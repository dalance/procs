use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HANDLE;

pub struct WorkDir {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    procfs: Option<PathBuf>,
}

impl WorkDir {
    pub fn new(header: Option<String>, procfs: Option<PathBuf>) -> Self {
        let header = header.unwrap_or_else(|| String::from("WorkDir"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            procfs,
        }
    }
}

impl Column for WorkDir {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = work_dir_of(proc.pid, &self.procfs).unwrap_or_default();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn work_dir_of(pid: i32, procfs: &Option<PathBuf>) -> Option<String> {
    let proc = crate::util::process_new(pid, procfs).ok()?;
    Some(proc.cwd().ok()?.to_string_lossy().into_owned())
}

/// Reads the current working directory of `pid` from its PEB.
///
/// The PEB address comes from `NtQueryInformationProcess` with
/// `ProcessBasicInformation`; the PEB and the process parameters it points at
/// are then read with `ReadProcessMemory`. Needs
/// `PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ`, so protected
/// processes (e.g. PPL) yield `None`.
#[cfg(target_os = "windows")]
fn work_dir_of(pid: i32, _procfs: &Option<PathBuf>) -> Option<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    // 0 is the idle process and has no PEB; negative pids are not real.
    if pid <= 0 {
        return None;
    }

    // SAFETY: `pid` is only handed to `OpenProcess`, and the handle it returns
    // is closed before this function returns.
    let handle: HANDLE = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
            FALSE,
            pid as u32,
        )
    };
    if handle.is_null() {
        return None;
    }

    let dir = read_work_dir(handle);
    unsafe { CloseHandle(handle) };
    dir
}

/// Reads the working directory out of the PEB of `handle`.
#[cfg(target_os = "windows")]
fn read_work_dir(handle: HANDLE) -> Option<String> {
    use windows_sys::Win32::Foundation::UNICODE_STRING;
    use crate::process::{
        process_peb_address, read_process_memory, PEB_PREFIX,
        RTL_USER_PROCESS_PARAMETERS_PREFIX,
    };
    use std::mem::{offset_of, size_of};
    use std::ptr;

    // The PEB address lives in the target's address space.
    let peb = process_peb_address(handle)?;

    // Read the leading PEB fields. `ProcessParameters` sits within the first
    // page, so a single read covers it.
    let mut peb_buf = [0u64; 0x100 / size_of::<u64>()];
    read_process_memory(handle, peb, &mut peb_buf)?;

    // SAFETY: `peb_buf` is 8-byte aligned and `ProcessParameters` is a
    // pointer-sized field at a pointer-aligned offset, so the read is aligned.
    let params_addr = unsafe {
        peb_buf
            .as_ptr()
            .cast::<u8>()
            .add(offset_of!(PEB_PREFIX, ProcessParameters))
            .cast::<usize>()
            .read()
    };
    // A 32-bit address from a 64-bit reader means a WOW64 process, whose
    // process parameters use the 32-bit layout; refuse rather than misparse.
    if params_addr == 0 || (size_of::<usize>() == 8 && params_addr >> 32 == 0) {
        return None;
    }

    // Read the leading process-parameters fields; `CurrentDirectory` sits
    // within the first page.
    let mut params_buf = [0u64; 0x100 / size_of::<u64>()];
    read_process_memory(handle, params_addr, &mut params_buf)?;

    // SAFETY: `params_buf` is 8-byte aligned and `CurrentDirectory` is at an
    // 8-byte-aligned offset, so the `UNICODE_STRING` read is aligned.
    let dos_path: UNICODE_STRING = unsafe {
        ptr::read(
            params_buf
                .as_ptr()
                .cast::<u8>()
                .add(offset_of!(RTL_USER_PROCESS_PARAMETERS_PREFIX, CurrentDirectory))
                .cast::<UNICODE_STRING>(),
        )
    };
    if dos_path.Buffer.is_null() || dos_path.Length == 0 {
        return None;
    }

    let bytes = dos_path.Length as usize;
    if bytes > MAX_WORK_DIR_BYTES {
        return None;
    }

    // The string lives in the target's address space, so it needs its own read.
    let mut chars = vec![0u16; bytes / 2];
    read_process_memory(handle, dos_path.Buffer as usize, &mut chars)?;

    Some(
        String::from_utf16_lossy(&chars)
            .trim_end_matches('\0')
            .to_owned(),
    )
}

/// Guard against a nonsensical `UNICODE_STRING::Length` turning into a huge
/// allocation. Real working directories are capped well below this.
#[cfg(target_os = "windows")]
const MAX_WORK_DIR_BYTES: usize = 64 * 1024;
