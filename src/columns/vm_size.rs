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
        let raw_content = proc.curr_proc.stat.vsize;
        let (size, unit) = unbytify::bytify(raw_content);
        let fmt_content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}

#[cfg(target_os = "macos")]
impl Column for VmSize {
    fn add(&mut self, proc: &ProcessInfo) {
        //let raw_content = proc.curr_task.ptinfo.pti_virtual_size;
        let raw_content = if let Some(ref curr_res) = proc.curr_res {
            curr_res.ri_lifetime_max_phys_footprint
        } else {
            0
        };
        let (size, unit) = unbytify::bytify(raw_content);
        let fmt_content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}
