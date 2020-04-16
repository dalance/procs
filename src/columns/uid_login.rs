use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

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
        let unit = String::from("");
        UidLogin {
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
            (format!("{}", uid), uid)
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32);
}
