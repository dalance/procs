use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct VmSize {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    max_width: usize,
}

impl VmSize {
    pub fn new() -> Self {
        let header = String::from("VmSize");
        let unit = String::from("[bytes]");
        VmSize {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for VmSize {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pid();
        let raw_content = proc.curr_proc.stat.vsize;
        let (size, unit) = unbytify::bytify(raw_content);
        let fmt_content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(u64);
}

#[cfg(target_os = "macos")]
impl Column for VmSize {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pbsd.pbi_pid as i32;
        let raw_content = proc.curr_proc.ptinfo.pti_virtual_size;
        let (size, unit) = unbytify::bytify(raw_content);
        let fmt_content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(u64);
}
