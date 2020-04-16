use crate::process::ProcessInfo;
use crate::Column;
#[cfg(target_os = "macos")]
use libproc::libproc::net_info::TcpSIState;
#[cfg(target_os = "linux")]
use procfs::net::{TcpNetEntry, TcpState};
#[cfg(target_os = "linux")]
use procfs::process::FDTarget;
use std::cmp;
use std::collections::HashMap;

pub struct TcpPort {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[cfg(target_os = "linux")]
    tcp_entry: Vec<TcpNetEntry>,
    #[cfg(target_os = "linux")]
    tcp6_entry: Vec<TcpNetEntry>,
}

impl TcpPort {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("TCP"));
        let unit = String::from("");
        TcpPort {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            #[cfg(target_os = "linux")]
            tcp_entry: procfs::net::tcp().unwrap_or_default(),
            #[cfg(target_os = "linux")]
            tcp6_entry: procfs::net::tcp6().unwrap_or_default(),
        }
    }
}

#[cfg(target_os = "linux")]
impl Column for TcpPort {
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
                let mut tcp_iter = self.tcp_entry.iter().chain(self.tcp6_entry.iter());
                let entry = tcp_iter.find(|&x| x.inode == *sock);
                if let Some(entry) = entry {
                    if entry.state == TcpState::Listen {
                        ports.push(entry.local_address.port());
                    }
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
impl Column for TcpPort {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut ports = Vec::new();
        for tcp in &proc.curr_tcps {
            match tcp.tcpsi_state.into() {
                TcpSIState::Listen => {
                    let port = crate::util::change_endian(tcp.tcpsi_ini.insi_lport as u32) >> 16;
                    ports.push(port);
                }
                _ => (),
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
