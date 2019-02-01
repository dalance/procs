use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Username {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl Username {
    pub fn new() -> Self {
        let header = String::from("User");
        let unit = String::from("");
        Username {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
        }
    }
}

impl Column for Username {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) {
        let user = users::get_user_by_uid(curr_proc.owner);
        let fmt_content = if let Some(user) = user {
            format!("{}", user.name().to_string_lossy())
        } else {
            format!("{}", curr_proc.owner)
        };
        let raw_content = fmt_content.clone();

        self.max_width = cmp::max(fmt_content.len(), self.max_width);

        self.fmt_contents.insert(curr_proc.pid(), fmt_content);
        self.raw_contents.insert(curr_proc.pid(), raw_content);
    }

    column_default!(String);
}
