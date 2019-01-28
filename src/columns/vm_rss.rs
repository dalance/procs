use crate::{column_default, Column};
use failure::Error;
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------------------------------
// VwRSS
// ---------------------------------------------------------------------------------------------------------------------

pub struct VmRSS {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    page_size: u64,
}

impl VmRSS {
    pub fn new() -> Self {
        let header = String::from("RSS");
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
        proc: &Process,
        _prev_proc: &Process,
        _prev_io: &ProcResult<Io>,
    ) -> Result<(), Error> {
        let (size, unit) = unbytify::bytify(proc.stat.rss as u64 * self.page_size);
        let content = format!("{}{}", size, unit.replace("i", "").replace("B", ""));

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(proc.pid(), String::from(content));
        Ok(())
    }

    column_default!();
}
