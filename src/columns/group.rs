use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Group {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
}

impl Group {
    pub fn new() -> Self {
        let header = String::from("Group");
        let unit = String::from("");
        Group {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
        }
    }
}

impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref status) = proc.curr_status {
            let gid = status.egid;
            if let Some(group) = users::get_group_by_gid(gid as u32) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{}", gid)
            }
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
