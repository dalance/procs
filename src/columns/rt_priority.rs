use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct RtPriority {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    max_width: usize,
}

impl RtPriority {
    pub fn new() -> Self {
        let header = String::from("RT Priority");
        let unit = String::from("");
        RtPriority {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for RtPriority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat.rt_priority.unwrap_or_default();
        let fmt_content = if let Some(p) = proc.curr_proc.stat.rt_priority {
            format!("{}", p)
        } else {
            String::from("")
        };

        self.fmt_contents.insert(proc.curr_proc.pid(), fmt_content);
        self.raw_contents.insert(proc.curr_proc.pid(), raw_content);
    }

    column_default!(u32);
}
