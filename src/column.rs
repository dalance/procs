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

    fn visible(&self) -> bool;

    fn header(&self) -> &str;

    fn unit(&self) -> &str;

    fn contents(&self) -> &HashMap<i32, String>;

    fn max_width(&self) -> usize;

    fn display_header(&self) -> String {
        if !self.visible() {
            String::from("")
        } else {
            let ret = format!(
                "{}{}",
                self.header(),
                " ".repeat(self.max_width() - self.header().len())
            );
            ret
        }
    }

    fn display_unit(&self) -> String {
        if !self.visible() {
            String::from("")
        } else {
            let ret = format!(
                "{}{}",
                self.unit(),
                " ".repeat(self.max_width() - self.unit().len())
            );
            ret
        }
    }

    fn display(&self, pid: i32) -> Option<String> {
        if !self.visible() {
            Some(String::from(""))
        } else if let Some(content) = self.contents().get(&pid) {
            let ret = format!(
                "{}{}",
                content,
                " ".repeat(self.max_width() - content.len())
            );
            Some(ret)
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
       fn visible(&self) -> bool {
           self.visible
       }
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
