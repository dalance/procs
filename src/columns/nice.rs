use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Nice {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i64>,
    width: usize,
}

impl Nice {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Nice"));
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
impl Column for Nice {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().nice;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "macos")]
impl Column for Nice {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.pbsd.pbi_nice as i64;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Nice {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.info.nice as i64;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}
