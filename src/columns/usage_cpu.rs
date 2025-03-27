use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct UsageCpu {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    width: usize,
}

impl UsageCpu {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("CPU"));
        let unit = String::from("[%]");
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for UsageCpu {
    fn add(&mut self, proc: &ProcessInfo) {
        let curr_stat = proc.curr_proc.stat();
        let prev_stat = &proc.prev_stat;

        let curr_time = curr_stat.utime + curr_stat.stime;
        let prev_time = prev_stat.utime + prev_stat.stime;
        let usage_ms = (curr_time - prev_time) * 1000 / procfs::ticks_per_second();
        let interval_ms = proc.interval.as_secs() * 1000 + u64::from(proc.interval.subsec_millis());
        let usage = usage_ms as f64 * 100.0 / interval_ms as f64;

        let fmt_content = format!("{usage:.1}");
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "macos")]
impl Column for UsageCpu {
    fn add(&mut self, proc: &ProcessInfo) {
        let curr_time =
            proc.curr_task.ptinfo.pti_total_user + proc.curr_task.ptinfo.pti_total_system;
        let prev_time =
            proc.prev_task.ptinfo.pti_total_user + proc.prev_task.ptinfo.pti_total_system;
        let usage_ms = (curr_time - prev_time) / 1000000u64;
        let interval_ms = proc.interval.as_secs() * 1000 + u64::from(proc.interval.subsec_millis());
        let usage = usage_ms as f64 * 100.0 / interval_ms as f64;

        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "windows")]
impl Column for UsageCpu {
    fn add(&mut self, proc: &ProcessInfo) {
        let curr_time = proc.cpu_info.curr_sys + proc.cpu_info.curr_user;
        let prev_time = proc.cpu_info.prev_sys + proc.cpu_info.prev_user;

        let usage_ms = (curr_time - prev_time) / 10000u64;
        let interval_ms = proc.interval.as_secs() * 1000 + u64::from(proc.interval.subsec_millis());
        let usage = usage_ms as f64 * 100.0 / interval_ms as f64;

        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for UsageCpu {
    fn add(&mut self, proc: &ProcessInfo) {
        let curr_time = (proc.curr_proc.info.rusage.utime.to_us()
            + proc.curr_proc.info.rusage.stime.to_us()) as u64;
        let prev_time = (proc.prev_proc.info.rusage.utime.to_us()
            + proc.prev_proc.info.rusage.stime.to_us()) as u64;
        let usage_ms = (curr_time - prev_time) / 1_000u64;
        let interval_ms = proc.interval.as_secs() * 1000 + u64::from(proc.interval.subsec_millis());
        let usage = usage_ms as f64 * 100.0 / interval_ms as f64;

        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}
