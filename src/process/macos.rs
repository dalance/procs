use crate::process::{ProcessInfoBase, ShowFilter, thread_key};
use libc::{c_char, c_int, c_void, size_t};
use libproc::libproc::file_info::{ListFDs, ProcFDType, pidfdinfo};
use libproc::libproc::net_info::{InSockInfo, SocketFDInfo, SocketInfoKind, TcpSockInfo};
use libproc::libproc::pid_rusage::{RUsageInfoV2, pidrusage};
use libproc::libproc::proc_pid::{ListThreads, listpidinfo, pidinfo, pidpath};
use libproc::libproc::task_info::TaskInfo;
use libproc::libproc::thread_info::ThreadInfo;
use mach2::{boolean, vm_types};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::{cmp, io, mem, ptr, thread};

pub struct ProcessInfo {
    /// The part of a row every platform has - the row key, the parent and the
    /// sampling window. The `Deref` below hands it out, so a column goes on
    /// writing `proc.pid` without reaching for `base`.
    ///
    /// Threads and processes share the key because every column keys its
    /// contents by it. `PROC_PIDLISTTHREADS` hands out 64 bit ids that mean
    /// nothing on their own and would land on real pids (or on each other) if
    /// they were used as keys directly, so a thread row negates its id: real
    /// pids are never negative, and the negation of a thread id is unique to
    /// that thread. The Pid column turns it back into the id.
    pub base: ProcessInfoBase,
    pub curr_proc: kinfo_proc,
    // Kept for the same reason freebsd keeps one: the two-sample shape of
    // `collect_proc` is shared, even if no column reads the previous one.
    #[allow(dead_code)]
    pub prev_proc: kinfo_proc,
    pub curr_task: TaskInfo,
    pub prev_task: TaskInfo,
    pub curr_path: Option<PathInfo>,
    pub curr_udps: Vec<InSockInfo>,
    pub curr_tcps: Vec<TcpSockInfo>,
    pub curr_res: Option<RUsageInfoV2>,
    pub prev_res: Option<RUsageInfoV2>,
    pub state: i32,
}

process_info_deref!();

pub fn collect_proc(
    interval: Duration,
    with_thread: bool,
    _procfs_path: &Option<PathBuf>,
    filter: ShowFilter,
) -> Vec<ProcessInfo> {
    let mut base_procs = HashMap::new();
    let mut ret = Vec::new();
    let arg_max = get_arg_max();
    let current_uid = uzers::get_current_uid();

    let uid_filter = if filter.other_users {
        None
    } else {
        Some(current_uid)
    };

    if let Some(procs) = sysctl_procs(uid_filter) {
        for kp in procs {
            if !filter.kthread && kp.kp_proc.p_flag & P_SYSTEM != 0 {
                continue;
            }

            // The uid comes with the process table, so dropping a process here
            // saves the resource usage query and everything the second pass
            // does for it.

            let pid = kp.kp_proc.p_pid;
            let task = pidinfo::<TaskInfo>(pid, 0).unwrap_or(unsafe { mem::zeroed() });
            let res = pidrusage::<RUsageInfoV2>(pid).ok();
            let time = Instant::now();

            // The thread list is part of the first sample: the second pass
            // pairs each current thread with the one the first pass saw.
            let mut threads = Vec::new();
            if with_thread {
                let threadids = listpidinfo::<ListThreads>(pid, task.pti_threadnum as usize);
                if let Ok(threadids) = threadids {
                    threads.reserve(task.pti_threadnum as usize);
                    for t in threadids {
                        // `0` is what the kernel reports for a thread it has
                        // no id for. It is not a thread to ask about, and it
                        // would collide with every other one of them.
                        if t == 0 {
                            continue;
                        }
                        if let Ok(thread) = pidinfo::<ThreadInfo>(pid, t) {
                            threads.push((t, thread));
                        }
                    }
                }
            }
            base_procs.insert(pid, (kp, task, res, time, threads));
        }
    }

    thread::sleep(interval);

    for curr_proc in sysctl_procs(uid_filter).unwrap_or_default() {
        // Either the process is gone, or libproc was never willing to talk
        // about it: keeping the sample taken before the sleep is what makes
        // the deltas come out as zero instead of as a jump.
        let pid = curr_proc.kp_proc.p_pid;
        let (prev_proc, prev_task, prev_res, prev_time, prev_threads) =
            match base_procs.remove(&pid) {
                Some(data) => data,
                None => continue,
            };

        let curr_task = pidinfo::<TaskInfo>(pid, 0).unwrap_or(prev_task);

        // The command line, from whichever source is willing to give one:
        // `KERN_PROCARGS2` gives all of it, `proc_pidpath` still gives the
        // executable for processes it will not, and the process table always
        // carries a name.
        let curr_path = get_path_info(pid, arg_max)
            .filter(|path| !path.cmd.is_empty())
            .or_else(|| pidpath(pid).ok().and_then(path_info_of_exe))
            .or_else(|| path_info_of_exe(command_of_comm(&curr_proc.kp_proc.p_comm)));

        let mut curr_tcps = Vec::new();
        let mut curr_udps = Vec::new();

        // The process table carries no open file count, and the default
        // `RLIMIT_NOFILE` soft limit is 256: a buffer that size covers the
        // sockets the port columns care about.
        let fds = listpidinfo::<ListFDs>(pid, 256);
        if let Ok(fds) = fds {
            for fd in fds {
                if let ProcFDType::Socket = fd.proc_fdtype.into()
                    && let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd)
                {
                    match socket.psi.soi_kind.into() {
                        SocketInfoKind::In => {
                            if socket.psi.soi_protocol == libc::IPPROTO_UDP {
                                let info = unsafe { socket.psi.soi_proto.pri_in };
                                curr_udps.push(info);
                            }
                        }
                        SocketInfoKind::Tcp => {
                            let info = unsafe { socket.psi.soi_proto.pri_tcp };
                            curr_tcps.push(info);
                        }
                        _ => (),
                    }
                }
            }
        }

        let curr_res = pidrusage::<RUsageInfoV2>(pid).ok();

        let curr_time = Instant::now();
        let interval = curr_time - prev_time;
        let ppid = curr_proc.kp_eproc.e_ppid;
        let mut state = 7;

        // What a thread row falls back to when the thread itself is unnamed,
        // which is most of them: the name of the process it belongs to.
        let thread_owner = curr_path
            .as_ref()
            .filter(|path| !path.name.is_empty())
            .map(|path| path.name.clone())
            .unwrap_or_else(|| command_of_comm(&curr_proc.kp_proc.p_comm));

        let threadids = listpidinfo::<ListThreads>(pid, curr_task.pti_threadnum as usize);
        let mut threads = Vec::new();

        if let Ok(threadids) = threadids {
            threads.reserve(curr_task.pti_threadnum as usize);
            for tid in threadids {
                // See the first pass: `0` is not a thread id.
                if tid == 0 {
                    continue;
                }
                if let Ok(thread) = pidinfo::<ThreadInfo>(pid, tid) {
                    let tstate = match thread.pth_run_state {
                        1 => 1, // TH_STATE_RUNNING
                        2 => 5, // TH_STATE_STOPPED
                        3 => {
                            if thread.pth_sleep_time > 20 {
                                4
                            } else {
                                3
                            }
                        } // TH_STATE_WAITING
                        4 => 2, // TH_STATE_UNINTERRUPTIBLE
                        5 => 6, // TH_STATE_HALTED
                        _ => 7,
                    };
                    state = cmp::min(tstate, state);

                    if with_thread {
                        let prev_thread = prev_threads
                            .iter()
                            .find(|(ptid, _)| *ptid == tid)
                            .map(|&(_, t)| t)
                            .unwrap_or(unsafe { mem::zeroed() });

                        let mut task = unsafe { mem::zeroed::<TaskInfo>() };
                        task.pti_total_user = thread.pth_user_time;
                        task.pti_total_system = thread.pth_system_time;
                        task.pti_priority = thread.pth_priority;
                        task.pti_threadnum = 1;

                        let mut prev = unsafe { mem::zeroed::<TaskInfo>() };
                        prev.pti_total_user = prev_thread.pth_user_time;
                        prev.pti_total_system = prev_thread.pth_system_time;
                        prev.pti_priority = prev_thread.pth_priority;
                        prev.pti_threadnum = 1;

                        let name = String::from_utf8_lossy(
                            &thread
                                .pth_name
                                .iter()
                                .take_while(|&&c| c != 0)
                                .map(|&c| c as u8)
                                .collect::<Vec<u8>>(),
                        )
                        .into_owned();

                        let path = PathInfo {
                            name: if name.is_empty() {
                                thread_owner.clone()
                            } else {
                                name
                            },
                            exe: PathBuf::new(),
                            root: PathBuf::new(),
                            cmd: Vec::new(),
                            env: Vec::new(),
                        };

                        // Negating the id is what keeps the row key unique:
                        // a thread can then neither be mistaken for the
                        // process it belongs to nor take the place of another
                        // thread. `0` in particular stops being reachable,
                        // which matters because the tree column reads "a ppid
                        // that is not in the table" as the root of the tree -
                        // and 0 is the ppid of launchd.
                        let key = thread_key(tid);

                        threads.push(ProcessInfo {
                            base: ProcessInfoBase::new(key, pid as i64, interval),
                            curr_proc,
                            prev_proc,
                            curr_task: task,
                            prev_task: prev,
                            curr_path: Some(path),
                            curr_udps: Vec::new(),
                            curr_tcps: Vec::new(),
                            curr_res: None,
                            prev_res: None,
                            state: tstate,
                        });
                    }
                }
            }
        }

        ret.push(ProcessInfo {
            base: ProcessInfoBase::new(pid as i64, ppid as i64, interval),
            curr_proc,
            prev_proc,
            curr_task,
            prev_task,
            curr_path,
            curr_udps,
            curr_tcps,
            curr_res,
            prev_res,
            state,
        });
        ret.extend(threads);
    }

    ret
}

// ---------------------------------------------------------------------------
// The process table
// ---------------------------------------------------------------------------

/// `KERN_PROC_ALL` from `<sys/sysctl.h>`: the whole process table. `libc`
/// does not export the `KERN_PROC_*` values for macOS.
const KERN_PROC_ALL: c_int = 0;

/// `KERN_PROC_UID` from `<sys/sysctl.h>`: fetch only processes whose
/// effective UID matches the supplied value. `libc` does not export it.
const KERN_PROC_UID: c_int = 5;

/// `P_SYSTEM` from `<sys/proc.h>`: a process the kernel owns, the ones
/// `ShowFilter::kthread` hides. It lives in the `P_*` namespace of
/// `kinfo_proc::kp_proc::p_flag`.
const P_SYSTEM: c_int = 0x00000200;

/// Every process on the system, or only the processes of a single user if
/// `uid` is set.
///
/// `libproc` answers for the processes `procs` owns only: `pidinfo`,
/// `pidrusage` and the rest of it fail with `EPERM` for anything else, which
/// is why a listing built on them alone is a listing of the current user's
/// processes, whatever the `ShowFilter` says. `sysctl` has no such rule and
/// hands out one `kinfo_proc` per process, whoever owns it, with the BSD half
/// of what the columns ask for - uid, ppid, pgid, tty, niceness, start time,
/// name - already in it.
fn sysctl_procs(uid: Option<libc::uid_t>) -> Option<Vec<kinfo_proc>> {
    let (mib, mib_len): (&[c_int], u32) = match uid {
        Some(uid) => (
            &[libc::CTL_KERN, libc::KERN_PROC, KERN_PROC_UID, uid as c_int],
            4,
        ),
        None => (&[libc::CTL_KERN, libc::KERN_PROC, KERN_PROC_ALL], 3),
    };
    let mut length: size_t = 0;

    unsafe {
        // Ask how large the table is ...
        if libc::sysctl(
            mib.as_ptr() as *mut c_int,
            mib_len,
            ptr::null_mut(),
            &mut length,
            ptr::null_mut(),
            0,
        ) != 0
        {
            return None;
        }

        // ... and then ask for more than that: the table keeps growing while
        // we are reading it, and a buffer that ends up too small gets an
        // ENOMEM and no data at all.
        length += length / 8 + mem::size_of::<kinfo_proc>();

        for _ in 0..8 {
            let count = length / mem::size_of::<kinfo_proc>();
            let mut procs: Vec<kinfo_proc> = Vec::new();
            procs.resize_with(count, || mem::zeroed());

            if libc::sysctl(
                mib.as_ptr() as *mut c_int,
                mib_len,
                procs.as_mut_ptr() as *mut c_void,
                &mut length,
                ptr::null_mut(),
                0,
            ) == 0
            {
                procs.truncate(length / mem::size_of::<kinfo_proc>());
                return Some(procs);
            }

            if io::Error::last_os_error().raw_os_error() != Some(libc::ENOMEM) {
                return None;
            }

            length += length / 8 + mem::size_of::<kinfo_proc>();
        }
    }

    None
}

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

pub struct PathInfo {
    pub name: String,
    #[allow(dead_code)]
    pub exe: PathBuf,
    /// The directory the executable was loaded from
    #[allow(dead_code)]
    pub root: PathBuf,
    pub cmd: Vec<String>,
    /// The environment, as `KERN_PROCARGS2` reported it: raw `KEY=VALUE`
    /// strings, the ones that follow the arguments in the same buffer.
    pub env: Vec<String>,
}

/// The bytes between `start` and `cp` as a string.
///
/// `KERN_PROCARGS2` promises nothing about encoding, and an argument or an
/// environment variable can hold any bytes at all, so invalid UTF-8 is
/// replaced instead of being assumed away.
unsafe fn get_str(cp: *mut u8, start: *mut u8) -> String {
    let len = cp as usize - start as usize;
    let part = unsafe { std::slice::from_raw_parts(start, len) };
    String::from_utf8_lossy(part).into_owned()
}

fn get_path_info(pid: i32, mut size: size_t) -> Option<PathInfo> {
    let mut proc_args = Vec::with_capacity(size);
    let ptr: *mut u8 = proc_args.as_mut_slice().as_mut_ptr();

    let mut mib: [c_int; 3] = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as c_int];

    unsafe {
        let ret = libc::sysctl(
            mib.as_mut_ptr(),
            3,
            ptr as *mut c_void,
            &mut size,
            ::std::ptr::null_mut(),
            0,
        );
        if ret != -1 {
            let mut n_args: c_int = 0;
            libc::memcpy(
                (&mut n_args) as *mut c_int as *mut c_void,
                ptr as *const c_void,
                ::std::mem::size_of::<c_int>(),
            );
            let mut cp = ptr.add(::std::mem::size_of::<c_int>());
            let mut start = cp;
            if cp < ptr.add(size) {
                while cp < ptr.add(size) && *cp != 0 {
                    cp = cp.offset(1);
                }
                let exe = Path::new(get_str(cp, start).as_str()).to_path_buf();
                let name = exe
                    .file_name()
                    .unwrap_or_else(|| OsStr::new(""))
                    .to_str()
                    .unwrap_or("")
                    .to_owned();
                let mut need_root = true;
                let mut root = Default::default();
                if exe.is_absolute()
                    && let Some(parent) = exe.parent()
                {
                    root = parent.to_path_buf();
                    need_root = false;
                }
                while cp < ptr.add(size) && *cp == 0 {
                    cp = cp.offset(1);
                }
                start = cp;
                let mut c = 0;
                let mut cmd = Vec::new();
                while c < n_args && cp < ptr.add(size) {
                    if *cp == 0 {
                        c += 1;
                        cmd.push(get_str(cp, start));
                        start = cp.offset(1);
                    }
                    cp = cp.offset(1);
                }
                start = cp;
                let mut env = Vec::new();
                while cp < ptr.add(size) {
                    if *cp == 0 {
                        if cp == start {
                            break;
                        }
                        env.push(get_str(cp, start));
                        start = cp.offset(1);
                    }
                    cp = cp.offset(1);
                }
                if need_root {
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
}

/// A `PathInfo` for a process whose command line nobody will tell us: `exe`
/// is both the executable and the whole command line.
fn path_info_of_exe(exe: String) -> Option<PathInfo> {
    if exe.is_empty() {
        return None;
    }

    let path = PathBuf::from(&exe);
    let name = path
        .file_name()
        .unwrap_or_else(|| OsStr::new(""))
        .to_str()
        .unwrap_or("")
        .to_owned();
    let root = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

    Some(PathInfo {
        exe: path,
        name,
        root,
        cmd: vec![exe],
        env: Vec::new(),
    })
}

/// The name carried by `comm`, a `p_comm` the kernel NUL pads and truncates
/// to `MAXCOMLEN` characters.
fn command_of_comm(comm: &[c_char]) -> String {
    let bytes = comm
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect::<Vec<u8>>();

    String::from_utf8_lossy(&bytes).into_owned()
}

// https://github.com/rust-psutil/rust-psutil/blob/main/src/process/os/macos/kinfo.rs

#[allow(non_camel_case_types)]
type caddr_t = *const libc::c_char;
#[allow(non_camel_case_types)]
type segsz_t = i32;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct vmspace {
    pub dummy: i32,
    pub dummy2: caddr_t,
    pub dummy3: [i32; 5],
    pub dummy4: [caddr_t; 3],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct pcred {
    pub pc_lock: [libc::c_char; 72],
    pub pc_ucred: *mut libc::xucred,
    pub p_ruid: libc::uid_t,
    pub p_svuid: libc::uid_t,
    pub p_rgid: libc::gid_t,
    pub p_svgid: libc::gid_t,
    pub p_refcnt: libc::c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct kinfo_proc {
    pub kp_proc: extern_proc,
    pub kp_eproc: kinfo_proc_eproc,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct run_sleep_queue {
    p_forw: vm_types::user_addr_t,
    p_back: vm_types::user_addr_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union p_un {
    pub p_st1: run_sleep_queue,
    pub p_starttime: libc::timeval,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct extern_proc {
    pub p_un: p_un,
    pub p_vmspace: vm_types::user_addr_t,
    pub p_sigacts: vm_types::user_addr_t,

    pub p_flag: libc::c_int,
    pub p_stat: libc::c_char,
    pub p_pid: libc::pid_t,
    pub p_oppid: libc::pid_t,
    pub p_dupfd: libc::c_int,
    pub user_stack: caddr_t,
    pub exit_thread: *mut libc::c_void,
    pub p_debugger: libc::c_int,
    pub sigwait: boolean::boolean_t,
    pub p_estcpu: libc::c_uint,
    pub p_cpticks: libc::c_int,
    pub p_pctcpu: u32,
    pub p_wchan: *mut libc::c_void,
    pub p_wmesg: *mut libc::c_char,
    pub p_swtime: libc::c_uint,
    pub p_slptime: libc::c_uint,
    pub p_realtimer: libc::itimerval,
    pub p_rtime: libc::timeval,
    pub p_uticks: u64,
    pub p_sticks: u64,
    pub p_iticks: u64,
    pub p_traceflag: libc::c_int,
    pub p_tracep: *mut libc::c_void,
    pub p_siglist: libc::c_int,
    // TODO: It was a pointer to `struct vnode`
    pub p_textvp: *mut libc::c_void,
    pub p_holdcnt: libc::c_int,
    pub p_sigmask: libc::sigset_t,
    pub p_sigignore: libc::sigset_t,
    pub p_sigcatch: libc::sigset_t,
    pub p_priority: libc::c_uchar,
    pub p_usrpri: libc::c_uchar,
    pub p_nice: libc::c_char,
    pub p_comm: [libc::c_char; 17],
    // TODO: It was a pointer to `struct proc`, declared at `bsd/sys/proc.h`
    pub p_pgrp: *mut libc::c_void,
    // TODO: It was a pointer to `struct user`, declared at `bsd/sys/user.h`
    // but it is not used anymore and we do not need it too
    pub p_addr: *mut libc::c_void,
    pub p_xstat: libc::c_ushort,
    pub p_acflag: libc::c_ushort,
    pub p_ru: *mut libc::rusage,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct kinfo_proc_eproc {
    // TODO: It should be a pointer to `struct proc`
    pub e_paddr: *mut libc::c_void,
    // TODO: It should be a pointer to `struct session`
    // but since we are not using it and it's declaration kinda big,
    // it was skipped. Same goes to `e_tsess` field below.
    pub e_sess: *mut libc::c_void,
    pub e_pcred: pcred,
    pub e_ucred: libc::xucred,
    pub e_vm: vmspace,
    pub e_ppid: libc::pid_t,
    pub e_pgid: libc::pid_t,
    pub e_jobc: libc::c_short,
    pub e_tdev: libc::dev_t,
    pub e_tpgid: libc::pid_t,
    pub e_tsess: *mut libc::c_void, // TODO: See `TODO` comment from above
    pub e_wmesg: [libc::c_char; 8],
    pub e_xsize: segsz_t,
    pub e_xrssize: libc::c_short,
    pub e_xccount: libc::c_short,
    pub e_xswrss: libc::c_short,
    pub e_flag: i32,
    pub e_login: [libc::c_char; 12],
    pub e_spare: [i32; 4],
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `KERN_PROCARGS2` hands the environment back along with the arguments,
    /// which is what the Env column reads.
    #[test]
    fn path_info_carries_the_environment() {
        let info = get_path_info(std::process::id() as i32, get_arg_max()).unwrap();
        assert!(!info.cmd.is_empty());
        assert!(!info.env.is_empty());
        assert!(info.env.iter().all(|entry| !entry.is_empty()));
    }
}
