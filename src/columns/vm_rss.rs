use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct VmRSS {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    page_size: u64,
}

impl VmRSS {
    pub fn new() -> Self {
        let header = String::from("RSS[bytes]");
        VmRSS {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
            page_size: page_size::get() as u64,
        }
    }
}

impl Column for VmRSS {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) -> () {
        let (size, unit) = unbytify::bytify(curr_proc.stat.rss as u64 * self.page_size);
        let content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    column_default!();
}
