use crate::config;
use config::ConfigSortOrder;
use procfs::{Io, ProcResult, Process, Status};
use std::time::Duration;

pub trait Column {
    fn add(
        &mut self,
        curr_proc: &Process,
        prev_proc: &Process,
        curr_io: &ProcResult<Io>,
        prev_io: &ProcResult<Io>,
        curr_status: &ProcResult<Status>,
        interval: &Duration,
    ) -> ();

    fn available(&self) -> bool {
        true
    }

    fn display_header(&self) -> String;
    fn display_unit(&self) -> String;
    fn display_content(&self, pid: i32) -> Option<String>;
    fn find_partial(&self, pid: i32, keyword: &str) -> bool;
    fn find_exact(&self, pid: i32, keyword: &str) -> bool;
    fn sorted_pid(&self, order: &ConfigSortOrder) -> Vec<i32>;
    fn reset_max_width(&mut self) -> ();
    fn update_max_width(&mut self, pid: i32) -> ();
}

#[macro_export]
macro_rules! column_default_display_header {
    () => {
        fn display_header(&self) -> String {
            crate::util::expand(&self.header, self.max_width)
        }
    };
}

#[macro_export]
macro_rules! column_default_display_unit {
    () => {
        fn display_unit(&self) -> String {
            crate::util::expand(&self.unit, self.max_width)
        }
    };
}

#[macro_export]
macro_rules! column_default_display_content {
    () => {
        fn display_content(&self, pid: i32) -> Option<String> {
            if let Some(content) = self.fmt_contents.get(&pid) {
                Some(crate::util::expand(content, self.max_width))
            } else {
                None
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_find_partial {
    () => {
        fn find_partial(&self, pid: i32, keyword: &str) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                content.find(keyword).is_some()
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_find_exact {
    () => {
        fn find_exact(&self, pid: i32, keyword: &str) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                content == keyword
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_sorted_pid {
    ($x:ty) => {
        fn sorted_pid(&self, order: &crate::config::ConfigSortOrder) -> Vec<i32> {
            let mut contents: Vec<(&i32, &$x)> = self.raw_contents.iter().collect();
            contents.sort_by_key(|&(_x, y)| y);
            if let crate::config::ConfigSortOrder::Descending = order { contents.reverse() }
            contents.iter().map(|(x, _y)| **x).collect()
        }
    };
}

#[macro_export]
macro_rules! column_default_reset_max_width {
    () => {
        fn reset_max_width(&mut self) {
            self.max_width = std::cmp::max(self.header.len(), self.unit.len());
        }
    };
}

#[macro_export]
macro_rules! column_default_update_max_width {
    () => {
        fn update_max_width(&mut self, pid: i32) {
            if let Some(content) = self.fmt_contents.get(&pid) {
                self.max_width = cmp::max(content.len(), self.max_width);
            }
        }
    };
}

#[macro_export]
macro_rules! column_default {
    ($x:ty) => {
        crate::column_default_display_header!();
        crate::column_default_display_unit!();
        crate::column_default_display_content!();
        crate::column_default_find_partial!();
        crate::column_default_find_exact!();
        crate::column_default_sorted_pid!($x);
        crate::column_default_reset_max_width!();
        crate::column_default_update_max_width!();
    };
}
