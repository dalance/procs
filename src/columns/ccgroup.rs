use crate::process::ProcessInfo;
use crate::{column_default, Column};
use regex::Regex;
use std::cmp;
use std::collections::HashMap;

pub struct Ccgroup {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    pat_user: Regex,
    pat_machine: Regex,
    pat_lxc_monitor: Regex,
    pat_lxc_payload: Regex,
    pat_scope: Regex,
    pat_service: Regex,
    pat_slice: Regex,
}

impl Ccgroup {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Cgroup (compressed)"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            pat_user: Regex::new(r"/user-([^/]*)\.slice").unwrap(),
            pat_machine: Regex::new(r"/machine-([^/]*)\.scope").unwrap(),
            pat_lxc_monitor: Regex::new(r"/lxc\.monitor\.([^/]*)").unwrap(),
            pat_lxc_payload: Regex::new(r"/lxc\.payload\.([^/]*)").unwrap(),
            pat_scope: Regex::new(r"/([^/]*)\.scope").unwrap(),
            pat_service: Regex::new(r"/([^/]*)\.service").unwrap(),
            pat_slice: Regex::new(r"/([^/]*)\.slice").unwrap(),
        }
    }
}

macro_rules! replace {
    ( $x: ident, $pat: expr, $fmt: literal) => {
        if let Some(x) = $pat.captures(&$x) {
            $pat.replace(&$x, &format!($fmt, &x[1])).to_string()
        } else {
            $x
        }
    };
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Ccgroup {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(cgroups) = &proc.curr_proc.cgroups() {
            let name = cgroups
                .last()
                .map_or_else(|| "".to_string(), |x| x.pathname.to_string());
            let name = name.replace("/system.slice", "/[S]");
            let name = name.replace("/user.slice", "/[U]");
            let name = replace!(name, self.pat_user, "/[U:{}]");
            let name = name.replace("/machine.slice", "/[M]");
            let name = replace!(name, self.pat_machine, "/[SNC:{}]");
            let name = replace!(name, self.pat_lxc_monitor, "/[LXC:{}]");
            let name = replace!(name, self.pat_lxc_payload, "/[lxc:{}]");
            let name = replace!(name, self.pat_scope, "/!{}");
            let name = replace!(name, self.pat_service, "/{}");
            replace!(name, self.pat_slice, "/[{}]")
        } else {
            "".to_string()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
