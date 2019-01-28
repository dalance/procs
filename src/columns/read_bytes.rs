use crate::{column_default, Column};
use failure::Error;
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------------------------------
// ReadBytes
// ---------------------------------------------------------------------------------------------------------------------

pub struct ReadBytes {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl ReadBytes {
    pub fn new() -> Self {
        let header = String::from("Read");
        ReadBytes {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
        }
    }
}

impl Column for ReadBytes {
    fn add(
        &mut self,
        proc: &Process,
        _prev_proc: &Process,
        prev_io: &ProcResult<Io>,
    ) -> Result<(), Error> {
        let cur_io = proc.io();
        let content = if cur_io.is_ok() && prev_io.is_ok() {
            let io = (cur_io.unwrap().read_bytes - prev_io.as_ref().unwrap().read_bytes)
                * crate::RATIO_TO_SECOND;
            let (size, unit) = unbytify::bytify(io);
            format!("{}{}", size, unit.replace("i", "").replace("B", ""))
        } else {
            String::from("")
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(proc.pid(), String::from(content));
        Ok(())
    }

    column_default!();
}
