use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Pid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    max_width: usize,
}

impl Pid {
    pub fn new() -> Self {
        let header = String::from("PID");
        let unit = String::from("");
        Pid {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pid();
        let raw_content = pid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(i32);
}

#[cfg(target_os = "macos")]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pbsd.pbi_pid as i32;
        let raw_content = pid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(i32);
}
