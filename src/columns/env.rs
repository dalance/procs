use crate::process::ProcessInfo;
use crate::{Column, column_default};
use std::cmp;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HANDLE;

pub struct Env {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[allow(dead_code)]
    procfs: Option<PathBuf>,
}

impl Env {
    pub fn new(header: Option<String>, procfs: Option<PathBuf>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Env"));
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

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Env {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut fmt_content = String::new();
        if let Ok(proc) = crate::util::process_new(proc.pid, &self.procfs)
            && let Ok(envs) = proc.environ()
        {
            for (k, v) in envs {
                fmt_content.push_str(&format!(
                    "{}=\"{}\" ",
                    k.to_string_lossy(),
                    v.to_string_lossy().replace('\"', "\\\"")
                ));
            }
        }
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for Env {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut fmt_content = String::new();
        for env in &proc.curr_proc.env {
            fmt_content.push_str(&format!("{} ", env.replace('\"', "\\\"")));
        }
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for Env {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = env_of(proc.pid).unwrap_or_default();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

/// Reads the environment block of `pid` from its PEB.
///
/// The PEB address comes from `NtQueryInformationProcess` with
/// `ProcessBasicInformation`; the PEB, the process parameters it points at
/// and the environment block are then read with `ReadProcessMemory`. Needs
/// `PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ`, so protected
/// processes (e.g. PPL) yield `None`.
#[cfg(target_os = "windows")]
fn env_of(pid: i32) -> Option<String> {
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

    let env = read_env(handle);
    unsafe { CloseHandle(handle) };
    env
}

/// Reads the environment block out of the PEB of `handle` and formats it
/// like the Linux column (`KEY="VALUE" ` per entry).
#[cfg(target_os = "windows")]
fn read_env(handle: HANDLE) -> Option<String> {
    use crate::process::{
        process_peb_address, read_process_memory, PEB_PREFIX,
        RTL_USER_PROCESS_PARAMETERS_PREFIX,
    };
    use std::mem::{offset_of, size_of};

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

    // Read the leading process-parameters fields. `Environment` sits within
    // the first page, but `EnvironmentSize` sits at 0x3F0 (64-bit) / 0x290
    // (32-bit), so the buffer must be sized to cover it.
    let mut params_buf = [0u64; 0x400 / size_of::<u64>()];
    read_process_memory(handle, params_addr, &mut params_buf)?;

    // SAFETY: `params_buf` is 8-byte aligned and `Environment` is a
    // pointer-sized field at a pointer-aligned offset, so the read is aligned.
    let env_addr = unsafe {
        params_buf
            .as_ptr()
            .cast::<u8>()
            .add(offset_of!(RTL_USER_PROCESS_PARAMETERS_PREFIX, Environment))
            .cast::<usize>()
            .read()
    };
    if env_addr == 0 || (size_of::<usize>() == 8 && env_addr >> 32 == 0) {
        return None;
    }

    // `EnvironmentSize` is the byte size of the block, including the final
    // double-NUL terminator. It is a `ULONG_PTR`, so it is pointer-sized.
    let env_size = unsafe {
        params_buf
            .as_ptr()
            .cast::<u8>()
            .add(offset_of!(RTL_USER_PROCESS_PARAMETERS_PREFIX, EnvironmentSize))
            .cast::<usize>()
            .read()
    };
    if env_size == 0 || env_size > MAX_ENV_BYTES {
        return None;
    }

    // The block lives in the target's address space, so it needs its own read.
    let mut chars = vec![0u16; env_size.div_ceil(2)];
    read_process_memory(handle, env_addr, &mut chars)?;

    Some(format_env_block(&chars))
}

/// Guard against a nonsensical `EnvironmentSize` turning into a huge
/// allocation. Real environment blocks are capped well below this.
#[cfg(target_os = "windows")]
const MAX_ENV_BYTES: usize = 64 * 1024;

/// Formats a raw environment block like the Linux column:
/// `KEY="VALUE" ` for each entry, with embedded quotes escaped.
#[cfg(target_os = "windows")]
fn format_env_block(block: &[u16]) -> String {
    let mut out = String::new();
    let mut start = 0usize;
    for i in 0..block.len() {
        if block[i] == 0 {
            if i == start {
                break; // double NUL: end of block
            }
            let entry = String::from_utf16_lossy(&block[start..i]);
            if let Some(eq) = entry.find('=') {
                let (key, value) = entry.split_at(eq);
                out.push_str(&format!(
                    "{}=\"{}\" ",
                    key,
                    value[1..].replace('"', "\\\"")
                ));
            }
            start = i + 1;
        }
    }
    out
}
