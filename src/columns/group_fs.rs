use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct GroupFs {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl GroupFs {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("File System Group"));
        let unit = String::from("");
        GroupFs {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

impl Column for GroupFs {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref status) = proc.curr_status {
            let gid = status.fgid;
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
