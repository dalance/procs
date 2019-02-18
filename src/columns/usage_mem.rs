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
            mem_total: procfs::meminfo().unwrap().mem_total,
        }
    }
}

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
