use crate::process::ProcessInfo;
#[cfg(target_os = "windows")]
use crate::util::format_sid;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Group {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl Group {
    pub fn new(abbr_sid: bool) -> Self {
        let header = String::from("Group");
        let unit = String::from("");
        Group {
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

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_task.pbsd.pbi_gid;
        let fmt_content = if let Some(group) = users::get_group_by_gid(gid) {
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

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut sid_name = &proc.groups[0];
        let mut kind = std::u64::MAX;
        for g in &proc.groups {
            if g.sid.len() > 3 && g.sid[1] == 5 && g.sid[2] == 32 && kind > g.sid[3] {
                sid_name = &g;
                kind = g.sid[3];
            }
        }

        let fmt_content = if let Some(name) = &sid_name.name {
            name.clone()
        } else {
            format_sid(&sid_name.sid, self.abbr_sid)
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String);
}
