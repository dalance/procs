use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Slot {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl Slot {
    pub fn new() -> Self {
        let header = String::from("");
        let unit = String::from("");
        Slot {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: 0,
            header,
            unit,
        }
    }
}

impl Column for Slot {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = "".to_string();
        let fmt_content = "".to_string();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
