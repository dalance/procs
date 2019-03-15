use crate::process::ProcessInfo;
#[cfg(target_os = "windows")]
use crate::util::format_sid;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Gid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    max_width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl Gid {
    pub fn new(abbr_sid: bool) -> Self {
        let header = String::from("GID");
        let unit = String::from("");
        Gid {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: 0,
            header,
            unit,
            abbr_sid,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            let gid = status.egid;
            (format!("{}", gid), gid)
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_task.pbsd.pbi_gid as i32;
        let fmt_content = format!("{}", gid);
        let raw_content = gid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for Gid {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut sid = &proc.groups[0].sid;
        let mut kind = std::u64::MAX;
        for g in &proc.groups {
            if g.sid.len() > 3 && g.sid[1] == 5 && g.sid[2] == 32 && kind > g.sid[3] {
                sid = &g.sid;
                kind = g.sid[3];
            }
        }

        let fmt_content = format_sid(&sid, self.abbr_sid);
        let raw_content = sid[sid.len() - 1] as i32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32);
}
