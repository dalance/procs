use crate::process::ProcessInfo;
use crate::util::bytify;
use crate::{Column, column_default};
use std::cmp;
use std::collections::HashMap;

pub struct InvoluntaryContextSw {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, u64>,
    width: usize,
}

impl InvoluntaryContextSw {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("InvoluntaryContextSw"));
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

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for InvoluntaryContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            if let Some(nonvoluntary_ctxt_switches) = status.nonvoluntary_ctxt_switches {
                let sw = nonvoluntary_ctxt_switches;
                (bytify(sw), sw)
            } else {
                (String::new(), 0)
            }
        } else {
            (String::new(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for InvoluntaryContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.ki_rusage.ru_nivcsw as u64;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}
