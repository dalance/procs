use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct VmRss {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    max_width: usize,
}

impl VmRss {
    pub fn new() -> Self {
        let header = String::from("VmRSS");
        let unit = String::from("[bytes]");
        VmRss {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for VmRss {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat.rss_bytes() as u64;
        let (size, unit) = unbytify::bytify(raw_content);
        let fmt_content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.fmt_contents.insert(proc.curr_proc.pid(), fmt_content);
        self.raw_contents.insert(proc.curr_proc.pid(), raw_content);
    }

    column_default!(u64);
}
