use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct WriteBytes {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    max_width: usize,
}

impl WriteBytes {
    pub fn new() -> Self {
        let header = String::from("Write");
        let unit = String::from("[B/s]");
        WriteBytes {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for WriteBytes {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if proc.curr_io.is_ok() && proc.prev_io.is_ok() {
            let interval_ms = proc.interval.as_secs() + u64::from(proc.interval.subsec_millis());
            let io = (proc.curr_io.as_ref().unwrap().write_bytes
                - proc.prev_io.as_ref().unwrap().write_bytes)
                * 1000
                / interval_ms;
            let (size, unit) = unbytify::bytify(io);
            (
                format!("{}{}", size, unit.replace("i", "").replace("B", "")),
                io,
            )
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.curr_proc.pid(), fmt_content);
        self.raw_contents.insert(proc.curr_proc.pid(), raw_content);
    }

    column_default!(u64);
}
