use procs_lib::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::io::Read;

pub struct SecContext {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl SecContext {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Context"));
        let unit = String::new();
        SecContext {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for SecContext {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(proc) = procfs::process::Process::new(proc.pid) {
            if let Ok(mut file) = proc.open_relative("attr/current") {
                let mut ret = String::new();
                let _ = file.read_to_string(&mut ret);
                ret.trim().to_string()
            } else {
                String::from("")
            }
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
