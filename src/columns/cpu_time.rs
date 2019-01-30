use crate::{column_default, util, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct CpuTime {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl CpuTime {
    pub fn new() -> Self {
        let header = String::from("CPU Time");
        let unit = String::from("");
        CpuTime {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
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
    ) -> () {
        let time_sec = (curr_proc.stat.utime + curr_proc.stat.stime)
            / procfs::ticks_per_second().unwrap_or(100) as u64;

        let content = format!("{}", util::parse_time(time_sec));

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
