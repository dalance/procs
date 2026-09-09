use crate::process::ProcessInfo;
use crate::util::bytify;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct ContextSw {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl ContextSw {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("ContextSw"));
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
impl Column for ContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            if let Some(voluntary_ctxt_switches) = status.voluntary_ctxt_switches
                && let Some(nonvoluntary_ctxt_switches) = status.nonvoluntary_ctxt_switches
            {
                let sw = voluntary_ctxt_switches
                    + nonvoluntary_ctxt_switches;
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

#[cfg(target_os = "macos")]
impl Column for ContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.pti_csw as u64;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for ContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content =
            (proc.curr_proc.info.rusage.nvcsw + proc.curr_proc.info.rusage.nivcsw) as u64;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}
