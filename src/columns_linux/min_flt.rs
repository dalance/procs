use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct MinFlt {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    max_width: usize,
}

impl MinFlt {
    pub fn new() -> Self {
        let header = String::from("MinorFaults");
        let unit = String::from("");
        MinFlt {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for MinFlt {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat.minflt;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.curr_proc.pid(), fmt_content);
        self.raw_contents.insert(proc.curr_proc.pid(), raw_content);
    }

    column_default!(u64);
}
