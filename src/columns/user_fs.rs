use crate::process::ProcessInfo;
use crate::util::USERS_CACHE;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
use uzers::Users;

pub struct UserFs {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl UserFs {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("File System User"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

impl Column for UserFs {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref status) = proc.curr_status {
            let uid = status.fuid;
            if let Some(user) = USERS_CACHE.with(|x| x.borrow_mut().get_user_by_uid(uid)) {
                format!("{}", user.name().to_string_lossy())
            } else {
                format!("{uid}")
            }
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
