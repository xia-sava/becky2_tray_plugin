//! Declarations from the Becky! Plug-In SDK (BeckyApi.h, Template.cpp).

use std::ffi::c_char;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

/// BKPLUGININFO. Strings are ANSI (the system code page) and NUL-terminated.
#[repr(C)]
pub struct PluginInfo {
    pub name: [c_char; 80],
    pub vendor: [c_char; 80],
    pub version: [c_char; 80],
    pub description: [c_char; 256],
}

impl PluginInfo {
    pub fn fill(&mut self, name: &str, vendor: &str, version: &str, description: &str) {
        copy_ascii(&mut self.name, name);
        copy_ascii(&mut self.vendor, vendor);
        copy_ascii(&mut self.version, version);
        copy_ascii(&mut self.description, description);
    }
}

fn copy_ascii(destination: &mut [c_char], source: &str) {
    let Some(capacity) = destination.len().checked_sub(1) else {
        return;
    };
    let length = source.len().min(capacity);
    for (slot, byte) in destination.iter_mut().zip(&source.as_bytes()[..length]) {
        *slot = *byte as c_char;
    }
    destination[length] = 0;
}

type GetWindowHandlesFn =
    unsafe extern "system" fn(*mut HWND, *mut HWND, *mut HWND, *mut HWND) -> i32;

/// Returns Becky!'s main window via BKA_GetWindowHandles exported by B2.exe.
pub fn main_window() -> Option<HWND> {
    // SAFETY: B2.exe is the process image, and BKA_GetWindowHandles has the
    // signature declared in BeckyApi.h.
    unsafe {
        let becky = GetModuleHandleA(std::ptr::null());
        if becky.is_null() {
            return None;
        }
        let address = GetProcAddress(becky, c"BKA_GetWindowHandles".as_ptr().cast())?;
        let get_window_handles: GetWindowHandlesFn = std::mem::transmute(address);
        let mut main = null_mut();
        let mut tree = null_mut();
        let mut list = null_mut();
        let mut view = null_mut();
        if get_window_handles(&mut main, &mut tree, &mut list, &mut view) == 0 || main.is_null() {
            return None;
        }
        Some(main)
    }
}
