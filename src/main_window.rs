//! Subclass of Becky!'s main window.
//!
//! The close button and Alt+F4 arrive as WM_SYSCOMMAND/SC_CLOSE. They are
//! turned into SC_MINIMIZE, and Becky! puts the minimized window into the
//! task tray. "Exit" in the menu posts WM_CLOSE directly, so it still quits.
//!
//! Pressing the left button on the task tray icon restores the hidden window
//! (handled by Becky!). While the window is shown, the same press minimizes it.

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsIconic, IsWindowVisible, SC_CLOSE, SC_MINIMIZE, WM_LBUTTONDOWN, WM_NCDESTROY, WM_SYSCOMMAND,
};

const SUBCLASS_ID: usize = 1;

/// The low four bits of the WM_SYSCOMMAND wParam are used internally by the system.
const SYSCOMMAND_MASK: WPARAM = 0xFFF0;

/// Callback message of Becky!'s task tray icon, observed with Becky! 2.
/// wParam is the icon ID and lParam is the mouse message.
const TRAY_CALLBACK_MESSAGE: u32 = 0x0BDA;
const TRAY_ICON_ID: WPARAM = 0x80;

pub fn attach(hwnd: HWND) -> bool {
    // SAFETY: hwnd is a window of the calling thread, as required by SetWindowSubclass.
    unsafe { SetWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID, 0) != 0 }
}

unsafe extern "system" fn subclass_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _reference_data: usize,
) -> LRESULT {
    #[cfg(feature = "trace-messages")]
    crate::trace::message(hwnd, message, wparam, lparam);

    // SAFETY: called by comctl32 for the subclassed window.
    unsafe {
        match message {
            WM_SYSCOMMAND if wparam & SYSCOMMAND_MASK == SC_CLOSE as WPARAM => {
                return DefSubclassProc(hwnd, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);
            }
            TRAY_CALLBACK_MESSAGE
                if wparam == TRAY_ICON_ID
                    && lparam as u32 == WM_LBUTTONDOWN
                    && IsWindowVisible(hwnd) != 0
                    && IsIconic(hwnd) == 0 =>
            {
                return DefSubclassProc(hwnd, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);
            }
            WM_NCDESTROY => {
                RemoveWindowSubclass(hwnd, Some(subclass_proc), SUBCLASS_ID);
            }
            _ => {}
        }
        DefSubclassProc(hwnd, message, wparam, lparam)
    }
}
