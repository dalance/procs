use crate::process::ProcessInfo;
use crate::util::bytify;
use crate::{Column, column_default};
use std::cmp;
use std::collections::HashMap;

/// Bytes received per second.
///
/// Windows only: the counters come from `ProcessNetworkIoInformation`
/// (`PROCESS_NETWORK_COUNTERS::BytesIn`), so the column is declared by
/// `os_windows.rs` alone. It reads a rate, which needs the two samples
/// `collect_proc` takes; a process that could not be opened, or a Windows
/// older than 11, reports zero.
pub struct RecvBytes {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u64>,
    width: usize,
}

impl RecvBytes {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Recv"));
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

impl Column for RecvBytes {
    fn add(&mut self, proc: &ProcessInfo) {
        let interval_ms =
            (proc.interval.as_secs() * 1000 + u64::from(proc.interval.subsec_millis())).max(1);
        let io = proc
            .net_info
            .curr_recv
            .saturating_sub(proc.net_info.prev_recv)
            * 1000
            / interval_ms;

        let raw_content = io;
        let fmt_content = bytify(raw_content);

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u64, true);
}
