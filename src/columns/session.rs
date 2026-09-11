use crate::process::ProcessInfo;
use crate::{column_default, Column};
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
        // A thread row is bracketed the way its pid is: the value comes from
        // the shared `stat`, and the bracket is what keeps it from reading
        // like the session of a process.
        let fmt_content = if proc.pid < 0 {
            format!("[{raw_content}]")
        } else {
            format!("{raw_content}")
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}

#[cfg(target_os = "macos")]
impl Column for Session {
    fn add(&mut self, proc: &ProcessInfo) {
        // A session belongs to a process, and a thread row is keyed by its
        // negated thread id, which is 64 bit wide: handing it to `getsid`
        // would truncate it and could report the session of an unrelated
        // process. So the row is left at 0.
        let raw_content = if proc.pid < 0{
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
