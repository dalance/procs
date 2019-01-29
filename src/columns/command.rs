use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Command {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl Command {
    pub fn new() -> Self {
        let header = String::from("Command");
        Command {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
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
    ) -> () {
        let content = if let Ok(cmd) = &curr_proc.cmdline() {
            if cmd.len() != 0 {
                let mut cmd = cmd
                    .iter()
                    .cloned()
                    .map(|mut x| {
                        x.push(' ');
                        x
                    })
                    .collect::<String>();
                cmd.pop();
                cmd
            } else {
                curr_proc.stat.comm.clone()
            }
        } else {
            curr_proc.stat.comm.clone()
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
