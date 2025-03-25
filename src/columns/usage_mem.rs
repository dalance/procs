use crate::process::ProcessInfo;
use crate::{column_default, Column};
#[cfg(any(target_os = "linux", target_os = "android"))]
use procfs::{Current, Meminfo, WithCurrentSystemInfo};
use std::cmp;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::mem::{size_of, zeroed};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};

pub struct UsageMem {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, u32>,
    width: usize,
    mem_total: u64,
}

impl UsageMem {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("MEM"));
        let unit = String::from("[%]");

        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            mem_total: get_mem_total(),
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn get_mem_total() -> u64 {
    let meminfo = Meminfo::current();
    if let Ok(meminfo) = meminfo {
        meminfo.mem_total
    } else {
        0
    }
}

#[cfg(target_os = "macos")]
fn get_mem_total() -> u64 {
    let mut mem_total: u64 = 0;
    let mut mib = [0, 0];

    unsafe {
        crate::util::get_sys_value(
            libc::CTL_HW as u32,
            libc::HW_MEMSIZE as u32,
            std::mem::size_of::<u64>(),
            &mut mem_total as *mut u64 as *mut libc::c_void,
            &mut mib,
        );
    }

    mem_total
}

#[cfg(target_os = "windows")]
fn get_mem_total() -> u64 {
    let mut info: PERFORMANCE_INFORMATION = unsafe { zeroed() };
    let ret = unsafe { GetPerformanceInfo(&mut info, size_of::<PERFORMANCE_INFORMATION>() as u32) };

    if ret != 0 {
        info.PhysicalTotal as u64 * info.PageSize as u64
    } else {
        0
    }
}

#[cfg(target_os = "freebsd")]
fn get_mem_total() -> u64 {
    let mut mem_total: u64 = 0;
    let name = std::ffi::CString::new("hw.availpages").unwrap();
    let mut size = std::mem::size_of::<u64>();
    let ptr: *mut u64 = &mut mem_total;

    unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            ptr as *mut libc::c_void,
            &mut size,
            std::ptr::null(),
            0,
        );
    }

    mem_total
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for UsageMem {
    fn add(&mut self, proc: &ProcessInfo) {
        let usage = proc.curr_proc.stat().rss_bytes().get() as f64 * 100.0 / self.mem_total as f64;
        let fmt_content = format!("{usage:.1}");
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "macos")]
impl Column for UsageMem {
    fn add(&mut self, proc: &ProcessInfo) {
        let usage = proc.curr_task.ptinfo.pti_resident_size as f64 * 100.0 / self.mem_total as f64;
        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "windows")]
impl Column for UsageMem {
    fn add(&mut self, proc: &ProcessInfo) {
        let usage = proc.memory_info.working_set_size as f64 * 100.0 / self.mem_total as f64;
        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}

#[cfg(target_os = "freebsd")]
impl Column for UsageMem {
    fn add(&mut self, proc: &ProcessInfo) {
        let usage = proc.curr_proc.info.rssize as f64 * 100.0 / self.mem_total as f64;
        let fmt_content = format!("{:.1}", usage);
        let raw_content = (usage * 1000.0) as u32;

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(u32, true);
}
