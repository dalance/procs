use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct UidSaved {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    max_width: usize,
}

impl UidSaved {
    pub fn new() -> Self {
        let header = String::from("SUID");
        let unit = String::from("");
        UidSaved {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for UidSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            let uid = status.suid;
            (format!("{}", uid), uid)
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}

#[cfg(target_os = "macos")]
impl Column for UidSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let uid = proc.curr_task.pbsd.pbi_svuid as i32;
        let fmt_content = format!("{}", uid);
        let raw_content = uid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}
