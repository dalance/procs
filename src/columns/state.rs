use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

#[cfg(target_os = "windows")]
use crate::process::{thread_state, wait_reason, ThreadState};

pub struct State {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, String>,
    width: usize,
}

impl State {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("State"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for State {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = format!("{}", proc.curr_proc.stat().state);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for State {
    fn add(&mut self, proc: &ProcessInfo) {
        let state = match proc.state {
            0 => "",
            1 => "R",
            2 => "U",
            3 => "S",
            4 => "I",
            5 => "T",
            6 => "H",
            8 => "Z",
            _ => "?",
        };
        let fmt_content = state.to_string();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for State {
    fn add(&mut self, proc: &ProcessInfo) {
        let info = &proc.curr_proc;
        let flag = info.ki_flag;
        let tdflags = info.ki_tdflags;
        let cr_flags = info.ki_cr_flags;
        let kiflag = info.ki_kiflag;

        let mut state = match info.ki_stat {
            libc::SSTOP => "T",
            libc::SSLEEP => {
                if (tdflags & libc::TDF_SINTR as i64) != 0 {
                    if info.ki_slptime >= 20 {
                        "I"
                    } else {
                        "S"
                    }
                } else {
                    "D"
                }
            }
            libc::SRUN | libc::SIDL => "R",
            libc::SWAIT => "W",
            libc::SLOCK => "L",
            libc::SZOMB => "Z",
            _ => "?",
        }
        .to_string();
        if (flag & libc::P_INMEM as i64) == 0 {
            state.push('W');
        }
        if info.ki_nice < libc::NZERO as i8
            || info.ki_pri.pri_class == crate::process::PRI_REALTIME
        {
            state.push('<');
        }
        if info.ki_nice > libc::NZERO as i8 || info.ki_pri.pri_class == crate::process::PRI_IDLE {
            state.push('N');
        }
        if (flag & libc::P_TRACED as i64) != 0 {
            state.push('X');
        }
        if (flag & libc::P_WEXIT as i64) != 0
            && info.ki_stat != libc::SZOMB as std::os::raw::c_char
        {
            state.push('E');
        }
        if (flag & libc::P_PPWAIT as i64) != 0 {
            state.push('V');
        }
        if (flag & libc::P_SYSTEM as i64) != 0 || info.ki_lock > 0 {
            state.push('L');
        }
        if (cr_flags & libc::KI_CRF_CAPABILITY_MODE as u32) != 0 {
            state.push('C');
        }
        if (kiflag & libc::KI_SLEADER as i64) != 0 {
            state.push('s');
        }
        if (flag & libc::P_CONTROLT as i64) != 0 && info.ki_pgid == info.ki_tpgid {
            state.push('+');
        }
        if (flag & libc::P_JAILED as i64) != 0 {
            state.push('J');
        }
        let fmt_content = state;
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

// ---------------------------------------------------------------------------
// Windows
//
// There is no per-process state to read: `SYSTEM_PROCESS_INFORMATION` only
// counts the threads, while `SYSTEM_THREAD_INFORMATION` is what carries the
// scheduler state. A row is therefore described by the threads behind it.
// ---------------------------------------------------------------------------

/// The letter `ps` prints for one thread's state.
///
/// `WaitReason` is only read while the thread is in `WAITING`: it is a leftover
/// from the last wait otherwise, and a thread that is runnable again still
/// carries the reason it sleeps on.
#[cfg(target_os = "windows")]
fn thread_state_char(state: ThreadState) -> char {
    use thread_state::*;

    match state.state {
        // Runnable, or on its way there: `TRANSITION` only means the kernel
        // stack is still being brought in.
        INITIALIZED | READY | RUNNING | STANDBY | TRANSITION | DEFERRED_READY => 'R',
        WAITING | GATE_WAIT | WAITING_FOR_PROCESS_IN_SWAP => match state.wait_reason {
            wait_reason::SUSPENDED | wait_reason::WR_SUSPENDED => 'T',
            _ => 'S',
        },
        TERMINATED => 'Z',
        _ => '?',
    }
}

/// The state letter of one row.
///
/// `state` is the most active thread's raw state, reduced by the snapshot
/// layer; `thread_count` is the `NumberOfThreads` the snapshot reported. It
/// is what separates a process that has no threads left from one whose
/// records simply could not be read.
#[cfg(target_os = "windows")]
fn state_char(state: Option<ThreadState>, thread_count: i32) -> char {
    match state {
        // No thread left at all: the process has terminated and is only still
        // listed because handles to it remain open.
        None if thread_count <= 0 => 'Z',
        // Threads were claimed but none could be read: not enough to call it.
        None => '?',
        Some(state) => thread_state_char(state),
    }
}

#[cfg(target_os = "windows")]
impl Column for State {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = String::from(state_char(proc.state, proc.thread));
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
