use crate::process::ProcessInfo;
#[cfg(not(target_os = "windows"))]
use crate::util::USERS_CACHE;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
#[cfg(not(target_os = "windows"))]
use uzers::Groups;

pub struct Group {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl Group {
    pub fn new(header: Option<String>, abbr_sid: bool) -> Self {
        let header = header.unwrap_or_else(|| String::from("Group"));
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
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref status) = proc.curr_status {
            let gid = status.egid;
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

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_task.pbsd.pbi_gid;
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

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        // A process whose token could not be opened - a protected process, or
        // any process when procs is not elevated - has no group list. Render
        // nothing rather than indexing a non-existent primary group.
        let Some(primary) = proc.groups.first() else {
            self.fmt_contents.insert(proc.pid, String::new());
            self.raw_contents.insert(proc.pid, String::new());
            return;
        };

        let mut sid = primary;
        let mut kind = u64::MAX;
        for g in &proc.groups {
            let subs = g.sub_authorities();
            if g.authority() == 5
                && subs.first() == Some(&32)
                && u64::from(subs.get(1).copied().unwrap_or(u32::MAX)) < kind
            {
                sid = g;
                kind = u64::from(subs[1]);
            }
        }

        let fmt_content = sid
            .display_name()
            .unwrap_or_else(|| sid.format(self.abbr_sid));
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for Group {
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

    column_default!(String, false);
}
