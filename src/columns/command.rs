use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Command {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl Command {
    pub fn new() -> Self {
        let header = String::from("Command");
        let unit = String::from("");
        Command {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Command {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pid();
        let fmt_content = if let Ok(cmd) = &proc.curr_proc.cmdline() {
            if !cmd.is_empty() {
                let mut cmd = cmd
                    .iter()
                    .cloned()
                    .map(|mut x| {
                        x.push(' ');
                        x
                    })
                    .collect::<String>();
                cmd.pop();
                cmd = cmd.replace("\n", " ").replace("\t", " ");
                cmd
            } else {
                proc.curr_proc.stat.comm.clone()
            }
        } else {
            proc.curr_proc.stat.comm.clone()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(String);
}

#[cfg(target_os = "macos")]
impl Column for Command {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.curr_proc.pbsd.pbi_pid as i32;
        let fmt_content = format!("{}", String::from_utf8_lossy(&proc.curr_proc.pbsd.pbi_name));
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(pid, fmt_content);
        self.raw_contents.insert(pid, raw_content);
    }

    column_default!(String);
}
