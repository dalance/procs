use crate::process::ProcessInfo;
use crate::util::bytify;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct VmData {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl VmData {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("VmData"));
        let unit = String::from("[bytes]");
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
impl Column for VmData {
    fn add(&mut self, proc: &ProcessInfo) {
        let (raw_content, fmt_content) = if let Some(ref curr_status) = proc.curr_status {
            if let Some(val) = curr_status.vmdata {
                let val = val.saturating_mul(1024);
                (val, bytify(val))
            } else {
                (0, String::new())
            }
        } else {
            (0, String::new())
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for VmData {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = (proc.curr_proc.info.dsize as u64).saturating_mul(4096);
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}
