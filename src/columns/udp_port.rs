use crate::process::ProcessInfo;
use crate::Column;
use procfs::{FDTarget, UdpNetEntry};
use std::cmp;
use std::collections::HashMap;

pub struct UdpPort {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
    udp_entry: Vec<UdpNetEntry>,
    udp6_entry: Vec<UdpNetEntry>,
}

impl UdpPort {
    pub fn new() -> Self {
        let header = String::from("UDP");
        let unit = String::from("");
        UdpPort {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
            udp_entry: procfs::udp().unwrap_or_default(),
            udp6_entry: procfs::udp6().unwrap_or_default(),
        }
    }
}

impl Column for UdpPort {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(fds) = proc.curr_proc.fd() {
            let mut socks = Vec::new();
            for fd in fds {
                if let FDTarget::Socket(x) = fd.target {
                    socks.push(x)
                }
            }

            let mut ports = Vec::new();
            for sock in &socks {
                let mut udp_iter = self.udp_entry.iter().chain(self.udp6_entry.iter());
                let entry = udp_iter.find(|&x| x.inode == *sock);
                if let Some(entry) = entry {
                    ports.push(entry.local_address.port());
                }
            }
            ports.sort();
            ports.dedup();

            format!("{:?}", ports)
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.curr_proc.pid(), fmt_content);
        self.raw_contents.insert(proc.curr_proc.pid(), raw_content);
    }

    fn find_exact(&self, pid: i32, keyword: &str) -> bool {
        if let Some(content) = self.fmt_contents.get(&pid) {
            let content = content.replace("[", "").replace("]", "");
            let content = content.split(',');
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

    crate::column_default_display_header!();
    crate::column_default_display_unit!();
    crate::column_default_display_content!();
    crate::column_default_find_partial!();
    crate::column_default_sorted_pid!(String);
    crate::column_default_reset_max_width!();
    crate::column_default_update_max_width!();
}
