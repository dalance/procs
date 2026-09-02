//! Minimal NT API surface that `windows-sys` does not export.
//!
//! Every structure below is transcribed field-for-field from
//! `winternl.h` (msys2 clang64); the source line is noted above each one.
//! Field names are kept verbatim so the definitions stay diffable against
//! the header, hence the `non_snake_case` allowances.

use std::ffi::c_void;
use std::mem::{align_of, size_of};
use std::ptr;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{HANDLE, HMODULE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Threading::IO_COUNTERS;

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

#[repr(C)]
#[allow(non_snake_case)]
#[derive(Copy, Clone)]
pub struct UNICODE_STRING {
    /// Byte length of `Buffer`, excluding any terminating NUL.
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct VM_COUNTERS {
    pub PeakVirtualSize: usize,
    pub VirtualSize: usize,
    pub PageFaultCount: u32,
    pub PeakWorkingSetSize: usize,
    pub WorkingSetSize: usize,
    pub QuotaPeakPagedPoolUsage: usize,
    pub QuotaPagedPoolUsage: usize,
    pub QuotaPeakNonPagedPoolUsage: usize,
    pub QuotaNonPagedPoolUsage: usize,
    pub PagefileUsage: usize,
    pub PeakPagefileUsage: usize,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_THREADS {
    pub KernelTime: i64,
    pub UserTime: i64,
    pub CreateTime: i64,
    pub WaitTime: u32,
    pub StartAddress: *mut c_void,
    pub ClientId: CLIENT_ID,
    pub Priority: i32,
    pub BasePriority: i32,
    pub ContextSwitchCount: u32,
    pub State: u32,
    pub WaitReason: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_PROCESS_INFORMATION {
    pub NextEntryOffset: u32,
    pub NumberOfThreads: u32,
    pub WorkingSetPrivateSize: u64,
    pub HardFaultCount: u32,
    pub NumberOfThreadsHighWatermark: u32,
    pub CycleTime: u64,
    pub CreateTime: i64,
    pub UserTime: i64,
    pub KernelTime: i64,
    pub ImageName: UNICODE_STRING,
    pub BasePriority: i32,
    pub UniqueProcessId: HANDLE,
    pub InheritedFromUniqueProcessId: HANDLE,
    pub HandleCount: u32,
    pub SessionId: u32,
    pub PageDirectoryBase: u32,
    pub VirtualMemoryCounters: VM_COUNTERS,
    pub PrivatePageCount: usize,
    pub IoCounters: IO_COUNTERS,
    /// C99-style flexible array member. The count is `NumberOfThreads`
    pub Threads: [SYSTEM_THREADS; 0],
    // String buffer for `ImageName`
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SYSTEM_PROCESS_INFORMATION>() == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SYSTEM_THREADS>() == 80);

impl SYSTEM_PROCESS_INFORMATION {
    /// Borrows the `Threads[]` flexible array trailing this process header.
    ///
    /// `NumberOfThreads` is the count the kernel stored, and for every entry
    /// except the last the following `SYSTEM_PROCESS_INFORMATION` (at
    /// `NextEntryOffset`) bounds the array, so the slice can never overreach
    /// the next entry. The last entry has `NextEntryOffset == 0`; there the
    /// buffer is sized by the kernel for exactly `NumberOfThreads` entries.
    pub fn threads(&self) -> ThreadSlice<'_> {
        let aligned = (self.Threads.as_ptr() as usize) % align_of::<SYSTEM_THREADS>() == 0;
        let len = if aligned {
            self.NumberOfThreads as usize
        } else {
            0
        };
        // SAFETY: `Threads` is the C99 flexible array member. `self` lives in
        // a live snapshot buffer and `len` is the count the kernel reported, so
        // the `len` `SYSTEM_THREADS` following the header are valid and the
        // pointer is aligned (verified above).
        let threads = unsafe { std::slice::from_raw_parts(self.Threads.as_ptr(), len) };
        ThreadSlice {
            threads,
            pid: self.UniqueProcessId as usize,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// SystemInformationClass
pub const SYSTEM_PROCESS_INFORMATION_CLASS: u32 = 5;
/// ProcessInformationClass
pub const PROCESS_COMMAND_LINE_INFORMATION_CLASS: u32 = 60;
/// NTSTATUS
pub const STATUS_SUCCESS: i32 = 0;
pub const STATUS_INFO_LENGTH_MISMATCH: i32 = 0xC000_0004u32 as i32;

/// Guard against a nonsensical `UNICODE_STRING::Length` turning into a huge
/// allocation. Real command lines are capped well below this.
const MAX_COMMAND_LINE_BYTES: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Function resolution
// ---------------------------------------------------------------------------

#[allow(non_snake_case)]
type NtQuerySystemInformationFn = unsafe extern "system" fn(
    SystemInformationClass: u32,
    SystemInformation: *mut c_void,
    SystemInformationLength: u32,
    ReturnLength: *mut u32,
) -> i32;

#[allow(non_snake_case)]
type NtQueryInformationProcessFn = unsafe extern "system" fn(
    ProcessHandle: HANDLE,
    ProcessInformationClass: u32,
    ProcessInformation: *mut c_void,
    ProcessInformationLength: u32,
    ReturnLength: *mut u32,
) -> i32;

static NT_QUERY_SYSTEM_INFORMATION: OnceLock<Option<NtQuerySystemInformationFn>> = OnceLock::new();
static NT_QUERY_INFORMATION_PROCESS: OnceLock<Option<NtQueryInformationProcessFn>> =
    OnceLock::new();

/// `ntdll.dll` is mapped into every user process, so a handle lookup is enough
/// and there is nothing to release. Stored as `usize` because a bare
/// `HMODULE` (a raw pointer) is not `Sync` and cannot live in a `static`.
fn ntdll() -> HMODULE {
    static HANDLE: OnceLock<usize> = OnceLock::new();
    let addr = *HANDLE.get_or_init(|| {
        let name: Vec<u16> = "ntdll.dll\0".encode_utf16().collect();
        unsafe { GetModuleHandleW(name.as_ptr()) as usize }
    });
    addr as HMODULE
}

fn ntdll_proc(name: &[u8]) -> Option<unsafe extern "system" fn() -> isize> {
    let module = ntdll();
    if module.is_null() {
        return None;
    }
    // SAFETY: `name` is NUL terminated and outlives the call.
    unsafe { GetProcAddress(module, name.as_ptr()) }
}

pub fn nt_query_system_information() -> Option<NtQuerySystemInformationFn> {
    *NT_QUERY_SYSTEM_INFORMATION.get_or_init(|| {
        ntdll_proc(b"NtQuerySystemInformation\0")
            // SAFETY: both are `extern "system"` fn pointers; the ABI matches.
            .map(|f| unsafe { std::mem::transmute::<_, NtQuerySystemInformationFn>(f) })
    })
}

pub fn nt_query_information_process() -> Option<NtQueryInformationProcessFn> {
    *NT_QUERY_INFORMATION_PROCESS.get_or_init(|| {
        ntdll_proc(b"NtQueryInformationProcess\0")
            // SAFETY: both are `extern "system"` fn pointers; the ABI matches.
            .map(|f| unsafe { std::mem::transmute::<_, NtQueryInformationProcessFn>(f) })
    })
}

// ---------------------------------------------------------------------------
// SystemProcessInformation
// ---------------------------------------------------------------------------

const INITIAL_BUF: usize = 64 * 1024;

/// Takes one `SystemProcessInformation` snapshot.
///
/// The buffer is a `Vec<u64>` rather than a `Vec<u8>` because the structures
/// require 8-byte alignment and a byte vector only guarantees 1.
pub fn query_system_processes() -> Option<Vec<u64>> {
    let query = nt_query_system_information()?;
    let mut words = vec![0u64; INITIAL_BUF / size_of::<u64>()];

    for _ in 0..8 {
        let mut ret_len: u32 = 0;
        let len = words.len() * size_of::<u64>();
        let status = unsafe {
            query(
                SYSTEM_PROCESS_INFORMATION_CLASS,
                words.as_mut_ptr().cast::<c_void>(),
                len as u32,
                ptr::addr_of_mut!(ret_len),
            )
        };

        if status == STATUS_INFO_LENGTH_MISMATCH {
            // `ret_len` is allowed to come back as 0, so double on our own too.
            let want = (ret_len as usize).max(len * 2) + 4096;
            words.resize(want.div_ceil(size_of::<u64>()), 0);
            continue;
        }
        if status < 0 {
            return None;
        }
        debug_assert_eq!(status, STATUS_SUCCESS);
        return Some(words);
    }

    None
}

/// Decoded `SYSTEM_THREADS`, with the raw handles already narrowed to ids.
pub struct ThreadInfo {
    pub tid: i32,
    pub create_time: i64,
    pub kernel_time: u64,
    pub user_time: u64,
    pub priority: i32,
}

/// The `SYSTEM_THREADS` array trailing a `SYSTEM_PROCESS_INFORMATION`.
pub struct ThreadSlice<'a> {
    threads: &'a [SYSTEM_THREADS],
    /// Owning process id, used to confirm the layout is what we expect.
    pid: usize,
}

impl ThreadSlice<'_> {
    pub fn len(&self) -> usize {
        self.threads.len()
    }

    pub fn get(&self, index: usize) -> Option<ThreadInfo> {
        let thread = self.threads.get(index)?;

        // Every thread carries its owning process id. If a future Windows
        // release moves the fields around this will not match, and the entry is
        // dropped instead of being read as garbage.
        if thread.ClientId.UniqueProcess as usize != self.pid {
            return None;
        }

        Some(ThreadInfo {
            tid: thread.ClientId.UniqueThread as usize as i32,
            create_time: thread.CreateTime,
            kernel_time: thread.KernelTime as u64,
            user_time: thread.UserTime as u64,
            priority: thread.Priority,
        })
    }
}

/// Walks the `SYSTEM_PROCESS_INFORMATION` entries of a buffer returned by
/// [`query_system_processes`].
pub struct ProcessIter<'a> {
    buf: &'a [u64],
    offset: usize,
}

impl<'a> ProcessIter<'a> {
    pub fn new(buf: &'a [u64]) -> Self {
        Self { buf, offset: 0 }
    }

    fn byte_len(&self) -> usize {
        std::mem::size_of_val(self.buf)
    }
}

impl<'a> Iterator for ProcessIter<'a> {
    type Item = &'a SYSTEM_PROCESS_INFORMATION;

    fn next(&mut self) -> Option<&'a SYSTEM_PROCESS_INFORMATION> {
        let limit = self.byte_len();
        let offset = self.offset;
        let header = size_of::<SYSTEM_PROCESS_INFORMATION>();
        if offset + header > limit {
            return None;
        }

        let base = self.buf.as_ptr() as *const u8;
        // SAFETY: the buffer is 8-byte aligned and `offset` only ever advances
        // by `NextEntryOffset`, which keeps each entry aligned.
        let info: &'a SYSTEM_PROCESS_INFORMATION =
            unsafe { &*base.add(offset).cast::<SYSTEM_PROCESS_INFORMATION>() };

        // `NextEntryOffset == 0` marks the last entry. Anything that would
        // rewind, or leave the next entry misaligned, is malformed - stop
        // rather than loop forever or dereference a bad pointer.
        self.offset = match info.NextEntryOffset as usize {
            0 => limit,
            next if next < header || next % align_of::<SYSTEM_PROCESS_INFORMATION>() != 0 => limit,
            next => offset.saturating_add(next),
        };

        Some(info)
    }
}

// ---------------------------------------------------------------------------
// ProcessCommandLineInformation
// ---------------------------------------------------------------------------

/// Reads the full command line of `handle`.
///
/// `ProcessCommandLineInformation` returns the `UNICODE_STRING` describing
/// `PEB->ProcessParameters->CommandLine`. The returned `UNICODE_STRING::Buffer`
/// is expected to point into the output buffer, where the kernel has copied
/// the command-line characters.
///
/// Needs `PROCESS_QUERY_LIMITED_INFORMATION` and Windows 8.1 or newer.
/// Anything else yields `None`.
pub fn process_command_line(handle: HANDLE) -> Option<String> {
    let query = nt_query_information_process()?;

    // Room for the header plus a typical command line.
    let mut buf = vec![0u64; (size_of::<UNICODE_STRING>() + 1024).div_ceil(size_of::<u64>())];
    let mut status = STATUS_INFO_LENGTH_MISMATCH;

    for _ in 0..4 {
        let mut ret_len: u32 = 0;
        status = unsafe {
            query(
                handle,
                PROCESS_COMMAND_LINE_INFORMATION_CLASS,
                buf.as_mut_ptr().cast::<c_void>(),
                (buf.len() * size_of::<u64>()) as u32,
                ptr::addr_of_mut!(ret_len),
            )
        };
        if status != STATUS_INFO_LENGTH_MISMATCH {
            break;
        }
        let want = (ret_len as usize).max(buf.len() * size_of::<u64>() * 2);
        if want > MAX_COMMAND_LINE_BYTES {
            return None;
        }
        buf.resize(want.div_ceil(size_of::<u64>()), 0);
    }
    if status < 0 {
        return None;
    }

    // SAFETY: the buffer is 8-byte aligned and the query reported success.
    let info: &UNICODE_STRING = unsafe { &*buf.as_ptr().cast::<UNICODE_STRING>() };
    if info.Buffer.is_null() || info.Length == 0 {
        return None;
    }
    let bytes = info.Length as usize;
    if bytes > MAX_COMMAND_LINE_BYTES {
        return None;
    }

    let start = buf.as_ptr() as usize;
    let end = start + buf.len() * size_of::<u64>();
    let addr = info.Buffer as usize;
    let last = addr.checked_add(bytes)?;
    if addr < start || last > end {
        return None;
    }

    // SAFETY: the range was just verified to lie inside `buf`, which is still alive.
    let chars = unsafe { std::slice::from_raw_parts(info.Buffer, bytes / 2) };

    Some(
        String::from_utf16_lossy(chars)
            .trim_end_matches('\0')
            .to_owned(),
    )
}

/// Copies the characters of `string` out of `buffer`, which must still be
/// alive (the `Buffer` pointer of an `ImageName` points inside it).
pub fn unicode_string_to_owned(string: &UNICODE_STRING) -> String {
    if string.Buffer.is_null() || string.Length == 0 {
        return String::new();
    }
    let len = (string.Length / 2) as usize;
    // SAFETY: `Buffer` points into the snapshot buffer, whose lifetime covers
    // the `'a` of the `ProcessEntry` this came from.
    let slice = unsafe { std::slice::from_raw_parts(string.Buffer, len) };
    String::from_utf16_lossy(slice)
}
