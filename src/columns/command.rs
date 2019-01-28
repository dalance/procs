use crate::{column_default, Column};
use failure::Error;
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------------------------------
// Command
// ---------------------------------------------------------------------------------------------------------------------

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
        proc: &Process,
        _prev_proc: &Process,
        _prev_io: &ProcResult<Io>,
    ) -> Result<(), Error> {
        let content = if let Ok(cmd) = &proc.cmdline() {
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
                proc.stat.comm.clone()
            }
        } else {
            proc.stat.comm.clone()
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(proc.pid(), String::from(content));
        Ok(())
    }

    column_default!();
}
