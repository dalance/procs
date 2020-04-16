use crate::process::ProcessInfo;
use crate::Column;
#[cfg(target_os = "linux")]
use procfs::net::UdpNetEntry;
#[cfg(target_os = "linux")]
use procfs::process::FDTarget;
use std::cmp;
use std::collections::HashMap;

pub struct UdpPort {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[cfg(target_os = "linux")]
    udp_entry: Vec<UdpNetEntry>,
    #[cfg(target_os = "linux")]
    udp6_entry: Vec<UdpNetEntry>,
}

impl UdpPort {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("UDP"));
        let unit = String::from("");
        UdpPort {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            #[cfg(target_os = "linux")]
            udp_entry: procfs::net::udp().unwrap_or_default(),
            #[cfg(target_os = "linux")]
            udp6_entry: procfs::net::udp6().unwrap_or_default(),
        }
    }
}

#[cfg(target_os = "linux")]
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

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
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
    crate::column_default_apply_visible!();
    crate::column_default_reset_width!();
    crate::column_default_update_width!();
    crate::column_default_get_width!();
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
impl Column for UdpPort {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut ports = Vec::new();
        for udp in &proc.curr_udps {
            let port = crate::util::change_endian(udp.insi_lport as u32) >> 16;
            if port != 0 {
                ports.push(port);
            }
        }
        ports.sort();
        ports.dedup();

        let fmt_content = format!("{:?}", ports);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
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
    crate::column_default_apply_visible!();
    crate::column_default_reset_width!();
    crate::column_default_update_width!();
    crate::column_default_get_width!();
}
