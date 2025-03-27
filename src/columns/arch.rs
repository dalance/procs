use crate::process::ProcessInfo;
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

const CTL_MAXNAME: i32 = 12;
const P_TRANSLATED: i32 = 131072;
const CPU_TYPE_X86_64: i32 = 16777223;
const CPU_TYPE_ARM64: i32 = 16777228;

pub struct Arch {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
}

#[cfg(target_os = "macos")]
impl Arch {
    pub fn new(header: Option<String>) -> Self {
        let header = header.unwrap_or_else(|| String::from("Arch"));
        let unit = String::new();
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit
        }
    }
}

#[cfg(target_os = "macos")]
impl Column for Arch {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.pid;
        let arch = arch_from_pid(pid);

        let fmt_content = format!("{}", arch);
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
pub fn arch_from_pid(pid: i32) -> &'static str {
    use {
        libc::{sysctl, sysctlnametomib, cpu_type_t, size_t, CTL_KERN, KERN_PROC, KERN_PROC_PID},
        std::{mem, ffi::CString},
        crate::process::kinfo_proc,
    };

    let mut mib = [0; CTL_MAXNAME as usize];
    let mut length = CTL_MAXNAME as size_t;
    let mut cpu_type: cpu_type_t = -1;
    let mut size = mem::size_of::<cpu_type_t>();

    let sysctl_name = CString::new("sysctl.proc_cputype").unwrap();
    if unsafe { sysctlnametomib(sysctl_name.as_ptr(), mib.as_mut_ptr(), &mut length) } != 0 {
        return "unknown";
    }

    mib[length as usize] = pid;
    length += 1;

    if unsafe { sysctl(mib.as_mut_ptr(), length as u32, &mut cpu_type as *mut _ as *mut _, &mut size, core::ptr::null_mut(), 0) } != 0 {
        return "unknown";
    }

    if cpu_type == CPU_TYPE_X86_64 {
        return "x86_64";
    }

    if cpu_type == CPU_TYPE_ARM64 {
        let mut proc_info: kinfo_proc = unsafe { mem::zeroed() };
        mib[0] = CTL_KERN;
        mib[1] = KERN_PROC;
        mib[2] = KERN_PROC_PID;
        mib[3] = pid;

        length = 4;
        size = mem::size_of::<kinfo_proc>();

        if unsafe { sysctl(mib.as_mut_ptr(), length as u32, &mut proc_info as *mut _ as *mut _, &mut size, core::ptr::null_mut(), 0) } != 0 {
            return "arm64";
        }

        if (proc_info.kp_proc.p_flag & P_TRANSLATED) != 0 {
            return "x86_64";
        }
        return "arm64";
    }

    "unknown"
}
