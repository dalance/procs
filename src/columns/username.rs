use crate::{column_default, Column};
use failure::Error;
use procfs::{Io, ProcResult, Process};
use std::cmp;
use std::collections::HashMap;

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
        proc: &Process,
        _prev_proc: &Process,
        _prev_io: &ProcResult<Io>,
    ) -> Result<(), Error> {
        let user = users::get_user_by_uid(proc.owner);
        let content = if let Some(user) = user {
            format!("{}", user.name().to_string_lossy())
        } else {
            format!("{}", proc.owner)
        };

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(proc.pid(), String::from(content));
        Ok(())
    }

    column_default!();
}
