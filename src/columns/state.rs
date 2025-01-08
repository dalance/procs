use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

pub struct State {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
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
        let mut state = 7;
        for t in &proc.curr_threads {
            let s = match t.pth_run_state {
                1 => 1, // TH_STATE_RUNNING
                2 => 5, // TH_STATE_STOPPED
                3 => {
                    if t.pth_sleep_time > 20 {
                        4
                    } else {
                        3
                    }
                } // TH_STATE_WAITING
                4 => 2, // TH_STATE_UNINTERRUPTIBLE
                5 => 6, // TH_STATE_HALTED
                _ => 7,
            };
            state = cmp::min(s, state);
        }
        let state = match state {
            0 => "",
            1 => "R",
            2 => "U",
            3 => "S",
            4 => "I",
            5 => "T",
            6 => "H",
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
        let info = &proc.curr_proc.info;
        let flag = info.flag;
        let tdflags = info.tdflags;
        let cr_flags = info.cr_flags;
        let kiflag = info.kiflag;

        let mut state = match info.stat {
            libc::SSTOP => "T",
            libc::SSLEEP => {
                if (tdflags & libc::TDF_SINTR as i64) != 0 {
                    if info.slptime >= 20 {
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
            state.push_str("W");
        }
        if info.nice < libc::NZERO as i8 || info.pri.class == bsd_kvm_sys::PRI_REALTIME as u8 {
            state.push_str("<");
        }
        if info.nice > libc::NZERO as i8 || info.pri.class == bsd_kvm_sys::PRI_IDLE as u8 {
            state.push_str("N");
        }
        if (flag & libc::P_TRACED as i64) != 0 {
            state.push_str("X");
        }
        if (flag & libc::P_WEXIT as i64) != 0 && info.stat != libc::SZOMB as std::os::raw::c_char {
            state.push_str("E");
        }
        if (flag & libc::P_PPWAIT as i64) != 0 {
            state.push_str("V");
        }
        if (flag & libc::P_SYSTEM as i64) != 0 || info.lock > 0 {
            state.push_str("L");
        }
        if (cr_flags & libc::KI_CRF_CAPABILITY_MODE as u32) != 0 {
            state.push_str("C");
        }
        if (kiflag & libc::KI_SLEADER as i64) != 0 {
            state.push_str("s");
        }
        if (flag & libc::P_CONTROLT as i64) != 0 && info.pgid == info.tpgid {
            state.push_str("+");
        }
        if (flag & libc::P_JAILED as i64) != 0 {
            state.push_str("J");
        }
        let fmt_content = state;
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
