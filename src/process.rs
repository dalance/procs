#[cfg(target_os = "macos")]
use libproc::libproc::proc_pid::{self, BSDInfo, ProcType, TaskAllInfo};
#[cfg(target_os = "linux")]
use procfs::{Io, ProcResult, Process, Status};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};
#[cfg(target_os = "macos")]
use sysinfo::{Process, ProcessExt, System, SystemExt};

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
    pub bsd: BSDInfo,
}

#[cfg(target_os = "macos")]
pub fn collect_proc(_interval: Duration) -> Vec<ProcessInfo> {
    let mut ret = Vec::new();

    if let Ok(procs) = proc_pid::listpids(ProcType::ProcAllPIDS) {
        for p in procs {
            if let Ok(curr_proc) = proc_pid::pidinfo::<TaskAllInfo>(p as i32, 0) {
                if let Ok(bsd) = proc_pid::pidinfo::<BSDInfo>(p as i32, 1) {
                    let proc = ProcessInfo { curr_proc, bsd };
                    ret.push(proc);
                }
            }
        }
    }

    ret
}
