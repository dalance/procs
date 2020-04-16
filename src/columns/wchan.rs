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
        let unit = String::from("");
        Wchan {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

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

    column_default!(String);
}
