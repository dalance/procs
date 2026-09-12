use anyhow::{Context, Error, anyhow};
use chrono::offset::TimeZone;
use chrono::{Local, NaiveDate};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{MaybeUninit, zeroed};
use std::path::PathBuf;
use std::ptr;
use std::str::FromStr;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
use windows_sys::Win32::Security::{
    AdjustTokenPrivileges, GetTokenInformation, LookupAccountNameW, LookupAccountSidW,
    LookupPrivilegeValueW, PSID, SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED, SECURITY_MAX_SID_SIZE, SID,
    TOKEN_ADJUST_PRIVILEGES, TOKEN_INFORMATION_CLASS, TOKEN_PRIVILEGES, TOKEN_QUERY, TOKEN_USER,
    TokenUser,
};
use windows_sys::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, OpenThread, PROCESS_QUERY_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, THREAD_QUERY_LIMITED_INFORMATION,
};

use super::ProcessInfoBase;
use super::ShowFilter;
use super::ntapi;
use crate::config::ConfigUserFilter;
use crate::process::thread_key;

/// Re-export the process-parameters prefix so the WorkDir column can read
/// `CurrentDirectory`.
pub use super::ntapi::RTL_USER_PROCESS_PARAMETERS_PREFIX;
/// Re-export the owned SID type so callers can use `crate::process::SID_MAX`.
pub use super::ntapi::SID_MAX;
/// Re-export the decoded thread record so callers can use
/// `crate::process::ThreadSnapshot`.
pub use super::ntapi::ThreadSnapshot;
/// Re-export the scheduler state so callers can use
/// `crate::process::ThreadState`.
pub use super::ntapi::ThreadState;
/// Re-export the image machine query so callers can use
/// `crate::process::process_image_machine`.
pub use super::ntapi::process_image_machine;
/// Re-export the PEB address query so callers can use
/// `crate::process::process_peb_address`.
pub use super::ntapi::process_peb_address;
/// Re-export the cross-process memory reader so columns can read
/// PEB-relative data.
pub use super::ntapi::read_process_memory;
/// Re-export the `KTHREAD_STATE` constants so the State column can map
/// them to letters.
pub use super::ntapi::thread_state;
/// Re-export the `KWAIT_REASON` constants so the State column can map
/// them to letters.
pub use super::ntapi::wait_reason;

pub struct ProcessInfo {
    /// The part of a row every platform has - the row key, the parent and the
    /// sampling window. The `Deref` below hands it out, so a column goes on
    /// writing `proc.pid` without reaching for `base`.
    pub base: ProcessInfoBase,
    /// Command line of the process, or `None` when the process exposes none
    /// (e.g. System, Idle). Falls back to `file_name` at display time.
    pub command: Option<String>,
    /// Image (executable) name, used as the Command fallback and by the
    /// FileName column. For a `--thread` row it is the thread's own name when
    /// it has one, and the image name of the process it belongs to otherwise.
    pub file_name: String,
    pub start_time: chrono::DateTime<chrono::Local>,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disk_info: DiskInfo,
    pub net_info: NetInfo,
    pub user: Option<SID_MAX>,
    pub priority: i32,
    pub thread: i32,
    pub session: i32,
    /// Scheduler state of the most active thread behind this row.
    ///
    /// Windows publishes no state for a process as a whole, only for each of
    /// its threads, so the State column derives the row's state from the most
    /// active one. For a `--thread` row it is that one thread's state.
    pub state: Option<ThreadState>,
}

process_info_deref!();

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

/// Cumulative network traffic sampled at both ends of the interval.
///
/// `ProcessNetworkIoInformation` is a Windows 11 addition and needs a process
/// handle, so both the class and the process can be missing: the counters stay
/// at zero then and the rate comes out as zero.
pub struct NetInfo {
    pub prev_recv: u64,
    pub prev_send: u64,
    pub curr_recv: u64,
    pub curr_send: u64,
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
    _procfs_path: &Option<PathBuf>,
    filter: ShowFilter,
) -> Vec<ProcessInfo> {
    let _ = set_privilege();

    let started = Instant::now();
    let prev = take_snapshot(with_thread, filter);
    thread::sleep(interval);
    let finished = Instant::now();
    let curr = take_snapshot(with_thread, filter);

    // Several columns divide by this, so never hand out a zero interval.
    let interval = finished
        .saturating_duration_since(started)
        .max(Duration::from_millis(1));

    let prev_procs: HashMap<i32, &ProcSnapshot> = prev.procs.iter().map(|p| (p.pid, p)).collect();
    let prev_threads: HashMap<u64, &ThreadSnapshot> =
        prev.threads.iter().map(|t| (t.tid, t)).collect();

    let SystemSnapshot { procs, threads } = curr;

    let mut ret = Vec::with_capacity(procs.len());

    for proc in procs {
        // Hide the kernel's own processes (Idle, System, Registry, Memory
        // Compression, ...) unless `--thread` was given. The decision is made
        // once in `take_snapshot` from the authoritative kernel classification
        // (or the parent-pid heuristic on a basic snapshot).
        if !filter.kthread && proc.is_kthread {
            continue;
        }

        let Some(prev) = prev_procs
            .get(&proc.pid)
            .copied()
            .filter(|p| p.create_time == proc.create_time || proc.create_time == 0)
        else {
            // Pid recycled
            continue;
        };

        let handles = ProcHandles::open(proc.pid);

        // The owner comes first. With a full snapshot it is already in hand and
        // `take_snapshot` has filtered on it, so anything that gets this far
        // matches; with the basic snapshot the token has to be opened anyway,
        // which is what this call does, and dropping the process here at least
        // saves reading its command line. The group list is deliberately not
        // collected here - `Group` and `Gid` query it themselves, so that only
        // enabling one of them pays for it.
        let user = proc.user_sid.or_else(|| handles.full.and_then(get_user));
        // A process whose *own* owner cannot be read counts as not the wanted
        // user and is dropped: there is no SID to compare, and keeping it
        // would let processes of every user through. A filter that is not set
        // at all - our own SID not having been readable - is the one case that
        // keeps everything, so that the listing is not left empty.
        if let Some(userid) = filter.userid
            && user != Some(userid)
        {
            continue;
        }

        let command = handles
            .any()
            .and_then(ntapi::process_command_line)
            .filter(|command| !command.is_empty());

        // `command` holds only the command line (when one is exposed); `file_name`
        // is the image name and acts as the Command fallback for processes that
        // have no command line (e.g. System, Idle).
        let file_name = image_fallback(&proc);

        let priority = proc.base_priority;

        ret.push(ProcessInfo {
            base: ProcessInfoBase::new(proc.pid as i64, proc.ppid as i64, interval),
            command,
            file_name,
            start_time: filetime_to_local(proc.create_time),
            cpu_info: CpuInfo {
                prev_sys: prev.kernel_time,
                prev_user: prev.user_time,
                curr_sys: proc.kernel_time,
                curr_user: proc.user_time,
            },
            disk_info: DiskInfo {
                prev_read: prev.read,
                prev_write: prev.write,
                curr_read: proc.read,
                curr_write: proc.write,
            },
            net_info: NetInfo {
                prev_recv: prev.recv,
                prev_send: prev.send,
                curr_recv: proc.recv,
                curr_send: proc.send,
            },
            // Last: this moves out of `proc`, which `prev` may still borrow.
            memory_info: proc.memory_info,
            user,
            priority,
            thread: proc.thread_count,
            session: proc.session_id as i32,
            state: proc.state,
        });
    }

    if with_thread {
        let owners: HashMap<i64, usize> = ret
            .iter()
            .enumerate()
            .map(|(idx, p)| (p.pid, idx))
            .collect();

        for thread in threads {
            // Skip threads whose owning process was filtered out above.
            let Some(&owner) = owners.get(&(thread.pid as i64)) else {
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

            let (command, file_name, user, session) = {
                let parent = &ret[owner];
                (
                    parent.command.clone(),
                    // A thread has no image of its own, so the name it is
                    // listed under is its own when it carries one - which is
                    // how the Command and FileName columns tell the threads
                    // of one process apart - and its process' otherwise.
                    thread_name(thread.tid).unwrap_or_else(|| parent.file_name.clone()),
                    parent.user,
                    parent.session,
                )
            };

            ret.push(ProcessInfo {
                base: ProcessInfoBase::new(thread_key(thread.tid), thread.pid as i64, interval),
                command,
                file_name,
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
                // Network traffic is only accounted per process, so a thread
                // row reports none rather than repeating its process'.
                net_info: NetInfo {
                    prev_recv: 0,
                    prev_send: 0,
                    curr_recv: 0,
                    curr_send: 0,
                },
                user,
                priority: thread.priority,
                thread: 1,
                session,
                state: Some(thread.state),
            });
        }
    }

    ret
}

// ---------------------------------------------------------------------------
// Current user
// ---------------------------------------------------------------------------

/// The SID of the user `procs` runs as.
static CURRENT_USER: OnceLock<Option<SID_MAX>> = OnceLock::new();

/// The SID of the user `procs` runs as, or `None` when it cannot be resolved.
///
/// Reading our own token needs no privilege, so a failure is not expected; a
/// failure yields `None`, which then matches nothing.
fn current_user_sid() -> Option<SID_MAX> {
    *CURRENT_USER.get_or_init(|| token_user(CURRENT_PROCESS_TOKEN))
}

/// The user a filter keeps, looked up from what the configuration names.
///
/// Windows has no uid, so what the filter compares is the whole SID. The two
/// keywords never reach this function - the configuration layer turns them
/// into `All` and `Myself` - and what is left is a user, tried as the SID it
/// is written as and then as the name of an account. A number is looked up as
/// a name like anything else, and so is almost never a user: there is nothing
/// on Windows for a uid to mean.
///
/// A name that resolves to nothing is a typo, and a listing filtered by a typo
/// comes out empty rather than wrong - empty with a reason beats empty without
/// one.
pub fn user_id_of(user: &ConfigUserFilter) -> Result<Option<SID_MAX>, Error> {
    Ok(match user {
        ConfigUserFilter::All => None,
        ConfigUserFilter::Myself => Some(
            current_user_sid().context("the SID of the user procs runs as could not be read")?,
        ),
        ConfigUserFilter::User(user) => {
            let sid = SID_MAX::from_str(user)
                .ok()
                .or_else(|| lookup_sid(user))
                .ok_or_else(|| anyhow!("no such user: {user}"))?;
            Some(sid)
        }
    })
}

/// The SID of the account with this name, or `None` when there is no such
/// account.
///
/// The name is resolved the way Windows resolves it: a bare name is looked for
/// among the machine's own accounts and then in the domain, while
/// `DOMAIN\name` - or `MACHINE\name` - says which of them to look in.
fn lookup_sid(name: &str) -> Option<SID_MAX> {
    let name = to_wide(name);
    let mut sid_len = 0;
    let mut domain_len = 0;
    let mut kind = 0;
    unsafe {
        // The first call only measures, and is what says whether the name is
        // one at all: with no buffers it fills in the two lengths, or leaves
        // them at zero.
        let _ = LookupAccountNameW(
            ptr::null(),
            name.as_ptr(),
            ptr::null_mut(),
            &mut sid_len,
            ptr::null_mut(),
            &mut domain_len,
            &mut kind,
        );

        // A SID never exceeds `SECURITY_MAX_SID_SIZE`; a longer one would not
        // fit in the `SID_MAX` the second call fills.
        if sid_len == 0 || sid_len > SECURITY_MAX_SID_SIZE {
            return None;
        }

        let mut sid: SID_MAX = zeroed();
        let mut domain = vec![0u16; domain_len as usize];
        let ret = LookupAccountNameW(
            ptr::null(),
            name.as_ptr(),
            &mut sid.sid as *mut SID as PSID,
            &mut sid_len,
            domain.as_mut_ptr(),
            &mut domain_len,
            &mut kind,
        );

        (ret != 0).then_some(sid)
    }
}

/// A string as a NUL terminated wide string, for the `*W` APIs.
fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

struct ProcSnapshot {
    pid: i32,
    ppid: i32,
    thread_count: i32,
    image_name: String,
    session_id: u32,
    create_time: i64,
    kernel_time: u64,
    user_time: u64,
    read: u64,
    write: u64,
    /// Cumulative network bytes received / sent, when the class that reports
    /// them is implemented and the process could be opened.
    recv: u64,
    send: u64,
    memory_info: MemoryInfo,
    base_priority: i32,
    /// Scheduler state of the most active thread.
    state: Option<ThreadState>,
    /// Process user SID straight from the snapshot, when it carried one.
    /// `None` means the token has to be opened to learn the user.
    user_sid: Option<SID_MAX>,
    /// Whether this process is one of the kernel's own: System, Secure System,
    /// Registry, Memory Compression. When the full snapshot classified it this
    /// is authoritative; for a basic snapshot it falls back to the classic
    /// Idle/System parent-pid heuristic. `procs` hides these unless `--thread`.
    is_kthread: bool,
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
fn take_snapshot(with_thread: bool, filter: ShowFilter) -> SystemSnapshot {
    let mut procs = Vec::new();
    let mut threads = Vec::new();

    let Some(buffer) = ntapi::query_system_processes() else {
        return SystemSnapshot { procs, threads };
    };

    // One entry of the buffer is `procs` itself, and it carries the SID the
    // filter compares against: reading it here, while the buffer is still the
    // only thing in hand, is what lets the filter drop a process before a
    // single handle is opened for it.
    let carries_sid = buffer.carries_user_sid();

    // `ProcessNetworkIoInformation` is a Windows 11 addition: on an older
    // build no handle is opened for it at all.
    let want_net = network_counters_supported();

    for entry in buffer.iter() {
        let info = entry.info();
        let pid = info.UniqueProcessId as usize as i32;
        let ppid = info.InheritedFromUniqueProcessId as usize as i32;

        // Dropping a process here is free and saves everything the rest of the
        // snapshot does for it: the handle for the network counters, a place in
        // the process list, and later on the handle and the command line read
        // in `collect_proc`.
        let user = if carries_sid { entry.user_sid() } else { None };
        // An owner that is not published yet - the class does not carry SIDs,
        // or this entry has none - is judged again in `collect_proc`, once its
        // token has been read.
        if let Some(userid) = filter.userid
            && user.is_some()
            && user != Some(userid)
        {
            continue;
        }

        let image_name = entry.image_name();

        // Unlike the rest of the snapshot, the network counters are read from
        // the process itself, which costs an `OpenProcess` each.
        let (recv, send) = if want_net {
            network_counters(pid)
        } else {
            (0, 0)
        };

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
            pid,
            ppid,
            thread_count: info.NumberOfThreads as i32,
            image_name,
            session_id: info.SessionId,
            create_time: info.CreateTime,
            kernel_time: info.KernelTime as u64,
            user_time: info.UserTime as u64,
            read: info.IoCounters.ReadTransferCount,
            write: info.IoCounters.WriteTransferCount,
            recv,
            send,
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
            state: entry.state(),
            user_sid: user,
            is_kthread,
        });

        // Idle (pid 0) is a stand-in the kernel keeps for the idle loop, not a
        // process with threads of its own: the records it carries have no
        // usable thread id - most of them report none at all, and one that
        // reports `0` would collapse onto Idle's own row, since a thread row
        // is keyed by its negated id and `-0` is `0` again. So it is listed as
        // one childless process.
        if with_thread && pid != 0 {
            threads.extend(entry.threads());
        }
    }

    SystemSnapshot { procs, threads }
}

/// Whether `ProcessNetworkIoInformation` answers on this machine.
///
/// The class is implemented from Windows 11 on, so the answer depends on the
/// OS build alone: it is probed once against this process - whose handle
/// carries every access right, so a failure means the class is missing rather
/// than the handle being too weak - and remembered. An older build then pays
/// nothing instead of an `OpenProcess` and a rejected query per process and
/// per sample.
fn network_counters_supported() -> bool {
    const UNKNOWN: u8 = 0;
    const YES: u8 = 1;
    const NO: u8 = 2;
    static SUPPORTED: AtomicU8 = AtomicU8::new(UNKNOWN);

    match SUPPORTED.load(Ordering::Acquire) {
        YES => true,
        NO => false,
        _ => {
            // SAFETY: the pseudo-handle needs no closing.
            let handle = unsafe { GetCurrentProcess() };
            let yes = ntapi::process_network_counters(handle).is_some();
            SUPPORTED.store(if yes { YES } else { NO }, Ordering::Release);
            yes
        }
    }
}

/// The cumulative network counters of `pid`, or `(0, 0)` when the handle
/// cannot be opened or the class is not implemented.
fn network_counters(pid: i32) -> (u64, u64) {
    let handles = ProcHandles::open(pid);
    match handles.any().and_then(ntapi::process_network_counters) {
        Some(counters) => (counters.BytesIn, counters.BytesOut),
        None => (0, 0),
    }
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
// Thread handles
// ---------------------------------------------------------------------------

/// The name the thread `tid` carries, if any.
///
/// Unlike everything else a row shows, the name is not part of the snapshot:
/// it costs an `OpenThread` and a query of its own, so it is paid for only
/// by the threads `--thread` actually lists. `None` when the thread cannot be
/// opened - it may have exited since the snapshot was taken - or when it was
/// never given a name.
fn thread_name(tid: u64) -> Option<String> {
    // Thread ids are 32 bit; a wider one is not one the kernel handed out.
    let tid = u32::try_from(tid).ok()?;
    let handle = unsafe { OpenThread(THREAD_QUERY_LIMITED_INFORMATION, FALSE, tid) };
    if handle.is_null() {
        return None;
    }

    let name = ntapi::thread_name(handle);
    unsafe {
        CloseHandle(handle);
    }
    name
}

// ---------------------------------------------------------------------------
// Privilege / token
// ---------------------------------------------------------------------------

/// The primary token of the calling process.
///
/// `NtCurrentProcessToken()` of `winnt.h`: a pseudo-handle the kernel
/// resolves to the caller's own token. It needs no `OpenProcessToken` and must
/// never be closed, so nothing here calls `CloseHandle` on it.
///
/// It is only good for **querying** the token (`GetTokenInformation`).
/// Adjusting privileges through it does not take effect - `set_privilege` has
/// to open a real handle for that.
const CURRENT_PROCESS_TOKEN: HANDLE = -4isize as *mut c_void;

fn set_privilege() -> bool {
    // The pseudo-handle cannot be used here: adjusting privileges needs a real
    // handle opened with `TOKEN_ADJUST_PRIVILEGES`.
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

/// The SID of the user the process `handle` runs as.
fn get_user(handle: HANDLE) -> Option<SID_MAX> {
    let mut token: HANDLE = unsafe { zeroed() };
    let ret = unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut token) };
    if ret == 0 {
        return None;
    }

    let sid = token_user(token);
    unsafe {
        CloseHandle(token);
    }
    sid
}

/// The SID of the user `token` belongs to.
///
/// `token` may be a real handle or the `CURRENT_PROCESS_TOKEN` pseudo-handle;
/// neither is closed here, so the caller keeps owning it.
fn token_user(token: HANDLE) -> Option<SID_MAX> {
    // The SID pointer lives inside this buffer, so it has to stay alive for
    // as long as `psid` is used.
    let buf = token_information(token, TokenUser)?;

    #[allow(clippy::cast_ptr_alignment)]
    let token_user = buf.as_ptr() as *const TOKEN_USER;
    let psid = unsafe { (*token_user).User.Sid };

    Some(unsafe { SID_MAX::from_psid(psid) })
}

/// Queries a token, returning the buffer that owns the result. Callers must
/// keep it alive: the returned information contains pointers into it.
///
/// Public so that columns can read a token class the collection does not
/// gather itself - `Group` reads `TokenGroups` this way, on demand.
pub fn token_information(
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The pseudo-handle has to stand for the same token that
    /// `OpenProcessToken` hands out for the current process.
    #[test]
    fn current_process_token_matches_open_process_token() {
        let via_pseudo = token_user(CURRENT_PROCESS_TOKEN);
        // SAFETY: `GetCurrentProcess` returns a pseudo-handle that must not be
        // closed, and `get_user` closes only the token it opens itself.
        let via_open = unsafe { get_user(GetCurrentProcess()) };

        assert!(via_pseudo.is_some());
        assert!(via_pseudo == via_open);
    }
}
