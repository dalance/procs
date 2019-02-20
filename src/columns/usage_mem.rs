use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct UsageMem {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    max_width: usize,
    mem_total: u64,
}

impl UsageMem {
    pub fn new() -> Self {
        let header = String::from("MEM");
        let unit = String::from("[%]");

        UsageMem {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
            mem_total: get_mem_total(),
        }
    }
}

#[cfg(target_os = "linux")]
fn get_mem_total() -> u64 {
    procfs::meminfo().unwrap().mem_total
}

#[cfg(target_os = "macos")]
fn get_mem_total() -> u64 {
    let mut mem_total: u64 = 0;
    let mut mib = [0, 0];
    crate::util::get_sys_value(
        libc::CTL_HW as u32,
        libc::HW_MEMSIZE as u32,
        std::mem::size_of::<u64>(),
        &mut mem_total as *mut u64 as *mut libc::c_void,
        &mut mib,
    );
    mem_total
}

#[cfg(target_os = "linux")]
impl Column for UsageMem {
    fn add(&mut self, proc: &ProcessInfo) {
        let usage = proc.curr_proc.stat.rss_bytes() as f64 * 100.0 / self.mem_total as f64;
        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32);
}

#[cfg(target_os = "macos")]
impl Column for UsageMem {
    fn add(&mut self, proc: &ProcessInfo) {
        let usage = proc.curr_task.ptinfo.pti_resident_size as f64 * 100.0 / self.mem_total as f64;
        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32);
}
