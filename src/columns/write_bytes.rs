use crate::{column_default, Column};
use failure::Error;
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------------------------------
// WriteBytes
// ---------------------------------------------------------------------------------------------------------------------

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
        proc: &Process,
        _prev_proc: &Process,
        prev_io: &ProcResult<Io>,
    ) -> Result<(), Error> {
        let cur_io = proc.io();
        let content = if cur_io.is_ok() && prev_io.is_ok() {
            let io = (cur_io.unwrap().write_bytes - prev_io.as_ref().unwrap().write_bytes)
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
