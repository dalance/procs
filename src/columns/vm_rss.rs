use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct VmRss {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl VmRss {
    pub fn new() -> Self {
        let header = String::from("RSS");
        let unit = String::from("[bytes]");
        VmRss {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
        }
    }
}

impl Column for VmRss {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let (size, unit) = unbytify::bytify(curr_proc.stat.rss_bytes() as u64);
        let content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), content);
    }

    column_default!();
}
