use gpui::Window;

const RESIZE_BORDER_DIP: u32 = 6;
const SETTINGS_SUBCLASS_ID: usize = 0x4e5951;

/// Remove the native frame and supply the resize hit tests normally provided
/// by WS_THICKFRAME. All other messages continue through GPUI's window proc.
pub(super) fn configure_settings_native_frame(window: &Window) -> bool {
    let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window) else {
        return false;
    };
    let raw_window_handle::RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return false;
    };
    let hwnd = handle.hwnd.get() as _;

    unsafe { configure_frameless_window(hwnd) && configure_dwm_frame(hwnd) }
}

unsafe fn configure_frameless_window(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::UI::Shell::SetWindowSubclass;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_STYLE, GetWindowLongPtrW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, WS_CAPTION, WS_THICKFRAME,
    };

    if unsafe { SetWindowSubclass(hwnd, Some(settings_subclass), SETTINGS_SUBCLASS_ID, 0) } == 0 {
        return false;
    }
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) };
    let frameless = style & !((WS_CAPTION | WS_THICKFRAME) as isize);
    if unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, frameless) } == 0 {
        return false;
    }
    unsafe {
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        ) != 0
    }
}

unsafe fn configure_dwm_frame(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE,
        DWMWCP_ROUND, DwmSetWindowAttribute,
    };

    let corner = DWMWCP_ROUND;
    let corner_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            &corner as *const _ as _,
            std::mem::size_of_val(&corner) as u32,
        )
    };
    let border = DWMWA_COLOR_NONE;
    let border_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR as u32,
            &border as *const _ as _,
            std::mem::size_of_val(&border) as u32,
        )
    };
    let caption = DWMWA_COLOR_NONE;
    let caption_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR as u32,
            &caption as *const _ as _,
            std::mem::size_of_val(&caption) as u32,
        )
    };
    corner_result >= 0 && border_result >= 0 && caption_result >= 0
}

unsafe extern "system" fn settings_subclass(
    hwnd: windows_sys::Win32::Foundation::HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
    _subclass_id: usize,
    _ref_data: usize,
) -> isize {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
    use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowRect, IsZoomed, PostMessageW, WM_NCDESTROY, WM_NCHITTEST, WM_NCLBUTTONDOWN,
        WM_SYSCOMMAND,
    };

    if message == WM_NCLBUTTONDOWN
        && let Some(command) = resize_sys_command(wparam as u32)
    {
        // Let the button-down return before entering the modal resize loop.
        // A synchronous send here re-enters the window proc and prevents real
        // mouse dragging even though a posted NCLBUTTONDOWN appears to work.
        unsafe {
            PostMessageW(hwnd, WM_SYSCOMMAND, command, lparam);
        }
        return 0;
    }
    if message == WM_NCHITTEST && unsafe { IsZoomed(hwnd) == 0 } {
        let mut rect = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut rect) } != 0 {
            let dpi = unsafe { GetDpiForWindow(hwnd) };
            let border = ((dpi * RESIZE_BORDER_DIP + 48) / 96) as i32;
            let x = (lparam as i16) as i32;
            let y = ((lparam >> 16) as i16) as i32;
            if let Some(hit) = resize_hit_test(x, y, &rect, border) {
                return hit as isize;
            }
        }
    }
    if message == WM_NCDESTROY {
        unsafe {
            RemoveWindowSubclass(hwnd, Some(settings_subclass), SETTINGS_SUBCLASS_ID);
        }
    }
    unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
}

fn resize_sys_command(hit: u32) -> Option<usize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{HTBOTTOMRIGHT, HTLEFT, SC_SIZE, WMSZ_LEFT};

    (HTLEFT..=HTBOTTOMRIGHT)
        .contains(&hit)
        .then(|| (SC_SIZE + hit - HTLEFT + WMSZ_LEFT) as usize)
}

fn resize_hit_test(
    x: i32,
    y: i32,
    rect: &windows_sys::Win32::Foundation::RECT,
    border: i32,
) -> Option<u32> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT,
    };

    if x < rect.left || x >= rect.right || y < rect.top || y >= rect.bottom {
        return None;
    }
    let left = x < rect.left + border;
    let right = x >= rect.right - border;
    let top = y < rect.top + border;
    let bottom = y >= rect.bottom - border;
    match (left, right, top, bottom) {
        (true, _, true, _) => Some(HTTOPLEFT),
        (_, true, true, _) => Some(HTTOPRIGHT),
        (true, _, _, true) => Some(HTBOTTOMLEFT),
        (_, true, _, true) => Some(HTBOTTOMRIGHT),
        (true, _, _, _) => Some(HTLEFT),
        (_, true, _, _) => Some(HTRIGHT),
        (_, _, true, _) => Some(HTTOP),
        (_, _, _, true) => Some(HTBOTTOM),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT,
        SC_SIZE,
    };

    use super::resize_hit_test;

    const RECT: RECT = RECT {
        left: 100,
        top: 200,
        right: 900,
        bottom: 700,
    };

    #[test]
    fn resize_hit_test_covers_eight_edges_without_capturing_the_interior() {
        for (x, y, expected) in [
            (100, 200, HTTOPLEFT),
            (450, 200, HTTOP),
            (899, 200, HTTOPRIGHT),
            (100, 450, HTLEFT),
            (899, 450, HTRIGHT),
            (100, 699, HTBOTTOMLEFT),
            (450, 699, HTBOTTOM),
            (899, 699, HTBOTTOMRIGHT),
        ] {
            assert_eq!(super::resize_hit_test(x, y, &RECT, 6), Some(expected));
        }
        assert_eq!(resize_hit_test(450, 450, &RECT, 6), None);
        assert_eq!(resize_hit_test(99, 450, &RECT, 6), None);
    }

    #[test]
    fn resize_hit_codes_map_to_the_standard_sys_commands() {
        for (offset, hit) in [
            HTLEFT,
            HTRIGHT,
            HTTOP,
            HTTOPLEFT,
            HTTOPRIGHT,
            HTBOTTOM,
            HTBOTTOMLEFT,
            HTBOTTOMRIGHT,
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                super::resize_sys_command(hit),
                Some((SC_SIZE + offset as u32 + 1) as usize)
            );
        }
        assert_eq!(super::resize_sys_command(0), None);
    }
}
