use procs_lib::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Pid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    width: usize,
}

impl Pid {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("PID"));
        let unit = String::new();
        Pid {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.pid;
        let fmt_content = match proc.curr_proc {
            procs_lib::ProcessTask::Process { .. } => format!("{raw_content}"),
            _ => format!("[{raw_content}]"),
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}

#[cfg(not(any(target_os = "linux", target_os = "android")))]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.pid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}
