use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct State {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl State {
    pub fn new() -> Self {
        let header = String::from("State");
        let unit = String::from("");
        State {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for State {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pid();
        let fmt_content = format!("{}", proc.curr_proc.stat.state);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(String);
}

#[cfg(target_os = "macos")]
impl Column for State {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pbsd.pbi_pid as i32;
        let state = match proc.bsd.pbi_status {
            1 => "R",
            2 => "U",
            3 => "S",
            4 => "T",
            5 => "I",
            6 => "Z",
            _ => "?",
        };
        let fmt_content = format!("{}", state);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(String);
}
