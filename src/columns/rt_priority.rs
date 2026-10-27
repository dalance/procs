use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct RtPriority {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, u32>,
    width: usize,
}

impl RtPriority {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("RT Priority"));
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
impl Column for RtPriority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().rt_priority.unwrap_or_default();
        let fmt_content = if let Some(p) = proc.curr_proc.stat().rt_priority {
            format!("{p}")
        } else {
            String::new()
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "windows")]
impl Column for RtPriority {
    fn add(&mut self, proc: &ProcessInfo) {
        // Windows has nothing like the POSIX real-time priority. This column
        // shows the kernel Base Priority (0-31), the scheduling priority the
        // kernel actually uses - the "real" priority of the process.
        let raw_content = proc.priority as u32;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}
