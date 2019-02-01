use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Pid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    max_width: usize,
}

impl Pid {
    pub fn new() -> Self {
        let header = String::from("PID");
        let unit = String::from("");
        Pid {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
        }
    }
}

impl Column for Pid {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let raw_content = curr_proc.pid();
        let fmt_content = format!("{}", raw_content);

        self.max_width = cmp::max(fmt_content.len(), self.max_width);

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(i32);
}
