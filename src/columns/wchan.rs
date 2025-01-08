use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Wchan {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl Wchan {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Wchan"));
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
impl Column for Wchan {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.wchan().unwrap_or_default();
        let fmt_content = if raw_content == "0" {
            String::from("-")
        } else {
            raw_content.clone()
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for Wchan {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = if let Ok(wmesg) = crate::util::ptr_to_cstr(&proc.curr_proc.info.wmesg) {
            wmesg.to_string_lossy().into_owned()
        } else {
            String::from("")
        };
        let fmt_content = raw_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
