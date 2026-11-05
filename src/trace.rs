//! Diagnostic log of window messages, written next to the DLL with the extension ".log".
//!
//! Records the windows of Becky!'s UI thread, the items of the menus passed to
//! BKC_OnMenuInit, and the messages those windows receive (WM_USER and above,
//! WM_SYSCOMMAND, WM_CLOSE), so that the notification sent from the task tray
//! icon can be identified.

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use windows_sys::Win32::Foundation::{HMODULE, HWND, LPARAM, LRESULT, MAX_PATH, WPARAM};
use windows_sys::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW,
};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GetClassNameW, GetMenuItemCount, GetMenuItemID, GetMenuStringW,
    GetWindowTextW, HMENU, IsIconic, IsWindowVisible, MF_BYPOSITION, WM_CLOSE, WM_NCDESTROY,
    WM_SYSCOMMAND, WM_USER,
};

const SUBCLASS_ID: usize = 2;
const TEXT_CAPACITY: usize = 256;

struct Log {
    file: File,
    started: Instant,
}

static LOG: Mutex<Option<Log>> = Mutex::new(None);

pub fn init() {
    let Some(path) = log_path() else {
        return;
    };
    let Ok(file) = OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    if let Ok(mut log) = LOG.lock() {
        *log = Some(Log { file, started: Instant::now() });
    }
    line(format_args!("---- start"));
}

fn log_path() -> Option<PathBuf> {
    let mut module: HMODULE = std::ptr::null_mut();
    let mut buffer = [0u16; MAX_PATH as usize];
    // SAFETY: the address of a function in this DLL identifies the DLL itself.
    let length = unsafe {
        let found = GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            init as *const () as *const u16,
            &mut module,
        );
        if found == 0 {
            return None;
        }
        GetModuleFileNameW(module, buffer.as_mut_ptr(), buffer.len() as u32) as usize
    };
    if length == 0 {
        return None;
    }
    let mut path = PathBuf::from(String::from_utf16_lossy(&buffer[..length]));
    path.set_extension("log");
    Some(path)
}

fn line(arguments: fmt::Arguments) {
    let Ok(mut guard) = LOG.lock() else {
        return;
    };
    let Some(log) = guard.as_mut() else {
        return;
    };
    let elapsed = log.started.elapsed().as_millis();
    let _ = writeln!(log.file, "{elapsed:>8} {arguments}");
}

pub fn start(main: HWND, attached: bool) {
    line(format_args!("main {main:?} subclassed={attached}"));
    // SAFETY: called on Becky!'s UI thread; the callback only reads window properties.
    unsafe {
        EnumThreadWindows(GetCurrentThreadId(), Some(visit_thread_window), main as LPARAM);
    }
}

unsafe extern "system" fn visit_thread_window(hwnd: HWND, main: LPARAM) -> i32 {
    // SAFETY: hwnd is a top-level window of the calling thread.
    unsafe {
        line(format_args!(
            "window {hwnd:?} class={:?} title={:?} visible={} iconic={}",
            class_name(hwnd),
            window_text(hwnd),
            IsWindowVisible(hwnd) != 0,
            IsIconic(hwnd) != 0,
        ));
        if hwnd as LPARAM != main {
            SetWindowSubclass(hwnd, Some(logging_proc), SUBCLASS_ID, 0);
        }
    }
    1
}

unsafe extern "system" fn logging_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _subclass_id: usize,
    _reference_data: usize,
) -> LRESULT {
    self::message(hwnd, message, wparam, lparam);
    // SAFETY: called by comctl32 for the subclassed window.
    unsafe {
        if message == WM_NCDESTROY {
            RemoveWindowSubclass(hwnd, Some(logging_proc), SUBCLASS_ID);
        }
        DefSubclassProc(hwnd, message, wparam, lparam)
    }
}

pub fn message(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) {
    if message < WM_USER && message != WM_SYSCOMMAND && message != WM_CLOSE {
        return;
    }
    // SAFETY: hwnd is a window receiving this message.
    let (visible, iconic) = unsafe { (IsWindowVisible(hwnd) != 0, IsIconic(hwnd) != 0) };
    line(format_args!(
        "msg {hwnd:?} {message:#06x} wparam={wparam:#x} lparam={lparam:#x} visible={visible} iconic={iconic}"
    ));
}

pub fn menu(hwnd: HWND, menu: HMENU, menu_type: i32) {
    line(format_args!("menu type={menu_type} hwnd={hwnd:?} class={:?}", class_name(hwnd)));
    // SAFETY: menu is the handle Becky! passed to BKC_OnMenuInit.
    unsafe {
        let count = GetMenuItemCount(menu);
        for position in 0..count.max(0) {
            let mut buffer = [0u16; TEXT_CAPACITY];
            let length = GetMenuStringW(
                menu,
                position as u32,
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                MF_BYPOSITION,
            );
            let text = String::from_utf16_lossy(&buffer[..length.max(0) as usize]);
            let id = GetMenuItemID(menu, position);
            line(format_args!("  item {position} id={id:#x} {text:?}"));
        }
    }
}

fn class_name(hwnd: HWND) -> String {
    let mut buffer = [0u16; TEXT_CAPACITY];
    // SAFETY: the buffer length is passed along with the buffer.
    let length = unsafe { GetClassNameW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..length.max(0) as usize])
}

fn window_text(hwnd: HWND) -> String {
    let mut buffer = [0u16; TEXT_CAPACITY];
    // SAFETY: the buffer length is passed along with the buffer.
    let length = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    String::from_utf16_lossy(&buffer[..length.max(0) as usize])
}
