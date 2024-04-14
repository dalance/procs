use procs_lib::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct TreeSlot {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

#[cfg_attr(tarpaulin, skip)]
impl TreeSlot {
    pub fn new() -> Self {
        let header = String::new();
        let unit = String::new();
        TreeSlot {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg_attr(tarpaulin, skip)]
impl Column for TreeSlot {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = String::new();
        let fmt_content = String::new();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
