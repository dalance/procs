use crate::{column_default, Column};
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct Username {
    pub visible: bool,
    header: String,
    contents: HashMap<i32, String>,
    max_width: usize,
}

impl Username {
    pub fn new() -> Self {
        let header = String::from("User");
        Username {
            visible: true,
            contents: HashMap::new(),
            max_width: header.len(),
            header: header,
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

    column_default!();
}
