use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct WriteBytes {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl WriteBytes {
    pub fn new() -> Self {
        let header = String::from("Write");
        WriteBytes {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
        }
    }
}

impl Column for WriteBytes {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        curr_io: &ProcResult<Io>,
        prev_io: &ProcResult<Io>,
        interval: &Duration,
    ) -> () {
        let content = if curr_io.is_ok() && prev_io.is_ok() {
            let interval_ms = interval.as_secs() + interval.subsec_millis() as u64;
            let io = (curr_io.as_ref().unwrap().write_bytes
                - prev_io.as_ref().unwrap().write_bytes)
                * 1000
                / interval_ms;
            let (size, unit) = unbytify::bytify(io);
            format!("{}{}", size, unit.replace("i", "").replace("B", ""))
        } else {
            String::from("")
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
