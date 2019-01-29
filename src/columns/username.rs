use crate::{column_default, Column, Util};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Username {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    mask: bool,
}

impl Username {
    pub fn new(mask: bool) -> Self {
        let header = String::from("User");
        let unit = String::from("");
        Username {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
            mask: mask,
        }
    }
}

impl Column for Username {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) -> () {
        let user = users::get_user_by_uid(curr_proc.owner);
        let content = if let Some(user) = user {
            format!("{}", user.name().to_string_lossy())
        } else {
            format!("{}", curr_proc.owner)
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    fn display(&self, pid: i32) -> Option<String> {
        if let Some(content) = self.contents().get(&pid) {
            let mut ret = format!(
                "{}{}",
                content,
                " ".repeat(self.max_width() - content.len())
            );
            if self.mask {
                ret = Util::mask(&ret);
            }
            Some(ret)
        } else {
            None
        }
    }

    column_default!();
}
