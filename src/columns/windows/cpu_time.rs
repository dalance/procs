use crate::process::ProcessInfo;
use crate::{column_default, util, Column};
use std::cmp;
use std::collections::HashMap;

pub struct CpuTime {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    max_width: usize,
}

impl CpuTime {
    pub fn new() -> Self {
        let header = String::from("CPU Time");
        let unit = String::from("");
        CpuTime {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for CpuTime {
    fn add(&mut self, proc: &ProcessInfo) {
        let time_sec = (proc.curr_proc.stat.utime + proc.curr_proc.stat.stime)
            / procfs::ticks_per_second().unwrap_or(100) as u64;

        let fmt_content = util::parse_time(time_sec).to_string();
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
            / 1000000000u64;

        let fmt_content = util::parse_time(time_sec).to_string();
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
        let (fmt_content, raw_content) = if let (Some(kernel), Some(user)) =
            (proc.cpu_info.curr_kernel, proc.cpu_info.curr_user)
        {
            let time_sec = (kernel + user) / 10000000u64;
            (util::parse_time(time_sec).to_string(), time_sec)
        } else {
            (String::default(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}
