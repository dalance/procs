use crate::process::ProcessInfo;
#[cfg(target_os = "windows")]
use crate::util::format_sid;
#[cfg(not(target_os = "windows"))]
use crate::util::USERS_CACHE;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
#[cfg(not(target_os = "windows"))]
use uzers::Users;

pub struct User {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl User {
    pub fn new(header: Option<String>, abbr_sid: bool) -> Self {
        let header = header.unwrap_or_else(|| String::from("User"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            abbr_sid,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for User {
    fn add(&mut self, proc: &ProcessInfo) {
        let user = USERS_CACHE.with(|x| x.borrow_mut().get_user_by_uid(proc.curr_proc.owner()));
        let fmt_content = if let Some(user) = user {
            format!("{}", user.name().to_string_lossy())
        } else {
            format!("{}", proc.curr_proc.owner())
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for User {
    fn add(&mut self, proc: &ProcessInfo) {
        let uid = proc.curr_task.pbsd.pbi_uid;
        let fmt_content =
            if let Some(user) = USERS_CACHE.with(|x| x.borrow_mut().get_user_by_uid(uid)) {
                format!("{}", user.name().to_string_lossy())
            } else {
                format!("{}", uid)
            };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for User {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(name) = &proc.user.name {
            name.clone()
        } else {
            format_sid(&proc.user.sid, self.abbr_sid)
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for User {
    fn add(&mut self, proc: &ProcessInfo) {
        let uid = proc.curr_proc.info.uid;
        let fmt_content =
            if let Some(user) = USERS_CACHE.with(|x| x.borrow_mut().get_user_by_uid(uid)) {
                format!("{}", user.name().to_string_lossy())
            } else {
                format!("{}", uid)
            };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
