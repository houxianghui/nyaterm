//! Suppress the DWM non-client border that Windows paints around undecorated
//! transparent windows. With `transparent` + `decorations: false` the
//! window has no native chrome, but DWM still draws a 1px ring along the straight
//! edges (the rounded corners are drawn by the webview and clip the ring away).

use tauri::WebviewWindow;

#[cfg(windows)]
use windows_sys::Win32::Foundation::HWND;

/// Disable DWM non-client-area rendering for a window so the default 1px border is
/// not painted. Safe to call on any platform; a no-op off Windows.
pub fn disable_dwm_border(window: &WebviewWindow) {
    #[cfg(windows)]
    {
        use windows_sys::Win32::Graphics::Dwm::{
            DWMWA_NCRENDERING_POLICY, DWMNCRP_DISABLED, DwmSetWindowAttribute,
        };

        let Ok(hwnd) = window.hwnd() else {
            tracing::warn!(
                window_label = window.label(),
                "failed to get HWND to disable DWM non-client border"
            );
            return;
        };
        let hwnd = hwnd.0 as HWND;

        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_NCRENDERING_POLICY as u32,
                &DWMNCRP_DISABLED as *const _ as *const core::ffi::c_void,
                core::mem::size_of_val(&DWMNCRP_DISABLED) as u32,
            );
        }
    }

    #[cfg(not(windows))]
    {
        let _ = window;
    }
}