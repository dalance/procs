use procfs::{Io, ProcResult, Process, Status};
use std::thread;
use std::time::{Duration, Instant};

pub struct ProcessInfo {
    pub pid: i32,
    pub curr_proc: Process,
    pub prev_proc: Process,
    pub curr_io: ProcResult<Io>,
    pub prev_io: ProcResult<Io>,
    pub curr_status: ProcResult<Status>,
    pub interval: Duration,
}

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
            pid,
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
