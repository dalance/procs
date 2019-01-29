use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Tty {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl Tty {
    pub fn new() -> Self {
        let header = String::from("TTY");
        let unit = String::from("");
        Tty {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
        }
    }
}

impl Column for Tty {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) -> () {
        let (major, minor) = curr_proc.stat.tty_nr();
        let content = if major == 136 {
            format!("pts/{}", minor)
        } else {
            String::from("")
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
