use libc::{c_int, c_void, size_t};
use libproc::libproc::bsd_info::BSDInfo;
use libproc::libproc::file_info::{pidfdinfo, ListFDs, ProcFDType};
use libproc::libproc::net_info::{InSockInfo, SocketFDInfo, SocketInfoKind, TcpSockInfo};
use libproc::libproc::pid_rusage::{pidrusage, RUsageInfoV2};
use libproc::libproc::proc_pid::{listpidinfo, pidinfo, ListThreads};
use libproc::libproc::task_info::{TaskAllInfo, TaskInfo};
use libproc::libproc::thread_info::ThreadInfo;
use libproc::processes::{pids_by_type, ProcFilter};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

pub struct ProcessInfo {
    pub pid: i32,
    pub ppid: i32,
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
pub fn collect_proc(
    interval: Duration,
    _with_thread: bool,
    _show_kthreads: bool,
) -> Vec<ProcessInfo> {
    let mut ret = Vec::new();

    thread::sleep(interval);

    if let Ok(procs) = pids_by_type(ProcFilter::All) {
        for p in procs {
            if let Some(proc) = collect_proc_info(p as i32, interval, _with_thread, _show_kthreads)
            {
                ret.push(proc);
            }
        }
    }
    ret
}


#[cfg_attr(tarpaulin, skip)]
pub fn collect_proc_info(
    p: i32,
    _interval: Duration,
    _with_thread: bool,
    _show_kthreads: bool,
) -> Option<ProcessInfo> {
    let arg_max = get_arg_max();

    if let Some(task) = base_task_t(p) {
        let (pid, prev_task, prev_res, prev_time) = task;
        let curr_task = if let Ok(task) = pidinfo::<TaskAllInfo>(pid, 0) {
            task
        } else {
            clone_task_all_info(&prev_task)
        };

        let curr_path = get_path_info(pid, arg_max);

        let threadids = listpidinfo::<ListThreads>(pid, curr_task.ptinfo.pti_threadnum as usize);
        let mut curr_threads = Vec::new();
        if let Ok(threadids) = threadids {
            for t in threadids {
                if let Ok(thread) = pidinfo::<ThreadInfo>(pid, t) {
                    curr_threads.push(thread);
                }
            }
        }

        let mut curr_tcps = Vec::new();
        let mut curr_udps = Vec::new();

        let fds = listpidinfo::<ListFDs>(pid, curr_task.pbsd.pbi_nfiles as usize);
        if let Ok(fds) = fds {
            for fd in fds {
                if let ProcFDType::Socket = fd.proc_fdtype.into() {
                    if let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) {
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
        }

        let curr_res = pidrusage::<RUsageInfoV2>(pid).ok();

        let curr_time = Instant::now();
        let interval = curr_time - prev_time;
        let ppid = curr_task.pbsd.pbi_ppid as i32;

        let proc = ProcessInfo {
            pid,
            ppid,
            curr_task,
            prev_task: clone_task_all_info(&prev_task),
            curr_path,
            curr_threads,
            curr_udps,
            curr_tcps,
            curr_res,
            prev_res: Some(clone_rusage_info_v2(&prev_res)),
            interval,
        };
        Some(proc)
    } else {
        None
    }
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

#[derive(Clone)]
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
            let mut cp = ptr.add(::std::mem::size_of::<c_int>());
            let mut start = cp;
            if cp < ptr.add(size) {
                while cp < ptr.add(size) && *cp != 0 {
                    cp = cp.offset(1);
                }
                let exe = Path::new(get_unchecked_str(cp, start).as_str()).to_path_buf();
                let name = exe
                    .file_name()
                    .unwrap_or_else(|| OsStr::new(""))
                    .to_str()
                    .unwrap_or("")
                    .to_owned();
                let mut need_root = true;
                let mut root = Default::default();
                if exe.is_absolute() {
                    if let Some(parent) = exe.parent() {
                        root = parent.to_path_buf();
                        need_root = false;
                    }
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
                        cmd.push(get_unchecked_str(cp, start));
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
                        env.push(get_unchecked_str(cp, start));
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

#[cfg_attr(tarpaulin, skip)]
fn clone_thread_info(src: &ThreadInfo) -> ThreadInfo {
    ThreadInfo {
        pth_user_time: src.pth_user_time,
        pth_system_time: src.pth_system_time,
        pth_cpu_usage: src.pth_cpu_usage,
        pth_policy: src.pth_policy,
        pth_run_state: src.pth_run_state,
        pth_flags: src.pth_flags,
        pth_sleep_time: src.pth_sleep_time,
        pth_curpri: src.pth_curpri,
        pth_priority: src.pth_priority,
        pth_maxpriority: src.pth_maxpriority,
        pth_name: src.pth_name,
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

#[cfg_attr(tarpaulin, skip)]
fn clone_rusage_info_v2(src: &RUsageInfoV2) -> RUsageInfoV2 {
    RUsageInfoV2 {
        ri_uuid: src.ri_uuid,
        ri_user_time: src.ri_user_time,
        ri_system_time: src.ri_system_time,
        ri_pkg_idle_wkups: src.ri_pkg_idle_wkups,
        ri_interrupt_wkups: src.ri_interrupt_wkups,
        ri_pageins: src.ri_pageins,
        ri_wired_size: src.ri_wired_size,
        ri_resident_size: src.ri_resident_size,
        ri_phys_footprint: src.ri_phys_footprint,
        ri_proc_start_abstime: src.ri_proc_start_abstime,
        ri_proc_exit_abstime: src.ri_proc_exit_abstime,
        ri_child_user_time: src.ri_child_user_time,
        ri_child_system_time: src.ri_child_system_time,
        ri_child_pkg_idle_wkups: src.ri_child_pkg_idle_wkups,
        ri_child_interrupt_wkups: src.ri_child_interrupt_wkups,
        ri_child_pageins: src.ri_child_pageins,
        ri_child_elapsed_abstime: src.ri_child_elapsed_abstime,
        ri_diskio_bytesread: src.ri_diskio_bytesread,
        ri_diskio_byteswritten: src.ri_diskio_byteswritten,
    }
}

fn clone_in_sock_info(src: &InSockInfo) -> InSockInfo {
    InSockInfo {
        insi_fport: src.insi_fport,
        insi_lport: src.insi_lport,
        insi_gencnt: src.insi_gencnt,
        insi_flags: src.insi_flags,
        insi_flow: src.insi_flow,
        insi_vflag: src.insi_vflag,
        insi_ip_ttl: src.insi_ip_ttl,
        rfu_1: src.rfu_1,
        insi_faddr: src.insi_faddr,
        insi_laddr: src.insi_laddr,
        insi_v4: src.insi_v4,
        insi_v6: src.insi_v6,
    }
}

fn clone_tcp_sock_info(src: &TcpSockInfo) -> TcpSockInfo {
    TcpSockInfo {
        tcpsi_ini: clone_in_sock_info(&src.tcpsi_ini),
        tcpsi_state: src.tcpsi_state,
        tcpsi_timer: src.tcpsi_timer,
        tcpsi_mss: src.tcpsi_mss,
        tcpsi_flags: src.tcpsi_flags,
        rfu_1: src.rfu_1,
        tcpsi_tp: src.tcpsi_tp,
    }
}
#[cfg_attr(tarpaulin, skip)]
impl Clone for ProcessInfo {
    fn clone(&self) -> ProcessInfo {
        ProcessInfo {
            pid: self.pid,
            ppid: self.ppid,
            curr_task: clone_task_all_info(&self.curr_task),
            prev_task: clone_task_all_info(&self.prev_task),
            curr_path: self.curr_path.clone(),
            curr_threads: self.curr_threads.iter().map(clone_thread_info).collect::<Vec<_>>(),
            curr_udps: self.curr_udps.iter().map(clone_in_sock_info).collect::<Vec<_>>(),
            curr_tcps: self.curr_tcps.iter().map(clone_tcp_sock_info).collect::<Vec<_>>(),
            curr_res: match &self.curr_res {
                Some(ruiv) => Some(clone_rusage_info_v2(&ruiv)),
                None => None,
            },
            prev_res: match &self.prev_res {
                Some(ruiv) => Some(clone_rusage_info_v2(&ruiv)),
                None => None,
            },
            interval: self.interval,
        }
    }
}
#[cfg_attr(tarpaulin, skip)]
pub fn base_task_t(p: i32) -> Option<(i32, TaskAllInfo, RUsageInfoV2, Instant)> {
    if let Ok(task) = pidinfo::<TaskAllInfo>(p as i32, 0) {
        let res = pidrusage::<RUsageInfoV2>(p as i32).unwrap();
        let time = Instant::now();
        Some((p as i32, task, res, time))
    } else {
        None
    }
}
