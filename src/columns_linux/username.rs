use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

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
            header,
            unit,
        }
    }
}

impl Column for Username {
    fn add(&mut self, proc: &ProcessInfo) {
        let user = users::get_user_by_uid(proc.curr_proc.owner);
        let fmt_content = if let Some(user) = user {
            format!("{}", user.name().to_string_lossy())
        } else {
            format!("{}", proc.curr_proc.owner)
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.curr_proc.pid(), fmt_content);
        self.raw_contents.insert(proc.curr_proc.pid(), raw_content);
    }

    column_default!(String);
}
