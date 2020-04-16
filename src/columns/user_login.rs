use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct UserLogin {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl UserLogin {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Login User"));
        let unit = String::from("");
        UserLogin {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

impl Column for UserLogin {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(uid) = proc.curr_proc.loginuid() {
            if let Some(user) = users::get_user_by_uid(uid as u32) {
                format!("{}", user.name().to_string_lossy())
            } else {
                format!("{}", uid)
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
