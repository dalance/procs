use crate::process::ShowFilter;
use bsd_kvm_sys::kinfo_proc;
use libc::{CTL_KERN, KERN_PROC, KERN_PROC_PROC, P_KPROC, c_void};
use std::collections::HashMap;
use std::path::PathBuf;
use std::ptr;
use std::thread;
use std::time::{Duration, Instant};

pub struct ProcessInfo {
    pub pid: i32,
    pub ppid: i32,
    pub curr_proc: kinfo_proc,
    pub prev_proc: kinfo_proc,
    pub interval: Duration,
}

fn get_processes() -> Vec<kinfo_proc> {
    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_PROC];
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
    let mut processes = vec![unsafe { std::mem::zeroed() }; count];
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

pub fn collect_proc(
    interval: Duration,
    _with_thread: bool,
    _procfs_path: &Option<PathBuf>,
    filter: ShowFilter,
) -> Vec<ProcessInfo> {
    let mut base_procs = HashMap::new();
    let mut ret = Vec::new();
    let current_uid = uzers::get_current_uid();

    for proc in get_processes() {
        if !filter.kthread && proc.ki_flag & (P_KPROC as i64) != 0 {
            continue;
        }
        // kvm hands over every process at once, so there is no per-process
        // work to save here - but dropping one avoids cloning it twice.
        if !filter.other_users && proc.ki_uid != current_uid {
            continue;
        }

        let time = Instant::now();
        base_procs.insert(proc.ki_pid, (proc, time));
    }

    thread::sleep(interval);

    for proc in get_processes() {
        if !filter.kthread && proc.ki_flag & (P_KPROC as i64) != 0 {
            continue;
        }
        if !filter.other_users && proc.ki_uid != current_uid {
            continue;
        }

        let pid = proc.ki_pid;
        if let Some((prev_proc, prev_time)) = base_procs.remove(&pid) {
            let curr_time = Instant::now();
            let interval = curr_time - prev_time;

            let proc = ProcessInfo {
                pid,
                ppid: proc.ki_ppid,
                curr_proc: proc,
                prev_proc,
                interval,
            };
            ret.push(proc);
        }
    }

    ret
}
