use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Priority {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i64>,
    max_width: usize,
}

impl Priority {
    pub fn new() -> Self {
        let header = String::from("Priority");
        let unit = String::from("");
        Priority {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat.priority;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64);
}

#[cfg(target_os = "macos")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.ptinfo.pti_priority as i64;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64);
}
