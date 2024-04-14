use procs_lib::ProcessInfo;
use crate::util::USERS_CACHE;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
use uzers::Groups;

pub struct GroupSaved {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl GroupSaved {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Saved Group"));
        let unit = String::new();
        GroupSaved {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for GroupSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref status) = proc.curr_status {
            let gid = status.sgid;
            if let Some(group) = USERS_CACHE.with(|x| x.borrow_mut().get_group_by_gid(gid)) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{gid}")
            }
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for GroupSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_task.pbsd.pbi_svgid;
        let fmt_content =
            if let Some(group) = USERS_CACHE.with(|x| x.borrow_mut().get_group_by_gid(gid)) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{}", gid)
            };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}

#[cfg(target_os = "freebsd")]
impl Column for GroupSaved {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_proc.info.svgid;
        let fmt_content =
            if let Some(group) = USERS_CACHE.with(|x| x.borrow_mut().get_group_by_gid(gid)) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{gid}")
            };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
