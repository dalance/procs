use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Ssb {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl Ssb {
    pub fn new() -> Self {
        let header = String::from("Speculative Store Bypass");
        let unit = String::from("");
        Ssb {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: 0,
            header,
            unit,
        }
    }
}

impl Column for Ssb {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref curr_status) = proc.curr_status {
            if let Some(ref val) = curr_status.speculation_store_bypass {
                val.clone()
            } else {
                String::from("")
            }
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
