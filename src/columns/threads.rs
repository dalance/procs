use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Threads {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, i64>,
    width: usize,
}

impl Threads {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Threads"));
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

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Threads {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().num_threads;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "macos")]
impl Column for Threads {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.pti_threadnum as i64;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "windows")]
impl Column for Threads {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = i64::from(proc.thread);
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Threads {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.ki_numthreads as i64;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}
