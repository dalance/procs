use crate::process::ProcessInfo;
use crate::Column;
#[cfg(target_os = "macos")]
use libproc::libproc::net_info::TcpSIState;
#[cfg(any(target_os = "linux", target_os = "android"))]
use procfs::net::{TcpNetEntry, TcpState};
#[cfg(any(target_os = "linux", target_os = "android"))]
use procfs::process::FDTarget;
use std::cmp;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
#[cfg(target_os = "windows")]
use windows_sys::Win32::NetworkManagement::IpHelper::{
    GetTcp6Table2, GetTcpTable2, MIB_TCP6TABLE2, MIB_TCPTABLE2, MIB_TCP_STATE, MIB_TCP_STATE_LISTEN,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Networking::WinSock::{ntohl, ntohs};

pub struct TcpPort {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "windows"))]
    tcp_entry: Vec<TcpNetEntry>,
    #[cfg(any(target_os = "linux", target_os = "android", target_os = "windows"))]
    tcp6_entry: Vec<TcpNetEntry>,
}

impl TcpPort {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("TCP"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            #[cfg(any(target_os = "linux", target_os = "android"))]
            tcp_entry: procfs::net::tcp().unwrap_or_default(),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            tcp6_entry: procfs::net::tcp6().unwrap_or_default(),
            #[cfg(target_os = "windows")]
            tcp_entry: get_tcp_entry_list().unwrap_or_default(),
            #[cfg(target_os = "windows")]
            tcp6_entry: get_tcp6_entry_list().unwrap_or_default(),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
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
            ports.sort_unstable();
            ports.dedup();

            format!("{ports:?}")
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    fn find_exact(&self, pid: i32, keyword: &str, _content_to_lowercase: bool) -> bool {
        if let Some(content) = self.fmt_contents.get(&pid) {
            let content = content.replace(['[', ']'], "");
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
    crate::column_default_display_json!();
    crate::column_default_find_partial!();
    crate::column_default_sorted_pid!(String);
    crate::column_default_apply_visible!();
    crate::column_default_reset_width!();
    crate::column_default_update_width!();
    crate::column_default_get_width!();
    crate::column_default_is_numeric!(false);
}

#[cfg(target_os = "macos")]
impl Column for TcpPort {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut ports = Vec::new();
        for tcp in &proc.curr_tcps {
            if let TcpSIState::Listen = tcp.tcpsi_state.into() {
                let port = crate::util::change_endian(tcp.tcpsi_ini.insi_lport as u32) >> 16;
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

    fn find_exact(&self, pid: i32, keyword: &str, _content_to_lowercase: bool) -> bool {
        if let Some(content) = self.fmt_contents.get(&pid) {
            let content = content.replace(['[', ']'], "");
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
    crate::column_default_display_json!();
    crate::column_default_find_partial!();
    crate::column_default_sorted_pid!(String);
    crate::column_default_apply_visible!();
    crate::column_default_reset_width!();
    crate::column_default_update_width!();
    crate::column_default_get_width!();
    crate::column_default_is_numeric!(false);
}

#[cfg(target_os = "windows")]
impl Column for TcpPort {
    fn add(&mut self, proc: &ProcessInfo) {
        let mut ports: Vec<_> = self
            .tcp_entry
            .iter()
            .chain(self.tcp6_entry.iter())
            .filter(|entry| entry.state == MIB_TCP_STATE_LISTEN && entry.pid == proc.pid as u32)
            .map(|entry| entry.local_address.port())
            .collect();
        ports.sort_unstable();
        ports.dedup();
        let fmt_content = format!("{:?}", ports);
        let raw_content = fmt_content.clone();
        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    fn find_exact(&self, pid: i32, keyword: &str, _content_to_lowercase: bool) -> bool {
        if let Some(content) = self.fmt_contents.get(&pid) {
            let content = content.replace(['[', ']'], "");
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
    crate::column_default_display_json!();
    crate::column_default_find_partial!();
    crate::column_default_sorted_pid!(String);
    crate::column_default_apply_visible!();
    crate::column_default_reset_width!();
    crate::column_default_update_width!();
    crate::column_default_get_width!();
    crate::column_default_is_numeric!(false);
}

#[derive(Debug, Clone)]
#[cfg(target_os = "windows")]
pub struct TcpNetEntry {
    pub local_address: SocketAddr,
    #[allow(dead_code)]
    pub remote_address: SocketAddr,
    pub state: MIB_TCP_STATE,
    pub pid: u32,
}

#[cfg(target_os = "windows")]
fn get_tcp_entry_list() -> Result<Vec<TcpNetEntry>, anyhow::Error> {
    let mut entry_list = Vec::new();

    let mut buffer_size = 0;
    let ret = unsafe { GetTcpTable2(std::ptr::null_mut(), &mut buffer_size, 0) };
    if ret != ERROR_INSUFFICIENT_BUFFER {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut buffer = vec![0u8; buffer_size as usize];
    let ret = unsafe {
        GetTcpTable2(
            buffer.as_mut_ptr() as *mut MIB_TCPTABLE2,
            &mut buffer_size,
            0,
        )
    };
    if ret != NO_ERROR {
        return Err(std::io::Error::last_os_error().into());
    }

    let tcp_table = unsafe { &*(buffer.as_ptr() as *const MIB_TCPTABLE2) };
    for i in 0..tcp_table.dwNumEntries {
        let entry = unsafe { &*tcp_table.table.as_ptr().add(i as usize) };
        entry_list.push(TcpNetEntry {
            local_address: SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::from(unsafe { ntohl(entry.dwLocalAddr) }),
                unsafe { ntohs(entry.dwLocalPort as u16) },
            )),
            remote_address: SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::from(entry.dwRemoteAddr),
                unsafe { ntohs(entry.dwRemotePort as u16) },
            )),
            pid: entry.dwOwningPid,
            state: entry.dwState as i32,
        });
    }

    Ok(entry_list)
}

#[cfg(target_os = "windows")]
fn get_tcp6_entry_list() -> Result<Vec<TcpNetEntry>, anyhow::Error> {
    let mut entry_list = Vec::new();

    let mut buffer_size = 0;
    let ret = unsafe { GetTcp6Table2(std::ptr::null_mut(), &mut buffer_size, 0) };
    if ret != ERROR_INSUFFICIENT_BUFFER {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut buffer = vec![0u8; buffer_size as usize];
    let ret = unsafe {
        GetTcp6Table2(
            buffer.as_mut_ptr() as *mut MIB_TCP6TABLE2,
            &mut buffer_size,
            0,
        )
    };
    if ret != NO_ERROR {
        return Err(std::io::Error::last_os_error().into());
    }

    let tcp_table = unsafe { &*(buffer.as_ptr() as *const MIB_TCP6TABLE2) };
    for i in 0..tcp_table.dwNumEntries {
        let entry = unsafe { *tcp_table.table.as_ptr().add(i as usize) };
        entry_list.push(TcpNetEntry {
            local_address: SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::from(u128::from_be_bytes(unsafe { entry.LocalAddr.u.Byte })),
                unsafe { ntohs(entry.dwLocalPort as u16) },
                0,
                entry.dwLocalScopeId,
            )),
            remote_address: SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::from(u128::from_be_bytes(unsafe { entry.RemoteAddr.u.Byte })),
                unsafe { ntohs(entry.dwRemotePort as u16) },
                0,
                entry.dwRemoteScopeId,
            )),
            pid: entry.dwOwningPid,
            state: entry.State,
        });
    }

    Ok(entry_list)
}

#[cfg(test)]
#[cfg(target_os = "windows")]
mod tests {
    use std::net::TcpListener;

    use windows_sys::Win32::NetworkManagement::IpHelper::MIB_TCP_STATE_LISTEN;

    use super::TcpPort;

    #[test]
    fn test_tcp_port() {
        assert!(test_ip_port("127.0.0.1:0"));
        assert!(test_ip_port("0.0.0.0:0"));
        assert!(test_ip_port("[::1]:0"));
        assert!(test_ip_port("[::]:0"));
    }

    fn test_ip_port(address: &str) -> bool {
        let listener = TcpListener::bind(address).unwrap();
        let tcp_port = TcpPort::new(None);
        let found = tcp_port
            .tcp_entry
            .iter()
            .chain(tcp_port.tcp6_entry.iter())
            .find(|entry| {
                entry.state == MIB_TCP_STATE_LISTEN
                    && entry.local_address == listener.local_addr().unwrap()
            });

        found.is_some()
    }
}
