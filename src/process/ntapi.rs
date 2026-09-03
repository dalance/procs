//! Minimal NT API surface that `windows-sys` does not export.
//!
//! Every structure below is transcribed field-for-field from
//! `winternl.h` (msys2 clang64); the source line is noted above each one.
//! Field names are kept verbatim so the definitions stay diffable against
//! the header, hence the `non_snake_case` allowances.

use std::ffi::c_void;
use std::marker::PhantomData;
use std::mem::{offset_of, size_of};
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

/// `SYSTEM_EXTENDED_THREAD_INFORMATION` - the thread record used by
/// `SystemExtendedProcessInformation` (57) and `SystemFullProcessInformation`
/// (148) in place of `SYSTEM_THREADS`.
///
/// The leading union member is a plain `SYSTEM_THREADS`, which is the only
/// part this crate reads; the trailing pointers are here so the array stride
/// (used to locate the process extension) is the real one.
#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_EXTENDED_THREAD_INFORMATION {
    pub ThreadInfo: SYSTEM_THREADS,
    pub StackBase: *mut c_void,
    pub StackLimit: *mut c_void,
    pub Win32StartAddress: *mut c_void,
    /// The base address of the memory region containing the TEB. Since VISTA.
    pub TebBaseAddress: *mut c_void,
    pub Reserved2: usize,
    pub Reserved3: usize,
    pub Reserved4: usize,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SYSTEM_PROCESS_INFORMATION>() == 256);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SYSTEM_THREADS>() == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SYSTEM_EXTENDED_THREAD_INFORMATION>() == 136);

/// `PROCESS_DISK_COUNTERS`
#[repr(C)]
#[allow(non_snake_case)]
pub struct PROCESS_DISK_COUNTERS {
    pub BytesRead: u64,
    pub BytesWritten: u64,
    pub ReadOperationCount: u64,
    pub WriteOperationCount: u64,
    pub FlushOperationCount: u64,
}

/// The leading, version-stable part of `SYSTEM_PROCESS_INFORMATION_EXTENSION`,
/// as returned by `SystemFullProcessInformation` (148).
///
/// Only the prefix is declared. Everything after `PackageFullNameOffset` -
/// `PROCESS_ENERGY_VALUES` and the fields behind it - has been resized and
/// reordered repeatedly across Windows 10 and 11 builds; measured sizes on
/// this machine are 368 bytes and up, so neither the trailing fields nor
/// `size_of` of the whole structure can be relied on. The variable-length
/// data the offsets point at sits *past* the real structure and is reached
/// purely through those offsets, which is why they are all this crate needs.
///
/// Only `UserSidOffset` is read today; the rest documents the layout and
/// guards the offsets with the assertions below.
#[repr(C)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct SYSTEM_PROCESS_INFORMATION_EXTENSION {
    pub DiskCounters: PROCESS_DISK_COUNTERS,
    pub ContextSwitches: u64,
    /// `HasStrongId` / `Classification` / `BackgroundActivityModerated` bits.
    pub Flags: u32,
    /// Byte offset from the start of this structure to the user SID, or 0.
    pub UserSidOffset: u32,
    /// Byte offset from the start of this structure to the package name, or 0.
    pub PackageFullNameOffset: u32,
}

const _: () = assert!(offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, DiskCounters) == 0);
const _: () = assert!(offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, ContextSwitches) == 40);
const _: () = assert!(offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, Flags) == 48);
const _: () = assert!(offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, UserSidOffset) == 52);
const _: () =
    assert!(offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, PackageFullNameOffset) == 56);
const _: () = assert!(size_of::<SYSTEM_PROCESS_INFORMATION_EXTENSION>() == 64);

/// Fixed part of a `SID`: revision, sub-authority count and the 6-byte
/// identifier authority. `windows-sys` types the trailing sub-authorities as
/// `[u32; 1]`, so `size_of::<SID>()` is 4 bytes more than this.
const SID_HEADER_BYTES: usize = 8;
const SID_REVISION: u8 = 1;
/// `SID_MAX_SUB_AUTHORITIES`
const SID_MAX_SUB_AUTHORITIES: usize = 15;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// SystemInformationClass
pub const SYSTEM_PROCESS_INFORMATION_CLASS: u32 = 5;
/// SystemInformationClass. Windows 8.1 and newer, and the caller has to be
/// elevated: without it the query returns `STATUS_ACCESS_DENIED`. Carries
/// the process extension, which includes the user SID.
pub const SYSTEM_FULL_PROCESS_INFORMATION_CLASS: u32 = 148;
/// ProcessInformationClass. Windows 8.1 and newer
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

/// Which information class produced a snapshot.
///
/// This decides how the entries are laid out: `Full` carries the wider thread
/// records and a `SYSTEM_PROCESS_INFORMATION_EXTENSION` after them.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SnapshotKind {
    /// `SystemProcessInformation` (5): always available.
    Basic,
    /// `SystemFullProcessInformation` (148): Windows 8.1 and elevation.
    Full,
}

impl SnapshotKind {
    /// Distance between two thread records of this snapshot.
    fn thread_stride(self) -> usize {
        match self {
            Self::Basic => size_of::<SYSTEM_THREADS>(),
            Self::Full => size_of::<SYSTEM_EXTENDED_THREAD_INFORMATION>(),
        }
    }

    /// Whether each entry carries a `SYSTEM_PROCESS_INFORMATION_EXTENSION`.
    fn has_extension(self) -> bool {
        match self {
            Self::Basic => false,
            Self::Full => true,
        }
    }
}

/// A process snapshot: the buffer the kernel filled plus the class that did it.
pub struct SystemProcessSnapshot {
    words: Vec<u64>,
    kind: SnapshotKind,
}

impl SystemProcessSnapshot {
    pub fn iter(&self) -> ProcessIter<'_> {
        ProcessIter::new(self)
    }
}

/// Takes one process snapshot.
///
/// `SystemFullProcessInformation` is tried first: it carries the process user
/// SID, which saves an `OpenProcessToken` per process. Any failure - an older
/// Windows that does not know the class, or a caller without the privilege it
/// requires - is reported as a negative status, so we quietly fall back to the
/// basic class.
///
/// The buffer is a `Vec<u64>` rather than a `Vec<u8>` because the structures
/// require 8-byte alignment and a byte vector only guarantees 1.
pub fn query_system_processes() -> Option<SystemProcessSnapshot> {
    if let Some(words) = query_system_info_class(SYSTEM_FULL_PROCESS_INFORMATION_CLASS) {
        return Some(SystemProcessSnapshot {
            words,
            kind: SnapshotKind::Full,
        });
    }

    let words = query_system_info_class(SYSTEM_PROCESS_INFORMATION_CLASS)?;
    Some(SystemProcessSnapshot {
        words,
        kind: SnapshotKind::Basic,
    })
}

fn query_system_info_class(class: u32) -> Option<Vec<u64>> {
    let query = nt_query_system_information()?;
    let mut words = vec![0u64; 0];

    for _ in 0..8 {
        let mut ret_len: u32 = 0;
        let len = words.len() * size_of::<u64>();
        let status = unsafe {
            query(
                class,
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

/// The thread records trailing a `SYSTEM_PROCESS_INFORMATION`.
///
/// Held as a base pointer plus a stride rather than a `&[SYSTEM_THREADS]`:
/// the records are `SYSTEM_EXTENDED_THREAD_INFORMATION` (136 bytes) in a full
/// snapshot, of which only the leading `SYSTEM_THREADS` part is read, and
/// neither variant is guaranteed to sit on an 8-byte boundary.
pub struct ThreadSlice<'a> {
    base: *const u8,
    stride: usize,
    len: usize,
    /// Owning process id, used to confirm the layout is what we expect.
    pid: usize,
    _lifetime: PhantomData<&'a [u64]>,
}

impl ThreadSlice<'_> {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<ThreadInfo> {
        if index >= self.len {
            return None;
        }

        // SAFETY: `index < len` and `len` was clamped so that
        // `base + (len - 1) * stride + size_of` stays inside the entry. The
        // record is copied because it can be misaligned.
        let thread: SYSTEM_THREADS = unsafe {
            self.base
                .add(index * self.stride)
                .cast::<SYSTEM_THREADS>()
                .read_unaligned()
        };

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

/// One entry of a snapshot: the process header plus everything that depends on
/// knowing where the entry begins and ends.
///
/// The header is a copy rather than a borrow. Entries of a full snapshot are
/// only guaranteed to be even-aligned - variable-length data such as a package
/// name makes `NextEntryOffset` arbitrary - so a `&SYSTEM_PROCESS_INFORMATION`
/// over the buffer would be misaligned often enough to matter.
pub struct ProcessEntry<'a> {
    buf: &'a [u64],
    /// Unaligned copy of the entry's leading `SYSTEM_PROCESS_INFORMATION`.
    header: SYSTEM_PROCESS_INFORMATION,
    /// Byte offset of this entry within `buf`.
    start: usize,
    /// Byte offset one past this entry: the bound for its variable-length data.
    end: usize,
    kind: SnapshotKind,
}

impl<'a> ProcessEntry<'a> {
    pub fn info(&self) -> &SYSTEM_PROCESS_INFORMATION {
        &self.header
    }

    /// The `ImageName`, reduced to the file name when the snapshot supplied a
    /// full path.
    ///
    /// `SystemFullProcessInformation` reports
    /// `\Device\HarddiskVolumeN\...\foo.exe` where `SystemProcessInformation`
    /// reports `foo.exe`. The command column falls back to this for processes
    /// whose command line cannot be read, and the kernel-thread heuristic
    /// looks for a '.', so both classes are normalised to the short form -
    /// output must not change just because the caller happened to be elevated.
    pub fn image_name(&self) -> String {
        let name = unicode_string_to_owned(&self.header.ImageName);
        match self.kind {
            SnapshotKind::Basic => name,
            SnapshotKind::Full => match name.rsplit('\\').next() {
                Some(file) if !file.is_empty() => file.to_owned(),
                _ => name,
            },
        }
    }

    /// The `Threads[]` flexible array trailing the process header.
    ///
    /// `NumberOfThreads` is the count the kernel stored. For every entry except
    /// the last, the following entry (at `NextEntryOffset`) bounds the array;
    /// the last entry has `NextEntryOffset == 0` and is bounded by the buffer,
    /// which the kernel sized for exactly `NumberOfThreads` records.
    pub fn threads(&self) -> ThreadSlice<'a> {
        let header = size_of::<SYSTEM_PROCESS_INFORMATION>();
        let stride = self.kind.thread_stride();
        let first = self.start.saturating_add(header);

        let available = self.end.saturating_sub(first);
        let wanted = (self.header.NumberOfThreads as usize).saturating_mul(stride);
        let len = wanted.min(available) / stride;

        ThreadSlice {
            // SAFETY: `first` is inside `buf`, which outlives `'a`.
            base: unsafe { self.buf.as_ptr().cast::<u8>().add(first) },
            stride,
            len,
            pid: self.header.UniqueProcessId as usize,
            _lifetime: PhantomData,
        }
    }

    /// The process user SID from the extension, as an 8-byte aligned copy.
    ///
    /// `None` for a basic snapshot, for an entry that reports no SID, or when
    /// the offsets and sizes do not add up - the extension layout is not
    /// version-stable, so nothing here is trusted before it is checked.
    /// Byte offset of this entry's `SYSTEM_PROCESS_INFORMATION_EXTENSION`, with
    /// the bounds already checked. `None` when the snapshot carried no extension
    /// (a basic class-5 snapshot) or the entry is too short to hold the prefix.
    fn extension_base(&self) -> Option<usize> {
        if !self.kind.has_extension() {
            return None;
        }
        let header = size_of::<SYSTEM_PROCESS_INFORMATION>();
        let stride = self.kind.thread_stride();
        let base = self
            .start
            .checked_add(header)?
            .checked_add((self.header.NumberOfThreads as usize).checked_mul(stride)?)?;

        // The fixed prefix we read must fit inside the entry; the variable
        // length data trailing it is reached only through the offsets.
        let prefix = size_of::<SYSTEM_PROCESS_INFORMATION_EXTENSION>();
        if base.saturating_add(prefix) > self.end {
            return None;
        }
        Some(base)
    }

    pub fn user_sid(&self) -> Option<Vec<u64>> {
        let base = self.extension_base()?;

        // SAFETY: `extension_base` already verified `base + prefix <= end`.
        let fields: SYSTEM_PROCESS_INFORMATION_EXTENSION = unsafe {
            self.buf
                .as_ptr()
                .cast::<u8>()
                .add(base)
                .cast::<SYSTEM_PROCESS_INFORMATION_EXTENSION>()
                .read_unaligned()
        };

        if fields.UserSidOffset == 0 {
            return None;
        }

        let offset = base.checked_add(fields.UserSidOffset as usize)?;
        copy_sid(self.buf.as_ptr().cast::<u8>(), offset, self.end)
    }

    /// The kernel process classification (`SYSTEM_PROCESS_CLASSIFICATION`),
    /// decoded from the low four bits of `Flags` (bit 0 is `HasStrongId`).
    ///
    /// Only a full snapshot carries the extension, so this is `None` for a
    /// basic class-5 snapshot. A non-zero value marks the kernel's own
    /// processes - System, Secure System, Registry, Memory Compression - as
    /// opposed to normal user processes (value 0). `procs` uses this to hide
    /// kernel threads unless `--thread` is given.
    pub fn classification(&self) -> Option<u32> {
        let base = self.extension_base()?;
        // `Flags` sits at offset 48 within the extension; `Classification` is
        // its bits 1..=4.
        let flags: u32 = unsafe {
            self.buf
                .as_ptr()
                .cast::<u8>()
                .add(base + offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, Flags))
                .cast::<u32>()
                .read_unaligned()
        };
        Some((flags >> 1) & 0xF)
    }
}

/// Copies the SID at byte `offset` (bounded by `end`) out of `base`.
///
/// Returned as `Vec<u64>` because a SID is read back through `PSID`, whose
/// sub-authorities are `u32`: they have to stay aligned, and a byte vector
/// only guarantees 1.
fn copy_sid(base: *const u8, offset: usize, end: usize) -> Option<Vec<u64>> {
    if offset.saturating_add(SID_HEADER_BYTES) > end {
        return None;
    }

    // SAFETY: `offset + 8 <= end <= buf.len()` was just checked. Read as bytes
    // rather than through `SID`: the record can be misaligned.
    let revision = unsafe { base.add(offset).read() };
    let count = unsafe { base.add(offset + 1).read() };
    if revision != SID_REVISION {
        return None;
    }

    let sub_authorities = usize::from(count);
    if sub_authorities > SID_MAX_SUB_AUTHORITIES {
        return None;
    }

    let bytes = SID_HEADER_BYTES + sub_authorities * size_of::<u32>();
    if offset.saturating_add(bytes) > end {
        return None;
    }

    let mut words = vec![0u64; bytes.div_ceil(size_of::<u64>())];
    // SAFETY: `words` is `bytes` rounded up, and `offset + bytes <= end`, so
    // the copy stays inside both allocations. `base` and `words` do not
    // overlap.
    unsafe {
        ptr::copy_nonoverlapping(base.add(offset), words.as_mut_ptr().cast::<u8>(), bytes);
    }

    Some(words)
}

/// Walks the entries of a [`SystemProcessSnapshot`].
pub struct ProcessIter<'a> {
    snapshot: &'a SystemProcessSnapshot,
    offset: usize,
}

impl<'a> ProcessIter<'a> {
    fn new(snapshot: &'a SystemProcessSnapshot) -> Self {
        Self {
            snapshot,
            offset: 0,
        }
    }

    fn byte_len(&self) -> usize {
        self.snapshot.words.len() * size_of::<u64>()
    }
}

impl<'a> Iterator for ProcessIter<'a> {
    type Item = ProcessEntry<'a>;

    fn next(&mut self) -> Option<ProcessEntry<'a>> {
        let limit = self.byte_len();
        let offset = self.offset;
        let header = size_of::<SYSTEM_PROCESS_INFORMATION>();
        if offset + header > limit {
            return None;
        }

        // SAFETY: `offset + size_of <= limit`, so the header lies wholly inside
        // the buffer. Copied rather than borrowed: entries of a full snapshot
        // are not guaranteed to be 8-byte aligned.
        let info: SYSTEM_PROCESS_INFORMATION = unsafe {
            self.snapshot
                .words
                .as_ptr()
                .cast::<u8>()
                .add(offset)
                .cast::<SYSTEM_PROCESS_INFORMATION>()
                .read_unaligned()
        };

        // `NextEntryOffset == 0` marks the last entry. An offset that would not
        // move past this header is malformed - stop rather than loop forever.
        // Alignment is deliberately not required: a full snapshot pads entries
        // to 2 bytes, so demanding 8 would silently truncate the process list.
        let next = info.NextEntryOffset as usize;
        let end = if next == 0 {
            limit
        } else if next < header {
            limit
        } else {
            offset.saturating_add(next).min(limit)
        };
        self.offset = end;

        Some(ProcessEntry {
            buf: &self.snapshot.words,
            header: info,
            start: offset,
            end,
            kind: self.snapshot.kind,
        })
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    /// Bytes of S-1-5-18 (Local System), as a `SID` would lay them out.
    fn sid_local_system() -> Vec<u8> {
        let mut bytes = vec![1u8, 2]; // Revision, SubAuthorityCount
        bytes.extend_from_slice(&[0, 0, 0, 0, 0, 5]); // Authority
        bytes.extend_from_slice(&5u32.to_le_bytes()); // SubAuthority[0]
        bytes.extend_from_slice(&18u32.to_le_bytes()); // SubAuthority[1]
        bytes
    }

    fn sid_as_words(sid: &[u8]) -> Vec<u64> {
        let mut words = vec![0u64; sid.len().div_ceil(size_of::<u64>())];
        // SAFETY: `words` is `sid.len()` rounded up, so the copy is in bounds.
        unsafe {
            ptr::copy_nonoverlapping(sid.as_ptr(), words.as_mut_ptr().cast::<u8>(), sid.len());
        }
        words
    }

    /// Builds a single-entry full snapshot whose user SID is `sid`, with the
    /// SID appended right after the 64-byte extension prefix (so
    /// `UserSidOffset == 64`). An empty `sid` drops the data and sets the
    /// offset to 0. `flags` is written into the extension `Flags` field so
    /// `classification()` can be exercised.
    fn make_snapshot(sid: &[u8], flags: u32) -> SystemProcessSnapshot {
        let mut header: SYSTEM_PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        header.NextEntryOffset = 0;
        header.NumberOfThreads = 0;
        header.UniqueProcessId = 1234 as HANDLE;

        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &header as *const SYSTEM_PROCESS_INFORMATION as *const u8,
                size_of::<SYSTEM_PROCESS_INFORMATION>(),
            )
        };

        let mut ext: SYSTEM_PROCESS_INFORMATION_EXTENSION = unsafe { std::mem::zeroed() };
        let has_sid = !sid.is_empty();
        ext.UserSidOffset = if has_sid { 64 } else { 0 };
        ext.Flags = flags;

        let ext_bytes = unsafe {
            std::slice::from_raw_parts(
                &ext as *const SYSTEM_PROCESS_INFORMATION_EXTENSION as *const u8,
                size_of::<SYSTEM_PROCESS_INFORMATION_EXTENSION>(),
            )
        };

        let mut bytes: Vec<u8> = Vec::with_capacity(
            size_of::<SYSTEM_PROCESS_INFORMATION>() + ext_bytes.len() + sid.len(),
        );
        bytes.extend_from_slice(header_bytes);
        bytes.extend_from_slice(ext_bytes);
        bytes.extend_from_slice(sid);

        let mut words = vec![0u64; bytes.len().div_ceil(size_of::<u64>())];
        // SAFETY: `words` is `bytes.len()` rounded up.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), words.as_mut_ptr().cast::<u8>(), bytes.len());
        }

        SystemProcessSnapshot {
            words,
            kind: SnapshotKind::Full,
        }
    }

    #[test]
    fn full_snapshot_resolves_user_sid() {
        let sid = sid_local_system();
        let expected = sid_as_words(&sid);
        let snap = make_snapshot(&sid, 0);

        let mut iter = snap.iter();
        let entry = iter.next().expect("one entry");
        assert_eq!(entry.info().UniqueProcessId as usize, 1234);
        assert_eq!(entry.user_sid(), Some(expected));
        assert!(iter.next().is_none());
    }

    #[test]
    fn full_snapshot_without_sid_is_none() {
        let snap = make_snapshot(&[], 0);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.user_sid(), None);
    }

    #[test]
    fn truncated_sid_is_rejected() {
        // SID data present but shorter than a full header -> bounds checked away.
        let snap = make_snapshot(&[1, 2, 0, 0, 0, 0, 0, 5], 0);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.user_sid(), None);
    }

    #[test]
    fn basic_snapshot_has_no_extension() {
        // Without the extension, `user_sid` must always be `None` regardless of
        // the bytes that happen to follow the header.
        let mut buf = vec![0u64; 256 / 8];
        let snap = SystemProcessSnapshot {
            words: buf.clone(),
            kind: SnapshotKind::Basic,
        };
        assert_eq!(snap.iter().next().unwrap().user_sid(), None);
        // Now point it at what would be a valid SID in a full snapshot.
        let sid = sid_local_system();
        buf.extend(sid_as_words(&sid));
        let snap = SystemProcessSnapshot {
            words: buf,
            kind: SnapshotKind::Basic,
        };
        assert_eq!(snap.iter().next().unwrap().user_sid(), None);
    }

    #[test]
    fn classification_normal_is_zero() {
        // `Flags = 0` -> Classification field is 0 -> a normal user process.
        let snap = make_snapshot(&[], 0);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.classification(), Some(0));
    }

    #[test]
    fn classification_skips_hasstrongid_bit() {
        // Bit 0 of `Flags` is `HasStrongId`, not `Classification`; it must
        // not bleed into the decoded value.
        let snap = make_snapshot(&[], 1);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.classification(), Some(0));
    }

    #[test]
    fn classification_system_is_nonzero() {
        // `Classification = System` (1) at bits 1..=4 -> `Flags = 1 << 1 = 2`.
        let snap = make_snapshot(&[], 1 << 1);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.classification(), Some(1));
        assert!(entry.classification().is_some_and(|c| c != 0));
    }

    #[test]
    fn classification_registry_nonzero() {
        // `Classification = Registry` (4) -> `Flags = 4 << 1 = 8`.
        let snap = make_snapshot(&[], 4 << 1);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.classification(), Some(4));
    }

    #[test]
    fn basic_snapshot_has_no_classification() {
        // Only the full snapshot carries the extension, so `classification`
        // is `None` regardless of the trailing bytes.
        let mut buf = vec![0u64; 256 / 8];
        let snap = SystemProcessSnapshot {
            words: buf.clone(),
            kind: SnapshotKind::Basic,
        };
        assert_eq!(snap.iter().next().unwrap().classification(), None);
        let sid = sid_local_system();
        buf.extend(sid_as_words(&sid));
        let snap = SystemProcessSnapshot {
            words: buf,
            kind: SnapshotKind::Basic,
        };
        assert_eq!(snap.iter().next().unwrap().classification(), None);
    }
}
