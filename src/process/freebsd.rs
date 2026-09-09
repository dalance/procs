use crate::process::ShowFilter;
use bsd_kvm::{Access, KernProc, Kvm, Process};
use std::os::raw::c_int;
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

const P_KPROC: c_int = 0x00000004;

pub fn collect_proc(
    interval: Duration,
    _with_thread: bool,
    _procfs_path: &Option<PathBuf>,
    filter: ShowFilter,
) -> Vec<ProcessInfo> {
    let mut base_procs = HashMap::new();
    let mut ret = Vec::new();
    let current_uid = uzers::get_current_uid();

    let kvm = Kvm::open(None, Some("/dev/null"), Access::ReadOnly);
    if let Ok(mut kvm) = kvm {
        for proc in kvm.get_process(KernProc::Proc, 0) {
            if !filter.kthread && proc.info.flag & P_KPROC != 0 {
                continue;
            }
            // kvm hands over every process at once, so there is no per-process
            // work to save here - but dropping one avoids cloning it twice.
            if !filter.other_users && proc.info.uid != current_uid {
                continue;
            }

            let time = Instant::now();
            base_procs.insert(proc.info.pid, (proc.clone(), time));
        }

        thread::sleep(interval);

        for proc in kvm.get_process(KernProc::Proc, 0) {
            if !filter.kthread && proc.info.flag & P_KPROC != 0 {
                continue;
            }
            if !filter.other_users && proc.info.uid != current_uid {
                continue;
            }

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
