use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

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
            header: header,
            unit: unit,
        }
    }
}

impl Column for Command {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let fmt_content = if let Ok(cmd) = &curr_proc.cmdline() {
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
                cmd = cmd.replace("\n", "  ");
                cmd
            } else {
                curr_proc.stat.comm.clone()
            }
        } else {
            curr_proc.stat.comm.clone()
        };
        let raw_content = fmt_content.clone();

        self.max_width = cmp::max(fmt_content.len(), self.max_width);

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(String);
}
