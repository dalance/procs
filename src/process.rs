#[cfg(target_os = "macos")]
use libc::{c_char, c_int, c_void, gid_t, int32_t, size_t, uid_t, uint32_t, uint64_t};
#[cfg(target_os = "macos")]
use libproc::libproc::proc_pid::{
    self, BSDInfo, PIDInfo, PidInfoFlavor, ProcType, TaskAllInfo, ThreadInfo,
};
#[cfg(target_os = "linux")]
use procfs::{Io, ProcResult, Process, Status};
#[cfg(target_os = "macos")]
use std::ffi::OsStr;
#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "linux")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::{Duration, Instant};

#[cfg(target_os = "linux")]
pub struct ProcessInfo {
    pub pid: i32,
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

#[cfg(target_os = "macos")]
pub struct ProcessInfo {
    pub pid: i32,
    pub curr_proc: TaskAllInfo,
    pub threads: Vec<ThreadInfo2>,
    pub path_info: Option<PathInfo>,
}

#[cfg(target_os = "macos")]
pub fn collect_proc(_interval: Duration) -> Vec<ProcessInfo> {
    let mut ret = Vec::new();
    let arg_max = get_arg_max();

    if let Ok(procs) = proc_pid::listpids(ProcType::ProcAllPIDS) {
        for p in procs {
            if let Ok(curr_proc) = proc_pid::pidinfo::<TaskAllInfo>(p as i32, 0) {
                let threadids = proc_pid::listthreads(
                    curr_proc.pbsd.pbi_pid as i32,
                    curr_proc.ptinfo.pti_threadnum,
                );
                let mut threads = Vec::new();
                if let Ok(threadids) = threadids {
                    for t in threadids {
                        if let Ok(thread) = proc_pid::pidinfo::<ThreadInfo2>(p as i32, t) {
                            threads.push(thread);
                        }
                    }
                }
                let pathinfo = get_path_info(p as i32, arg_max);

                let proc = ProcessInfo {
                    pid: p as i32,
                    curr_proc,
                    threads,
                    path_info,
                };
                ret.push(proc);
            }
        }
    }

    ret
}

#[cfg(target_os = "macos")]
fn get_arg_max() -> size_t {
    let mut mib: [c_int; 2] = [libc::CTL_KERN, libc::KERN_ARGMAX];
    let mut arg_max = 0i32;
    let mut size = ::std::mem::size_of::<c_int>();
    unsafe {
        while libc::sysctl(
            mib.as_mut_ptr(),
            2,
            (&mut arg_max) as *mut i32 as *mut c_void,
            &mut size,
            ::std::ptr::null_mut(),
            0,
        ) == -1
        {}
    }
    arg_max as size_t
}

#[cfg(target_os = "macos")]
struct PathInfo {
    exe: PathBuf,
    name: String,
    root: PathBuf,
    cmd: Vec<String>,
    env: Vec<String>,
}

#[cfg(target_os = "macos")]
unsafe fn get_unchecked_str(cp: *mut u8, start: *mut u8) -> String {
    let len = cp as usize - start as usize;
    let part = Vec::from_raw_parts(start, len, len);
    let tmp = String::from_utf8_unchecked(part.clone());
    ::std::mem::forget(part);
    tmp
}

#[cfg(target_os = "macos")]
fn get_cmd_env(pid: i32, size: size_t) -> Option<PathInfo> {
    let mut proc_args = Vec::with_capacity(size as usize);
    let ptr: *mut u8 = proc_args.as_mut_slice().as_mut_ptr();

    let mut mib: [c_int; 3] = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as c_int];

    if libc::sysctl(
        mib.as_mut_ptr(),
        3,
        ptr as *mut c_void,
        &mut size,
        ::std::ptr::null_mut(),
        0,
    ) != -1
    {
        let mut n_args: c_int = 0;
        libc::memcpy(
            (&mut n_args) as *mut c_int as *mut c_void,
            ptr as *const c_void,
            ::std::mem::size_of::<c_int>(),
        );
        let mut cp = ptr.offset(::std::mem::size_of::<c_int>() as isize);
        let mut start = cp;
        if cp < ptr.offset(size as isize) {
            while cp < ptr.offset(size as isize) && *cp != 0 {
                cp = cp.offset(1);
            }
            let exe = Path::new(get_unchecked_str(cp, start).as_str()).to_path_buf();
            let name = exe
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .unwrap_or_else(|| "")
                .to_owned();
            let mut need_root = true;
            let mut root = Default::default();
            if exe.is_absolute() {
                if let Some(parent) = exe.parent() {
                    root = parent.to_path_buf();
                    need_root = false;
                }
            }
            while cp < ptr.offset(size as isize) && *cp == 0 {
                cp = cp.offset(1);
            }
            start = cp;
            let mut c = 0;
            let mut cmd = Vec::new();
            while c < n_args && cp < ptr.offset(size as isize) {
                if *cp == 0 {
                    c += 1;
                    cmd.push(get_unchecked_str(cp, start));
                    start = cp.offset(1);
                }
                cp = cp.offset(1);
            }
            start = cp;
            let mut env = Vec::new();
            while cp < ptr.offset(size as isize) {
                if *cp == 0 {
                    if cp == start {
                        break;
                    }
                    env.push(get_unchecked_str(cp, start));
                    start = cp.offset(1);
                }
                cp = cp.offset(1);
            }
            if need_root == true {
                for env in env.iter() {
                    if env.starts_with("PATH=") {
                        root = Path::new(&env[6..]).to_path_buf();
                        break;
                    }
                }
            }

            Some(PathInfo {
                exe,
                name,
                root,
                cmd,
                env,
            })
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
const MAXTHREADNAMESIZE: usize = 64;

#[cfg(target_os = "macos")]
#[repr(C)]
pub struct ThreadInfo2 {
    pub pth_user_time: uint64_t,               // user run time
    pub pth_system_time: uint64_t,             // system run time
    pub pth_cpu_usage: int32_t,                // scaled cpu usage percentage
    pub pth_policy: int32_t,                   // scheduling policy in effect
    pub pth_run_state: int32_t,                // run state (see below)
    pub pth_flags: int32_t,                    // various flags (see below)
    pub pth_sleep_time: int32_t,               // number of seconds that thread
    pub pth_curpri: int32_t,                   // cur priority
    pub pth_priority: int32_t,                 // priority
    pub pth_maxpriority: int32_t,              // max priority
    pub pth_name: [c_char; MAXTHREADNAMESIZE], // thread name, if any
}

#[cfg(target_os = "macos")]
impl PIDInfo for ThreadInfo2 {
    fn flavor() -> PidInfoFlavor {
        PidInfoFlavor::ThreadInfo
    }
}

#[cfg(target_os = "macos")]
impl Default for ThreadInfo2 {
    fn default() -> ThreadInfo2 {
        ThreadInfo2 {
            pth_user_time: 0,
            pth_system_time: 0,
            pth_cpu_usage: 0,
            pth_policy: 0,
            pth_run_state: 0,
            pth_flags: 0,
            pth_sleep_time: 0,
            pth_curpri: 0,
            pth_priority: 0,
            pth_maxpriority: 0,
            pth_name: [0; MAXTHREADNAMESIZE],
        }
    }
}
