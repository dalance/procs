use crate::process::ProcessInfo;
#[cfg(not(target_os = "macos"))]
use crate::column_default;
use crate::Column;
use std::cmp;
use std::collections::HashMap;

pub struct Pid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    /// macOS only: the real thread id of the rows that are threads, by row
    /// key. A thread's key is a synthetic one that says nothing, so JSON has
    /// to go through here instead of through `raw_contents`.
    #[cfg(target_os = "macos")]
    thread_ids: HashMap<i32, u64>,
    width: usize,
}

impl Pid {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("PID"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            #[cfg(target_os = "macos")]
            thread_ids: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.pid;
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
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.pid;
        // A thread keeps its real id in `tid`; its `pid` is only a key.
        let fmt_content = match proc.tid {
            Some(tid) => {
                self.thread_ids.insert(proc.pid, tid);
                format!("[{tid}]")
            }
            None => format!("{raw_content}"),
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    fn display_json(&self, pid: i32) -> String {
        let value = match self.thread_ids.get(&pid) {
            Some(tid) => tid.to_string(),
            None => self
                .raw_contents
                .get(&pid)
                .map(|x| x.to_string())
                .unwrap_or_else(|| String::from("")),
        };
        format!("\"{}\": {}", self.header, value)
    }

    crate::column_default_display_header!();
    crate::column_default_display_unit!();
    crate::column_default_display_content!();
    crate::column_default_find_partial!();
    crate::column_default_find_exact!();
    crate::column_default_sorted_pid!(i32);
    crate::column_default_apply_visible!();
    crate::column_default_reset_width!();
    crate::column_default_update_width!();
    crate::column_default_get_width!();
    crate::column_default_is_numeric!(true);
}

#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos")))]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.pid;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}
