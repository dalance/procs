use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Gid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl Gid {
    pub fn new(header: Option<String>, abbr_sid: bool) -> Self {
        let header = header.unwrap_or_else(|| String::from("GID"));
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
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            let gid = status.egid;
            (format!("{gid}"), gid)
        } else {
            (String::new(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "macos")]
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_task.pbsd.pbi_gid;
        let fmt_content = format!("{}", gid);
        let raw_content = gid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "windows")]
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        // A process whose token could not be opened - a protected process, or
        // any process when procs is not elevated - has no group list. Render
        // nothing rather than indexing a non-existent primary group.
        let Some(primary) = proc.groups.first() else {
            self.fmt_contents.insert(proc.pid, String::new());
            self.raw_contents.insert(proc.pid, 0);
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

        let fmt_content = sid.format(self.abbr_sid);
        let raw_content = sid.sub_authorities().last().copied().unwrap_or(0);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_proc.info.svgid;
        let fmt_content = format!("{}", gid);
        let raw_content = gid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}
