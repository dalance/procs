use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Separator {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    separator: String,
}

impl Separator {
    pub fn new(separator: &str) -> Self {
        let header = String::from(separator);
        let unit = String::from(separator);
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            separator: String::from(separator),
        }
    }
}

impl Column for Separator {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = self.separator.clone();
        let fmt_content = self.separator.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    fn sortable(&self) -> bool {
        false
    }

    column_default!(String, false);
}
