use crate::config::ConfigSortOrder;
use crate::process::{ProcessInfo, row_sort_key};
use crate::{Column, column_default_without_sorted_pid};
use std::cmp;
use std::collections::HashMap;

pub struct Pid {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, i64>,
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

impl Column for Pid {
    fn add(&mut self, proc: &ProcessInfo) {
        // The id the row stands for, not the key that carries it: a thread row
        // is keyed by its negated thread id, and the value behind this column -
        // what `--json` reports and what a regex search reads - has to be the
        // thread id itself rather than the internal negation of it.
        //
        // Negating the key cannot overflow: it is either a process id, which
        // is not negative, or a `thread_key`, which is never `i64::MIN`.
        let raw_content = if proc.pid >= 0 { proc.pid } else { -proc.pid };
        // Printed bracketed the way `ps` prints a kernel thread, which is what
        // keeps the id of a thread from reading like the id of a process.
        let fmt_content = if proc.pid >= 0 {
            format!("{raw_content}")
        } else {
            format!("[{raw_content}]")
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    /// Rows are ordered by the id they stand for rather than by the key that
    /// carries it - see [`crate::process::row_sort_key`].
    fn sorted_pid(&self, order: &ConfigSortOrder) -> Vec<i64> {
        let mut contents: Vec<(&i64, &i64)> = self.raw_contents.iter().collect();
        contents.sort_by_key(|&(pid, _)| row_sort_key(*pid));
        if matches!(*order, ConfigSortOrder::Descending) {
            contents.reverse()
        }
        contents.iter().map(|(pid, _)| **pid).collect()
    }

    column_default_without_sorted_pid!(true);
}
