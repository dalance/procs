use crate::{column_default, Column};
use chrono::{DateTime, Local};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct StartTime {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    boot_time: DateTime<Local>,
}

impl StartTime {
    pub fn new() -> Self {
        let header = String::from("Start");
        StartTime {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
            boot_time: procfs::boot_time().unwrap(),
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
    ) -> () {
        let start_time = chrono::Duration::seconds(
            curr_proc.stat.starttime / procfs::ticks_per_second().unwrap_or(100),
        );
        let start_time = self
            .boot_time
            .checked_add_signed(start_time)
            .unwrap_or(self.boot_time);
        let content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
