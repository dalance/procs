use bsd_kvm::{Access, KernProc, Kvm, Process};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

pub struct ProcessInfo {
    pub pid: i32,
    pub ppid: i32,
    pub curr_proc: Process,
    pub prev_proc: Process,
    pub interval: Duration,
}

pub fn collect_proc(
    interval: Duration,
    _with_thread: bool,
    _show_kthreads: bool,
    _procfs_path: &Option<PathBuf>,
) -> Vec<ProcessInfo> {
    let mut base_procs = HashMap::new();
    let mut ret = Vec::new();

    let kvm = Kvm::open(None, Some("/dev/null"), Access::ReadOnly);
    if let Ok(mut kvm) = kvm {
        for proc in kvm.get_process(KernProc::Proc, 0) {
            let time = Instant::now();
            base_procs.insert(proc.info.pid, (proc.clone(), time));
        }

        thread::sleep(interval);

        for proc in kvm.get_process(KernProc::Proc, 0) {
            let pid = proc.info.pid;
            if let Some((prev_proc, prev_time)) = base_procs.remove(&pid) {
                let curr_time = Instant::now();
                let curr_proc = proc.clone();
                let interval = curr_time - prev_time;

                let proc = ProcessInfo {
                    pid: proc.info.pid,
                    ppid: proc.info.ppid,
                    curr_proc,
                    prev_proc,
                    interval,
                };
                ret.push(proc);
            }
        }
    }

    ret
}
