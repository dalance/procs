use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Session {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    width: usize,
}

impl Session {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Session"));
        let unit = String::from("");
        Session {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Session {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().session;
        let fmt_content = match proc.curr_proc {
            crate::process::ProcessTask::Process(_) => format!("{}", raw_content),
            _ => format!("[{}]", raw_content),
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}
