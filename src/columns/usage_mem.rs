use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct UsageMem {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    mem_total: u64,
}

impl UsageMem {
    pub fn new() -> Self {
        let header = String::from("MEM");
        UsageMem {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
            mem_total: procfs::meminfo().unwrap().mem_total,
        }
    }
}

impl Column for UsageMem {
    fn add(
        &mut self,
        curr_proc: &Process,
        prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        interval: &Duration,
    ) -> () {
        let usage = curr_proc.stat.rss_bytes();
        let content = format!("{:.1}%", usage as f64 * 100.0 / self.mem_total as f64);

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
