//! Becky! 2 plugin that sends the main window to the task tray instead of closing it.

mod becky;
mod main_window;
#[cfg(feature = "trace-messages")]
mod trace;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::HMENU;

/// Called when Becky! collects plug-in information.
/// Becky! ignores the plug-in unless the name and the vendor are filled.
///
/// # Safety
/// `info` must be null or point to a writable BKPLUGININFO.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn BKC_OnPlugInInfo(info: *mut becky::PluginInfo) -> i32 {
    // SAFETY: guaranteed by the caller.
    if let Some(info) = unsafe { info.as_mut() } {
        info.fill(
            "Close to Tray",
            "ISIGE, Akira",
            env!("CARGO_PKG_VERSION"),
            "Sends the main window to the task tray instead of closing it.",
        );
    }
    0
}

/// Called after Becky!'s main window is created.
#[unsafe(no_mangle)]
pub extern "system" fn BKC_OnStart() -> i32 {
    #[cfg(feature = "trace-messages")]
    trace::init();

    let Some(main) = becky::main_window() else {
        return 0;
    };
    let attached = main_window::attach(main);

    #[cfg(feature = "trace-messages")]
    trace::start(main, attached);
    #[cfg(not(feature = "trace-messages"))]
    let _ = attached;
    0
}

/// Called when a menu of Becky! is initialized.
#[unsafe(no_mangle)]
pub extern "system" fn BKC_OnMenuInit(hwnd: HWND, menu: HMENU, menu_type: i32) -> i32 {
    #[cfg(feature = "trace-messages")]
    trace::menu(hwnd, menu, menu_type);
    #[cfg(not(feature = "trace-messages"))]
    let _ = (hwnd, menu, menu_type);
    0
}
