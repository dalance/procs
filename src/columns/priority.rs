use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Priority {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i64>,
    width: usize,
}

impl Priority {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Priority"));
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
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().priority;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "macos")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.ptinfo.pti_priority as i64;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "windows")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        // This is the kernel Base Priority, which is not the same as the user-mode priority class.
        // To get the user-mode priority class, we would need GetPriorityClass / NtQueryInformationProcess syscall
        // on the process handle, but that would require additional permissions and is not always available.
        let raw_content = proc.priority as i64;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.info.pri.level as i64 - 100;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}
