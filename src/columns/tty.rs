use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Tty {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl Tty {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("TTY"));
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

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Tty {
    fn add(&mut self, proc: &ProcessInfo) {
        let (major, minor) = proc.curr_proc.stat().tty_nr();
        let fmt_content = if major == 136 {
            format!("pts/{minor}")
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for Tty {
    fn add(&mut self, proc: &ProcessInfo) {
        let dev = proc.curr_task.pbsd.e_tdev;
        let major = (dev >> 24) & 0xff;
        let minor = dev & 0xffffff;
        let fmt_content = if major == 16 {
            format!("s{:03}", minor)
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for Tty {
    fn add(&mut self, proc: &ProcessInfo) {
        let dev = proc.curr_proc.info.tdev;

        let mut buf = [0u8; 256];

        let name = std::ffi::CString::new("kern.devname").unwrap();
        let mut buf_size = std::mem::size_of::<[u8; 256]>();
        let buf_ptr = buf.as_mut_ptr();
        let dev_size = std::mem::size_of::<u64>();
        let dev_ptr: *const u64 = &dev;

        unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                buf_ptr as *mut libc::c_void,
                &mut buf_size,
                dev_ptr as *const libc::c_void,
                dev_size,
            );
        }

        let fmt_content = if let Ok(devname) = std::ffi::CStr::from_bytes_until_nul(&buf) {
            devname.to_string_lossy().into_owned()
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
