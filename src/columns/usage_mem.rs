use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct UsageMem {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    mem_total: u64,
}

impl UsageMem {
    pub fn new() -> Self {
        let header = String::from("MEM");
        let unit = String::from("[%]");
        UsageMem {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
            mem_total: procfs::meminfo().unwrap().mem_total,
        }
    }
}

impl Column for UsageMem {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let usage = curr_proc.stat.rss_bytes();
        let content = format!("{:.1}", usage as f64 * 100.0 / self.mem_total as f64);

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), content);
    }

    column_default!();
}
