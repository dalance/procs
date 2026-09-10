use crate::config::ConfigSortOrder;
use crate::process::{ProcessInfo, row_sort_key, thread_id};
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
        let raw_content = proc.pid;
        // A thread row carries its thread id negated, and is printed
        // bracketed the way `ps` prints a kernel thread - which is what keeps
        // the id of a thread from reading like the id of a process.
        let fmt_content = match thread_id(proc.pid) {
            Some(tid) => format!("[{tid}]"),
            None => format!("{raw_content}"),
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
