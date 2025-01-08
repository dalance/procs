use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct FileName {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl FileName {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("FileName"));
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
impl Column for FileName {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().comm.clone();
        let fmt_content = raw_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for FileName {
    fn add(&mut self, proc: &ProcessInfo) {
        let comm = crate::util::ptr_to_cstr(proc.curr_proc.info.comm.as_ref());
        let comm = if let Ok(comm) = comm {
            comm.to_string_lossy().into_owned()
        } else {
            String::from("")
        };
        let fmt_content = comm;
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
