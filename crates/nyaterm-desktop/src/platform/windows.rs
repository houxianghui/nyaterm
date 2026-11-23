use std::sync::Once;

use gpui::Window;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Dwm::{
    DWMNCRP_DISABLED, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_NCRENDERING_POLICY,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
};
use windows_sys::Win32::System::LibraryLoader::{
    GetModuleHandleW, GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW,
};
use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;
use windows_sys::core::{s, w};

static NATIVE_MENU_THEME_INIT: Once = Once::new();

pub(super) fn window_material_capabilities() -> super::WindowMaterialCapabilities {
    let composition_disabled = std::env::var("GPUI_DISABLE_DIRECT_COMPOSITION")
        .is_ok_and(|value| value == "true" || value == "1");
    let build = windows_version()
        .and_then(|version| (version.dwMajorVersion >= 10).then_some(version.dwBuildNumber));
    // This matches GPUI's dynamically loaded entry point; application code never
    // calls it or duplicates the backend's native material implementation.
    let accent_api_available = unsafe {
        let module = GetModuleHandleW(w!("user32.dll"));
        !module.is_null() && GetProcAddress(module, s!("SetWindowCompositionAttribute")).is_some()
    };
    let supported = window_material_supported(build, composition_disabled, accent_api_available);
    super::WindowMaterialCapabilities {
        transparent: supported,
        blurred: supported,
    }
}

fn window_material_supported(
    build: Option<u32>,
    composition_disabled: bool,
    accent_api_available: bool,
) -> bool {
    build.is_some_and(|build| build >= 17763) && !composition_disabled && accent_api_available
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DarkMenuApi {
    Legacy,
    PreferredAppMode,
}

impl DarkMenuApi {
    fn for_version(major: u32, minor: u32, build: u32) -> Option<Self> {
        if major != 10 || minor != 0 || build < 17763 {
            None
        } else if build < 18362 {
            Some(Self::Legacy)
        } else {
            Some(Self::PreferredAppMode)
        }
    }
}

fn windows_version() -> Option<OSVERSIONINFOW> {
    // RtlGetVersion reports the actual OS version without depending on the
    // executable's supportedOS manifest, which GetVersionEx would require.
    type RtlGetVersion = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
    let mut version = OSVERSIONINFOW {
        dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    // SAFETY: ntdll is loaded for the lifetime of every Windows process. Its
    // documented export takes this sized OSVERSIONINFOW structure and returns NTSTATUS.
    unsafe {
        let module = GetModuleHandleW(w!("ntdll.dll"));
        if module.is_null() {
            return None;
        }
        let procedure = GetProcAddress(module, s!("RtlGetVersion"))?;
        let get_version: RtlGetVersion = std::mem::transmute(procedure);
        (get_version(&mut version) >= 0).then_some(version)
    }
}

pub(super) fn enable_native_menu_dark_mode() {
    NATIVE_MENU_THEME_INIT.call_once(|| {
        let Some(version) = windows_version() else {
            return;
        };
        let Some(api) = DarkMenuApi::for_version(
            version.dwMajorVersion,
            version.dwMinorVersion,
            version.dwBuildNumber,
        ) else {
            return;
        };

        // These uxtheme exports are undocumented. Match Tao's build guards:
        // ordinal 135 changed ABI in Windows 10 1903, so never call it before
        // selecting the signature for the actual OS version.
        // SAFETY: the DLL comes only from System32; every export is checked
        // before use and called with the signature for the supported build.
        unsafe {
            let module = LoadLibraryExW(
                w!("uxtheme.dll"),
                std::ptr::null_mut(),
                LOAD_LIBRARY_SEARCH_SYSTEM32,
            );
            if module.is_null() {
                return;
            }
            // Keep this module reference for the process lifetime: unloading
            // uxtheme here could discard the process-wide menu theme policy.
            let Some(procedure) = GetProcAddress(module, 135_usize as *const u8) else {
                return;
            };
            match api {
                DarkMenuApi::Legacy => {
                    type AllowDarkModeForApp = unsafe extern "system" fn(bool) -> bool;
                    let allow_dark: AllowDarkModeForApp = std::mem::transmute(procedure);
                    allow_dark(true);
                }
                DarkMenuApi::PreferredAppMode => {
                    // PreferredAppMode::AllowDark permits system-controlled
                    // dark menus; ForceDark would override the user's theme.
                    type SetPreferredAppMode = unsafe extern "system" fn(i32) -> i32;
                    let set_mode: SetPreferredAppMode = std::mem::transmute(procedure);
                    set_mode(1);
                }
            }
            if let Some(procedure) = GetProcAddress(module, 104_usize as *const u8) {
                type RefreshImmersiveColorPolicyState = unsafe extern "system" fn();
                let refresh: RefreshImmersiveColorPolicyState = std::mem::transmute(procedure);
                refresh();
            }
        }
    });
}

/// Windows 11 introduced the DWM corner-preference attribute.
const WINDOWS_11_MIN_BUILD: u32 = 22000;

fn rounded_corners_supported(build: u32) -> bool {
    build >= WINDOWS_11_MIN_BUILD
}

/// Round the custom-framed workspace windows through DWM.
///
/// GPUI hides the native caption but keeps `WS_THICKFRAME`. With DWM's default
/// non-client rendering, Windows 11 draws the (unpainted) frame area around the
/// client and leaves the visible content square; requesting `DWMWCP_ROUND`
/// alone changes nothing. Disabling non-client rendering
/// (`DWMNCRP_DISABLED`) together with `DWMWCP_ROUND` makes the rounded client
/// edge visible (verified 2026-11-23). Failures are non-fatal: the window keeps
/// its square corners instead of blocking creation.
pub(super) fn apply_rounded_window_corners(window: &Window) -> bool {
    let Some(version) = windows_version() else {
        return false;
    };
    if !rounded_corners_supported(version.dwBuildNumber) {
        return false;
    }
    // `Window` has an inherent typed `window_handle()`; call the raw-handle
    // trait explicitly to reach the native HWND.
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return false;
    };
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return false;
    };
    // SAFETY: the HWND belongs to the live gpui window. Each attribute takes a
    // reference to one value of its documented type.
    let hwnd: HWND = win32.hwnd.get() as *mut core::ffi::c_void;
    let nc_rendering: i32 = DWMNCRP_DISABLED;
    let preference: i32 = DWMWCP_ROUND;
    let border_color: u32 = DWMWA_COLOR_NONE;
    let nc_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY as u32,
            &nc_rendering as *const i32 as *const core::ffi::c_void,
            std::mem::size_of_val(&nc_rendering) as u32,
        )
    };
    let corner_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &preference as *const i32 as *const core::ffi::c_void,
            std::mem::size_of_val(&preference) as u32,
        )
    };
    // Keep the DWM 1px border from painting a straight edge over the rounded outline.
    let _ = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR as u32,
            &border_color as *const u32 as *const core::ffi::c_void,
            std::mem::size_of_val(&border_color) as u32,
        )
    };
    nc_result == 0 && corner_result == 0
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
    use windows_sys::core::w;

    use super::{
        DarkMenuApi, enable_native_menu_dark_mode, rounded_corners_supported, windows_version,
    };

    #[test]
    fn rounded_window_corners_require_windows_11() {
        for (build, expected) in [
            (0, false),
            (19045, false),
            (21999, false),
            (22000, true),
            (22631, true),
            (26100, true),
        ] {
            assert_eq!(rounded_corners_supported(build), expected, "build {build}");
        }
    }

    #[test]
    fn native_window_material_requires_supported_build_composition_and_accent_api() {
        for (build, disabled, api, expected) in [
            (None, false, true, false),
            (Some(17762), false, true, false),
            (Some(17763), false, true, true),
            (Some(19045), false, true, true),
            (Some(22621), false, true, true),
            (Some(22621), true, true, false),
            (Some(22621), false, false, false),
        ] {
            assert_eq!(
                super::window_material_supported(build, disabled, api),
                expected
            );
        }
    }

    #[test]
    fn native_menu_api_uses_actual_windows_build_to_select_ordinal_135_abi() {
        for (major, minor, build, expected) in [
            (6, 3, 9600, None),
            (10, 0, 17134, None),
            (10, 0, 17762, None),
            (10, 0, 17763, Some(DarkMenuApi::Legacy)),
            (10, 0, 18361, Some(DarkMenuApi::Legacy)),
            (10, 0, 18362, Some(DarkMenuApi::PreferredAppMode)),
            (10, 0, 19045, Some(DarkMenuApi::PreferredAppMode)),
            (10, 0, 26100, Some(DarkMenuApi::PreferredAppMode)),
            (11, 0, 26100, None),
        ] {
            assert_eq!(DarkMenuApi::for_version(major, minor, build), expected);
        }
    }

    #[test]
    fn native_menu_initialization_enables_process_dark_mode_and_is_idempotent() {
        let version = windows_version().expect("actual Windows version");
        if DarkMenuApi::for_version(
            version.dwMajorVersion,
            version.dwMinorVersion,
            version.dwBuildNumber,
        )
        .is_none()
        {
            // Unsupported systems intentionally retain the native light menu.
            enable_native_menu_dark_mode();
            return;
        }
        enable_native_menu_dark_mode();
        enable_native_menu_dark_mode();
        // SAFETY: initialization keeps uxtheme loaded. Ordinal 137 is
        // IsDarkModeAllowedForApp on the supported Windows builds above.
        unsafe {
            let module = GetModuleHandleW(w!("uxtheme.dll"));
            assert!(!module.is_null());
            let procedure =
                GetProcAddress(module, 137_usize as *const u8).expect("IsDarkModeAllowedForApp");
            let is_allowed: unsafe extern "system" fn() -> bool = std::mem::transmute(procedure);
            assert!(
                is_allowed(),
                "native menus must be allowed to follow the system dark theme"
            );
        }
    }
}
