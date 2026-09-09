use crate::process::ProcessInfo;
#[cfg(target_os = "windows")]
use crate::process::SID_MAX;
#[cfg(not(target_os = "windows"))]
use crate::util::USERS_CACHE;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;
#[cfg(not(target_os = "windows"))]
use uzers::Groups;

pub struct Group {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[allow(dead_code)]
    abbr_sid: bool,
}

impl Group {
    pub fn new(header: Option<String>, abbr_sid: bool) -> Self {
        let header = header.unwrap_or_else(|| String::from("Group"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            abbr_sid,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(ref status) = proc.curr_status {
            let gid = status.egid;
            if let Some(group) = USERS_CACHE.with(|x| x.borrow_mut().get_group_by_gid(gid)) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{gid}")
            }
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_proc.kp_eproc.e_pcred.p_rgid;
        let fmt_content =
            if let Some(group) = USERS_CACHE.with(|x| x.borrow_mut().get_group_by_gid(gid)) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{}", gid)
            };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        // The token is opened here rather than while collecting, so a run
        // that does not show this column never pays for it.
        let fmt_content = groups_of(proc.pid)
            .as_deref()
            .and_then(primary_group)
            .map(|sid| {
                sid.display_name()
                    .unwrap_or_else(|| sid.format(self.abbr_sid))
            })
            .unwrap_or_default();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

/// The token groups of `pid`, or `None` when its token cannot be opened - a
/// protected process, or any process when procs is not elevated.
///
/// Reading the groups costs an `OpenProcess` and two token calls per process,
/// so it is done here, lazily, instead of during collection: only a run that
/// displays a group column pays for it, and only for the pids it displays.
/// A thread row carries a thread id rather than a pid, so its lookup fails
/// and it renders as empty - the same thing `Arch` and `WorkDir` do.
#[cfg(target_os = "windows")]
pub fn groups_of(pid: i32) -> Option<Vec<SID_MAX>> {
    use std::mem::zeroed;
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::Security::{TOKEN_GROUPS, TOKEN_QUERY, TokenGroups};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
    };

    // 0 is the idle process, which owns no token; negative pids are not real.
    if pid <= 0 {
        return None;
    }

    // SAFETY: `pid` is only handed to `OpenProcess`, and the handle it hands
    // back is closed before this function returns.
    let handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION, FALSE, pid as u32) };
    if handle.is_null() {
        return None;
    }

    let mut token: HANDLE = unsafe { zeroed() };
    let ret = unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut token) };
    if ret == 0 {
        unsafe { CloseHandle(handle) };
        return None;
    }

    // The SID pointers live inside this buffer, so it has to outlive them.
    let buf = crate::process::token_information(token, TokenGroups);
    unsafe {
        CloseHandle(token);
        CloseHandle(handle);
    }
    let buf = buf?;

    let mut groups = Vec::new();
    #[allow(clippy::cast_ptr_alignment)]
    let token_groups = buf.as_ptr() as *const TOKEN_GROUPS;

    // SAFETY: `token_information` returns only when `GetTokenInformation`
    // succeeded, so `buf` holds a `TOKEN_GROUPS` followed by `GroupCount`
    // `SID_AND_ATTRIBUTES`. Every SID is copied into an owned `SID_MAX`
    // before `buf` is dropped, so no pointer outlives the buffer.
    unsafe {
        let sa = (*token_groups).Groups.as_ptr();
        for i in 0..(*token_groups).GroupCount {
            let psid = (*sa.offset(i as isize)).Sid;
            groups.push(SID_MAX::from_psid(psid));
        }
    }

    Some(groups)
}

/// The group a process is attributed to: the lowest-numbered well-known
/// builtin group (`S-1-5-32-x`), or the primary group of the token when it
/// carries no builtin group at all.
#[cfg(target_os = "windows")]
pub fn primary_group(groups: &[SID_MAX]) -> Option<&SID_MAX> {
    let mut sid = groups.first()?;
    let mut kind = u64::MAX;
    for g in groups {
        let subs = g.sub_authorities();
        if g.authority() == 5
            && subs.first() == Some(&32)
            && u64::from(subs.get(1).copied().unwrap_or(u32::MAX)) < kind
        {
            sid = g;
            kind = u64::from(subs[1]);
        }
    }
    Some(sid)
}

#[cfg(target_os = "freebsd")]
impl Column for Group {
    fn add(&mut self, proc: &ProcessInfo) {
        let gid = proc.curr_proc.ki_svgid;
        let fmt_content =
            if let Some(group) = USERS_CACHE.with(|x| x.borrow_mut().get_group_by_gid(gid)) {
                format!("{}", group.name().to_string_lossy())
            } else {
                format!("{gid}")
            };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
