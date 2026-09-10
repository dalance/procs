use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(target_os = "macos")]
use crate::process::thread_id;
#[cfg(target_os = "macos")]
use nix::unistd::{self, Pid};
use std::cmp;
use std::collections::HashMap;

pub struct Session {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, i32>,
    width: usize,
}

impl Session {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Session"));
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

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Session {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().session;
        let fmt_content = match proc.curr_proc {
            crate::process::ProcessTask::Process { .. } => format!("{raw_content}"),
            _ => format!("[{raw_content}]"),
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "macos")]
impl Column for Session {
    fn add(&mut self, proc: &ProcessInfo) {
        // A thread row is keyed by its negated thread id, which is 64 bit
        // wide: handing it to `getsid` would truncate it and could report the
        // session of an unrelated process. A session belongs to a process, so
        // the row is left at 0.
        let raw_content = if thread_id(proc.pid).is_some() {
            0
        } else {
            unistd::getsid(Some(Pid::from_raw(proc.pid as i32)))
                .map(|x| x.as_raw())
                .unwrap_or(0)
        };
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "windows")]
impl Column for Session {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.session;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for Session {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.ki_sid;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}
