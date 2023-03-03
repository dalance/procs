use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct WorkDir {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl WorkDir {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("WorkDir"));
        let unit = String::new();
        WorkDir {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for WorkDir {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(proc) = procfs::process::Process::new(proc.pid) {
            if let Ok(path) = proc.cwd() {
                path.to_string_lossy().to_string()
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
