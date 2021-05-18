use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(not(target_os = "windows"))]
use chrono::offset::TimeZone;
#[cfg(target_os = "linux")]
use chrono::DateTime;
use chrono::{Duration, Local};
#[cfg(target_os = "linux")]
use lazy_static::lazy_static;
use std::cmp;
use std::collections::HashMap;

#[cfg(target_os = "linux")]
lazy_static! {
    static ref TICKS_PER_SECOND: i64 = procfs::ticks_per_second().unwrap();
}

pub struct ElapsedTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, Duration>,
    width: usize,
    #[cfg(target_os = "linux")]
    boot_time: DateTime<Local>,
}

impl ElapsedTime {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Elapsed"));
        let unit = String::from("");
        ElapsedTime {
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

fn format_duration(duration: Duration) -> String {
    let years = duration.num_weeks() as f64 / 52.0;
    let weeks = duration.num_days() as f64 / 7.0;
    let days = duration.num_hours() as f64 / 24.0;
    let hours = duration.num_minutes() as f64 / 60.0;
    let minutes = duration.num_seconds() as f64 / 60.0;
    let seconds = duration.num_seconds();

    if years > 1.0 {
        format!("{:.1}years", years)
    } else if weeks > 1.0 {
        format!("{:.1}weeks", weeks)
    } else if days > 1.0 {
        format!("{:.1}days", days)
    } else if hours > 1.0 {
        format!("{:.1}hours", hours)
    } else if minutes > 1.0 {
        format!("{:.1}minutes", minutes)
    } else {
        format!("{:.1}seconds", seconds)
    }
}

#[cfg(target_os = "linux")]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let starttime = proc.curr_proc.stat().starttime;
        let seconds_since_boot = starttime as f32 / *TICKS_PER_SECOND as f32;
        let start_time =
            self.boot_time + Duration::milliseconds((seconds_since_boot * 1000.0) as i64);
        let raw_content = Local::now().signed_duration_since(start_time);
        let fmt_content = format!("{}", format_duration(raw_content));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let start_time = Local.timestamp(proc.curr_task.pbsd.pbi_start_tvsec as i64, 0);
        let raw_content = Local::now().signed_duration_since(start_time);
        let fmt_content = format!("{}", format_duration(raw_content));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for ElapsedTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = Local::now().signed_duration_since(proc.start_time);
        let fmt_content = format!("{}", format_duration(raw_content));

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(Duration);
}
