use std::ptr;
use windows_sys::{w, Win32::{Foundation::TRUE, System::Threading::CreateMutexW, UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR}}};

mod app;
mod fsuipc;
mod vatsim;
mod aircraft;
mod worker;
mod server;

fn main() {
    // Ensure only one instance is running
    if !check_unique_instance() {
        unsafe { MessageBoxW(0, w!("Traffic Viewer is already running!"), w!("Traffic Viewer"), MB_ICONERROR) };
        std::process::exit(1);
    }

    // Run the app
    if let Err(error) = app::run() {
        unsafe { MessageBoxW(0, wide_null(format!("Error: {}", error.to_string())).as_ptr(), w!("Traffic Viewer"), MB_ICONERROR) };
        std::process::exit(1);
    }
}

fn check_unique_instance() -> bool {
    unsafe {
        CreateMutexW(ptr::null(), TRUE, w!("CMTrafficViewer")) != 0
    }
}

/// Allocates a utf-16, null-terminated version of the `&str` given.
///
/// **Note:** This will not filter any null characters (`'\0'`) that are in the
/// string. If you have an internal null Windows will think it means the end of
/// the string and not see your full string, which will probably make it do
/// something you don't want.
#[inline]
pub fn wide_null(s: impl AsRef<str>) -> Vec<u16> {
  s.as_ref().encode_utf16().chain(Some(0)).collect()
}