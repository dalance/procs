use crate::{column_default, Column};
#[cfg(not(udp))]
use procfs::TcpNetEntry as UdpNetEntry;
#[cfg(not(udp))]
use procfs::{FDTarget, Io, ProcResult, Process};
#[cfg(udp)]
use procfs::{FDTarget, Io, ProcResult, Process, UdpNetEntry};
use std::cmp;
use std::collections::HashMap;
use std::time::Duration;

pub struct UdpPort {
    header: String,
    unit: String,
    contents: HashMap<i32, String>,
    max_width: usize,
    udp_entry: Vec<UdpNetEntry>,
}

impl UdpPort {
    pub fn new() -> Self {
        let header = String::from("UDP");
        let unit = String::from("");
        UdpPort {
            contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header: header,
            unit: unit,
            #[cfg(not(udp))]
            udp_entry: procfs::tcp().unwrap(),
            #[cfg(udp)]
            udp_entry: procfs::udp().unwrap(),
        }
    }
}

impl Column for UdpPort {
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
            let entry = self.udp_entry.iter().find(|&x| x.inode == *sock);
            if let Some(entry) = entry {
                ports.push(entry.local_address.port());
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
