use crate::process::{ProcessInfo, thread_id};
use crate::{column_default, Column};
use std::cmp;
use std::collections::HashMap;

#[cfg(target_os = "macos")]
const CTL_MAXNAME: i32 = 12;
#[cfg(target_os = "macos")]
const P_TRANSLATED: i32 = 131072;
#[cfg(target_os = "macos")]
const CPU_TYPE_X86_64: i32 = 16777223;
#[cfg(target_os = "macos")]
const CPU_TYPE_ARM64: i32 = 16777228;

pub struct Arch {
    header: String,
    unit: String,
    fmt_contents: HashMap<i64, String>,
    raw_contents: HashMap<i64, String>,
    width: usize,
}

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

impl Column for Arch {
    fn add(&mut self, proc: &ProcessInfo) {
        let pid = proc.pid;
        // A thread row is left blank: an image belongs to a process, and its
        // key is a negated thread id the OS does not know - on macOS that id
        // is 64 bit wide, so handing it to `sysctl` would truncate it and
        // could name a different process.
        let arch = if thread_id(pid).is_some() {
            ""
        } else {
            arch_from_pid(pid)
        };

        let fmt_content = arch.to_string();
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    column_default!(String, false);
}

// ---------------------------------------------------------------------------
// Windows
// ---------------------------------------------------------------------------

/// `IMAGE_FILE_MACHINE_*` constants.
///
/// <https://learn.microsoft.com/zh-cn/windows/win32/sysinfo/image-file-machine-constants>
#[cfg(target_os = "windows")]
mod machine {
    pub const UNKNOWN: u16 = 0x0000;
    pub const ALPHA: u16 = 0x0184;
    /// Same value as `IMAGE_FILE_MACHINE_AXP64`.
    pub const ALPHA64: u16 = 0x0284;
    pub const AM33: u16 = 0x01d3;
    pub const AMD64: u16 = 0x8664;
    pub const ARM: u16 = 0x01c0;
    pub const ARM64: u16 = 0xaa64;
    /// x86_64 code that runs on ARM64 through emulation.
    pub const ARM64EC: u16 = 0xa641;
    /// Image loadable both as ARM64 and as ARM64EC.
    pub const ARM64X: u16 = 0xa64e;
    /// ARMv7 / Thumb-2; `IMAGE_FILE_MACHINE_ARMV7` shares this value.
    pub const ARMNT: u16 = 0x01c4;
    pub const CEE: u16 = 0xc0ee;
    pub const CEF: u16 = 0x0cef;
    pub const EBC: u16 = 0x0ebc;
    pub const I386: u16 = 0x014c;
    pub const IA64: u16 = 0x0200;
    pub const M32R: u16 = 0x9041;
    pub const MIPS16: u16 = 0x0266;
    pub const MIPSFPU: u16 = 0x0366;
    pub const MIPSFPU16: u16 = 0x0466;
    pub const POWERPC: u16 = 0x01f0;
    pub const POWERPCFP: u16 = 0x01f1;
    pub const R3000: u16 = 0x0162;
    pub const R4000: u16 = 0x0166;
    pub const R10000: u16 = 0x0168;
    pub const SH3: u16 = 0x01a2;
    pub const SH3DSP: u16 = 0x01a3;
    pub const SH3E: u16 = 0x01a4;
    pub const SH4: u16 = 0x01a6;
    pub const SH5: u16 = 0x01a8;
    pub const THUMB: u16 = 0x01c2;
    pub const TRICORE: u16 = 0x0520;
    pub const WCEMIPSV2: u16 = 0x0169;
}

/// Reports the architecture of the image `pid` was started from.
///
/// WOW64 processes report their own architecture, not the host's: a 32-bit
/// process on an x86_64 machine yields `x86`.
#[cfg(target_os = "windows")]
pub fn arch_from_pid(pid: i64) -> &'static str {
    use windows_sys::Win32::Foundation::{CloseHandle, FALSE, HANDLE};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    // 0 is the idle process and has no image; negative pids are not real.
    if pid <= 0 {
        return "unknown";
    }

    // SAFETY: `pid` is only handed to `OpenProcess`, and the handle it hands
    // back is closed before this function returns.
    let handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid as u32) };
    if handle.is_null() {
        return "unknown";
    }

    let arch = crate::process::process_image_machine(handle)
        .map(arch_from_machine)
        .unwrap_or("unknown");
    unsafe { CloseHandle(handle) };

    arch
}

/// Names an `IMAGE_FILE_MACHINE_*` value the way the rest of the crate spells
/// architectures (`x86_64`, `arm64`, ...).
#[cfg(target_os = "windows")]
fn arch_from_machine(machine: u16) -> &'static str {
    use machine::*;

    match machine {
        UNKNOWN => "unknown",
        I386 => "x86",
        AMD64 => "x86_64",
        ARM | ARMNT | THUMB => "arm",
        ARM64 => "arm64",
        ARM64EC => "arm64ec",
        ARM64X => "arm64x",
        IA64 => "ia64",
        EBC => "ebc",
        ALPHA | ALPHA64 => "alpha",
        MIPS16 | MIPSFPU | MIPSFPU16 | R3000 | R4000 | R10000 | WCEMIPSV2 => "mips",
        POWERPC | POWERPCFP => "ppc",
        SH3 | SH3DSP | SH3E | SH4 | SH5 => "sh",
        M32R => "m32r",
        AM33 => "am33",
        TRICORE => "tricore",
        CEF => "cef",
        // Pure IL assembly: no architecture of its own.
        CEE => "msil",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// macOS
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
pub fn arch_from_pid(pid: i64) -> &'static str {
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

    mib[length as usize] = pid as libc::c_int;
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
        mib[3] = pid as libc::c_int;

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

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{arch_from_machine, machine};

    #[test]
    fn machine_names() {
        assert_eq!(arch_from_machine(machine::AMD64), "x86_64");
        assert_eq!(arch_from_machine(machine::I386), "x86");
        assert_eq!(arch_from_machine(machine::ARM64), "arm64");
        assert_eq!(arch_from_machine(machine::ARM64EC), "arm64ec");
        // ARMv7 and Thumb-2 are both 32-bit ARM.
        assert_eq!(arch_from_machine(machine::ARMNT), "arm");
        assert_eq!(arch_from_machine(machine::THUMB), "arm");
        // No image, and a value outside the table.
        assert_eq!(arch_from_machine(machine::UNKNOWN), "unknown");
        assert_eq!(arch_from_machine(0x1234), "unknown");
    }
}
