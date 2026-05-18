use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Edge {
    Top,
    Bottom,
}

impl Edge {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bottom" => Edge::Bottom,
            _ => Edge::Top,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn to_abe(self) -> u32 {
        match self {
            Edge::Top => 1,
            Edge::Bottom => 3,
        }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::Edge;
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::{HWND, RECT};
    use windows::Win32::UI::Shell::{
        SHAppBarMessage, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, MoveWindow, SM_CXSCREEN, SM_CYSCREEN,
    };

    pub fn register(hwnd_isize: isize, edge: Edge, height_px: i32) -> bool {
        unsafe {
            let hwnd = HWND(hwnd_isize as *mut _);
            let mut data: APPBARDATA = zeroed();
            data.cbSize = size_of::<APPBARDATA>() as u32;
            data.hWnd = hwnd;
            data.uCallbackMessage = 0;
            data.uEdge = edge.to_abe();

            let ok = SHAppBarMessage(ABM_NEW, &mut data);
            if ok == 0 {
                return false;
            }

            let mut rect = RECT {
                left: 0,
                top: 0,
                right: get_primary_width(),
                bottom: 0,
            };
            match edge {
                Edge::Top => {
                    rect.top = 0;
                    rect.bottom = height_px;
                }
                Edge::Bottom => {
                    let h = get_primary_height();
                    rect.bottom = h;
                    rect.top = h - height_px;
                }
            }
            data.rc = rect;
            SHAppBarMessage(ABM_QUERYPOS, &mut data);

            match edge {
                Edge::Top => data.rc.bottom = data.rc.top + height_px,
                Edge::Bottom => data.rc.top = data.rc.bottom - height_px,
            }
            SHAppBarMessage(ABM_SETPOS, &mut data);

            let _ = MoveWindow(
                hwnd,
                data.rc.left,
                data.rc.top,
                data.rc.right - data.rc.left,
                data.rc.bottom - data.rc.top,
                true,
            );
            true
        }
    }

    pub fn unregister(hwnd_isize: isize) -> bool {
        unsafe {
            let hwnd = HWND(hwnd_isize as *mut _);
            let mut data: APPBARDATA = zeroed();
            data.cbSize = size_of::<APPBARDATA>() as u32;
            data.hWnd = hwnd;
            SHAppBarMessage(ABM_REMOVE, &mut data) != 0
        }
    }

    fn get_primary_width() -> i32 {
        unsafe { GetSystemMetrics(SM_CXSCREEN) }
    }
    fn get_primary_height() -> i32 {
        unsafe { GetSystemMetrics(SM_CYSCREEN) }
    }
}

#[cfg(target_os = "windows")]
pub use imp::{register, unregister};

#[cfg(not(target_os = "windows"))]
pub fn register(_: isize, _: Edge, _: i32) -> bool {
    false
}
#[cfg(not(target_os = "windows"))]
pub fn unregister(_: isize) -> bool {
    false
}

/// Apply WS_EX_NOACTIVATE + force sharp (non-rounded) corners to a window.
/// - NOACTIVATE prevents the window from becoming foreground (immune to
///   shell-activation auto-hide for Start menu / Win+D etc).
/// - DwmSetWindowAttribute(DWMWCP_DONOTROUND) overrides Windows 11's
///   default rounded corners. Without this, the OS draws rounded corners
///   on the window frame regardless of any CSS border-radius value.
/// Clicks and data-tauri-drag-region still work after these flags.
#[cfg(target_os = "windows")]
pub fn apply_widget_styles(hwnd_isize: isize) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        WS_EX_TOPMOST,
    };
    unsafe {
        let hwnd = HWND(hwnd_isize as *mut _);

        // 1) Extended styles: NOACTIVATE + TOOLWINDOW + TOPMOST
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let add = (WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0 | WS_EX_TOPMOST.0) as isize;
        let new = cur | add;
        if new != cur {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new);
        }

        // 2) Disable Windows 11's default rounded corners (DWM)
        let pref: i32 = DWMWCP_DONOTROUND.0;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const i32 as *const _,
            std::mem::size_of::<i32>() as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_widget_styles(_: isize) {}

pub struct AppBarGuard {
    pub hwnd: isize,
    pub active: bool,
}

impl AppBarGuard {
    pub fn new() -> Self {
        Self { hwnd: 0, active: false }
    }
}

impl Drop for AppBarGuard {
    fn drop(&mut self) {
        if self.active && self.hwnd != 0 {
            unregister(self.hwnd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_from_str_defaults_to_top() {
        assert_eq!(Edge::from_str("top"), Edge::Top);
        assert_eq!(Edge::from_str("TOP"), Edge::Top);
        assert_eq!(Edge::from_str("bottom"), Edge::Bottom);
        assert_eq!(Edge::from_str("invalid"), Edge::Top);
        assert_eq!(Edge::from_str(""), Edge::Top);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn edge_to_abe_constants() {
        assert_eq!(Edge::Top.to_abe(), 1); // ABE_TOP
        assert_eq!(Edge::Bottom.to_abe(), 3); // ABE_BOTTOM
    }

    #[test]
    fn guard_inactive_does_nothing_on_drop() {
        let g = AppBarGuard::new();
        assert!(!g.active);
        assert_eq!(g.hwnd, 0);
        drop(g); // must not panic / not crash
    }
}
