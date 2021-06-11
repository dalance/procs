use procfs::process::{FDInfo, Io, Process, Stat, Status, TasksIter};
use procfs::{ProcError, ProcessCgroup};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

pub enum ProcessTask {
    Process(Process),
    Task { stat: Stat, owner: u32 },
}

impl ProcessTask {
    pub fn stat(&self) -> &Stat {
        match self {
            ProcessTask::Process(x) => &x.stat,
            ProcessTask::Task { stat: x, owner: _ } => x,
        }
    }

    pub fn cmdline(&self) -> Result<Vec<String>, ProcError> {
        match self {
            ProcessTask::Process(x) => x.cmdline(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn cgroups(&self) -> Result<Vec<ProcessCgroup>, ProcError> {
        match self {
            ProcessTask::Process(x) => x.cgroups(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn fd(&self) -> Result<Vec<FDInfo>, ProcError> {
        match self {
            ProcessTask::Process(x) => x.fd(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn loginuid(&self) -> Result<u32, ProcError> {
        match self {
            ProcessTask::Process(x) => x.loginuid(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn owner(&self) -> u32 {
        match self {
            ProcessTask::Process(x) => x.owner,
            ProcessTask::Task { stat: _, owner: x } => *x,
        }
    }

    pub fn wchan(&self) -> Result<String, ProcError> {
        match self {
            ProcessTask::Process(x) => x.wchan(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }
}

pub struct ProcessInfo {
    pub pid: i32,
    pub ppid: i32,
    pub curr_proc: ProcessTask,
    pub prev_proc: ProcessTask,
    pub curr_io: Option<Io>,
    pub prev_io: Option<Io>,
    pub curr_status: Option<Status>,
    pub interval: Duration,
}

pub fn collect_proc(interval: Duration, with_thread: bool) -> Vec<ProcessInfo> {
    let mut base_procs = Vec::new();
    let mut base_tasks = HashMap::new();
    let mut ret = Vec::new();

    if let Ok(all_proc) = procfs::process::all_processes() {
        for proc in all_proc {
            let io = proc.io().ok();
            let time = Instant::now();
            if with_thread {
                if let Ok(iter) = proc.tasks() {
                    collect_task(iter, &mut base_tasks);
                }
            }
            base_procs.push((proc.pid(), proc, io, time));
        }
    }

    thread::sleep(interval);

    for (pid, prev_proc, prev_io, prev_time) in base_procs {
        let curr_proc = if let Ok(proc) = Process::new(pid) {
            proc
        } else {
            prev_proc.clone()
        };
        let curr_io = curr_proc.io().ok();
        let curr_status = curr_proc.status().ok();
        let curr_time = Instant::now();
        let interval = curr_time - prev_time;
        let ppid = curr_proc.stat.ppid;
        let owner = curr_proc.owner;

        let mut curr_tasks = HashMap::new();
        if with_thread {
            if let Ok(iter) = curr_proc.tasks() {
                collect_task(iter, &mut curr_tasks);
            }
        }

        let curr_proc = ProcessTask::Process(curr_proc);
        let prev_proc = ProcessTask::Process(prev_proc);

        let proc = ProcessInfo {
            pid,
            ppid,
            curr_proc,
            prev_proc,
            curr_io,
            prev_io,
            curr_status,
            interval,
        };

        ret.push(proc);

        for (tid, (pid, curr_stat, curr_status, curr_io)) in curr_tasks {
            if let Some((_, prev_stat, _, prev_io)) = base_tasks.remove(&tid) {
                let proc = ProcessInfo {
                    pid: tid,
                    ppid: pid,
                    curr_proc: ProcessTask::Task {
                        stat: curr_stat,
                        owner,
                    },
                    prev_proc: ProcessTask::Task {
                        stat: prev_stat,
                        owner,
                    },
                    curr_io,
                    prev_io,
                    curr_status,
                    interval,
                };
                ret.push(proc);
            }
        }
    }

    ret
}

fn collect_task(iter: TasksIter, map: &mut HashMap<i32, (i32, Stat, Option<Status>, Option<Io>)>) {
    for task in iter {
        let task = if let Ok(x) = task {
            x
        } else {
            continue;
        };
        if task.tid != task.pid {
            let stat = if let Ok(x) = task.stat() {
                x
            } else {
                continue;
            };
            let status = task.status().ok();
            let io = task.io().ok();
            map.insert(task.tid, (task.pid, stat, status, io));
        }
    }
}
