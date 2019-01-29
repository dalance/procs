use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct State {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl State {
    pub fn new() -> Self {
        let header = String::from("State");
        let unit = String::from("");
        State {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
        }
    }
}

impl Column for State {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) -> () {
        let content = format!("{}", curr_proc.stat.state);

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
