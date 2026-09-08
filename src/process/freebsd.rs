use crate::process::OnlyFilter;
use bsd_kvm::{Access, KernProc, Kvm, Process};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;
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
    show_kthreads: bool,
    _procfs_path: &Option<PathBuf>,
    only: OnlyFilter,
) -> Vec<ProcessInfo> {
    let mut base_procs = HashMap::new();
    let mut ret = Vec::new();

    let kvm = Kvm::open(None, Some("/dev/null"), Access::ReadOnly);
    if let Ok(mut kvm) = kvm {
        for proc in kvm.get_process(KernProc::Proc, 0) {
            if !show_kthreads && proc.info.flag & 0x00000004 /* P_KPROC */ != 0 {
                continue;
            }
            // kvm hands over every process at once, so there is no per-process
            // work to save here - but dropping one avoids cloning it twice.
            if !only.matches(proc.info.sid, proc.info.uid) {
                continue;
            }

            let time = Instant::now();
            base_procs.insert(proc.info.pid, (proc.clone(), time));
        }

        thread::sleep(interval);

        for proc in kvm.get_process(KernProc::Proc, 0) {
            let pid = proc.info.pid;
            if !only.matches(proc.info.sid, proc.info.uid) {
                continue;
            }
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

// ---------------------------------------------------------------------------
// Current user / session
// ---------------------------------------------------------------------------

impl OnlyFilter {
    /// Whether a process of `session` owned by `uid` is kept.
    fn matches(self, session: i32, uid: u32) -> bool {
        if self.current_session {
            match current_session() {
                // `getsid` failed, so there is nothing to compare against -
                // better to keep than to hide everything.
                None => {}
                Some(current) if current != session => return false,
                _ => {}
            }
        }
        if self.current_user && uid != current_uid() {
            return false;
        }
        true
    }
}

/// The uid `procs` runs as.
fn current_uid() -> u32 {
    uzers::get_current_uid()
}

/// The session id of `procs` itself, queried once.
fn current_session() -> Option<i32> {
    static CURRENT: OnceLock<Option<i32>> = OnceLock::new();
    *CURRENT.get_or_init(|| {
        let sid = unsafe { libc::getsid(0) };
        (sid >= 0).then_some(sid)
    })
}
