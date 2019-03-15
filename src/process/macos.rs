use crate::libproc::libproc::proc_pid::{
    self, BSDInfo, InSockInfo, ListFDs, ListThreads, ProcFDType, ProcType, RUsageInfoV2,
    SocketFDInfo, SocketInfoKind, TaskAllInfo, TaskInfo, TcpSockInfo, ThreadInfo,
};
use libc::{c_int, c_void, size_t};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

pub struct ProcessInfo {
    pub pid: i32,
    pub curr_task: TaskAllInfo,
    pub prev_task: TaskAllInfo,
    pub curr_path: Option<PathInfo>,
    pub curr_threads: Vec<ThreadInfo>,
    pub curr_udps: Vec<InSockInfo>,
    pub curr_tcps: Vec<TcpSockInfo>,
    pub curr_res: Option<RUsageInfoV2>,
    pub prev_res: Option<RUsageInfoV2>,
    pub interval: Duration,
}

#[cfg_attr(tarpaulin, skip)]
pub fn collect_proc(interval: Duration) -> Vec<ProcessInfo> {
    let mut base_procs = Vec::new();
    let mut ret = Vec::new();
    let arg_max = get_arg_max();

    if let Ok(procs) = proc_pid::listpids(ProcType::ProcAllPIDS) {
        for p in procs {
            if let Ok(task) = proc_pid::pidinfo::<TaskAllInfo>(p as i32, 0) {
                let res = proc_pid::pidrusage::<RUsageInfoV2>(p as i32).ok();
                let time = Instant::now();
                base_procs.push((p as i32, task, res, time));
            }
        }
    }

    thread::sleep(interval);

    for (pid, prev_task, prev_res, prev_time) in base_procs {
        let curr_task = if let Ok(task) = proc_pid::pidinfo::<TaskAllInfo>(pid, 0) {
            task
        } else {
            clone_task_all_info(&prev_task)
        };

        let curr_path = get_path_info(pid, arg_max);

        let threadids =
            proc_pid::listpidinfo::<ListThreads>(pid, curr_task.ptinfo.pti_threadnum as usize);
        let mut curr_threads = Vec::new();
        if let Ok(threadids) = threadids {
            for t in threadids {
                if let Ok(thread) = proc_pid::pidinfo::<ThreadInfo>(pid, t) {
                    curr_threads.push(thread);
                }
            }
        }

        let mut curr_tcps = Vec::new();
        let mut curr_udps = Vec::new();

        let fds = proc_pid::listpidinfo::<ListFDs>(pid, curr_task.pbsd.pbi_nfiles as usize);
        if let Ok(fds) = fds {
            for fd in fds {
                match fd.proc_fdtype.into() {
                    ProcFDType::Socket => {
                        if let Ok(socket) = proc_pid::pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) {
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
                    _ => (),
                }
            }
        }

        let curr_res = proc_pid::pidrusage::<RUsageInfoV2>(pid).ok();

        let curr_time = Instant::now();
        let interval = curr_time - prev_time;

        let proc = ProcessInfo {
            pid,
            curr_task,
            prev_task,
            curr_path,
            curr_threads,
            curr_udps,
            curr_tcps,
            curr_res,
            prev_res,
            interval,
        };

        ret.push(proc);
    }

    ret
}

#[cfg_attr(tarpaulin, skip)]
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
    pub exe: PathBuf,
    pub root: PathBuf,
    pub cmd: Vec<String>,
    pub env: Vec<String>,
}

#[cfg_attr(tarpaulin, skip)]
unsafe fn get_unchecked_str(cp: *mut u8, start: *mut u8) -> String {
    let len = cp as usize - start as usize;
    let part = Vec::from_raw_parts(start, len, len);
    let tmp = String::from_utf8_unchecked(part.clone());
    ::std::mem::forget(part);
    tmp
}

#[cfg_attr(tarpaulin, skip)]
fn get_path_info(pid: i32, mut size: size_t) -> Option<PathInfo> {
    let mut proc_args = Vec::with_capacity(size as usize);
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
}

#[cfg_attr(tarpaulin, skip)]
fn clone_task_all_info(src: &TaskAllInfo) -> TaskAllInfo {
    let pbsd = BSDInfo {
        pbi_flags: src.pbsd.pbi_flags,
        pbi_status: src.pbsd.pbi_status,
        pbi_xstatus: src.pbsd.pbi_xstatus,
        pbi_pid: src.pbsd.pbi_pid,
        pbi_ppid: src.pbsd.pbi_ppid,
        pbi_uid: src.pbsd.pbi_uid,
        pbi_gid: src.pbsd.pbi_gid,
        pbi_ruid: src.pbsd.pbi_ruid,
        pbi_rgid: src.pbsd.pbi_rgid,
        pbi_svuid: src.pbsd.pbi_svuid,
        pbi_svgid: src.pbsd.pbi_svgid,
        rfu_1: src.pbsd.rfu_1,
        pbi_comm: src.pbsd.pbi_comm,
        pbi_name: src.pbsd.pbi_name,
        pbi_nfiles: src.pbsd.pbi_nfiles,
        pbi_pgid: src.pbsd.pbi_pgid,
        pbi_pjobc: src.pbsd.pbi_pjobc,
        e_tdev: src.pbsd.e_tdev,
        e_tpgid: src.pbsd.e_tpgid,
        pbi_nice: src.pbsd.pbi_nice,
        pbi_start_tvsec: src.pbsd.pbi_start_tvsec,
        pbi_start_tvusec: src.pbsd.pbi_start_tvusec,
    };
    let ptinfo = TaskInfo {
        pti_virtual_size: src.ptinfo.pti_virtual_size,
        pti_resident_size: src.ptinfo.pti_resident_size,
        pti_total_user: src.ptinfo.pti_total_user,
        pti_total_system: src.ptinfo.pti_total_system,
        pti_threads_user: src.ptinfo.pti_threads_user,
        pti_threads_system: src.ptinfo.pti_threads_system,
        pti_policy: src.ptinfo.pti_policy,
        pti_faults: src.ptinfo.pti_faults,
        pti_pageins: src.ptinfo.pti_pageins,
        pti_cow_faults: src.ptinfo.pti_cow_faults,
        pti_messages_sent: src.ptinfo.pti_messages_sent,
        pti_messages_received: src.ptinfo.pti_messages_received,
        pti_syscalls_mach: src.ptinfo.pti_syscalls_mach,
        pti_syscalls_unix: src.ptinfo.pti_syscalls_unix,
        pti_csw: src.ptinfo.pti_csw,
        pti_threadnum: src.ptinfo.pti_threadnum,
        pti_numrunning: src.ptinfo.pti_numrunning,
        pti_priority: src.ptinfo.pti_priority,
    };
    TaskAllInfo { pbsd, ptinfo }
}
