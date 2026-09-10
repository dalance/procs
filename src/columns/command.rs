use crate::process::ProcessInfo;
use crate::util::sanitize_control_chars;
use crate::{column_default, Column};
#[cfg(target_os = "freebsd")]
use libc::{CTL_KERN, KERN_PROC, KERN_PROC_ARGS, c_void};
use std::cmp;
use std::collections::HashMap;
#[cfg(target_os = "freebsd")]
use std::ptr;

#[cfg(target_os = "freebsd")]
pub(crate) fn get_process_args(pid: i32) -> Vec<String> {
    let mut mib = [CTL_KERN, KERN_PROC, KERN_PROC_ARGS, pid];
    let mut size = 0usize;
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
        || size == 0
    {
        return Vec::new();
    }

    let mut bytes = vec![0u8; size];
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            bytes.as_mut_ptr() as *mut c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    } != 0
    {
        return Vec::new();
    }
    bytes.truncate(size);
    bytes
        .split(|byte| *byte == 0)
        .filter(|arg| !arg.is_empty())
        .map(|arg| String::from_utf8_lossy(arg).into_owned())
        .collect()
}

pub struct Command {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

impl Command {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Command"));
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
impl Column for Command {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(cmd) = &proc.curr_proc.cmdline() {
            if !cmd.is_empty() {
                let mut cmd = cmd
                    .iter()
                    .cloned()
                    .map(|mut x| {
                        x.push(' ');
                        x
                    })
                    .collect::<String>();
                cmd.pop();
                cmd
            } else {
                format!("[{}]", proc.curr_proc.stat().comm)
            }
        } else {
            proc.curr_proc.stat().comm.clone()
        };
        let fmt_content = sanitize_control_chars(&fmt_content);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for Command {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(path) = &proc.curr_path {
            if !path.cmd.is_empty() {
                let mut cmd = path
                    .cmd
                    .iter()
                    .cloned()
                    .map(|mut x| {
                        x.push(' ');
                        x
                    })
                    .collect::<String>();
                cmd.pop();
                cmd
            } else if proc.tid.is_some() {
                // A thread has no command line of its own, so show the name
                // the collector settled on - the thread's own when it has one,
                // the owning process's otherwise - bracketed like its pid.
                format!("[{}]", path.name)
            } else {
                String::from("")
            }
        } else {
            String::from("")
        };
        let fmt_content = sanitize_control_chars(&fmt_content);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "windows")]
impl Column for Command {
    fn add(&mut self, proc: &ProcessInfo) {
        // Show the command line when present, otherwise fall back to the image
        // name (e.g. System, Idle, or protected processes with no command line).
        let fmt_content = proc
            .command
            .as_ref()
            .filter(|c| !c.is_empty())
            .cloned()
            .unwrap_or_else(|| proc.file_name.clone());
        let fmt_content = sanitize_control_chars(&fmt_content);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "freebsd")]
impl Column for Command {
    fn add(&mut self, proc: &ProcessInfo) {
        let args = get_process_args(proc.pid);
        let command = if args.is_empty() {
            let comm = crate::util::ptr_to_cstr(proc.curr_proc.ki_comm.as_ref());
            if let Ok(comm) = comm {
                format!("[{}]", comm.to_string_lossy())
            } else {
                String::from("")
            }
        } else {
            let mut x = String::from("");
            for arg in &args {
                x.push_str(&arg);
                x.push_str(" ");
            }
            x
        };
        let fmt_content = command;
        let fmt_content = sanitize_control_chars(&fmt_content);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}
