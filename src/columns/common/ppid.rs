use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Ppid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    max_width: usize,
}

impl Ppid {
    pub fn new() -> Self {
        let header = String::from("Parent PID");
        let unit = String::from("");
        Ppid {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Ppid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat.ppid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for Ppid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.pbsd.pbi_ppid as i32;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for Ppid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.ppid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}
