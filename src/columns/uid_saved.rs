use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct UidSaved {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    width: usize,
}

impl UidSaved {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("SUID"));
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
impl Column for UidSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            let uid = status.suid;
            (format!("{uid}"), uid)
        } else {
            (String::new(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "macos")]
impl Column for UidSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let uid = proc.curr_task.pbsd.pbi_svuid;
        let fmt_content = format!("{}", uid);
        let raw_content = uid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for UidSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let uid = proc.curr_proc.info.svuid;
        let fmt_content = format!("{}", uid);
        let raw_content = uid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}
