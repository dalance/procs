use crate::process::ProcessInfo;
use crate::util::bytify;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct VmTotal {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl VmTotal {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("VmTotal"));
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
impl Column for VmTotal {
    fn add(&mut self, proc: &ProcessInfo) {
        use procfs::WithCurrentSystemInfo;
        let raw_content = proc.curr_proc.stat().rss_bytes().get();
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "macos")]
impl Column for VmTotal {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = match &proc.curr_res {
            Some(mem) => mem.ri_phys_footprint,
            None => proc.curr_task.ptinfo.pti_resident_size,
        };
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "windows")]
impl Column for VmTotal {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.memory_info.working_set_size;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for VmTotal {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = (proc.curr_proc.info.rssize as u64).saturating_mul(4096);
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}
