use crate::{column_default, Column};
use procfs::{FDTarget, Io, ProcResult, Process, TcpNetEntry, TcpState};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct TcpPort {
    pub visible: bool,
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    tcp_entry: Vec<TcpNetEntry>,
}

impl TcpPort {
    pub fn new() -> Self {
        let header = String::from("TCP");
        let unit = String::from("");
        TcpPort {
            visible: true,
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
            tcp_entry: procfs::tcp().unwrap(),
        }
    }
}

impl Column for TcpPort {
    fn add(
        &mut self,
        curr_proc: &Process,
        _prev_proc: &Process,
        _curr_io: &ProcResult<Io>,
        _prev_io: &ProcResult<Io>,
        _interval: &Duration,
    ) -> () {
        let mut socks = Vec::new();
        if let Ok(fds) = curr_proc.fd() {
            for fd in fds {
                match fd.target {
                    FDTarget::Socket(x) => socks.push(x),
                    _ => (),
                }
            }
        }
        let mut ports = Vec::new();
        for sock in &socks {
            let entry = self.tcp_entry.iter().find(|&x| x.inode == *sock);
            if let Some(entry) = entry {
                if entry.state == TcpState::Listen {
                    ports.push(entry.local_address.port());
                }
            }
        }
        let content = format!("{:?}", ports);

        self.max_width = cmp::max(content.len(), self.max_width);

        self.contents.insert(curr_proc.pid(), String::from(content));
    }

    fn find_exact(&self, pid: i32, keyword: &str) -> bool {
        if let Some(content) = self.contents().get(&pid) {
            let content = content.replace("[", "").replace("]", "");
            let content = content.split(",");
            for c in content {
                if c == keyword {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    column_default!();
}
