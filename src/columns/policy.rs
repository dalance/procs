use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Policy {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl Policy {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Policy"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Policy {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = match proc.curr_proc.stat().policy.map(|x| x as i32) {
            Some(libc::SCHED_BATCH) => String::from("B"),
            Some(libc::SCHED_FIFO) => String::from("FF"),
            Some(libc::SCHED_IDLE) => String::from("IDL"),
            Some(libc::SCHED_OTHER) => String::from("TS"),
            Some(libc::SCHED_RR) => String::from("RR"),
            _ => String::new(),
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "android")]
impl Column for Policy {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = match proc.curr_proc.stat().policy.map(|x| x as i32) {
            Some(libc::SCHED_NORMAL) => String::from("N"),
            Some(libc::SCHED_FIFO) => String::from("FF"),
            Some(libc::SCHED_RR) => String::from("RR"),
            Some(libc::SCHED_BATCH) => String::from("B"),
            Some(libc::SCHED_IDLE) => String::from("IDL"),
            Some(libc::SCHED_DEADLINE) => String::from("D"),
            _ => String::from(""),
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for Policy {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = match proc.curr_task.ptinfo.pti_policy {
            1 => String::from("TS"),
            2 => String::from("RR"),
            4 => String::from("FF"),
            _ => String::from(""),
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
