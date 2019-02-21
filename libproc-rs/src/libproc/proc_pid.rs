extern crate libc;
extern crate errno;

use self::libc::{uint64_t, int64_t, uint32_t, int32_t, uint16_t, uint8_t, c_short, c_ushort, c_void, c_int, c_uchar, uid_t, gid_t, c_char, off_t, in_addr, sockaddr_un, in6_addr, SOCK_MAXADDRLEN, IF_NAMESIZE};
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

impl PIDInfo for ThreadInfo {
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::ThreadInfo }
}

impl Default for ThreadInfo {
    fn default() -> ThreadInfo {
        ThreadInfo {
            pth_user_time  : 0,
            pth_system_time: 0,
            pth_cpu_usage  : 0,
            pth_policy     : 0,
            pth_run_state  : 0,
            pth_flags      : 0,
            pth_sleep_time : 0,
            pth_curpri     : 0,
            pth_priority   : 0,
            pth_maxpriority: 0,
            pth_name       : [0; MAXTHREADNAMESIZE],
        }
    }
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

    fn proc_pidfdinfo(pid : c_int, fd : c_int, flavor : c_int, buffer : *mut c_void, buffersize : c_int) -> c_int;

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

// This trait is needed for polymorphism on listpidinfo types, also abstracting flavor in order to provide
// type-guaranteed flavor correctness
pub trait ListPIDInfo {
    type Item;
    fn flavor() -> PidInfoFlavor;
}

/// Returns the information of the process that match pid passed in.
/// `max_len` is the maximum number of array to return.
/// The length of return value: `Vec<T::Item>` may be less than `max_len`.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use libproc::libproc::proc_pid::{listpidinfo, pidinfo, ListFDs, TaskAllInfo, ProcFDType};
///
/// fn listpidinfo_test() {
///     use std::process;
///     let pid = process::id() as i32;
///
///     if let Ok(info) = pidinfo::<TaskAllInfo>(pid, 0) {
///         if let Ok(fds) = listpidinfo::<ListFDs>(pid, info.pbsd.pbi_nfiles as usize) {
///             for fd in &fds {
///                 let fd_type = ProcFDType::from(fd.proc_fdtype);
///                 println!("File Descriptor: {}, Type: {:?}", fd.proc_fd, fd_type);
///             }
///         }
///     }
/// }
/// ```
pub fn listpidinfo<T: ListPIDInfo>(pid : i32, max_len: usize) -> Result<Vec<T::Item>, String> {
    let flavor = T::flavor() as i32;
    let buffer_size = mem::size_of::<T::Item>() as i32 * max_len as i32;
    let mut buffer = Vec::<T::Item>::with_capacity(max_len);
    let buffer_ptr = unsafe {
        buffer.set_len(max_len);
        buffer.as_mut_ptr() as *mut c_void
    };

    let ret: i32;

    unsafe {
        ret = proc_pidinfo( pid, flavor, 0, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        let actual_len = ret as usize / mem::size_of::<T::Item>();
        buffer.truncate(actual_len);
        Ok(buffer)
    }
}

#[test]
fn listpidinfo_test() {
    use std::process;
    let pid = process::id() as i32;

    match pidinfo::<TaskAllInfo>(pid, 0) {
        Ok(info) => {
            match listpidinfo::<ListThreads>(pid, info.ptinfo.pti_threadnum as usize) {
                Ok(threads) => assert!(threads.len()>0),
                Err(err) => assert!(false, "Error retrieving process info: {}", err)
            }
            match listpidinfo::<ListFDs>(pid, info.pbsd.pbi_nfiles as usize) {
                Ok(fds) => assert!(fds.len()>0),
                Err(err) => assert!(false, "Error retrieving process info: {}", err)
            }
        },
        Err(err) => assert!(false, "Error retrieving process info: {}", err)
    };
}

pub struct ListThreads;

impl ListPIDInfo for ListThreads {
    type Item = uint64_t;
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::ListThreads }
}

pub struct ListFDs;

impl ListPIDInfo for ListFDs {
    type Item = ProcFDInfo;
    fn flavor() -> PidInfoFlavor { PidInfoFlavor::ListFDs }
}

#[repr(C)]
pub struct ProcFDInfo {
    pub proc_fd: int32_t,
    pub proc_fdtype: uint32_t,
}

#[derive(Copy, Clone, Debug)]
pub enum ProcFDType {
    /// AppleTalk
    ATalk    = 0,
    /// Vnode
    VNode    = 1,
    /// Socket
    Socket   = 2,
    /// POSIX shared memory
    PSHM     = 3,
    /// POSIX semaphore
    PSEM     = 4,
    /// Kqueue
    KQueue   = 5,
    /// Pipe
    Pipe     = 6,
    /// FSEvents
    FSEvents = 7,
    /// Unknown
    Unknown,
}

impl From<uint32_t> for ProcFDType {
    fn from(value: uint32_t) -> ProcFDType {
        match value {
            0 => ProcFDType::ATalk   ,
            1 => ProcFDType::VNode   ,
            2 => ProcFDType::Socket  ,
            3 => ProcFDType::PSHM    ,
            4 => ProcFDType::PSEM    ,
            5 => ProcFDType::KQueue  ,
            6 => ProcFDType::Pipe    ,
            7 => ProcFDType::FSEvents,
            _ => ProcFDType::Unknown ,
        }
    }
}

// This trait is needed for polymorphism on pidfdinfo types, also abstracting flavor in order to provide
// type-guaranteed flavor correctness
pub trait PIDFDInfo: Default {
    fn flavor() -> PidFDInfoFlavor;
}

/// Returns the information of the process that match pid passed in.
///
/// # Examples
///
/// ```
/// use std::io::Write;
/// use std::net::TcpListener;
/// use libproc::libproc::proc_pid::{listpidinfo, pidinfo, pidfdinfo, ListFDs, ListThreads, BSDInfo, ProcFDType, SocketFDInfo, SocketInfoKind};
///
/// fn pidfdinfo_test() {
///     use std::process;
///     let pid = process::id() as i32;
///
///     // Open TCP port:8000 to test.
///     let _listener = TcpListener::bind("127.0.0.1:8000");
///
///     if let Ok(info) = pidinfo::<BSDInfo>(pid, 0) {
///         if let Ok(fds) = listpidinfo::<ListFDs>(pid, info.pbi_nfiles as usize) {
///             for fd in &fds {
///                 match fd.proc_fdtype.into() {
///                     ProcFDType::Socket => {
///                         if let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) {
///                             match SocketInfoKind::from(socket.psi.soi_kind) {
///                                 Some(SocketInfoKind::Tcp) => {
///                                     // access to the member of `soi_proto` is unsafe becasuse of union type.
///                                     let info = unsafe { socket.psi.soi_proto.pri_tcp };
///
///                                     // change endian and cut off because the member of insi_lport is network endian and 16bit witdh.
///                                     let mut port = 0;
///                                     port |= info.tcpsi_ini.insi_lport >> 8 & 0x00ff;
///                                     port |= info.tcpsi_ini.insi_lport << 8 & 0xff00;
///
///                                     // access to the member of `insi_laddr` is unsafe becasuse of union type.
///                                     let s_addr = unsafe { info.tcpsi_ini.insi_laddr.ina_46.i46a_addr4.s_addr };
///
///                                     // change endian because the member of insi_laddr is network endian.
///                                     let mut addr = 0;
///                                     addr |= s_addr >> 24 & 0x000000ff;
///                                     addr |= s_addr >> 8  & 0x0000ff00;
///                                     addr |= s_addr << 8  & 0x00ff0000;
///                                     addr |= s_addr << 24 & 0xff000000;
///
///                                     println!("{}.{}.{}.{}:{}", addr >> 24 & 0xff, addr >> 16 & 0xff, addr >> 8 & 0xff, addr & 0xff, port);
///                                     assert_eq!(port, 8000);
///                                     assert_eq!(addr>>24&0xff, 127);
///                                     assert_eq!(addr>>16&0xff, 0  );
///                                     assert_eq!(addr>>8 &0xff, 0  );
///                                     assert_eq!(addr>>0 &0xff, 1  );
///                                 },
///                                 _ => (),
///                             }
///                         }
///                     },
///                     _ => (),
///                 }
///             }
///         }
///     }
/// }
/// ```
///
pub fn pidfdinfo<T: PIDFDInfo>(pid : i32, fd: int32_t) -> Result<T, String> {
    let flavor = T::flavor() as i32;
    let buffer_size = mem::size_of::<T>() as i32;
    let mut pidinfo = T::default();
    let buffer_ptr = &mut pidinfo as *mut _ as *mut c_void;
    let ret: i32;

    unsafe {
        ret = proc_pidfdinfo(pid, fd, flavor, buffer_ptr, buffer_size);
    };

    if ret <= 0 {
        Err(get_errno_with_message(ret))
    } else {
        Ok(pidinfo)
    }
}

#[test]
fn pidfdinfo_test() {
    use std::process;
    use std::net::TcpListener;
    let pid = process::id() as i32;

    let _listener = TcpListener::bind("127.0.0.1:65535");

    match pidinfo::<BSDInfo>(pid, 0) {
        Ok(info) => {
            match listpidinfo::<ListFDs>(pid, info.pbi_nfiles as usize) {
                Ok(fds) => {
                    for fd in fds {
                        match fd.proc_fdtype.into() {
                            ProcFDType::Socket => {
                                if let Ok(socket) = pidfdinfo::<SocketFDInfo>(pid, fd.proc_fd) {
                                    match SocketInfoKind::from(socket.psi.soi_kind) {
                                        Some(SocketInfoKind::Tcp) => unsafe {
                                            let info = socket.psi.soi_proto.pri_tcp;
                                            assert_eq!(socket.psi.soi_protocol,
                                            libc::IPPROTO_TCP);
                                            assert_eq!(info.tcpsi_ini.insi_lport as u32, 65535);
                                        }
                                        _ => assert!(false),
                                    }
                                }
                            },
                            _ => (),
                        }
                    }
                },
                Err(err) => assert!(false, "Error retrieving process info: {}", err)
            }
        },
        Err(err) => assert!(false, "Error retrieving process info: {}", err)
    };
}

#[repr(C)]
#[derive(Default)]
pub struct SocketFDInfo {
    pub pfi: ProcFileInfo,
    pub psi: SocketInfo,
}

impl PIDFDInfo for SocketFDInfo {
    fn flavor() -> PidFDInfoFlavor { PidFDInfoFlavor::SocketInfo }
}

#[repr(C)]
#[derive(Default)]
pub struct ProcFileInfo {
    pub fi_openflags: uint32_t,
    pub fi_status   : uint32_t,
    pub fi_offset   : off_t,
    pub fi_type     : int32_t,
    pub rfu_1       : int32_t,
}

#[derive(Copy, Clone, Debug)]
pub enum SocketInfoKind {
    Generic   = 0,
    /// IPv4 and IPv6 Sockets
    In        = 1,
    /// TCP Sockets
    Tcp       = 2,
    /// Unix Domain Sockets
    Un        = 3,
    /// PF_NDRV Sockets
    Ndrv      = 4,
    /// Kernel Event Sockets
    KernEvent = 5,
    /// Kernel Control Sockets
    KernCtl   = 6
}

impl SocketInfoKind {
    pub fn from(value: c_int) -> Option<SocketInfoKind> {
        match value {
            0 => Some(SocketInfoKind::Generic  ),
            1 => Some(SocketInfoKind::In       ),
            2 => Some(SocketInfoKind::Tcp      ),
            3 => Some(SocketInfoKind::Un       ),
            4 => Some(SocketInfoKind::Ndrv     ),
            5 => Some(SocketInfoKind::KernEvent),
            6 => Some(SocketInfoKind::KernCtl  ),
            _ => None
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct SocketInfo {
    pub soi_stat    : VInfoStat,
    pub soi_so      : uint64_t,
    pub soi_pcb     : uint64_t,
    pub soi_type    : c_int,
    pub soi_protocol: c_int,
    pub soi_family  : c_int,
    pub soi_options : c_short,
    pub soi_linger  : c_short,
    pub soi_state   : c_short,
    pub soi_qlen    : c_short,
    pub soi_incqlen : c_short,
    pub soi_qlimit  : c_short,
    pub soi_timeo   : c_short,
    pub soi_error   : c_ushort,
    pub soi_oobmark : uint32_t,
    pub soi_rcv     : SockBufInfo,
    pub soi_snd     : SockBufInfo,
    pub soi_kind    : c_int,
    pub rfu_1       : uint32_t,
    pub soi_proto   : SocketInfoProto,
}

#[repr(C)]
#[derive(Default)]
pub struct VInfoStat {
    pub vst_dev          : uint32_t,
    pub vst_mode         : uint16_t,
    pub vst_nlink        : uint16_t,
    pub vst_ino          : uint64_t,
    pub vst_uid          : uid_t,
    pub vst_gid          : gid_t,
    pub vst_atime        : int64_t,
    pub vst_atimensec    : int64_t,
    pub vst_mtime        : int64_t,
    pub vst_mtimensec    : int64_t,
    pub vst_ctime        : int64_t,
    pub vst_ctimensec    : int64_t,
    pub vst_birthtime    : int64_t,
    pub vst_birthtimensec: int64_t,
    pub vst_size         : off_t,
    pub vst_blocks       : int64_t,
    pub vst_blksize      : int32_t,
    pub vst_flags        : uint32_t,
    pub vst_gen          : uint32_t,
    pub vst_rdev         : uint32_t,
    pub vst_qspare       : [int64_t; 2],
}

#[repr(C)]
#[derive(Default)]
pub struct SockBufInfo {
    pub sbi_cc   : uint32_t,
    pub sbi_hiwat: uint32_t,
    pub sbi_mbcnt: uint32_t,
    pub sbi_mbmax: uint32_t,
    pub sbi_lowat: uint32_t,
    pub sbi_flags: c_short,
    pub sbi_timeo: c_short,
}

#[repr(C)]
pub union SocketInfoProto {
    pub pri_in        : InSockInfo,
    pub pri_tcp       : TcpSockInfo,
    pub pri_un        : UnSockInfo,
    pub pri_ndrv      : NdrvInfo,
    pub pri_kern_event: KernEventInfo,
    pub pri_kern_ctl  : KernCtlInfo,
}

impl Default for SocketInfoProto {
    fn default() -> SocketInfoProto {
        SocketInfoProto {
            pri_in: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct In4In6Addr {
    pub i46a_pad32: [uint32_t; 3],
    pub i46a_addr4: in_addr,
}

impl Default for In4In6Addr {
    fn default() -> In4In6Addr {
        In4In6Addr {
            i46a_pad32: [0; 3],
            i46a_addr4: in_addr{s_addr: 0},
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InSockInfo {
    pub insi_fport : c_int,
    pub insi_lport : c_int,
    pub insi_gencnt: uint64_t,
    pub insi_flags : uint32_t,
    pub insi_flow  : uint32_t,
    pub insi_vflag : uint8_t,
    pub insi_ip_ttl: uint8_t,
    pub rfu_1      : uint32_t,
    pub insi_faddr : InSIAddr,
    pub insi_laddr : InSIAddr,
    pub insi_v4    : InSIV4,
    pub insi_v6    : InSIV6,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InSIV4 {
    pub in4_top: c_uchar,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct InSIV6 {
    pub in6_hlim   : uint8_t,
    pub in6_cksum  : c_int,
    pub in6_ifindex: c_ushort,
    pub in6_hops   : c_short,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union InSIAddr {
    pub ina_46: In4In6Addr,
    pub ina_6 : in6_addr,
}

impl Default for InSIAddr {
    fn default() -> InSIAddr {
        InSIAddr {
            ina_46: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TcpSIState {
    /// Closed
    Closed      = 0,
    /// Listening for connection
    Listen      = 1,
    /// Active, have sent syn
    SynSent     = 2,
    /// Have send and received syn
    SynReceived = 3,
    /// Established
    Established = 4,
    /// Rcvd fin, waiting for close
    CloseWait   = 5,
    /// Have closed, sent fin
    FinWait1    = 6,
    /// Closed xchd FIN; await FIN ACK
    Closing     = 7,
    /// Had fin and close; await FIN ACK
    LastAck     = 8,
    /// Have closed, fin is acked
    FinWait2    = 9,
    /// In 2*msl quiet wait after close
    TimeWait    = 10,
    /// Pseudo state: reserved
    Reserved    = 11,
}

impl TcpSIState {
    pub fn from(value: c_int) -> Option<TcpSIState> {
        match value {
            0  => Some(TcpSIState::Closed     ),
            1  => Some(TcpSIState::Listen     ),
            2  => Some(TcpSIState::SynSent    ),
            3  => Some(TcpSIState::SynReceived),
            4  => Some(TcpSIState::Established),
            5  => Some(TcpSIState::CloseWait  ),
            6  => Some(TcpSIState::FinWait1   ),
            7  => Some(TcpSIState::Closing    ),
            8  => Some(TcpSIState::LastAck    ),
            9  => Some(TcpSIState::FinWait2   ),
            10 => Some(TcpSIState::TimeWait   ),
            11 => Some(TcpSIState::Reserved   ),
            _  => None
        }
    }
}

const TSI_T_NTIMERS : usize = 4;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct TcpSockInfo {
    pub tcpsi_ini  : InSockInfo,
    pub tcpsi_state: c_int,
    pub tcpsi_timer: [c_int; TSI_T_NTIMERS],
    pub tcpsi_mss  : c_int,
    pub tcpsi_flags: uint32_t,
    pub rfu_1      : uint32_t,
    pub tcpsi_tp   : uint64_t,
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct UnSockInfo {
    pub unsi_conn_so : uint64_t,
    pub unsi_conn_pcb: uint64_t,
    pub unsi_addr    : UnSIAddr,
    pub unsi_caddr   : UnSIAddr,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union UnSIAddr {
    pub ua_sun  : sockaddr_un,
    pub ua_dummy: [c_char; SOCK_MAXADDRLEN as usize],
}

impl Default for UnSIAddr {
    fn default() -> UnSIAddr {
        UnSIAddr {
            ua_dummy: [0; SOCK_MAXADDRLEN as usize],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct NdrvInfo {
    pub ndrvsi_if_family: uint32_t,
    pub ndrvsi_if_unit  : uint32_t,
    pub ndrvsi_if_name  : [c_char; IF_NAMESIZE],
}

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct KernEventInfo {
    pub kesi_vendor_code_filter: uint32_t,
    pub kesi_class_filter      : uint32_t,
    pub kesi_subclass_filter   : uint32_t,
}

const MAX_KCTL_NAME : usize = 96;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KernCtlInfo {
    pub kcsi_id         : uint32_t,
    pub kcsi_reg_unit   : uint32_t,
    pub kcsi_flags      : uint32_t,
    pub kcsi_recvbufsize: uint32_t,
    pub kcsi_sendbufsize: uint32_t,
    pub kcsi_unit       : uint32_t,
    pub kcsi_name       : [c_char; MAX_KCTL_NAME],
}

impl Default for KernCtlInfo {
    fn default() -> KernCtlInfo {
        KernCtlInfo {
            kcsi_id         : 0,
            kcsi_reg_unit   : 0,
            kcsi_flags      : 0,
            kcsi_recvbufsize: 0,
            kcsi_sendbufsize: 0,
            kcsi_unit       : 0,
            kcsi_name       : [0; MAX_KCTL_NAME],
        }
    }
}

