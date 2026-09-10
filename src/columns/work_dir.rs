use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(target_os = "freebsd")]
use libc::{c_void, CTL_KERN, KERN_PROC, KERN_PROC_CWD};
#[cfg(target_os = "macos")]
use libproc::libproc::proc_pid::{PIDInfo, PidInfoFlavor, pidinfo};
use std::cmp;
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(target_os = "freebsd")]
use std::ptr;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::HANDLE;

pub struct WorkDir {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, String>,
    width: usize,
    #[allow(dead_code)]
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

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for WorkDir {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(proc) = crate::util::process_new(proc.pid, &self.procfs) {
            if let Ok(path) = proc.cwd() {
                path.to_string_lossy().to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for WorkDir {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = work_dir_of(proc.pid).unwrap_or_default();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for WorkDir {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = work_dir_of(proc.pid)
            .map(|dir| dir.to_string_lossy().into_owned())
            .unwrap_or_default();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

/// The directory the process is sitting in, from `proc_pidinfo` with
/// `PROC_PIDVNODEPATHINFO`.
///
/// Unlike the environment, this one is not already in hand: the collector
/// never asks for it, so it is read here, once per row, only when the column
/// is on screen. `PathInfo::root` is no substitute - that is where the
/// executable was loaded from, not where the process has since moved to.
///
/// A thread row is left blank: a directory belongs to a process, and the row
/// stands for a thread of one.
#[cfg(target_os = "macos")]
impl Column for WorkDir {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if crate::process::thread_id(proc.pid).is_some() {
            String::new()
        } else {
            work_dir_of(proc.pid)
                .map(|dir| dir.to_string_lossy().into_owned())
                .unwrap_or_default()
        };

        self.fmt_contents.insert(proc.pid, fmt_content.clone());
        self.raw_contents.insert(proc.pid, fmt_content);
    }

    column_default!(String, false);
}

/// The current working directory of `pid`, from FreeBSD's `KERN_PROC_CWD`.
#[cfg(target_os = "freebsd")]
fn work_dir_of(pid: i64) -> Option<PathBuf> {
    // KERN_PROC_CWD returns struct kinfo_file, not a plain path. The path is
    // the final PATH_MAX-byte field in the FreeBSD kinfo_file ABI.
    const KINFO_FILE_PATH_OFFSET: usize = 368;

    if pid <= 0 {
        return None;
    }

    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_CWD, pid as i32];
    let mut size = 0usize;
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
        || size == 0
    {
        return None;
    }

    let mut bytes = vec![0u8; size];
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            bytes.as_mut_ptr() as *mut c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
    {
        return None;
    }
    bytes.truncate(size);
    if bytes.len() <= KINFO_FILE_PATH_OFFSET {
        return None;
    }
    let path = &bytes[KINFO_FILE_PATH_OFFSET..];
    let end = path.iter().position(|byte| *byte == 0).unwrap_or(path.len());
    if end == 0 {
        return None;
    }

    Some(PathBuf::from(String::from_utf8_lossy(&path[..end]).into_owned()))
}

/// The current working directory of `pid`.
///
/// `KERN_PROCARGS2` carries the arguments and the environment, but not the
/// directory a process sits in, so `proc_pidinfo` with
/// `PROC_PIDVNODEPATHINFO` is the only source for it. The kernel answers for
/// the processes of the user `procs` runs as and refuses the rest - the wall
/// the rest of `libproc` hits too - so `None` here means "not ours to see"
/// far more often than it means "no directory".
///
/// Note that this is not what `PathInfo::root` holds: that field is the
/// directory the executable was loaded from, `proc_pidpath` territory, and a
/// process that has changed directory since keeps pointing at the old one.
#[cfg(target_os = "macos")]
fn work_dir_of(pid: i64) -> Option<PathBuf> {
    // 0 is the kernel task and has no vnode paths; the negative pids are the
    // synthetic keys the thread rows carry.
    if pid <= 0 {
        return None;
    }

    let info = pidinfo::<VNodePathInfo>(pid as i32, 0).ok()?;
    // libc declares `vip_path` as `[[c_char; 32]; 32]`, not `[c_char; MAXPATHLEN]`.
    let path = crate::util::ptr_to_cstr(info.0.pvi_cdir.vip_path.as_flattened()).ok()?;
    Some(PathBuf::from(path.to_string_lossy().into_owned()))
}

/// libproc's `pidinfo` passes `&mut T` to `proc_pidinfo` sized by
/// `size_of::<T>()`, so this must keep the exact layout of
/// `proc_vnodepathinfo`.
#[cfg(target_os = "macos")]
#[repr(transparent)]
struct VNodePathInfo(libc::proc_vnodepathinfo);

#[cfg(target_os = "macos")]
impl PIDInfo for VNodePathInfo {
    fn flavor() -> PidInfoFlavor {
        PidInfoFlavor::VNodePathInfo
    }
}

/// Reads the current working directory of `pid` from its PEB.
///
/// The PEB address comes from `NtQueryInformationProcess` with
/// `ProcessBasicInformation`; the PEB and the process parameters it points at
/// are then read with `ReadProcessMemory`. Needs
/// `PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ`, so protected
/// processes (e.g. PPL) yield `None`.
#[cfg(target_os = "windows")]
fn work_dir_of(pid: i64) -> Option<String> {
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

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    /// The kernel and the process agree about where the process is.
    ///
    /// The two paths are canonicalized because the kernel reports the real
    /// path - `/private/tmp` rather than `/tmp` - while the standard library
    /// is free to report either.
    #[test]
    fn work_dir_of_self() {
        let cwd = std::env::current_dir().unwrap().canonicalize().unwrap();
        let dir = work_dir_of(std::process::id() as i64)
            .unwrap()
            .canonicalize()
            .unwrap();
        assert_eq!(dir, cwd);
    }

    /// Pids there is nothing to ask about: 0 is the kernel task, and the
    /// negative keys the thread rows carry are not pids at all.
    #[test]
    fn work_dir_of_an_unaskable_pid() {
        assert_eq!(work_dir_of(0), None);
        assert_eq!(work_dir_of(-1), None);
    }
}
