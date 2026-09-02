use chrono::offset::TimeZone;
use chrono::{Local, NaiveDate};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{MaybeUninit, zeroed};
use std::path::PathBuf;
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupAccountSidW, LookupPrivilegeValueW, PSID,
    SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED, SID, TOKEN_ADJUST_PRIVILEGES, TOKEN_GROUPS,
    TOKEN_INFORMATION_CLASS, TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER, TokenGroups, TokenUser,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

use super::ntapi;

pub struct ProcessInfo {
    pub pid: i32,
    pub command: String,
    pub ppid: i32,
    pub start_time: chrono::DateTime<chrono::Local>,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disk_info: DiskInfo,
    pub user: Option<SidName>,
    pub groups: Vec<SidName>,
    pub priority: i32,
    pub thread: i32,
    pub interval: Duration,
}

pub struct MemoryInfo {
    pub page_fault_count: u64,
    pub peak_working_set_size: u64,
    pub working_set_size: u64,
    #[allow(dead_code)]
    pub quota_peak_paged_pool_usage: u64,
    pub quota_paged_pool_usage: u64,
    #[allow(dead_code)]
    pub quota_peak_non_paged_pool_usage: u64,
    pub quota_non_paged_pool_usage: u64,
    #[allow(dead_code)]
    pub page_file_usage: u64,
    pub peak_page_file_usage: u64,
    pub private_usage: u64,
}

impl MemoryInfo {
    /// Used for threads, which do not own memory of their own.
    fn empty() -> Self {
        Self {
            page_fault_count: 0,
            peak_working_set_size: 0,
            working_set_size: 0,
            quota_peak_paged_pool_usage: 0,
            quota_paged_pool_usage: 0,
            quota_peak_non_paged_pool_usage: 0,
            quota_non_paged_pool_usage: 0,
            page_file_usage: 0,
            peak_page_file_usage: 0,
            private_usage: 0,
        }
    }
}

pub struct DiskInfo {
    pub prev_read: u64,
    pub prev_write: u64,
    pub curr_read: u64,
    pub curr_write: u64,
}

pub struct CpuInfo {
    pub prev_sys: u64,
    pub prev_user: u64,
    pub curr_sys: u64,
    pub curr_user: u64,
}

#[derive(Clone)]
pub struct SidName {
    pub sid: Vec<u64>,
    pub name: Option<String>,
    #[allow(dead_code)]
    pub domainname: Option<String>,
}

pub fn collect_proc(
    interval: Duration,
    with_thread: bool,
    show_kthreads: bool,
    _procfs_path: &Option<PathBuf>,
) -> Vec<ProcessInfo> {
    let _ = set_privilege();

    let started = Instant::now();
    let prev = take_snapshot(with_thread);
    thread::sleep(interval);
    let finished = Instant::now();
    let curr = take_snapshot(with_thread);

    // Several columns divide by this, so never hand out a zero interval.
    let interval = finished
        .saturating_duration_since(started)
        .max(Duration::from_millis(1));

    let prev_procs: HashMap<i32, &ProcSnapshot> = prev.procs.iter().map(|p| (p.pid, p)).collect();
    let prev_threads: HashMap<i32, &ThreadSnapshot> =
        prev.threads.iter().map(|t| (t.tid, t)).collect();

    let SystemSnapshot { procs, threads } = curr;

    let mut ret = Vec::with_capacity(procs.len());

    for proc in procs {
        // Idle and System have no parent process; treat them like the kernel
        // threads other platforms hide behind the same flag.
        if !show_kthreads
            && (proc.ppid == 0 /* Idle, System */ || (proc.ppid == 4 /* System */ && !proc.image_name.contains('.')))
        {
            continue;
        }

        let prev = prev_procs
            .get(&proc.pid)
            .copied()
            // A recycled pid would otherwise pair up with an unrelated process.
            .filter(|p| p.create_time == proc.create_time || proc.create_time == 0);

        let (prev_sys, prev_user, prev_read, prev_write) = match prev {
            Some(p) => (p.kernel_time, p.user_time, p.read, p.write),
            // Started between the two samples: report no delta.
            None => (proc.kernel_time, proc.user_time, proc.read, proc.write),
        };

        let handles = ProcHandles::open(proc.pid);

        let command = handles
            .any()
            .and_then(ntapi::process_command_line)
            .filter(|command| !command.is_empty())
            .unwrap_or_else(|| image_fallback(&proc));

        let (user, groups) = match handles.full {
            Some(handle) => (get_user(handle), get_groups(handle)),
            None => (None, None),
        };
        let priority = proc.base_priority;

        ret.push(ProcessInfo {
            pid: proc.pid,
            command,
            ppid: proc.ppid,
            start_time: filetime_to_local(proc.create_time),
            cpu_info: CpuInfo {
                prev_sys,
                prev_user,
                curr_sys: proc.kernel_time,
                curr_user: proc.user_time,
            },
            memory_info: proc.memory_info,
            disk_info: DiskInfo {
                prev_read,
                prev_write,
                curr_read: proc.read,
                curr_write: proc.write,
            },
            user,
            groups: groups.unwrap_or_default(),
            priority,
            thread: proc.thread_count,
            interval,
        });
    }

    if with_thread {
        let owners: HashMap<i32, usize> = ret
            .iter()
            .enumerate()
            .map(|(idx, p)| (p.pid, idx))
            .collect();

        for thread in threads {
            // Skip threads whose owning process was filtered out above.
            let Some(&owner) = owners.get(&thread.pid) else {
                continue;
            };

            let prev = prev_threads
                .get(&thread.tid)
                .copied()
                .filter(|p| p.create_time == thread.create_time || thread.create_time == 0);
            let (prev_sys, prev_user) = match prev {
                Some(p) => (
                    thread.kernel_time.saturating_sub(p.kernel_time),
                    thread.user_time.saturating_sub(p.user_time),
                ),
                None => (thread.kernel_time, thread.user_time),
            };

            let (command, user, groups) = {
                let parent = &ret[owner];
                (
                    parent.command.clone(),
                    parent.user.clone(),
                    parent.groups.clone(),
                )
            };

            ret.push(ProcessInfo {
                pid: thread.tid,
                command,
                ppid: thread.pid,
                start_time: filetime_to_local(thread.create_time),
                cpu_info: CpuInfo {
                    prev_sys,
                    prev_user,
                    curr_sys: thread.kernel_time,
                    curr_user: thread.user_time,
                },
                memory_info: MemoryInfo::empty(),
                disk_info: DiskInfo {
                    prev_read: 0,
                    prev_write: 0,
                    curr_read: 0,
                    curr_write: 0,
                },
                user,
                groups,
                priority: thread.priority,
                thread: 1,
                interval,
            });
        }
    }

    ret
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

struct ProcSnapshot {
    pid: i32,
    ppid: i32,
    thread_count: i32,
    image_name: String,
    create_time: i64,
    kernel_time: u64,
    user_time: u64,
    read: u64,
    write: u64,
    memory_info: MemoryInfo,
    base_priority: i32,
}

struct ThreadSnapshot {
    tid: i32,
    pid: i32,
    create_time: i64,
    kernel_time: u64,
    user_time: u64,
    priority: i32,
}

struct SystemSnapshot {
    procs: Vec<ProcSnapshot>,
    threads: Vec<ThreadSnapshot>,
}

/// One `NtQuerySystemInformation(SystemProcessInformation)` call covers every
/// process on the machine - no process handle required.
fn take_snapshot(with_thread: bool) -> SystemSnapshot {
    let mut procs = Vec::new();
    let mut threads = Vec::new();

    let Some(buffer) = ntapi::query_system_processes() else {
        return SystemSnapshot { procs, threads };
    };

    for info in ntapi::ProcessIter::new(&buffer) {
        procs.push(ProcSnapshot {
            pid: info.UniqueProcessId as usize as i32,
            ppid: info.InheritedFromUniqueProcessId as usize as i32,
            thread_count: info.NumberOfThreads as i32,
            image_name: ntapi::unicode_string_to_owned(&info.ImageName),
            create_time: info.CreateTime,
            kernel_time: info.KernelTime as u64,
            user_time: info.UserTime as u64,
            read: info.IoCounters.ReadTransferCount,
            write: info.IoCounters.WriteTransferCount,
            memory_info: MemoryInfo {
                page_fault_count: u64::from(info.VirtualMemoryCounters.PageFaultCount),
                peak_working_set_size: info.VirtualMemoryCounters.PeakWorkingSetSize as u64,
                working_set_size: info.VirtualMemoryCounters.WorkingSetSize as u64,
                quota_peak_paged_pool_usage: info.VirtualMemoryCounters.QuotaPeakPagedPoolUsage
                    as u64,
                quota_paged_pool_usage: info.VirtualMemoryCounters.QuotaPagedPoolUsage as u64,
                quota_peak_non_paged_pool_usage: info
                    .VirtualMemoryCounters
                    .QuotaPeakNonPagedPoolUsage
                    as u64,
                quota_non_paged_pool_usage: info.VirtualMemoryCounters.QuotaNonPagedPoolUsage
                    as u64,
                page_file_usage: info.VirtualMemoryCounters.PagefileUsage as u64,
                peak_page_file_usage: info.VirtualMemoryCounters.PeakPagefileUsage as u64,
                private_usage: info.PrivatePageCount as u64,
            },
            base_priority: info.BasePriority,
        });

        if with_thread {
            let pid = info.UniqueProcessId as usize as i32;
            let slice = info.threads();
            for idx in 0..slice.len() {
                let Some(thread) = slice.get(idx) else {
                    continue;
                };
                threads.push(ThreadSnapshot {
                    tid: thread.tid,
                    pid,
                    create_time: thread.create_time,
                    kernel_time: thread.kernel_time,
                    user_time: thread.user_time,
                    priority: thread.priority,
                });
            }
        }
    }

    SystemSnapshot { procs, threads }
}

/// Idle and System have no command line and, depending on privileges, no
/// readable image name either.
fn image_fallback(proc: &ProcSnapshot) -> String {
    if !proc.image_name.is_empty() {
        return proc.image_name.clone();
    }
    match proc.pid {
        0 => String::from("[System Idle Process]"),
        _ => String::new(),
    }
}

fn filetime_to_local(time: i64) -> chrono::DateTime<Local> {
    let base = NaiveDate::from_ymd_opt(1601, 1, 1)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .unwrap();
    let elapsed = chrono::Duration::seconds(time.max(0) / 10_000_000);
    Local.from_utc_datetime(&(base + elapsed))
}

// ---------------------------------------------------------------------------
// Process handles
// ---------------------------------------------------------------------------

/// `PROCESS_QUERY_INFORMATION` is needed for the token and
/// `PROCESS_QUERY_LIMITED_INFORMATION` is sufficient for command lines.
const FULL_ACCESS: u32 = PROCESS_QUERY_INFORMATION;
/// Protected processes refuse the above but often allow the limited variant,
/// which is all `ProcessCommandLineInformation` needs.
const LIMITED_ACCESS: u32 = PROCESS_QUERY_LIMITED_INFORMATION;

struct ProcHandles {
    full: Option<HANDLE>,
    limited: Option<HANDLE>,
}

impl ProcHandles {
    fn open(pid: i32) -> Self {
        if pid <= 0 {
            return Self {
                full: None,
                limited: None,
            };
        }

        let full = open_process(pid, FULL_ACCESS);
        let limited = if full.is_some() {
            None
        } else {
            open_process(pid, LIMITED_ACCESS)
        };

        Self { full, limited }
    }

    /// Any handle usable for `NtQueryInformationProcess`.
    fn any(&self) -> Option<HANDLE> {
        self.full.or(self.limited)
    }
}

impl Drop for ProcHandles {
    fn drop(&mut self) {
        for handle in [self.full, self.limited].into_iter().flatten() {
            unsafe {
                CloseHandle(handle);
            }
        }
    }
}

fn open_process(pid: i32, access: u32) -> Option<HANDLE> {
    let handle = unsafe { OpenProcess(access, FALSE, pid as u32) };
    if handle.is_null() { None } else { Some(handle) }
}

// ---------------------------------------------------------------------------
// Privilege / token
// ---------------------------------------------------------------------------

fn set_privilege() -> bool {
    let handle = unsafe { GetCurrentProcess() };
    let mut token: HANDLE = unsafe { zeroed() };
    let ret = unsafe { OpenProcessToken(handle, TOKEN_ADJUST_PRIVILEGES, &mut token) };
    if ret == 0 {
        return false;
    }

    let mut tps: TOKEN_PRIVILEGES = unsafe { zeroed() };
    tps.PrivilegeCount = 1;
    // `SE_DEBUG_NAME` is already a NUL terminated wide string.
    let ret =
        unsafe { LookupPrivilegeValueW(ptr::null(), SE_DEBUG_NAME, &mut tps.Privileges[0].Luid) };
    if ret == 0 {
        unsafe {
            CloseHandle(token);
        }
        return false;
    }

    tps.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    let ret = unsafe {
        AdjustTokenPrivileges(
            token,
            FALSE,
            &tps as *const _,
            0,
            ptr::null::<TOKEN_PRIVILEGES>() as *mut TOKEN_PRIVILEGES,
            ptr::null::<u32>() as *mut u32,
        )
    };

    unsafe {
        CloseHandle(token);
    }

    ret != 0
}

fn get_user(handle: HANDLE) -> Option<SidName> {
    let mut token: HANDLE = unsafe { zeroed() };
    let ret = unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut token) };
    if ret == 0 {
        return None;
    }

    let sid = get_token_information(token, TokenUser);
    unsafe {
        CloseHandle(token);
    }

    // The SID pointer lives inside this buffer, so it has to stay alive for
    // as long as `psid` is used.
    let buf = sid?;

    #[allow(clippy::cast_ptr_alignment)]
    let token_user = buf.as_ptr() as *const TOKEN_USER;
    let psid = unsafe { (*token_user).User.Sid };

    let (name, domainname) = match get_name_cached(psid) {
        Some((name, domainname)) => (Some(name), Some(domainname)),
        None => (None, None),
    };

    Some(SidName {
        sid: get_sid(psid),
        name,
        domainname,
    })
}

fn get_groups(handle: HANDLE) -> Option<Vec<SidName>> {
    let mut token: HANDLE = unsafe { zeroed() };
    let ret = unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut token) };
    if ret == 0 {
        return None;
    }

    let groups = get_token_information(token, TokenGroups);
    unsafe {
        CloseHandle(token);
    }

    // The SID pointers live inside this buffer, so it has to outlive them.
    let buf = groups?;

    let mut ret = Vec::new();
    #[allow(clippy::cast_ptr_alignment)]
    let token_groups = buf.as_ptr() as *const TOKEN_GROUPS;

    unsafe {
        let sa = (*token_groups).Groups.as_ptr();
        for i in 0..(*token_groups).GroupCount {
            let psid = (*sa.offset(i as isize)).Sid;
            let (name, domainname) = if let Some((x, y)) = get_name_cached(psid) {
                (Some(x), Some(y))
            } else {
                (None, None)
            };

            ret.push(SidName {
                sid: get_sid(psid),
                name,
                domainname,
            });
        }
    }

    Some(ret)
}

/// Queries a token, returning the buffer that owns the result. Callers must
/// keep it alive: the returned information contains pointers into it.
fn get_token_information(
    token: HANDLE,
    class: TOKEN_INFORMATION_CLASS,
) -> Option<Vec<MaybeUninit<u8>>> {
    let mut cb_needed = 0;
    let _ = unsafe {
        GetTokenInformation(
            token,
            class,
            ptr::null::<c_void>() as *mut c_void,
            0,
            &mut cb_needed,
        )
    };

    let mut buf: Vec<MaybeUninit<u8>> = Vec::with_capacity(cb_needed as usize);
    unsafe {
        buf.set_len(cb_needed as usize);
    }

    let ret = unsafe {
        GetTokenInformation(
            token,
            class,
            buf.as_mut_ptr() as *mut c_void,
            cb_needed,
            &mut cb_needed,
        )
    };

    if ret == 0 { None } else { Some(buf) }
}

fn get_sid(psid: PSID) -> Vec<u64> {
    let mut ret = Vec::new();
    let psid = psid as *const SID;
    unsafe {
        let mut ia = 0;
        ia |= u64::from((*psid).IdentifierAuthority.Value[0]) << 40;
        ia |= u64::from((*psid).IdentifierAuthority.Value[1]) << 32;
        ia |= u64::from((*psid).IdentifierAuthority.Value[2]) << 24;
        ia |= u64::from((*psid).IdentifierAuthority.Value[3]) << 16;
        ia |= u64::from((*psid).IdentifierAuthority.Value[4]) << 8;
        ia |= u64::from((*psid).IdentifierAuthority.Value[5]);

        ret.push(u64::from((*psid).Revision));
        ret.push(ia);
        let cnt = (*psid).SubAuthorityCount;
        let sa = (*psid).SubAuthority.as_ptr();
        for i in 0..cnt {
            ret.push(u64::from(*sa.offset(i as isize)));
        }

        ret
    }
}

/// Account name and domain name, cached per SID.
type AccountName = Option<(String, String)>;

// Keyed by the SID itself rather than by its address: every query allocates a
// fresh buffer, and a freed one gets its address recycled by the next query,
// which would hand back another account's name.
thread_local!(
    pub static NAME_CACHE: RefCell<HashMap<Vec<u64>, AccountName>> = RefCell::new(HashMap::new());
);

fn get_name_cached(psid: PSID) -> Option<(String, String)> {
    let key = get_sid(psid);
    NAME_CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some(x) = c.get(&key) {
            x.clone()
        } else {
            let x = get_name(psid);
            c.insert(key, x.clone());
            x
        }
    })
}

fn get_name(psid: PSID) -> Option<(String, String)> {
    let mut cc_name = 0;
    let mut cc_domainname = 0;
    let mut pe_use = 0;
    unsafe {
        let _ = LookupAccountSidW(
            ptr::null::<u16>() as *mut u16,
            psid,
            ptr::null::<u16>() as *mut u16,
            &mut cc_name,
            ptr::null::<u16>() as *mut u16,
            &mut cc_domainname,
            &mut pe_use,
        );

        if cc_name == 0 || cc_domainname == 0 {
            return None;
        }

        let mut name: Vec<u16> = Vec::with_capacity(cc_name as usize);
        let mut domainname: Vec<u16> = Vec::with_capacity(cc_domainname as usize);
        name.set_len(cc_name as usize);
        domainname.set_len(cc_domainname as usize);
        let ret = LookupAccountSidW(
            ptr::null::<u16>() as *mut u16,
            psid,
            name.as_mut_ptr(),
            &mut cc_name,
            domainname.as_mut_ptr(),
            &mut cc_domainname,
            &mut pe_use,
        );

        if ret == 0 {
            return None;
        }

        let name = from_wide_ptr(name.as_ptr());
        let domainname = from_wide_ptr(domainname.as_ptr());
        Some((name, domainname))
    }
}

fn from_wide_ptr(ptr: *const u16) -> String {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    assert!(!ptr.is_null());
    let len = (0..isize::MAX)
        .position(|i| unsafe { *ptr.offset(i) == 0 })
        .unwrap();
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    OsString::from_wide(slice).to_string_lossy().into_owned()
}
