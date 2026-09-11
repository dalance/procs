use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Pgid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, i32>,
    width: usize,
}

impl Pgid {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("PGID"));
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
impl Column for Pgid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().pgrp;
        // A thread row is bracketed the way its pid is: the value comes from
        // the shared `stat`, and the bracket is what keeps it from reading
        // like the process group of a process.
        let fmt_content = if proc.pid < 0 {
            format!("[{raw_content}]")
        } else {
            format!("{raw_content}")
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "macos")]
impl Column for Pgid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.kp_eproc.e_pgid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Pgid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.ki_pgid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}
