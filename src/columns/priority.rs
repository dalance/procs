use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Priority {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i64>,
    width: usize,
}

impl Priority {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Priority"));
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
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.stat().priority;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "macos")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_task.ptinfo.pti_priority as i64;
        let fmt_content = format!("{}", raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "windows")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        // User-mode priority class - what the user configured via Task Manager,
        // `STARTUPINFO`, or `SetPriorityClass`. It is queried only when this
        // column is actually displayed: `add` runs once per process for the
        // processes the column is shown for. The kernel Base Priority lives in
        // `proc.priority` and is surfaced by the `RtPriority` column.
        let (raw_content, fmt_content) = match priority_class_of(proc.pid) {
            Some(class) => (class as i64, priority_class_name(class).to_string()),
            // Protected processes (e.g. PPL) refuse the open, so fall back to
            // the kernel priority, which is always available.
            None => (proc.priority as i64, proc.priority.to_string()),
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}

#[cfg(target_os = "windows")]
fn priority_class_of(pid: i32) -> Option<u32> {
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::System::Threading::{
        GetPriorityClass, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    if pid <= 0 {
        return None;
    }

    // SAFETY: `pid` is only handed to `OpenProcess`, and the handle it returns
    // is closed before this function returns.
    let handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid as u32) };
    if handle.is_null() {
        return None;
    }

    let class = unsafe { GetPriorityClass(handle) };
    unsafe { CloseHandle(handle) };

    // A return of 0 means the call failed.
    if class == 0 { None } else { Some(class) }
}

#[cfg(target_os = "windows")]
fn priority_class_name(class: u32) -> &'static str {
    use windows_sys::Win32::System::Threading::{
        ABOVE_NORMAL_PRIORITY_CLASS, BELOW_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS,
        IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, REALTIME_PRIORITY_CLASS,
    };

    match class {
        IDLE_PRIORITY_CLASS => "Idle",
        BELOW_NORMAL_PRIORITY_CLASS => "BelowNormal",
        NORMAL_PRIORITY_CLASS => "Normal",
        ABOVE_NORMAL_PRIORITY_CLASS => "AboveNormal",
        HIGH_PRIORITY_CLASS => "High",
        REALTIME_PRIORITY_CLASS => "Realtime",
        _ => "Unknown",
    }
}

#[cfg(target_os = "freebsd")]
impl Column for Priority {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.curr_proc.info.pri.level as i64 - 100;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i64, true);
}
