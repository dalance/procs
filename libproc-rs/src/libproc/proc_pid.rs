extern crate libc;
extern crate errno;

use self::libc::{uint64_t, uint32_t, int32_t, c_void, c_int, uid_t, gid_t, c_char};
use self::errno::errno;
use std::ptr;
use std::mem;

// Since we cannot access C macros for constants from Rust - I have had to redefine this, based on Apple's source code
// See http://opensource.apple.com/source/Libc/Libc-594.9.4/darwin/libproc.c
// buffersize must be more than PROC_PIDPATHINFO_SIZE
// buffersize must be less than PROC_PIDPATHINFO_MAXSIZE
//
// See http://opensource.apple.com//source/xnu/xnu-1456.1.26/bsd/sys/proc_info.h
// #define PROC_PIDPATHINFO_SIZE		(MAXPATHLEN)
// #define PROC_PIDPATHINFO_MAXSIZE	(4*MAXPATHLEN)
// in http://opensource.apple.com//source/xnu/xnu-1504.7.4/bsd/sys/param.h
// #define	MAXPATHLEN	PATH_MAX
// in https://opensource.apple.com/source/xnu/xnu-792.25.20/bsd/sys/syslimits.h
// #define	PATH_MAX		 1024
const MAXPATHLEN: usize = 1024;
const PROC_PIDPATHINFO_MAXSIZE: usize = 4 * MAXPATHLEN;

// from http://opensource.apple.com//source/xnu/xnu-1456.1.26/bsd/sys/proc_info.h
const MAXTHREADNAMESIZE : usize = 64;

// From http://opensource.apple.com//source/xnu/xnu-1456.1.26/bsd/sys/proc_info.h and
// http://fxr.watson.org/fxr/source/bsd/sys/proc_info.h?v=xnu-2050.18.24
#[derive(Copy, Clone)]
pub enum ProcType {
    ProcAllPIDS     = 1,
    ProcPGRPOnly    = 2,
    ProcTTYOnly     = 3,
    ProcUIDOnly     = 4,
    ProcRUIDOnly    = 5,
    ProcPPIDOnly    = 6
}

// from http://opensource.apple.com//source/xnu/xnu-1504.7.4/bsd/sys/param.h
const MAXCOMLEN	: usize = 16;

// This trait is needed for polymorphism on pidinfo types, also abstracting flavor in order to provide
// type-guaranteed flavor correctness
pub trait PIDInfo: Default {
    fn flavor() -> PidInfoFlavor;
}

// structures from http://opensource.apple.com//source/xnu/xnu-1456.1.26/bsd/sys/proc_info.h
#[repr(C)]
#[derive(Default)]
pub struct TaskInfo {
    pub pti_virtual_size        : uint64_t,     // virtual memory size (bytes)
    pub pti_resident_size       : uint64_t,     // resident memory size (bytes)
    pub pti_total_user          : uint64_t,     // total time
    pub pti_total_system        : uint64_t,
    pub pti_threads_user        : uint64_t,     // existing threads only
    pub pti_threads_system      : uint64_t,
    pub pti_policy              : int32_t,      // default policy for new threads
    pub pti_faults              : int32_t,      // number of page faults
    pub pti_pageins             : int32_t,      // number of actual pageins
    pub pti_cow_faults          : int32_t,      // number of copy-on-write faults
    pub pti_messages_sent       : int32_t,      // number of messages sent
    pub pti_messages_received   : int32_t,      // number of messages received
    pub pti_syscalls_mach       : int32_t,      // number of mach system calls
    pub pti_syscalls_unix       : int32_t,      // number of unix system calls
    pub pti_csw                 : int32_t,      // number of context switches
    pub pti_threadnum           : int32_t,      // number of threads in the task
    pub pti_numrunning          : int32_t,      // number of running threads
    pub pti_priority            : int32_t       // task priority
}

impl PIDInfo for TaskInfo {
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::TaskInfo }
}

#[repr(C)]
#[derive(Default)]
pub struct BSDInfo {
    pub pbi_flags               : uint32_t,                 // 64bit; emulated etc
    pub pbi_status              : uint32_t,
    pub pbi_xstatus             : uint32_t,
    pub pbi_pid                 : uint32_t,
    pub pbi_ppid                : uint32_t,
    pub pbi_uid                 : uid_t,
    pub pbi_gid                 : gid_t,
    pub pbi_ruid                : uid_t,
    pub pbi_rgid                : gid_t,
    pub pbi_svuid               : uid_t,
    pub pbi_svgid               : gid_t,
    pub rfu_1                   : uint32_t,                 // reserved
    pub pbi_comm                : [c_char; MAXCOMLEN],
    pub pbi_name                : [c_char; 2 * MAXCOMLEN],  // empty if no name is registered
    pub pbi_nfiles              : uint32_t,
    pub pbi_pgid                : uint32_t,
    pub pbi_pjobc               : uint32_t,
    pub e_tdev                  : uint32_t,                 // controlling tty dev
    pub e_tpgid                 : uint32_t,                 // tty process group id
    pub pbi_nice                : int32_t,
    pub pbi_start_tvsec         : uint64_t,
    pub pbi_start_tvusec        : uint64_t
}

impl PIDInfo for BSDInfo {
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::TBSDInfo }
}

#[repr(C)]
#[derive(Default)]
pub struct TaskAllInfo {
    pub pbsd : BSDInfo,
    pub ptinfo : TaskInfo
}

impl PIDInfo for TaskAllInfo {
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::TaskAllInfo }
}

#[repr(C)]
pub struct ThreadInfo {
    pub pth_user_time           : uint64_t,                     // user run time
    pub pth_system_time         : uint64_t,                     // system run time
    pub pth_cpu_usage           : int32_t,                      // scaled cpu usage percentage
    pub pth_policy              : int32_t,                      // scheduling policy in effect
    pub pth_run_state           : int32_t,                      // run state (see below)
    pub pth_flags               : int32_t,                      // various flags (see below)
    pub pth_sleep_time          : int32_t,                      // number of seconds that thread
    pub pth_curpri              : int32_t,                      // cur priority
    pub pth_priority            : int32_t,                      // priority
    pub pth_maxpriority         : int32_t,                      // max priority
    pub pth_name                : [c_char; MAXTHREADNAMESIZE]   // thread name, if any
}

#[derive(Default)]
pub struct WorkQueueInfo {
    pub pwq_nthreads            : uint32_t,     // total number of workqueue threads
    pub pwq_runthreads          : uint32_t,     // total number of running workqueue threads
    pub pwq_blockedthreads      : uint32_t,     // total number of blocked workqueue threads
    pub reserved                : [uint32_t;1]  // reserved for future use
}

impl PIDInfo for WorkQueueInfo {
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::WorkQueueInfo }
}

// From http://opensource.apple.com/source/xnu/xnu-1504.7.4/bsd/kern/proc_info.c
pub enum PidInfoFlavor {
    ListFDs         =  1,   // list of ints?
    TaskAllInfo     =  2,   // struct proc_taskallinfo
    TBSDInfo        =  3,   // struct proc_bsdinfo
    TaskInfo        =  4,   // struct proc_taskinfo
    ThreadInfo      =  5,   // struct proc_threadinfo
    ListThreads     =  6,   // list if int thread ids
    RegionInfo      =  7,
    RegionPathInfo  =  8,   // string?
    VNodePathInfo   =  9,   // string?
    ThreadPathInfo  = 10,   // String?
    PathInfo        = 11,   // String
    WorkQueueInfo   = 12    // struct proc_workqueueinfo
}

pub enum PidInfo {
    ListFDs(Vec<i32>),      // File Descriptors used by Process
    TaskAllInfo(TaskAllInfo),
    TBSDInfo(BSDInfo),
    TaskInfo(TaskInfo),
    ThreadInfo(ThreadInfo),
    ListThreads(Vec<i32>),  // thread ids
    RegionInfo(String),     // String??
    RegionPathInfo(String),
    VNodePathInfo(String),
    ThreadPathInfo(String),
    PathInfo(String),
    WorkQueueInfo(WorkQueueInfo)
}

pub enum PidFDInfoFlavor {
    VNodeInfo       = 1,
    VNodePathInfo   = 2,
    SocketInfo      = 3,
    PSEMInfo        = 4,
    PSHMInfo        = 5,
    PipeInfo        = 6,
    KQueueInfo      = 7,
    ATalkInfo       = 8
}

// this extern block links to the libproc library
// Original signatures of functions can be found at http://opensource.apple.com/source/Libc/Libc-594.9.4/darwin/libproc.c
#[link(name = "proc", kind = "dylib")]
extern {
    fn proc_listpids(proc_type: uint32_t, typeinfo: uint32_t, buffer: *mut c_void, buffersize: uint32_t) -> c_int;

    fn proc_pidinfo(pid : c_int, flavor : c_int, arg: uint64_t, buffer : *mut c_void, buffersize : c_int) -> c_int;

//    fn proc_pidfdinfo(pid : c_int, fd : c_int, flavor : c_int, buffer : *mut c_void, buffersize : c_int) -> c_int;

    fn proc_name(pid: c_int, buffer: *mut c_void, buffersize: uint32_t) -> c_int;

    fn proc_regionfilename(pid: c_int, address: uint64_t, buffer: *mut c_void, buffersize: uint32_t) -> c_int;

    fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffersize: uint32_t) -> c_int;

    fn proc_libversion(major: *mut c_int, minor: *mut c_int) -> c_int;
}


pub fn get_errno_with_message(ret: i32) -> String {
    let e = errno();
    let code = e.0 as i32;
    format!("return code = {}, errno = {}, message = '{}'", ret, code, e)
}

/// Returns the PIDs of the processes active that match the ProcType passed in
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use libproc::libproc::proc_pid;
///
/// match proc_pid::listpids(proc_pid::ProcType::ProcAllPIDS) {
///     Ok(pids) => {
///         assert!(pids.len() > 1);
///         println!("Found {} processes using listpids()", pids.len());
///     },
///     Err(err) => assert!(false, "Error listing pids")
/// }
/// ```
pub fn listpids(proc_types: ProcType) -> Result<Vec<u32>, String> {
    let buffer_size = unsafe { proc_listpids(proc_types as u32, 0, ptr::null_mut(), 0) };
    if buffer_size <= 0 {
        return Err(get_errno_with_message(buffer_size))
    }

    let capacity = buffer_size as usize / mem::size_of::<u32>();
    let mut pids: Vec<u32> = Vec::with_capacity(capacity);
    let buffer_ptr = pids.as_mut_ptr() as *mut c_void;

    let ret = unsafe { proc_listpids(proc_types as u32, 0, buffer_ptr, buffer_size as u32) };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        let items_count = ret as usize / mem::size_of::<u32>() - 1;
        unsafe {
            pids.set_len(items_count);
        }

        Ok(pids)
    }
}

#[test]
fn listpids_test() {
    match listpids(ProcType::ProcAllPIDS) {
        Ok(pids) => {
            assert!(pids.len() > 1);
            println!("Found {} processes using listpids()", pids.len());
        },
        Err(err) => assert!(false, "Error listing pids: {}", err)
    }
}


/// Returns the PIDs of the process that match pid passed in.
///
/// arg - is "geavily not documented" and need to look at code for each flavour here
/// http://opensource.apple.com/source/xnu/xnu-1504.7.4/bsd/kern/proc_info.c
/// to figure out what it's doing.... Pull-Requests welcome!
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use libproc::libproc::proc_pid::{pidinfo, BSDInfo};
///
/// fn pidinfo_test() {
///     use std::process;
///     let pid = process::id() as i32;
///
///     match pidinfo::<BSDInfo>(pid, 0) {
///         Ok(info) => assert_eq!(info.pbi_pid as i32, pid),
///         Err(err) => assert!(false, "Error retrieving process info: {}", err)
///     };
/// }
/// ```
///
pub fn pidinfo<T: PIDInfo>(pid : i32, arg: uint64_t) -> Result<T, String> {
    let flavor = T::flavor() as i32;
    let buffer_size = mem::size_of::<T>() as i32;
    let mut pidinfo = T::default();
    let buffer_ptr = &mut pidinfo as *mut _ as *mut c_void;
    let ret: i32;

    unsafe {
        ret = proc_pidinfo(pid, flavor, arg, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        Ok(pidinfo)
    }
}

#[test]
fn pidinfo_test() {
    use std::process;
    let pid = process::id() as i32;

    match pidinfo::<BSDInfo>(pid, 0) {
        Ok(info) => assert_eq!(info.pbi_pid as i32, pid),
        Err(err) => assert!(false, "Error retrieving process info: {}", err)
    };
}

pub fn regionfilename(pid: i32, address: u64) -> Result<String, String> {
    let mut regionfilenamebuf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE - 1);
    let buffer_ptr = regionfilenamebuf.as_mut_ptr() as *mut c_void;
    let buffer_size = regionfilenamebuf.capacity() as u32;
    let ret: i32;

    unsafe {
        ret = proc_regionfilename(pid, address, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        unsafe {
            regionfilenamebuf.set_len(ret as usize);
        }

        match String::from_utf8(regionfilenamebuf) {
            Ok(regionfilename) => Ok(regionfilename),
            Err(e) => Err(format!("Invalid UTF-8 sequence: {}", e))
        }
    }
}

#[test]
// This checks that it can find the regionfilename of the region at address 0, of the init process with PID 1
fn regionfilename_test() {
    match regionfilename(1, 0) {
        // run tests with 'cargo test -- --nocapture' to see the test output
        Ok(regionfilename) => println!("Region Filename (at address = 0) of init process PID = 1 is '{}'", regionfilename),
        Err(message) => assert!(true, message)
    }
}

pub fn pidpath(pid: i32) -> Result<String, String> {
    let mut pathbuf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE - 1);
    let buffer_ptr = pathbuf.as_mut_ptr() as *mut c_void;
    let buffer_size = pathbuf.capacity() as u32;
    let ret: i32;

    unsafe {
        ret = proc_pidpath(pid, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        unsafe {
            pathbuf.set_len(ret as usize);
        }

        match String::from_utf8(pathbuf) {
            Ok(path) => Ok(path),
            Err(e) => Err(format!("Invalid UTF-8 sequence: {}", e))
        }
    }
}

#[test]
// This checks that it can find the path of the init process with PID 1
fn pidpath_test_init_pid() {
    match pidpath(1) {
        // run tests with 'cargo test -- --nocapture' to see the test output
        Ok(path) => println!("Path of init process with PID = 1 is '{}'", path),
        Err(message) => assert!(false, message)
    }
}

#[test]
#[should_panic]
// This checks that it cannot find the path of the process with pid -1
fn pidpath_test_unknown_pid() {
    match pidpath(-1) {
        // run tests with 'cargo test -- --nocapture' to see the test output
        Ok(path) => assert!(false, "It found the path of process Pwith ID = -1 (path = {}), that's not possible\n", path),
        Err(message) => assert!(false, message)
    }
}

/// Returns the major and minor version numbers of the native librproc library being used
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use libproc::libproc::proc_pid;
///
/// match proc_pid::libversion() {
///     Ok((major, minor)) => println!("Libversion: {}.{}", major, minor),
///     Err(err) => writeln!(&mut std::io::stderr(), "Error: {}", err).unwrap()
/// }
/// ```
pub fn libversion() -> Result<(i32, i32), String> {
    let mut major = 0;
    let mut minor = 0;
    let ret: i32;

    unsafe {
        ret = proc_libversion(&mut major, &mut minor);
    };

    // return value of 0 indicates success (inconsistent with other functions... :-( )
    if ret == 0 {
        Ok((major, minor))
    } else {
        Err(get_errno_with_message(ret))
    }
}

#[test]
fn libversion_test() {
    match libversion() {
        Ok((major, minor)) => {
            // run tests with 'cargo test -- --nocapture' to see the test output
            println!("Major = {}, Minor = {}", major, minor);
        },
        Err(message) => assert!(false, message)
    }
}

/// Returns the name of the process with the specified pid
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use libproc::libproc::proc_pid;
///
/// match proc_pid::name(1) {
///     Ok(name) => println!("Name: {}", name),
///     Err(err) => writeln!(&mut std::io::stderr(), "Error: {}", err).unwrap()
/// }
/// ```
pub fn name(pid: i32) -> Result<String, String> {
    let mut namebuf: Vec<u8> = Vec::with_capacity(PROC_PIDPATHINFO_MAXSIZE - 1);
    let buffer_ptr = namebuf.as_ptr() as *mut c_void;
    let buffer_size = namebuf.capacity() as u32;
    let ret: i32;

    unsafe {
        ret = proc_name(pid, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        unsafe {
            namebuf.set_len(ret as usize);
        }

        match String::from_utf8(namebuf) {
            Ok(name) => Ok(name),
            Err(e) => Err(format!("Invalid UTF-8 sequence: {}", e))
        }
    }
}

#[test]
// error: Process didn't exit successfully: `/Users/andrew/workspace/libproc-rs/target/debug/libproc-503ad0ba07eb6318` (signal: 11, SIGSEGV: invalid memory reference)
// This checks that it can find the name of the init process with PID 1
fn name_test_init_pid() {
    match pidpath(1) {
        // run tests with 'cargo test -- --nocapture' to see the test output
        Ok(path) => println!("Name of init process PID = 1 is '{}'", path),
        Err(message) => assert!(true, message)
    }
}

/// Returns the Thread IDs of the process that match pid passed in.
/// `threadnum` is the maximum number of threads to return.
/// The length of return value: `Vec<uint64_t>` may be less than `threadnum`.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use libproc::libproc::proc_pid::{listthreads, pidinfo, TaskInfo};
///
/// fn listthreads_test() {
///     use std::process;
///     let pid = process::id() as i32;
///
///     match pidinfo::<TaskInfo>(pid, 0) {
///         Ok(info) => {
///             match listthreads(pid, info.pti_threadnum) {
///                 Ok(threads) => assert!(threads.len()>0),
///                 Err(err) => assert!(false, "Error retrieving process info: {}", err)
///             }
///         },
///         Err(err) => assert!(false, "Error retrieving process info: {}", err)
///     };
/// }
/// ```
pub fn listthreads(pid: i32, threadnum: i32) -> Result<Vec<uint64_t>, String> {
    let buffer_size = mem::size_of::<uint64_t>() as i32 * threadnum;
    let mut buffer = Vec::<uint64_t>::with_capacity(threadnum as usize);
    let buffer_ptr = unsafe {
        buffer.set_len(threadnum as usize);
        buffer.as_mut_ptr() as *mut c_void
    };

    let ret: i32;

    unsafe {
        ret = proc_pidinfo( pid, PidInfoFlavor::ListThreads as i32, 0, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        let actual_len = ret as usize / mem::size_of::<uint64_t>();
        buffer.truncate(actual_len);
        Ok(buffer)
    }
}

#[test]
fn listthreads_test() {
    use std::process;
    let pid = process::id() as i32;

    match pidinfo::<TaskInfo>(pid, 0) {
        Ok(info) => {
            match listthreads(pid, info.pti_threadnum) {
                Ok(threads) => assert!(threads.len()>0),
                Err(err) => assert!(false, "Error retrieving process info: {}", err)
            }
        },
        Err(err) => assert!(false, "Error retrieving process info: {}", err)
    };
}
