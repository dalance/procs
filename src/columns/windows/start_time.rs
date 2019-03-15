use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(target_os = "macos")]
use chrono::offset::TimeZone;
use chrono::{DateTime, Local};
use std::cmp;
use std::collections::HashMap;

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
            max_width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = proc.curr_proc.stat.starttime();
        let raw_content = start_time;
        let fmt_content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = Local.timestamp(proc.curr_task.pbsd.pbi_start_tvsec as i64, 0);
        let raw_content = start_time;
        let fmt_content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.start_time;
        let fmt_content = format!("{}", proc.start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>);
}
