use crate::process::{ProcessInfoBase, ShowFilter, thread_key};
use libc::{
    CTL_KERN, KERN_PROC, KERN_PROC_INC_THREAD, KERN_PROC_PROC, KERN_PROC_UID, P_KPROC, c_void,
    kinfo_proc,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

/// The two `pri_class` values `State` reads, from `sys/priority.h`.
///
/// `libc` binds the `struct priority` they live in but not the class
/// constants themselves, and they are the ABI - `PRI_ITHD` 1, `PRI_REALTIME`
/// 2, `PRI_TIMESHARE` 3, `PRI_IDLE` 4 - so the numbers are written out here
/// rather than guessed at the use site.
pub const PRI_REALTIME: u8 = 2;
pub const PRI_IDLE: u8 = 4;

pub struct ProcessInfo {
    /// The part of a row every platform has - the row key, the parent and the
    /// sampling window. The `Deref` below hands it out, so a column goes on
    /// writing `proc.pid` without reaching for `base`.
    pub base: ProcessInfoBase,
    /// The kernel's `struct kinfo_proc`, as `libc` binds it.
    ///
    /// FreeBSD pins the layout of this structure by size - `KINFO_PROC_SIZE`,
    /// 1088 bytes on the 64-bit architectures - so the ABI is fixed even
    /// though fields keep being carved out of the `ki_spare*` arrays. That is
    /// what makes `libc`'s binding safe to read directly: it was checked
    /// against the real headers on FreeBSD 15.1, where all 54 of the sizes
    /// and field offsets this crate reads agree exactly.
    pub curr_proc: kinfo_proc,
    pub prev_proc: kinfo_proc,
}

process_info_deref!();

fn get_processes(with_thread: bool, uid: Option<libc::uid_t>) -> Vec<kinfo_proc> {
    let proc_selector = if uid.is_some() {
        KERN_PROC_UID
    } else {
        KERN_PROC_PROC
    } | if with_thread { KERN_PROC_INC_THREAD } else { 0 };
    let mut mib = vec![CTL_KERN, KERN_PROC, proc_selector];
    if let Some(uid) = uid {
        mib.push(uid as libc::c_int);
    }
    let mut size = 0usize;
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
        || size == 0
    {
        return Vec::new();
    }

    let count = size.div_ceil(std::mem::size_of::<kinfo_proc>());
    let mut processes: Vec<kinfo_proc> = vec![unsafe { std::mem::zeroed() }; count];
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            processes.as_mut_ptr() as *mut c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
    {
        return Vec::new();
    }
    processes.truncate(size / std::mem::size_of::<kinfo_proc>());
    processes
}

fn row_key(proc: &kinfo_proc, with_thread: bool, process_pids: &mut HashSet<i32>) -> (i64, i64) {
    let is_thread = with_thread && !process_pids.insert(proc.ki_pid);
    if is_thread {
        (thread_key(proc.ki_tid as u64), proc.ki_pid as i64)
    } else {
        (proc.ki_pid as i64, proc.ki_ppid as i64)
    }
}

pub fn collect_proc(
    interval: Duration,
    with_thread: bool,
    _procfs_path: &Option<PathBuf>,
    filter: ShowFilter,
) -> Vec<ProcessInfo> {
    let mut base_procs = HashMap::new();
    let mut ret = Vec::new();
    // The *effective* uid, because that is the one `KERN_PROC_UID` compares:
    // the header calls it "by effective uid" and the kernel tests
    // `p_ucred->cr_uid`. Handing it the real uid instead asks a different
    // question - for the processes whose effective uid happens to equal *our
    // real* uid - and that only looks right while the two are equal. They come
    // apart for a setuid `procs`, where the real uid is still the caller's
    // while the effective one is root's. (`KERN_PROC_RUID` is the selector that
    // matches the real uid.)
    //
    // This also keeps the filter agreeing with the User column, which shows
    // `ki_uid`, the effective uid.
    let current_uid = uzers::get_effective_uid();
    let mut process_pids = HashSet::new();

    let uid = (!filter.other_users).then_some(current_uid);
    for proc in get_processes(with_thread, uid) {
        if !filter.kthread && proc.ki_flag & (P_KPROC as i64) != 0 {
            continue;
        }

        let time = Instant::now();
        let (key, _) = row_key(&proc, with_thread, &mut process_pids);
        base_procs.insert(key, (proc, time));
    }

    thread::sleep(interval);
    let mut process_pids = HashSet::new();

    for proc in get_processes(with_thread, uid) {
        if !filter.kthread && proc.ki_flag & (P_KPROC as i64) != 0 {
            continue;
        }

        let (key, ppid) = row_key(&proc, with_thread, &mut process_pids);
        if let Some((prev_proc, prev_time)) = base_procs.remove(&key) {
            let curr_time = Instant::now();
            let interval = curr_time - prev_time;

            let proc = ProcessInfo {
                base: ProcessInfoBase::new(key, ppid, interval),
                curr_proc: proc,
                prev_proc,
            };
            ret.push(proc);
        }
    }

    ret
}
