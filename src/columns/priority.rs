use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process, Status};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Priority {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i64>,
    max_width: usize,
}

impl Priority {
    pub fn new() -> Self {
        let header = String::from("Priority");
        let unit = String::from("");
        Priority {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for Priority {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _curr_status: &ProcResult<Status>,
        _interval: &Duration,
    ) {
        let raw_content = curr_proc.stat.priority;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(i64);
}
