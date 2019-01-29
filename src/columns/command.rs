use crate::{column_default, Column, Util};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Command {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    mask: bool,
}

impl Command {
    pub fn new(mask: bool) -> Self {
        let header = String::from("Command");
        let unit = String::from("");
        Command {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
            mask: mask,
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

    fn display(&self, pid: i32) -> Option<String> {
        if let Some(content) = self.contents().get(&pid) {
            let mut ret = format!(
                "{}{}",
                content,
                " ".repeat(self.max_width() - content.len())
            );
            if self.mask {
                ret = Util::mask(&ret);
            }
            Some(ret)
        } else {
            None
        }
    }

    column_default!();
}
