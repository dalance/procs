use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Processor {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    width: usize,
}

impl Processor {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Processor"));
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
impl Column for Processor {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().processor.unwrap_or_default();
        let fmt_content = if let Some(p) = proc.curr_proc.stat().processor {
            format!("{p}")
        } else {
            String::new()
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Processor {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.info.lastcpu;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}
