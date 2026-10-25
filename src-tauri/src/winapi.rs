//! Thin wrappers around the few Win32 calls Framewise needs.
//! All are polled cheaply by the tracking thread; no hooks or injected DLLs.

use std::path::Path;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::ProcessStatus::EnumProcesses;
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, OpenInputDesktop, DESKTOP_CONTROL_FLAGS, DESKTOP_READOBJECTS,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub fn foreground_process_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return None;
        }
        let mut pid = 0u32;
        if GetWindowThreadProcessId(hwnd, Some(&mut pid)) == 0 || pid == 0 {
            return None;
        }
        process_name(pid)
    }
}

pub fn process_name(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let name = image_name(handle);
        let _ = CloseHandle(handle);
        name
    }
}

/// Full image path for a currently running executable name (e.g. "mpvnet.exe").
pub fn process_path(name: &str) -> Option<String> {
    let wanted = name.trim();
    if wanted.is_empty() {
        return None;
    }
    for pid in pids() {
        let Some(path) = process_path_for_pid(pid) else {
            continue;
        };
        let matches = Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().eq_ignore_ascii_case(wanted))
            .unwrap_or(false);
        if matches {
            return Some(path);
        }
    }
    None
}

fn process_path_for_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let path = image_path(handle);
        let _ = CloseHandle(handle);
        path
    }
}

unsafe fn image_path(handle: HANDLE) -> Option<String> {
    let mut buf = [0u16; 32768];
    let mut size = buf.len() as u32;
    QueryFullProcessImageNameW(
        handle,
        PROCESS_NAME_FORMAT(0),
        PWSTR(buf.as_mut_ptr()),
        &mut size,
    )
    .ok()?;
    Some(String::from_utf16_lossy(&buf[..size as usize]))
}

unsafe fn image_name(handle: HANDLE) -> Option<String> {
    let path = image_path(handle)?;
    Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

/// Milliseconds since the last keyboard/mouse input (32-bit tick wrapping is handled).
pub fn idle_milliseconds() -> u64 {
    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut info).as_bool() {
            let now = GetTickCount();
            now.wrapping_sub(info.dwTime) as u64
        } else {
            0
        }
    }
}

/// The secure/lock desktop cannot be opened while the session is locked.
pub fn is_session_locked() -> bool {
    unsafe {
        match OpenInputDesktop(DESKTOP_CONTROL_FLAGS(0), false, DESKTOP_READOBJECTS) {
            Ok(desktop) => {
                let _ = CloseDesktop(desktop);
                false
            }
            Err(_) => true,
        }
    }
}

fn pids() -> Vec<u32> {
    let mut capacity = 2048usize;
    loop {
        let mut buf = vec![0u32; capacity];
        let mut needed = 0u32;
        let cb = (buf.len() * std::mem::size_of::<u32>()) as u32;
        if unsafe { EnumProcesses(buf.as_mut_ptr(), cb, &mut needed) }.is_err() {
            return Vec::new();
        }
        let used = needed as usize / std::mem::size_of::<u32>();
        if used < capacity {
            buf.truncate(used);
            return buf;
        }
        capacity *= 2;
        if capacity > 65536 {
            return Vec::new();
        }
    }
}

fn filetime_ticks(ft: FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

fn cpu_ticks(pid: u32) -> Option<u64> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut creation = FILETIME::default();
        let mut exit = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let result = GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user);
        let _ = CloseHandle(handle);
        result.ok()?;
        Some(filetime_ticks(kernel) + filetime_ticks(user))
    }
}

/// Total CPU ticks across every running Adobe Media Encoder process.
/// Used only when the opt-in background render setting is enabled.
pub fn media_encoder_cpu_ticks() -> u64 {
    let mut total = 0u64;
    for pid in pids() {
        let Some(name) = process_name(pid) else {
            continue;
        };
        if name.eq_ignore_ascii_case("Adobe Media Encoder.exe") {
            if let Some(ticks) = cpu_ticks(pid) {
                total = total.saturating_add(ticks);
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_name_never_panics_for_invalid_pid() {
        let _ = process_name(0);
        let _ = process_name(u32::MAX);
    }
}
