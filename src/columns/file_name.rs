use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(target_os = "freebsd")]
use libc::{CTL_KERN, KERN_PROC, KERN_PROC_PATHNAME, c_void};
use std::cmp;
use std::collections::HashMap;
#[cfg(target_os = "freebsd")]
use std::ffi::CStr;
#[cfg(target_os = "freebsd")]
use std::ptr;
#[cfg(target_os = "freebsd")]
use std::path::Path;

pub struct FileName {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl FileName {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("FileName"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "freebsd")]
pub(crate) fn get_process_path(pid: i32) -> Option<String> {
    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_PATHNAME, pid];
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
    CStr::from_bytes_until_nul(&bytes[..size])
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for FileName {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().comm.clone();
        let fmt_content = raw_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for FileName {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.file_name.clone();
        let fmt_content = raw_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for FileName {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = get_process_path(proc.pid)
            .and_then(|path| Path::new(&path).file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_default();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for FileName {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(path) = &proc.curr_path {
            path.name.clone()
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
