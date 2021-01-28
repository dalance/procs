use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Tty {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl Tty {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("TTY"));
        let unit = String::from("");
        Tty {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Tty {
    fn add(&mut self, proc: &ProcessInfo) {
        let (major, minor) = proc.curr_proc.stat().tty_nr();
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

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for Tty {
    fn add(&mut self, proc: &ProcessInfo) {
        let dev = proc.curr_task.pbsd.e_tdev;
        let major = (dev >> 24) & 0xff;
        let minor = dev & 0xffffff;
        let fmt_content = if major == 16 {
            format!("s{:03}", minor)
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
