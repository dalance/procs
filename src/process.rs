#[cfg(target_os = "macos")]
use libc::{c_char, c_int, c_void, gid_t, int32_t, uid_t, uint32_t, uint64_t};
#[cfg(target_os = "macos")]
use libproc::libproc::proc_pid::{
    self, BSDInfo, PIDInfo, PidInfoFlavor, ProcType, TaskAllInfo, ThreadInfo,
};
#[cfg(target_os = "linux")]
use procfs::{Io, ProcResult, Process, Status};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
pub struct ProcessInfo {
    pub curr_proc: Process,
    pub prev_proc: Process,
    pub curr_io: ProcResult<Io>,
    pub prev_io: ProcResult<Io>,
    pub curr_status: ProcResult<Status>,
    pub interval: Duration,
}

#[cfg(target_os = "linux")]
pub fn collect_proc(interval: Duration) -> Vec<ProcessInfo> {
    let mut base_procs = Vec::new();
    let mut ret = Vec::new();

    for proc in procfs::all_processes() {
        let io = proc.io();
        let time = Instant::now();
        base_procs.push((proc.pid(), proc, io, time));
    }

    thread::sleep(interval);

    for (pid, prev_proc, prev_io, prev_time) in base_procs {
        let curr_proc = if let Ok(proc) = Process::new(pid) {
            proc
        } else {
            prev_proc.clone()
        };
        let curr_io = curr_proc.io();
        let curr_status = curr_proc.status();
        let curr_time = Instant::now();
        let interval = curr_time - prev_time;

        let proc = ProcessInfo {
            curr_proc,
            prev_proc,
            curr_io,
            prev_io,
            curr_status,
            interval,
        };

        ret.push(proc);
    }

    ret
}

#[cfg(target_os = "macos")]
pub struct ProcessInfo {
    pub curr_proc: TaskAllInfo,
    pub threads: Vec<ThreadInfo2>,
}

#[cfg(target_os = "macos")]
pub fn collect_proc(_interval: Duration) -> Vec<ProcessInfo> {
    let mut ret = Vec::new();

    if let Ok(procs) = proc_pid::listpids(ProcType::ProcAllPIDS) {
        for p in procs {
            if let Ok(curr_proc) = proc_pid::pidinfo::<TaskAllInfo>(p as i32, 0) {
                let threadids = proc_pid::listthreads(
                    curr_proc.pbsd.pbi_pid as i32,
                    curr_proc.ptinfo.pti_threadnum as usize,
                );
                let mut threads = Vec::new();
                if let Ok((len, threadids)) = threadids {
                    dbg!((len, threadids.len()));
                    for t in threadids {
                        if let Ok(thread) = proc_pid::pidinfo::<ThreadInfo2>(p as i32, t) {
                            threads.push(thread);
                        }
                    }
                }
                let proc = ProcessInfo { curr_proc, threads };
                ret.push(proc);
            }
        }
    }

    ret
}

const MAXTHREADNAMESIZE: usize = 64;

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct ThreadInfo2 {
    pub pth_user_time: uint64_t,               // user run time
    pub pth_system_time: uint64_t,             // system run time
    pub pth_cpu_usage: int32_t,                // scaled cpu usage percentage
    pub pth_policy: int32_t,                   // scheduling policy in effect
    pub pth_run_state: int32_t,                // run state (see below)
    pub pth_flags: int32_t,                    // various flags (see below)
    pub pth_sleep_time: int32_t,               // number of seconds that thread
    pub pth_curpri: int32_t,                   // cur priority
    pub pth_priority: int32_t,                 // priority
    pub pth_maxpriority: int32_t,              // max priority
    pub pth_name: [c_char; MAXTHREADNAMESIZE], // thread name, if any
}

#[cfg(target_os = "macos")]
impl PIDInfo for ThreadInfo2 {
    fn flavor() -> PidInfoFlavor {
        PidInfoFlavor::ThreadInfo
    }
}

#[cfg(target_os = "macos")]
impl Default for ThreadInfo2 {
    fn default() -> ThreadInfo2 {
        ThreadInfo2 {
            pth_user_time: 0,
            pth_system_time: 0,
            pth_cpu_usage: 0,
            pth_policy: 0,
            pth_run_state: 0,
            pth_flags: 0,
            pth_sleep_time: 0,
            pth_curpri: 0,
            pth_priority: 0,
            pth_maxpriority: 0,
            pth_name: [0; MAXTHREADNAMESIZE],
        }
    }
}
