use crate::config;
use config::ConfigSortOrder;
use procfs::{Io, ProcResult, Process};
use std::time::Duration;

pub trait Column {
    fn add(
        &mut self,
        curr_proc: &Process,
        prev_proc: &Process,
        curr_io: &ProcResult<Io>,
        prev_io: &ProcResult<Io>,
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
macro_rules! column_default {
    ($x:ty) => {
        fn display_header(&self) -> String {
            crate::util::expand(&self.header, self.max_width)
        }
        fn display_unit(&self) -> String {
            crate::util::expand(&self.unit, self.max_width)
        }
        fn display_content(&self, pid: i32) -> Option<String> {
            if let Some(content) = self.fmt_contents.get(&pid) {
                Some(crate::util::expand(content, self.max_width))
            } else {
                None
            }
        }
        fn find_partial(&self, pid: i32, keyword: &str) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                content.find(keyword).is_some()
            } else {
                false
            }
        }
        fn find_exact(&self, pid: i32, keyword: &str) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                content == keyword
            } else {
                false
            }
        }
        fn sorted_pid(&self, order: &crate::config::ConfigSortOrder) -> Vec<i32> {
            let mut contents: Vec<(&i32, &$x)> = self.raw_contents.iter().collect();
            contents.sort_by_key(|&(_x, y)| y);
            if let crate::config::ConfigSortOrder::Descending = order { contents.reverse() }
            contents.iter().map(|(x, _y)| **x).collect()
        }
        fn reset_max_width(&mut self) {
            self.max_width = std::cmp::max(self.header.len(), self.unit.len());
        }
        fn update_max_width(&mut self, pid: i32) {
            if let Some(content) = self.fmt_contents.get(&pid) {
                self.max_width = cmp::max(content.len(), self.max_width);
            }
        }
    };
}
