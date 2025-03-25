use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(not(target_os = "windows"))]
use chrono::offset::TimeZone;
#[cfg(any(target_os = "linux", target_os = "android"))]
use chrono::DateTime;
use chrono::{Duration, Local};
#[cfg(any(target_os = "linux", target_os = "android"))]
use once_cell::sync::Lazy;
use std::cmp;
use std::collections::HashMap;

#[cfg(any(target_os = "linux", target_os = "android"))]
static TICKS_PER_SECOND: Lazy<u64> = Lazy::new(procfs::ticks_per_second);

pub struct ElapsedTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, Duration>,
    width: usize,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    boot_time: DateTime<Local>,
}

impl ElapsedTime {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Elapsed"));
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

fn format_duration(duration: Duration) -> String {
    let years = duration.num_weeks() as f64 / 52.0;
    let weeks = duration.num_days() as f64 / 7.0;
    let days = duration.num_hours() as f64 / 24.0;
    let hours = duration.num_minutes() as f64 / 60.0;
    let minutes = duration.num_seconds() as f64 / 60.0;
    let seconds = duration.num_seconds();

    if years > 1.0 {
        format!("{years:.1}years")
    } else if weeks > 1.0 {
        format!("{weeks:.1}weeks")
    } else if days > 1.0 {
        format!("{days:.1}days")
    } else if hours > 1.0 {
        format!("{hours:.1}hours")
    } else if minutes > 1.0 {
        format!("{minutes:.1}minutes")
    } else {
        format!("{seconds:.1}seconds")
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let starttime = proc.curr_proc.stat().starttime;
        let seconds_since_boot = starttime as f32 / *TICKS_PER_SECOND as f32;
        let start_time = self.boot_time
            + Duration::try_milliseconds((seconds_since_boot * 1000.0) as i64).unwrap_or_default();
        let raw_content = Local::now().signed_duration_since(start_time);
        let fmt_content = format_duration(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration, false);
}

#[cfg(target_os = "macos")]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = Local
            .timestamp_opt(proc.curr_task.pbsd.pbi_start_tvsec as i64, 0)
            .unwrap();
        let raw_content = Local::now().signed_duration_since(start_time);
        let fmt_content = format_duration(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration, false);
}

#[cfg(target_os = "windows")]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = Local::now().signed_duration_since(proc.start_time);
        let fmt_content = format_duration(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration, false);
}

#[cfg(target_os = "freebsd")]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = Local
            .timestamp_opt(proc.curr_proc.info.start.sec as i64, 0)
            .unwrap();
        let raw_content = Local::now().signed_duration_since(start_time);
        let fmt_content = format_duration(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration, false);
}
