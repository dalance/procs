use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Tty {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl Tty {
    pub fn new() -> Self {
        let header = String::from("TTY");
        let unit = String::from("");
        Tty {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for Tty {
    fn add(&mut self, proc: &ProcessInfo) {
        let (major, minor) = proc.curr_proc.stat.tty_nr();
        let fmt_content = if major == 136 {
            format!("pts/{}", minor)
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
