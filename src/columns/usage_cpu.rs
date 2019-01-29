use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct UsageCPU {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl UsageCPU {
    pub fn new() -> Self {
        let header = String::from("CPU");
        UsageCPU {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
        }
    }
}

impl Column for UsageCPU {
    fn add(
        &mut self,
        curr_proc: &Process,
        prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        interval: &Duration,
    ) -> () {
        let curr_time = curr_proc.stat.utime + curr_proc.stat.stime;
        let prev_time = prev_proc.stat.utime + prev_proc.stat.stime;
        let usage_ms =
            (curr_time - prev_time) * 1000 / procfs::ticks_per_second().unwrap_or(100) as u64;
        let interval_ms = interval.as_secs() + interval.subsec_millis() as u64;

        let content = format!("{:.1}%", usage_ms as f64 * 100.0 / interval_ms as f64);

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
