use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

const UID_NOT_SET: u32 = 0xffffffff;

pub struct UidLogin {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    width: usize,
}

impl UidLogin {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("LoginUID"));
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

impl Column for UidLogin {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Ok(uid) = proc.curr_proc.loginuid() {
            if uid == UID_NOT_SET {
                (String::new(), uid)
            } else {
                (format!("{uid}"), uid)
            }
        } else {
            (String::new(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}
