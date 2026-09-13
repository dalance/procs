use crate::process::{ProcessInfoBase, ShowFilter, thread_key};
use procfs::ProcError;
use procfs::ProcessCGroup;
use procfs::process::{FDInfo, Io, Process, Stat, StatFlags, Status, TasksIter};
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

pub enum ProcessTask {
    Process {
        stat: Stat,
        owner: u32,
        proc: Process,
    },
    Task {
        stat: Stat,
        owner: u32,
    },
}

impl ProcessTask {
    pub fn stat(&self) -> &Stat {
        match self {
            ProcessTask::Process { stat: x, .. } => x,
            ProcessTask::Task { stat: x, .. } => x,
        }
    }

    pub fn cmdline(&self) -> Result<Vec<String>, ProcError> {
        match self {
            ProcessTask::Process { proc: x, .. } => x.cmdline(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn cgroups(&self) -> Result<Vec<ProcessCGroup>, ProcError> {
        match self {
            ProcessTask::Process { proc: x, .. } => x.cgroups().map(|x| x.0),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn fd(&self) -> Result<Vec<FDInfo>, ProcError> {
        match self {
            ProcessTask::Process { proc: x, .. } => x.fd()?.collect(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn loginuid(&self) -> Result<u32, ProcError> {
        match self {
            ProcessTask::Process { proc: x, .. } => x.loginuid(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }

    pub fn owner(&self) -> u32 {
        match self {
            ProcessTask::Process { owner: x, .. } => *x,
            ProcessTask::Task { owner: x, .. } => *x,
        }
    }

    pub fn wchan(&self) -> Result<String, ProcError> {
        match self {
            ProcessTask::Process { proc: x, .. } => x.wchan(),
            _ => Err(ProcError::Other("not supported".to_string())),
        }
    }
}

pub struct ProcessInfo {
    /// The part of a row every platform has - the row key, the parent and the
    /// sampling window. The `Deref` below hands it out, so a column goes on
    /// writing `proc.pid` without reaching for `base`.
    ///
    /// The key is a process id when it is positive and the negated thread id
    /// of a thread row when it is negative: a task's id lives in the same
    /// namespace as a process id, so it can stand for a row of its own - but
    /// only while no process is using the same number, and a recycled id is
    /// indistinguishable from the one that held it before. Negating it says
    /// "this row is a thread" and leaves the Pid column free to print the id
    /// it came from.
    pub base: ProcessInfoBase,
    pub curr_proc: ProcessTask,
    pub prev_stat: Stat,
    pub curr_io: Option<Io>,
    pub prev_io: Option<Io>,
    pub curr_status: Option<Status>,
}

process_info_deref!();

/// Hands the processes out one at a time instead of collecting them first.
///
/// A `ProcessInfo` owns an open `/proc/<pid>` directory, so returning the whole
/// listing as a `Vec` means holding one descriptor per process at once. The
/// caller consumes each entry before the next one is read (see `View::new`),
/// which keeps the peak at a single descriptor no matter how many processes
/// there are - otherwise a low `RLIMIT_NOFILE` makes the second pass fail
/// halfway and the processes it could not open disappear without a word.
pub fn collect_proc(
    interval: Duration,
    with_thread: bool,
    procfs_path: &Option<PathBuf>,
    filter: ShowFilter,
) -> impl Iterator<Item = ProcessInfo> + '_ {
    let mut base_procs = Vec::new();
    let mut base_tasks = HashMap::new();

    let all_proc = if let Some(x) = procfs_path {
        procfs::process::all_processes_with_root(x)
    } else {
        procfs::process::all_processes()
    };

    if let Ok(all_proc) = all_proc {
        for proc in all_proc.flatten() {
            if let Ok(stat) = proc.stat() {
                // The owner is read only when something is compared against
                // it: with no uid to filter on, it is a `/proc/<pid>` open per
                // process for nothing.
                let owner = if filter.userid.is_some() {
                    proc.uid().ok()
                } else {
                    None
                };
                if let Some(userid) = filter.userid
                    && owner != Some(userid)
                {
                    continue;
                }

                let io = proc.io().ok();
                let time = Instant::now();
                if with_thread && let Ok(iter) = proc.tasks() {
                    collect_task(iter, &mut base_tasks);
                }
                base_procs.push((proc.pid(), stat, io, time));
            }
        }
    }

    thread::sleep(interval);

    base_procs
        .into_iter()
        .filter_map(move |(pid, prev_stat, prev_io, prev_time)| {
            let curr_proc = crate::util::process_new(pid.into(), procfs_path).ok()?;
            let curr_stat = curr_proc.stat().ok()?;

            if prev_stat.starttime != curr_stat.starttime {
                // Pid recycled
                return None;
            }

            let curr_owner = curr_proc.uid().ok()?;

            if let Some(userid) = filter.userid
                && curr_owner != userid
            {
                return None;
            }

            let curr_io = curr_proc.io().ok();
            let curr_status = curr_proc.status().ok();
            let curr_time = Instant::now();
            let interval = curr_time - prev_time;
            let ppid = curr_stat.ppid;

            if !filter.kthread
                && curr_stat
                    .flags()
                    .unwrap_or(StatFlags::empty())
                    .contains(StatFlags::PF_KTHREAD)
            {
                return None;
            }

            let mut curr_tasks = HashMap::new();
            if with_thread && let Ok(iter) = curr_proc.tasks() {
                collect_task(iter, &mut curr_tasks);
            }

            let curr_proc = ProcessTask::Process {
                stat: curr_stat,
                owner: curr_owner,
                proc: curr_proc,
            };

            let proc = ProcessInfo {
                base: ProcessInfoBase::new(pid as i64, ppid as i64, interval),
                curr_proc,
                prev_stat,
                curr_io,
                prev_io,
                curr_status,
            };

            let mut tasks = Vec::new();

            for (tid, (pid, curr_stat, curr_status, curr_io)) in curr_tasks {
                if let Some((_, prev_stat, _, prev_io)) = base_tasks.remove(&tid) {
                    let proc = ProcessInfo {
                        base: ProcessInfoBase::new(thread_key(tid as u64), pid as i64, interval),
                        curr_proc: ProcessTask::Task {
                            stat: curr_stat,
                            owner: curr_owner,
                        },
                        prev_stat,
                        curr_io,
                        prev_io,
                        curr_status,
                    };
                    tasks.push(proc);
                }
            }

            Some(std::iter::once(proc).chain(tasks))
        })
        .flatten()
}

#[allow(clippy::type_complexity)]
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
