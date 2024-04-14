use procs_lib::ProcessInfo;
use crate::{column_default, util, Column};
use std::cmp;
use std::collections::HashMap;

pub struct CpuTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl CpuTime {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("CPU Time"));
        let unit = String::new();
        CpuTime {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for CpuTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let time_sec = (proc.curr_proc.stat().utime + proc.curr_proc.stat().stime)
            / procfs::ticks_per_second();

        let fmt_content = util::parse_time(time_sec);
        let raw_content = time_sec;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for CpuTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let time_sec = (proc.curr_task.ptinfo.pti_total_user
            + proc.curr_task.ptinfo.pti_total_system)
            / 1_000_000_000u64;

        let fmt_content = util::parse_time(time_sec);
        let raw_content = time_sec;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for CpuTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let time_sec = (proc.cpu_info.curr_sys + proc.cpu_info.curr_user) / 10_000_000u64;

        let fmt_content = util::parse_time(time_sec);
        let raw_content = time_sec;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "freebsd")]
impl Column for CpuTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let time_sec = ((proc.curr_proc.info.rusage.utime.sec * 1_000_000i64
            + proc.curr_proc.info.rusage.utime.usec
            + proc.curr_proc.info.rusage.stime.sec * 1_000_000i64
            + proc.curr_proc.info.rusage.stime.usec)
            / 1_000_000) as u64;

        let fmt_content = util::parse_time(time_sec);
        let raw_content = time_sec;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}
