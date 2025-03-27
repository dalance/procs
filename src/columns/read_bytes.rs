use crate::process::ProcessInfo;
use crate::util::bytify;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct ReadBytes {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl ReadBytes {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Read"));
        let unit = String::from("[B/s]");
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for ReadBytes {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if proc.curr_io.is_some() && proc.prev_io.is_some() {
            let interval_ms = proc.interval.as_secs() + u64::from(proc.interval.subsec_millis());
            let io = (proc.curr_io.as_ref().unwrap().read_bytes
                - proc.prev_io.as_ref().unwrap().read_bytes)
                * 1000
                / interval_ms;
            (bytify(io), io)
        } else {
            (String::new(), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "macos")]
impl Column for ReadBytes {
    fn add(&mut self, proc: &ProcessInfo) {
        let (fmt_content, raw_content) = if proc.curr_res.is_some() && proc.prev_res.is_some() {
            let interval_ms = proc.interval.as_secs() + u64::from(proc.interval.subsec_millis());
            let io = (proc.curr_res.as_ref().unwrap().ri_diskio_bytesread
                - proc.prev_res.as_ref().unwrap().ri_diskio_bytesread)
                * 1000
                / interval_ms;
            (bytify(io), io)
        } else {
            (String::from(""), 0)
        };

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "windows")]
impl Column for ReadBytes {
    fn add(&mut self, proc: &ProcessInfo) {
        let interval_ms = proc.interval.as_secs() + u64::from(proc.interval.subsec_millis());
        let io = (proc.disk_info.curr_read - proc.disk_info.prev_read) * 1000 / interval_ms;

        let raw_content = io;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}

#[cfg(target_os = "freebsd")]
impl Column for ReadBytes {
    fn add(&mut self, proc: &ProcessInfo) {
        // io block size: 128KB
        let block_size = 128 * 1024;
        let interval_ms = proc.interval.as_secs() + u64::from(proc.interval.subsec_millis());
        let io = (proc.curr_proc.info.rusage.inblock as u64
            - proc.prev_proc.info.rusage.inblock as u64)
            * block_size
            * 1000
            / interval_ms;

        let raw_content = io;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}
