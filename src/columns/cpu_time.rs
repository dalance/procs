use crate::{column_default, util, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct CpuTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    max_width: usize,
}

impl CpuTime {
    pub fn new() -> Self {
        let header = String::from("CPU Time");
        let unit = String::from("");
        CpuTime {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for CpuTime {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let time_sec = (curr_proc.stat.utime + curr_proc.stat.stime)
            / procfs::ticks_per_second().unwrap_or(100) as u64;

        let fmt_content = util::parse_time(time_sec).to_string();
        let raw_content = time_sec;

        self.max_width = cmp::max(fmt_content.len(), self.max_width);

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(u64);
}
