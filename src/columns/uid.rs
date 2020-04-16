use crate::process::ProcessInfo;
#[cfg(target_os = "windows")]
use crate::util::format_sid;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Uid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl Uid {
    pub fn new(header: Option<String>, abbr_sid: bool) -> Self {
        let header = header.unwrap_or_else(|| String::from("UID"));
        let unit = String::from("");
        Uid {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            abbr_sid,
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for Uid {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if let Some(ref status) = proc.curr_status {
            let uid = status.euid;
            (format!("{}", uid), uid)
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for Uid {
    fn add(&mut self, proc: &ProcessInfo) {
        let uid = proc.curr_task.pbsd.pbi_uid;
        let fmt_content = format!("{}", uid);
        let raw_content = uid;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32);
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
impl Column for Uid {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = format_sid(&proc.user.sid, self.abbr_sid);
        let raw_content = proc.user.sid[proc.user.sid.len() - 1] as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32);
}
