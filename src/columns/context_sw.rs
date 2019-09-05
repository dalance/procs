use crate::process::ProcessInfo;
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
    pub fn new() -> Self {
        let header = String::from("ContextSw");
        let unit = String::from("");
        ContextSw {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for ContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            if status.voluntary_ctxt_switches.is_some()
                && status.nonvoluntary_ctxt_switches.is_some()
            {
                let sw = status.voluntary_ctxt_switches.unwrap()
                    + status.nonvoluntary_ctxt_switches.unwrap();
                let (size, unit) = unbytify::bytify(sw);
                (
                    format!("{}{}", size, unit.replace("i", "").replace("B", "")),
                    sw,
                )
            } else {
                (String::from(""), 0)
            }
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for ContextSw {
    fn add(&mut self, proc: &ProcessInfo) {
        let sw = proc.curr_task.ptinfo.pti_csw as u64;
        let (size, unit) = unbytify::bytify(sw);
        let fmt_content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));
        let raw_content = sw;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64);
}
