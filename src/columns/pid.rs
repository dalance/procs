use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct Pid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, i32>,
    width: usize,
}

impl Pid {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("PID"));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escapes_custom_header() {
        for header in ["PID", "Process \"ID\"", "Path\\ID", "Process\nID", "Process\tID"] {
            let mut column = Pid::new(Some(header.into()));
            column.raw_contents.insert(201, 201);
            column.fmt_contents.insert(201, "201".into());
            let json = format!("{{{}}}", column.display_json(201));
            let row: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(row[header], 201);
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

#[cfg(not(any(target_os = "linux", target_os = "android")))]
impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        let raw_content = proc.pid;
        let fmt_content = format!("{raw_content}");

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(i32, true);
}
