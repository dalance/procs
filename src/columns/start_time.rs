use crate::{column_default, Column};
use chrono::{DateTime, Local};
use procfs::{Io, ProcResult, Process, Status};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct StartTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, DateTime<Local>>,
    max_width: usize,
}

impl StartTime {
    pub fn new() -> Self {
        let header = String::from("Start");
        let unit = String::from("");
        StartTime {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for StartTime {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _curr_status: &ProcResult<Status>,
        _interval: &Duration,
    ) {
        let start_time = curr_proc.stat.starttime();
        let raw_content = start_time;
        let fmt_content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(DateTime<Local>);
}
