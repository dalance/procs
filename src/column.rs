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
       fn contents(&self) -> &HashMap<i32, String> {
           &self.contents
       }
       fn max_width(&self) -> usize {
           self.max_width
       }
    };
}
