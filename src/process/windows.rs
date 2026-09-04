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

/// Re-export the owned SID type so callers can use `crate::process::SID_MAX`.
pub use super::ntapi::SID_MAX;

pub struct ProcessInfo {
    pub pid: i32,
    pub command: String,
    pub ppid: i32,
    pub start_time: chrono::DateTime<chrono::Local>,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disk_info: DiskInfo,
    pub user: Option<SID_MAX>,
    pub groups: Vec<SID_MAX>,
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
        // Hide the kernel's own processes (Idle, System, Registry, Memory
        // Compression, ...) unless `--thread` was given. The decision is made
        // once in `take_snapshot` from the authoritative kernel classification
        // (or the parent-pid heuristic on a basic snapshot).
        if !show_kthreads && proc.is_kthread {
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

        // The snapshot SID saves an `OpenProcessToken`; the token is only
        // opened for processes the snapshot could not name.
        let user = proc
            .user_sid
            .clone()
            .or_else(|| handles.full.and_then(get_user));
        let groups = handles.full.and_then(get_groups);
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
    /// Process user SID straight from the snapshot, when it carried one.
    /// `None` means the token has to be opened to learn the user.
    user_sid: Option<SID_MAX>,
    /// Whether this process is one of the kernel's own: System, Secure System,
    /// Registry, Memory Compression. When the full snapshot classified it this
    /// is authoritative; for a basic snapshot it falls back to the classic
    /// Idle/System parent-pid heuristic. `procs` hides these unless `--thread`.
    is_kthread: bool,
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

/// One `NtQuerySystemInformation` call covers every process on the machine -
/// no process handle required.
///
/// `SystemFullProcessInformation` is used when it is available, which also
/// hands over each process' user SID.
fn take_snapshot(with_thread: bool) -> SystemSnapshot {
    let mut procs = Vec::new();
    let mut threads = Vec::new();

    let Some(buffer) = ntapi::query_system_processes() else {
        return SystemSnapshot { procs, threads };
    };

    for entry in buffer.iter() {
        let info = entry.info();
        let ppid = info.InheritedFromUniqueProcessId as usize as i32;
        let image_name = entry.image_name();

        // Hide the kernel's own processes. The full snapshot's classification is
        // authoritative - a non-zero `SYSTEM_PROCESS_CLASSIFICATION` marks System,
        // Secure System, Registry and Memory Compression, which carry real parent
        // PIDs and the classic ppid heuristic would miss. The parent-pid rule is
        // kept as a safety net for Idle (pid 0): the kernel reports it as `Normal`
        // (classification 0) yet it is not a user process and must stay hidden.
        let is_kthread = entry.classification().is_some_and(|c| c != 0)
            || ppid == 0
            || (ppid == 4 && !image_name.contains('.'));

        procs.push(ProcSnapshot {
            pid: info.UniqueProcessId as usize as i32,
            ppid,
            thread_count: info.NumberOfThreads as i32,
            image_name,
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
            user_sid: entry.user_sid(),
            is_kthread,
        });

        if with_thread {
            let pid = info.UniqueProcessId as usize as i32;
            let slice = entry.threads();
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

fn get_user(handle: HANDLE) -> Option<SID_MAX> {
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

    Some(unsafe { SID_MAX::from_psid(psid) })
}

fn get_groups(handle: HANDLE) -> Option<Vec<SID_MAX>> {
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
            ret.push(SID_MAX::from_psid(psid));
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

/// Account name and domain name, cached per SID.
type AccountName = Option<(String, String)>;

// Keyed by the SID itself rather than by its address: every query allocates a
// fresh buffer, and a freed one gets its address recycled by the next query,
// which would hand back another account's name.
thread_local!(
    pub static NAME_CACHE: RefCell<HashMap<SID_MAX, AccountName>> = RefCell::new(HashMap::new());
);

impl SID_MAX {
    /// Resolves the account name for this SID at display time, using the
    /// thread-local cache. Returns `None` when the name cannot be looked up
    /// (e.g. a protected process or a SID with no resolvable account).
    pub fn display_name(&self) -> Option<String> {
        NAME_CACHE
            .with(|c| {
                let mut c = c.borrow_mut();
                if let Some(x) = c.get(self) {
                    x.clone()
                } else {
                    // `SID_MAX` embeds the official `SID` with matching alignment,
                    // so its address is a valid `PSID`.
                    let x = self.get_name();
                    c.insert(*self, x.clone());
                    x
                }
            })
            .map(|(name, _)| name)
    }

    fn get_name(&self) -> Option<(String, String)> {
        let psid = &self.sid as *const SID as PSID;
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
