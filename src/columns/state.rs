use crate::{column_default, Column};
use failure::Error;
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;

pub struct State {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl State {
    pub fn new() -> Self {
        let header = String::from("State");
        State {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
        }
    }
}

impl Column for State {
    fn add(
        &mut self,
        proc: &Process,
        _prev_proc: &Process,
        _prev_io: &ProcResult<Io>,
    ) -> Result<(), Error> {
        let content = format!("{}", proc.stat.state);

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(proc.pid(), String::from(content));
        Ok(())
    }

    column_default!();
}
