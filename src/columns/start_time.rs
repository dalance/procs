use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct StartTime {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl StartTime {
    pub fn new() -> Self {
        let header = String::from("Start");
        let unit = String::from("");
        StartTime {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
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
        _interval: &Duration,
    ) {
        let start_time = curr_proc.stat.starttime();
        let content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), content);
    }

    column_default!();
}
