use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(not(target_os = "windows"))]
use chrono::offset::TimeZone;
#[cfg(target_os = "linux")]
use chrono::Duration;
use chrono::{DateTime, Local};
#[cfg(target_os = "linux")]
use lazy_static::lazy_static;
use std::cmp;
use std::collections::HashMap;

#[cfg(target_os = "linux")]
lazy_static! {
    static ref TICKS_PER_SECOND: i64 = procfs::ticks_per_second().unwrap();
}

pub struct StartTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, DateTime<Local>>,
    width: usize,
    #[cfg(target_os = "linux")]
    boot_time: DateTime<Local>,
}

impl StartTime {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Start"));
        let unit = String::from("");
        StartTime {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            #[cfg(target_os = "linux")]
            boot_time: procfs::boot_time().unwrap_or_else(|_| Local.timestamp(0, 0)),
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let starttime = proc.curr_proc.stat().starttime;
        let seconds_since_boot = starttime as f32 / *TICKS_PER_SECOND as f32;
        let raw_content =
            self.boot_time + Duration::milliseconds((seconds_since_boot * 1000.0) as i64);
        let fmt_content = format!("{}", raw_content.format("%Y/%m/%d %H:%M"));

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
