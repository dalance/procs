use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Env {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[allow(dead_code)]
    procfs: Option<PathBuf>,
}

impl Env {
    pub fn new(header: Option<String>, procfs: Option<PathBuf>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Env"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            procfs,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Env {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut fmt_content = String::new();
        if let Ok(proc) = crate::util::process_new(proc.pid, &self.procfs) {
            if let Ok(envs) = proc.environ() {
                for (k, v) in envs {
                    fmt_content.push_str(&format!(
                        "{}=\"{}\" ",
                        k.to_string_lossy(),
                        v.to_string_lossy().replace('\"', "\\\"")
                    ));
                }
            }
        }
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for Env {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut fmt_content = String::new();
        for env in &proc.curr_proc.env {
            fmt_content.push_str(&format!("{} ", env.replace('\"', "\\\"")));
        }
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
