use crate::util;
use procfs::{Io, ProcResult, Process};
use std::collections::HashMap;
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

    fn header(&self) -> &str;

    fn unit(&self) -> &str;

    fn contents(&self) -> &HashMap<i32, String>;

    fn max_width(&self) -> usize;

    fn display_header(&self) -> String {
        util::expand(self.header(), self.max_width())
    }

    fn display_unit(&self) -> String {
        util::expand(self.unit(), self.max_width())
    }

    fn display(&self, pid: i32) -> Option<String> {
        if let Some(content) = self.contents().get(&pid) {
            Some(util::expand(content, self.max_width()))
        } else {
            None
        }
    }

    fn find(&self, pid: i32, keyword: &str) -> bool {
        if let Some(content) = self.contents().get(&pid) {
            content.find(keyword).is_some()
        } else {
            false
        }
    }

    fn find_exact(&self, pid: i32, keyword: &str) -> bool {
        if let Some(content) = self.contents().get(&pid) {
            content == keyword
        } else {
            false
        }
    }
}

#[macro_export]
macro_rules! column_default {
    () => {
       fn header(&self) -> &str {
           &self.header
       }
       fn unit(&self) -> &str {
           &self.unit
       }
       fn contents(&self) -> &HashMap<i32, String> {
           &self.contents
       }
       fn max_width(&self) -> usize {
           self.max_width
       }
    };
}
