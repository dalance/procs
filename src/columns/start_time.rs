use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(not(target_os = "windows"))]
use chrono::offset::TimeZone;
#[cfg(any(target_os = "linux", target_os = "android"))]
use chrono::Duration;
use chrono::{DateTime, Local};
#[cfg(any(target_os = "linux", target_os = "android"))]
use once_cell::sync::Lazy;
use std::cmp;
use std::collections::HashMap;

#[cfg(any(target_os = "linux", target_os = "android"))]
static TICKS_PER_SECOND: Lazy<u64> = Lazy::new(procfs::ticks_per_second);

pub struct StartTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, DateTime<Local>>,
    width: usize,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    boot_time: DateTime<Local>,
}

impl StartTime {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Start"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            #[cfg(any(target_os = "linux", target_os = "android"))]
            boot_time: procfs::boot_time().unwrap_or_else(|_| Local.timestamp_opt(0, 0).unwrap()),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let starttime = proc.curr_proc.stat().starttime;
        let seconds_since_boot = starttime as f32 / *TICKS_PER_SECOND as f32;
        let raw_content = self.boot_time
            + Duration::try_milliseconds((seconds_since_boot * 1000.0) as i64).unwrap_or_default();
        let fmt_content = format!("{}", raw_content.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>, false);
}

#[cfg(target_os = "macos")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = Local
            .timestamp_opt(proc.curr_task.pbsd.pbi_start_tvsec as i64, 0)
            .unwrap();
        let raw_content = start_time;
        let fmt_content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>, false);
}

#[cfg(target_os = "windows")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.start_time;
        let fmt_content = format!("{}", proc.start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>, false);
}

#[cfg(target_os = "freebsd")]
impl Column for StartTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = Local
            .timestamp_opt(proc.curr_proc.info.start.sec as i64, 0)
            .unwrap();
        let raw_content = start_time;
        let fmt_content = format!("{}", start_time.format("%Y/%m/%d %H:%M"));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(DateTime<Local>, false);
}
