use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct UsageCpu {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    max_width: usize,
}

impl UsageCpu {
    pub fn new() -> Self {
        let header = String::from("CPU");
        let unit = String::from("[%]");
        UsageCpu {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for UsageCpu {
    fn add(
        &mut self,
        curr_proc: &Process,
        prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        interval: &Duration,
    ) {
        let curr_time = curr_proc.stat.utime + curr_proc.stat.stime;
        let prev_time = prev_proc.stat.utime + prev_proc.stat.stime;
        let usage_ms =
            (curr_time - prev_time) * 1000 / procfs::ticks_per_second().unwrap_or(100) as u64;
        let interval_ms = interval.as_secs() + u64::from(interval.subsec_millis());
        let usage = usage_ms as f64 * 100.0 / interval_ms as f64;

        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(u32);
}
