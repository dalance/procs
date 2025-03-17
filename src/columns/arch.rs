use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

#[cfg(target_os = "macos")]
use crate::util::arch_from_pid;

pub struct Arch {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

#[cfg(target_os = "macos")]
impl Arch {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Arch"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit
        }
    }
}

#[cfg(target_os = "macos")]
impl Column for Arch {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.pid;
        let arch = arch_from_pid(pid);

        let fmt_content = format!("{}", arch);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}