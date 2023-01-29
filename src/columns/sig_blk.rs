use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct SigBlk {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl SigBlk {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("SigBlk"));
        let unit = String::new();
        SigBlk {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

impl Column for SigBlk {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            let val = status.sigblk;
            (format!("{val:016x}"), val)
        } else {
            (String::new(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}
