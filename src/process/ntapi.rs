//! Minimal NT API surface that `windows-sys` does not export.
//!
//! Every structure below is transcribed field-for-field from
//! `winternl.h` (msys2 clang64); the source line is noted above each one.
//! Field names are kept verbatim so the definitions stay diffable against
//! the header, hence the `non_snake_case` allowances.

use std::ffi::c_void;
use std::fmt::Write;
use std::mem::{offset_of, size_of};
use std::ptr;
use std::slice;

use windows_sys::Wdk::System::SystemInformation::{
    NtQuerySystemInformation, SystemProcessInformation,
};
use windows_sys::Wdk::System::SystemServices::VM_COUNTERS;
use windows_sys::Wdk::System::Threading::{
    NtQueryInformationProcess, ProcessBasicInformation, ProcessCommandLineInformation,
    ProcessImageInformation,
};
use windows_sys::Win32::Foundation::UNICODE_STRING;
use windows_sys::Win32::Foundation::{HANDLE, STATUS_INFO_LENGTH_MISMATCH, STATUS_SUCCESS};
use windows_sys::Win32::Security::{PSID, SECURITY_MAX_SID_SIZE, SID};
use windows_sys::Win32::System::Threading::{IO_COUNTERS, PROCESS_BASIC_INFORMATION};
use windows_sys::Win32::System::WindowsProgramming::CLIENT_ID;

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// `SYSTEM_THREAD_INFORMATION` - the thread record of
/// `SystemProcessInformation` (5).
///
/// `SystemFullProcessInformation` (148) returns
/// `SYSTEM_EXTENDED_THREAD_INFORMATION` instead, which begins with this
/// structure and appends the extended pointers.
#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_THREAD_INFORMATION {
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
    /// C99-style flexible array member. The count is `NumberOfThreads`. The
    /// records are `SYSTEM_THREAD_INFORMATION` (80 bytes) for
    /// `SystemProcessInformation` (5) and `SYSTEM_EXTENDED_THREAD_INFORMATION`
    /// (136 bytes) for `SystemFullProcessInformation` (148).
    pub Threads: [u64; 0],
    // String buffer for `ImageName`
}

/// `SYSTEM_EXTENDED_THREAD_INFORMATION` - the wider thread record returned by
/// `SystemFullProcessInformation` (148) in place of
/// `SYSTEM_THREAD_INFORMATION`.
///
/// Only the leading `SYSTEM_THREAD_INFORMATION` is read; the trailing pointers
/// are declared so the array stride - which is what locates the process
/// extension - keeps its real width.
#[repr(C)]
#[allow(non_snake_case)]
pub struct SYSTEM_EXTENDED_THREAD_INFORMATION {
    pub ThreadInfo: SYSTEM_THREAD_INFORMATION,

    // Extended
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
const _: () = assert!(size_of::<SYSTEM_THREAD_INFORMATION>() == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SYSTEM_EXTENDED_THREAD_INFORMATION>() == 136);
const _: () = assert!(offset_of!(SYSTEM_EXTENDED_THREAD_INFORMATION, ThreadInfo) == 0);

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

/// `SECTION_IMAGE_INFORMATION` - the output of `ProcessImageInformation` (37).
///
/// `Machine` is the `IMAGE_FILE_MACHINE_*` of the executable image, so it
/// reports the architecture of the process itself: a 32-bit process on a
/// 64-bit host still reports `IMAGE_FILE_MACHINE_I386`.
///
/// The header declares `SubSystemVersion` and `OperatingSystemVersion` as
/// unions of two `USHORT`s and `ImageFlags` as a union of bit fields; they are
/// spelled out as their members here, which leaves the layout untouched.
#[repr(C)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub struct SECTION_IMAGE_INFORMATION {
    pub TransferAddress: *mut c_void,
    pub ZeroBits: u32,
    /// `SIZE_T` in the header, which is why the next field is padded to an
    /// 8-byte boundary and the structure ends up 64 bytes wide.
    pub MaximumStackSize: usize,
    pub CommittedStackSize: usize,
    pub SubSystemType: u32,
    pub SubSystemMinorVersion: u16,
    pub SubSystemMajorVersion: u16,
    pub MajorOperatingSystemVersion: u16,
    pub MinorOperatingSystemVersion: u16,
    pub ImageCharacteristics: u16,
    pub DllCharacteristics: u16,
    pub Machine: u16,
    pub ImageContainsCode: u8,
    pub ImageFlags: u8,
    pub LoaderFlags: u32,
    pub ImageFileSize: u32,
    pub CheckSum: u32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<SECTION_IMAGE_INFORMATION>() == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(SECTION_IMAGE_INFORMATION, Machine) == 48);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(size_of::<SECTION_IMAGE_INFORMATION>() == 48);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(offset_of!(SECTION_IMAGE_INFORMATION, Machine) == 32);

/// The leading, version-stable part of `PEB`, up to and including
/// `ProcessParameters`.
///
/// Everything after `ProcessParameters` has grown and been reordered across
/// Windows versions, but the prefix has stayed put since XP, which is all the
/// working-directory read needs.
#[repr(C)]
#[allow(non_snake_case)]
pub struct PEB_PREFIX {
    pub InheritedAddressSpace: u8,
    pub ReadImageFileExecOptions: u8,
    pub BeingDebugged: u8,
    pub BitField: u8,
    pub Mutant: HANDLE,
    pub ImageBaseAddress: *mut c_void,
    pub Ldr: *mut c_void,
    pub ProcessParameters: *mut c_void,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(PEB_PREFIX, ProcessParameters) == 0x20);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(offset_of!(PEB_PREFIX, ProcessParameters) == 0x10);

/// `CURDIR` - the current-directory record of
/// `RTL_USER_PROCESS_PARAMETERS`.
#[repr(C)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
pub struct CURDIR {
    pub DosPath: UNICODE_STRING,
    pub Handle: HANDLE,
}

/// The leading, version-stable part of `RTL_USER_PROCESS_PARAMETERS`, up to
/// and including `CurrentDirectory`.
///
/// The fields after `CurrentDirectory` (`DllPath`, `ImagePathName`,
/// `CommandLine`, ...) are not needed here; the prefix layout has been stable
/// since XP.
#[repr(C)]
#[allow(non_snake_case)]
pub struct RTL_USER_PROCESS_PARAMETERS_PREFIX {
    pub MaximumLength: u32,
    pub Length: u32,
    pub Flags: u32,
    pub DebugFlags: u32,
    pub ConsoleHandle: HANDLE,
    pub ConsoleFlags: u32,
    pub StandardInput: HANDLE,
    pub StandardOutput: HANDLE,
    pub StandardError: HANDLE,
    pub CurrentDirectory: CURDIR,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(offset_of!(RTL_USER_PROCESS_PARAMETERS_PREFIX, CurrentDirectory) == 0x38);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(offset_of!(RTL_USER_PROCESS_PARAMETERS_PREFIX, CurrentDirectory) == 0x24);

/// Fixed part of a `SID`: revision, sub-authority count and the 6-byte
/// identifier authority. `windows-sys` types the trailing sub-authorities as
/// `[u32; 1]`, so `size_of::<SID>()` is 4 bytes more than this.
const SID_HEADER_BYTES: usize = 8;
const SID_REVISION: u8 = 1;
/// `SID_MAX_SUB_AUTHORITIES`
const SID_MAX_SUB_AUTHORITIES: usize = 15;

/// An owned, fixed-size `SID` that can be stored by value.
///
/// `windows-sys`'s `SID` types its trailing sub-authorities as `[u32; 1]`, so
/// it cannot hold a full SID by value. `SID_MAX` embeds the official `SID` as
/// its first field and pads it out to `SECURITY_MAX_SID_SIZE` (68 bytes, the
/// largest a SID can be: 8-byte header + 15 sub-authorities). The `#[repr(C)]`
/// layout and the embedded `SID` give it the same alignment (4) as `SID`, so
/// the first `8 + SubAuthorityCount * 4` bytes are a valid `PSID`.
///
/// Equality and hashing compare only the live SID bytes, never the padding.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SID_MAX {
    /// The official `SID` header and first sub-authority.
    pub sid: SID,
    /// Padding to `SECURITY_MAX_SID_SIZE`; never read or compared.
    _pad: [u8; SECURITY_MAX_SID_SIZE as usize - size_of::<SID>()],
}

impl SID_MAX {
    /// The live bytes of this SID (header + `SubAuthorityCount` sub-authorities),
    /// as a slice that is a valid `PSID` when cast to a pointer.
    ///
    /// The count is clamped to `SID_MAX_SUB_AUTHORITIES` so a corrupted
    /// `SubAuthorityCount` cannot make the slice outrun the fixed buffer.
    pub fn as_bytes(&self) -> &[u8] {
        let count = (self.sid.SubAuthorityCount as usize).min(SID_MAX_SUB_AUTHORITIES);
        let len = SID_HEADER_BYTES + count * size_of::<u32>();
        // SAFETY: `self` is exactly `SECURITY_MAX_SID_SIZE` bytes, and `len` is
        // clamped to at most 68, so the slice stays strictly in bounds.
        unsafe { slice::from_raw_parts(self as *const Self as *const u8, len) }
    }

    /// Copies the SID pointed to by `psid` into a new `SID_MAX`, zeroing the
    /// padding. `psid` must point at a valid SID.
    ///
    /// # Safety
    /// `psid` must be a valid `PSID` whose `SubAuthorityCount` is at most 15.
    pub unsafe fn from_psid(psid: PSID) -> SID_MAX {
        let count = unsafe { (psid.cast::<u8>()).add(1).read() } as usize;

        // Guard against a malformed PSID overflowing the fixed buffer.
        assert!(
            count <= SID_MAX_SUB_AUTHORITIES,
            "PSID has {} sub-authorities, exceeding the maximum {}",
            count,
            SID_MAX_SUB_AUTHORITIES
        );

        let len = SID_HEADER_BYTES + count * size_of::<u32>();

        // Zero-initialise the whole struct (header + padding).
        let mut ret: SID_MAX = unsafe { std::mem::zeroed() };

        // Copy the whole live SID by bytes, like `memcpy`, so the header and
        // all sub-authorities land in `ret` even though `SID` only declares one.
        unsafe {
            ptr::copy_nonoverlapping(
                psid.cast::<u8>(),
                (&mut ret.sid as *mut SID).cast::<u8>(),
                len,
            );
        }
        ret
    }

    /// The sub-authorities of this SID, as a zero-allocation slice.
    pub fn sub_authorities(&self) -> &[u32] {
        let count = (self.sid.SubAuthorityCount as usize).min(SID_MAX_SUB_AUTHORITIES);
        if count == 0 {
            return &[];
        }

        // SAFETY: `SID_MAX` is `#[repr(C)]` and embeds a `[u32; 1]`, so it is
        // at least 4-aligned. `SubAuthority` sits at offset 8, a multiple of 4,
        // so the pointer is properly aligned for `u32`. The sub-authorities are
        // contiguous, immediately following `SubAuthority[0]` in the buffer.
        let sub_ptr = ptr::addr_of!(self.sid.SubAuthority) as *const u32;
        unsafe { slice::from_raw_parts(sub_ptr, count) }
    }

    /// The 6-byte IdentifierAuthority of this SID, as a `u64`.
    pub fn authority(&self) -> u64 {
        let v = self.sid.IdentifierAuthority.Value;
        u64::from_be_bytes([0, 0, v[0], v[1], v[2], v[3], v[4], v[5]])
    }

    /// Formats this SID as a string, abbreviating the middle sub-authorities
    /// when `abbr` is set (e.g. `S-1-5-...-18` instead of the full form).
    pub fn format(&self, abbr: bool) -> String {
        let revision = self.sid.Revision;
        let authority = self.authority();
        let subs = self.sub_authorities();
        let count = subs.len();

        let mut ret = format!("S-{}-{}", revision, authority);
        if count == 0 {
            return ret;
        }

        // Append with `std::fmt::Write` to avoid O(N^2) string rebuilds.
        write!(&mut ret, "-{}", subs[0]).unwrap();

        if count > 1 {
            if abbr {
                write!(&mut ret, "-...-{}", subs[count - 1]).unwrap();
            } else {
                for sub in &subs[1..] {
                    write!(&mut ret, "-{sub}").unwrap();
                }
            }
        }
        ret
    }
}

impl std::fmt::Display for SID_MAX {
    /// The full, unabbreviated SID string (e.g. `S-1-5-21-...-1001`).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format(false))
    }
}

impl PartialEq for SID_MAX {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl Eq for SID_MAX {}

impl std::fmt::Debug for SID_MAX {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The formatted SID string is more useful than the raw byte array.
        f.debug_struct("SID_MAX")
            .field("sid", &self.format(false))
            .finish()
    }
}

impl std::hash::Hash for SID_MAX {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// SystemInformationClass. Windows 8.1 and newer, and the caller has to be
/// elevated: without it the query returns `STATUS_ACCESS_DENIED`. Carries
/// the process extension, which includes the user SID.
pub const SYSTEM_FULL_PROCESS_INFORMATION_CLASS: i32 = 148;

/// Guard against a nonsensical `UNICODE_STRING::Length` turning into a huge
/// allocation. Real command lines are capped well below this.
const MAX_COMMAND_LINE_BYTES: usize = 64 * 1024;

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
    ///
    /// Class 5 lays them out as `SYSTEM_THREAD_INFORMATION` (80 bytes) and
    /// class 148 as `SYSTEM_EXTENDED_THREAD_INFORMATION` (136 bytes). Both are
    /// multiples of 8, so every record in the array stays 8-byte aligned.
    fn thread_stride(self) -> usize {
        match self {
            Self::Basic => size_of::<SYSTEM_THREAD_INFORMATION>(),
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

    let words = query_system_info_class(SystemProcessInformation)?;
    Some(SystemProcessSnapshot {
        words,
        kind: SnapshotKind::Basic,
    })
}

fn query_system_info_class(class: i32) -> Option<Vec<u64>> {
    let mut words = vec![0u64; 0];
    // A `Vec<u64>` is allocated 8-byte aligned, which is what the
    // `SYSTEM_PROCESS_INFORMATION` structures in the snapshot require.
    debug_assert_eq!(words.as_ptr() as usize % 8, 0);

    for _ in 0..8 {
        let mut ret_len: u32 = 0;
        let len = words.len() * size_of::<u64>();
        let status = unsafe {
            NtQuerySystemInformation(
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

/// The scheduler state of one thread, as `SYSTEM_THREAD_INFORMATION` reports
/// it.
///
/// Both values are kept raw. `WaitReason` is a leftover from the last wait once
/// a thread is no longer waiting, so it only means anything while `State` is
/// `Waiting`; turning the pair into a display letter is the State column's job.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ThreadState {
    /// `KTHREAD_STATE`.
    pub state: u32,
    /// `KWAIT_REASON`.
    pub wait_reason: u32,
}

/// `KTHREAD_STATE`, as reported by `SYSTEM_THREAD_INFORMATION::State`.
///
/// <https://ntdoc.m417z.com/kthread_state>
pub mod thread_state {
    pub const INITIALIZED: u32 = 0;
    pub const READY: u32 = 1;
    pub const RUNNING: u32 = 2;
    pub const STANDBY: u32 = 3;
    pub const TERMINATED: u32 = 4;
    pub const WAITING: u32 = 5;
    pub const TRANSITION: u32 = 6;
    pub const DEFERRED_READY: u32 = 7;
    /// No longer produced by the kernel; kept so the table stays complete.
    pub const GATE_WAIT: u32 = 8;
    pub const WAITING_FOR_PROCESS_IN_SWAP: u32 = 9;
}

/// The `KWAIT_REASON` values that mark a thread stopped rather than waiting
/// for something.
///
/// <https://ntdoc.m417z.com/kwait_reason>
pub mod wait_reason {
    /// Set by `NtSuspendThread`.
    pub const SUSPENDED: u32 = 5;
    pub const WR_SUSPENDED: u32 = 12;
}

/// The state pair of one raw thread record.
fn decode_state(thread: &SYSTEM_THREAD_INFORMATION) -> ThreadState {
    ThreadState {
        state: thread.State,
        wait_reason: thread.WaitReason,
    }
}

/// How active a thread is; lower wins.
///
/// The order mirrors the State column's letters: runnable beats waiting,
/// waiting beats suspended, suspended beats terminated, and anything the
/// kernel has not been seen to produce is least active. `WaitReason` only
/// matters while the thread is `WAITING` - it is a leftover from the last
/// wait otherwise.
fn activity(state: ThreadState) -> u8 {
    use thread_state::*;

    match state.state {
        // Runnable, or on its way there: `TRANSITION` only means the kernel
        // stack is still being brought in.
        INITIALIZED | READY | RUNNING | STANDBY | TRANSITION | DEFERRED_READY => 0,
        WAITING | GATE_WAIT | WAITING_FOR_PROCESS_IN_SWAP => match state.wait_reason {
            wait_reason::SUSPENDED | wait_reason::WR_SUSPENDED => 2,
            _ => 1,
        },
        TERMINATED => 3,
        _ => 4,
    }
}

/// One thread of a process, decoded from a `SYSTEM_THREAD_INFORMATION`.
///
/// The raw handles are narrowed to ids and the times are converted, so this is
/// what the rest of the crate works with; the kernel buffer is not kept alive
/// for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThreadSnapshot {
    pub tid: i32,
    /// Owning process id.
    pub pid: i32,
    pub create_time: i64,
    pub kernel_time: u64,
    pub user_time: u64,
    pub priority: i32,
    /// Raw scheduler state at the moment of the snapshot.
    pub state: ThreadState,
}

/// One entry of a snapshot: the process header plus everything that depends on
/// knowing where the entry begins and ends.
///
/// The header is borrowed straight from the `Vec<u64>` snapshot rather than
/// copied. The buffer is 8-byte aligned (it is a `Vec<u64>`), and the kernel
/// places each entry on an 8-byte boundary (`NextEntryOffset` is a multiple of
/// 8), so a `&SYSTEM_PROCESS_INFORMATION` over the buffer is always aligned.
pub struct ProcessEntry<'a> {
    buf: &'a [u64],
    /// The entry's leading `SYSTEM_PROCESS_INFORMATION`, borrowed from the
    /// snapshot buffer. Aligned, so it is read directly rather than copied.
    info: &'a SYSTEM_PROCESS_INFORMATION,
    /// Byte offset of this entry within `buf`.
    start: usize,
    /// Byte offset one past this entry: the bound for its variable-length data.
    end: usize,
    kind: SnapshotKind,
}

impl<'a> ProcessEntry<'a> {
    pub fn info(&self) -> &SYSTEM_PROCESS_INFORMATION {
        self.info
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
        let name = unicode_string_to_owned(&self.info.ImageName);
        match self.kind {
            SnapshotKind::Basic => name,
            SnapshotKind::Full => match name.rsplit('\\').next() {
                Some(file) if !file.is_empty() => file.to_owned(),
                _ => name,
            },
        }
    }

    /// The `Threads[]` flexible array trailing the process header, decoded into
    /// owned [`ThreadSnapshot`]s.
    ///
    /// `NumberOfThreads` is the count the kernel stored. For every entry except
    /// the last, the following entry (at `NextEntryOffset`) bounds the array;
    /// the last entry has `NextEntryOffset == 0` and is bounded by the buffer,
    /// which the kernel sized for exactly `NumberOfThreads` records. Records
    /// that would reach past the entry are clipped, so nothing is read outside
    /// the entry it belongs to.
    pub fn threads(&self) -> Vec<ThreadSnapshot> {
        let (base, stride, len) = self.threads_span();
        let mut ret = Vec::with_capacity(len);
        let pid = self.info.UniqueProcessId as usize as i32;

        for index in 0..len {
            // SAFETY: `base + index * stride` is inside the entry (`index <
            // len`) and 8-byte aligned, so a `&SYSTEM_THREAD_INFORMATION`
            // over it is valid. Both record types start with that structure
            // (see the `ThreadInfo` offset assertion), which is why the stride
            // is the only thing that differs between the two classes.
            let thread: &SYSTEM_THREAD_INFORMATION =
                unsafe { &*base.add(index * stride).cast::<SYSTEM_THREAD_INFORMATION>() };

            ret.push(ThreadSnapshot {
                tid: thread.ClientId.UniqueThread as usize as i32,
                pid,
                create_time: thread.CreateTime,
                kernel_time: thread.KernelTime as u64,
                user_time: thread.UserTime as u64,
                priority: thread.Priority,
                state: decode_state(thread),
            });
        }

        ret
    }

    /// The `Threads[]` flexible array as a raw span: the first record, the
    /// distance between two records, and how many of them may be read.
    ///
    /// `NumberOfThreads` is the count the kernel stored, but only the records
    /// that fit between the process header and the end of this entry can be
    /// read, so the two are reconciled here and every caller gets the clamped
    /// count.
    fn threads_span(&self) -> (*const u8, usize, usize) {
        let stride = self.kind.thread_stride();
        let first = self
            .start
            .saturating_add(size_of::<SYSTEM_PROCESS_INFORMATION>());

        let available = self.end.saturating_sub(first);
        let wanted = (self.info.NumberOfThreads as usize).saturating_mul(stride);
        let len = wanted.min(available) / stride;

        // `info` borrows the entry that starts at `start`, so this is
        // `start + 256`: inside the buffer, and 8-byte aligned like every
        // entry. Each stride is a multiple of 8, so every record is aligned
        // too, and `len` was clamped so the last one stays inside the entry.
        (self.info.Threads.as_ptr().cast::<u8>(), stride, len)
    }

    /// The scheduler state of the most active thread of this entry.
    ///
    /// `SYSTEM_PROCESS_INFORMATION` carries no state for the process itself, so
    /// the per-thread states are the only thing a process state can be derived
    /// from. The most active thread wins, in the same order the State column
    /// prints its letters; the letter mapping itself stays in the column.
    /// `None` when no thread record could be read; the caller separates a
    /// terminated process from an unreadable one with `NumberOfThreads`.
    pub fn state(&self) -> Option<ThreadState> {
        let (base, stride, len) = self.threads_span();
        let mut best: Option<ThreadState> = None;
        let mut best_rank = u8::MAX;

        for index in 0..len {
            // SAFETY: `base + index * stride` is inside the entry (`index <
            // len`) and 8-byte aligned, so a `&SYSTEM_THREAD_INFORMATION`
            // over it is valid. Both record types start with that structure
            // (see the `ThreadInfo` offset assertion), which is why the stride
            // is the only thing that differs between the two classes.
            let thread: &SYSTEM_THREAD_INFORMATION =
                unsafe { &*base.add(index * stride).cast::<SYSTEM_THREAD_INFORMATION>() };
            let state = decode_state(thread);
            let rank = activity(state);
            if rank < best_rank {
                best_rank = rank;
                best = Some(state);
            }
        }

        best
    }

    /// The process user SID from the extension, as the raw `SID` bytes.
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
            .checked_add((self.info.NumberOfThreads as usize).checked_mul(stride)?)?;

        // The fixed prefix we read must fit inside the entry; the variable
        // length data trailing it is reached only through the offsets.
        let prefix = size_of::<SYSTEM_PROCESS_INFORMATION_EXTENSION>();
        if base.saturating_add(prefix) > self.end {
            return None;
        }
        Some(base)
    }

    pub fn user_sid(&self) -> Option<SID_MAX> {
        let base = self.extension_base()?;

        // SAFETY: `extension_base` already verified `base + prefix <= end`, and
        // `base` is a multiple of 8 (`start` + header + N*stride), so the
        // extension is aligned; borrow it instead of copying it out.
        let fields: &SYSTEM_PROCESS_INFORMATION_EXTENSION = unsafe {
            &*self
                .buf
                .as_ptr()
                .cast::<u8>()
                .add(base)
                .cast::<SYSTEM_PROCESS_INFORMATION_EXTENSION>()
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
        // its bits 1..=4. `base` is 8-aligned, so a `u32` read at `base + 48`
        // is aligned.
        let flags: u32 = unsafe {
            self.buf
                .as_ptr()
                .cast::<u8>()
                .add(base + offset_of!(SYSTEM_PROCESS_INFORMATION_EXTENSION, Flags))
                .cast::<u32>()
                .read()
        };
        Some((flags >> 1) & 0xF)
    }
}

/// Copies the SID at byte `offset` (bounded by `end`) out of `base`.
///
/// Returned as an owned [`SID_MAX`], which can be handed straight back to
/// `LookupAccountSidW` as a `PSID`; the sub-authorities are read unaligned, so
/// no alignment guarantee is needed.
fn copy_sid(base: *const u8, offset: usize, end: usize) -> Option<SID_MAX> {
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

    let mut sid = SID_MAX {
        sid: SID::default(),
        _pad: [0u8; SECURITY_MAX_SID_SIZE as usize - size_of::<SID>()],
    };
    // SAFETY: `sid` is `SECURITY_MAX_SID_SIZE` bytes, and `offset + bytes <=
    // end`, so the copy stays inside both allocations. `base` and `sid` do not
    // overlap. Copying the whole live SID by bytes lands the header and all
    // sub-authorities into `sid` even though `SID` only declares one.
    unsafe {
        ptr::copy_nonoverlapping(
            base.add(offset),
            (&mut sid.sid as *mut SID).cast::<u8>(),
            bytes,
        );
    }

    Some(sid)
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

        // SAFETY: the buffer is a `Vec<u64>`, so it is 8-byte aligned, and the
        // kernel lays each entry on an 8-byte boundary (`NextEntryOffset` is a
        // multiple of 8). `offset + size_of <= limit`, so the header is fully
        // inside the buffer and aligned; borrow it directly instead of copying.
        debug_assert_eq!(offset % 8, 0);
        let info: &SYSTEM_PROCESS_INFORMATION = unsafe {
            &*self
                .snapshot
                .words
                .as_ptr()
                .cast::<u8>()
                .add(offset)
                .cast::<SYSTEM_PROCESS_INFORMATION>()
        };

        // `NextEntryOffset == 0` marks the last entry. An offset that would not
        // move past this header is malformed - stop rather than loop forever.
        let next = info.NextEntryOffset as usize;
        let end = if next == 0 || next < header {
            limit
        } else {
            offset.saturating_add(next).min(limit)
        };
        self.offset = end;

        Some(ProcessEntry {
            buf: &self.snapshot.words,
            info,
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
    // Room for the header plus a typical command line.
    let mut buf = vec![0u64; (size_of::<UNICODE_STRING>() + 1024).div_ceil(size_of::<u64>())];
    let mut status = STATUS_INFO_LENGTH_MISMATCH;

    for _ in 0..4 {
        let mut ret_len: u32 = 0;
        status = unsafe {
            NtQueryInformationProcess(
                handle,
                ProcessCommandLineInformation,
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

/// Reads the `IMAGE_FILE_MACHINE_*` of the executable image of `handle`.
///
/// `ProcessImageInformation` only needs `PROCESS_QUERY_LIMITED_INFORMATION`,
/// so it also answers for protected processes that refuse the full query
/// rights. Anything older than Vista yields `None`.
pub fn process_image_machine(handle: HANDLE) -> Option<u16> {
    // SAFETY: a zeroed `SECTION_IMAGE_INFORMATION` is a valid output buffer -
    // every field is an integer or a pointer, and only `Machine` is read.
    let mut info: SECTION_IMAGE_INFORMATION = unsafe { std::mem::zeroed() };
    let mut ret_len: u32 = 0;

    let status = unsafe {
        NtQueryInformationProcess(
            handle,
            ProcessImageInformation,
            ptr::addr_of_mut!(info).cast::<c_void>(),
            size_of::<SECTION_IMAGE_INFORMATION>() as u32,
            ptr::addr_of_mut!(ret_len),
        )
    };
    if status != STATUS_SUCCESS {
        return None;
    }
    // Short writes leave `Machine` unset.
    if (ret_len as usize) < offset_of!(SECTION_IMAGE_INFORMATION, Machine) + size_of::<u16>() {
        return None;
    }

    Some(info.Machine)
}

/// Reads the PEB base address of `handle`.
///
/// `ProcessBasicInformation` only needs `PROCESS_QUERY_LIMITED_INFORMATION`.
/// The returned address lives in the target process's address space and is
/// only meaningful to `ReadProcessMemory`.
pub fn process_peb_address(handle: HANDLE) -> Option<usize> {
    // SAFETY: a zeroed `PROCESS_BASIC_INFORMATION` is a valid output buffer -
    // every field is an integer or a pointer, and only `PebBaseAddress` is read.
    let mut info: PROCESS_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
    let mut ret_len: u32 = 0;

    let status = unsafe {
        NtQueryInformationProcess(
            handle,
            ProcessBasicInformation,
            ptr::addr_of_mut!(info).cast::<c_void>(),
            size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            ptr::addr_of_mut!(ret_len),
        )
    };
    if status != STATUS_SUCCESS {
        return None;
    }
    // Short writes leave `PebBaseAddress` unset.
    if (ret_len as usize)
        < offset_of!(PROCESS_BASIC_INFORMATION, PebBaseAddress) + size_of::<usize>()
    {
        return None;
    }

    Some(info.PebBaseAddress as usize)
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
        let snap = make_snapshot(&sid, 0);

        let mut iter = snap.iter();
        let entry = iter.next().expect("one entry");
        assert_eq!(entry.info().UniqueProcessId as usize, 1234);
        let got = entry.user_sid().expect("a user SID");
        assert_eq!(got.as_bytes(), sid);
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

    /// One thread record, zeroed apart from the fields the crate reads.
    fn thread_record(tid: usize, pid: usize) -> SYSTEM_THREAD_INFORMATION {
        let mut thread: SYSTEM_THREAD_INFORMATION = unsafe { std::mem::zeroed() };
        thread.KernelTime = 100;
        thread.UserTime = 200;
        thread.CreateTime = 1_234_567;
        thread.ClientId = CLIENT_ID {
            UniqueProcess: pid as HANDLE,
            UniqueThread: tid as HANDLE,
        };
        thread.Priority = 8;
        thread
    }

    /// Widens a record into the layout a full snapshot uses: the leading
    /// `SYSTEM_THREAD_INFORMATION` followed by the extended pointers.
    fn extend(thread: SYSTEM_THREAD_INFORMATION) -> SYSTEM_EXTENDED_THREAD_INFORMATION {
        SYSTEM_EXTENDED_THREAD_INFORMATION {
            ThreadInfo: thread,
            StackBase: ptr::null_mut(),
            StackLimit: ptr::null_mut(),
            Win32StartAddress: ptr::null_mut(),
            TebBaseAddress: ptr::null_mut(),
            Reserved2: 0,
            Reserved3: 0,
            Reserved4: 0,
        }
    }

    /// Builds a single-entry snapshot holding `threads` verbatim after the
    /// process header, while the header claims `number_of_threads` - passing a
    /// count larger than the records present exercises the bounds clamp.
    ///
    /// `T` is whichever thread record the snapshot class uses.
    fn make_snapshot_with_threads<T>(
        threads: &[T],
        number_of_threads: u32,
        kind: SnapshotKind,
    ) -> SystemProcessSnapshot {
        let mut header: SYSTEM_PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        // `NextEntryOffset == 0` marks the last entry, which is bounded by the
        // buffer itself.
        header.NextEntryOffset = 0;
        header.NumberOfThreads = number_of_threads;
        header.UniqueProcessId = 1234 as HANDLE;

        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                (&header as *const SYSTEM_PROCESS_INFORMATION).cast::<u8>(),
                size_of::<SYSTEM_PROCESS_INFORMATION>(),
            )
        };
        let mut bytes: Vec<u8> = header_bytes.to_vec();

        for thread in threads {
            let record = unsafe {
                std::slice::from_raw_parts((thread as *const T).cast::<u8>(), size_of::<T>())
            };
            bytes.extend_from_slice(record);
        }
        // The snapshot is a `Vec<u64>`, so round the buffer up to a whole word.
        bytes.resize(bytes.len().div_ceil(size_of::<u64>()) * size_of::<u64>(), 0);

        let mut words = vec![0u64; bytes.len() / size_of::<u64>()];
        // SAFETY: `words` holds exactly `bytes.len()` bytes.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), words.as_mut_ptr().cast::<u8>(), bytes.len());
        }

        SystemProcessSnapshot { words, kind }
    }

    #[test]
    fn threads_returns_every_record() {
        let records = [
            thread_record(11, 1234),
            thread_record(12, 1234),
            thread_record(13, 1234),
        ];
        let snap = make_snapshot_with_threads(&records, 3, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");

        let got = entry.threads();
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].tid, 11);
        assert_eq!(got[1].tid, 12);
        assert_eq!(got[2].tid, 13);
        assert_eq!(got[0].pid, 1234);
        assert_eq!(got[0].create_time, 1_234_567);
        assert_eq!(got[0].kernel_time, 100);
        assert_eq!(got[0].user_time, 200);
        assert_eq!(got[2].priority, 8);
    }

    #[test]
    fn threads_is_empty_without_records() {
        let snap =
            make_snapshot_with_threads::<SYSTEM_THREAD_INFORMATION>(&[], 0, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");
        assert!(entry.threads().is_empty());
    }

    #[test]
    fn threads_are_clipped_to_the_entry() {
        // The header claims three records but only two fit before the buffer
        // ends; the third must not be handed out.
        let records = [thread_record(11, 1234), thread_record(12, 1234)];
        let snap = make_snapshot_with_threads(&records, 3, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");

        let got = entry.threads();
        assert_eq!(got.len(), 2);
        assert_eq!(got[1].tid, 12);
    }

    /// A thread record sitting in a given `KTHREAD_STATE` / `KWAIT_REASON`.
    fn thread_record_in(
        tid: usize,
        pid: usize,
        state: u32,
        wait_reason: u32,
    ) -> SYSTEM_THREAD_INFORMATION {
        let mut thread = thread_record(tid, pid);
        thread.State = state;
        thread.WaitReason = wait_reason;
        thread
    }

    #[test]
    fn state_picks_the_most_active_thread() {
        // `Running` beats `Waiting`, which beats `Terminated`.
        let records = [
            thread_record_in(11, 1234, thread_state::WAITING, 13),
            thread_record_in(12, 1234, thread_state::TERMINATED, 0),
            thread_record_in(13, 1234, thread_state::RUNNING, 0),
        ];
        let snap = make_snapshot_with_threads(&records, 3, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");

        assert_eq!(
            entry.state(),
            Some(ThreadState {
                state: thread_state::RUNNING,
                wait_reason: 0
            })
        );
        // The full record carries the same pair, so a `--thread` row and the
        // process it belongs to cannot disagree.
        assert_eq!(
            entry.threads()[2].state,
            ThreadState {
                state: thread_state::RUNNING,
                wait_reason: 0
            }
        );
    }

    #[test]
    fn state_prefers_waiting_over_suspended() {
        // A suspended thread waits, and says why; a plain wait is more active.
        let records = [
            thread_record_in(11, 1234, thread_state::WAITING, wait_reason::SUSPENDED),
            thread_record_in(12, 1234, thread_state::WAITING, 13),
        ];
        let snap = make_snapshot_with_threads(&records, 2, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");

        assert_eq!(
            entry.state(),
            Some(ThreadState {
                state: thread_state::WAITING,
                wait_reason: 13
            })
        );
    }

    #[test]
    fn state_of_a_full_snapshot_uses_the_wider_stride() {
        // The state words live at the start of `SYSTEM_EXTENDED_THREAD_INFORMATION`,
        // so reading them means stepping by 136 rather than 80.
        let records = [
            extend(thread_record_in(11, 1234, thread_state::TERMINATED, 0)),
            extend(thread_record_in(12, 1234, thread_state::WAITING, 12)),
        ];
        let snap = make_snapshot_with_threads(&records, 2, SnapshotKind::Full);
        let entry = snap.iter().next().expect("one entry");

        assert_eq!(
            entry.state(),
            Some(ThreadState {
                state: thread_state::WAITING,
                wait_reason: 12
            })
        );
    }

    #[test]
    fn state_is_none_without_records() {
        // A process whose threads have all exited decodes to no state at all,
        // which is what lets the State column call it terminated.
        let snap =
            make_snapshot_with_threads::<SYSTEM_THREAD_INFORMATION>(&[], 0, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");
        assert_eq!(entry.state(), None);
    }

    #[test]
    fn state_is_clipped_to_the_entry() {
        // Same bound as `threads`: a record past the end of the entry is not
        // read even though the header claims it.
        let records = [
            thread_record_in(11, 1234, thread_state::RUNNING, 0),
            thread_record_in(12, 1234, thread_state::WAITING, 0),
        ];
        let snap = make_snapshot_with_threads(&records, 3, SnapshotKind::Basic);
        let entry = snap.iter().next().expect("one entry");

        assert_eq!(
            entry.state(),
            Some(ThreadState {
                state: thread_state::RUNNING,
                wait_reason: 0
            })
        );
    }

    #[test]
    fn threads_of_a_full_snapshot_use_the_wider_stride() {
        // Class 148 lays the threads out as `SYSTEM_EXTENDED_THREAD_INFORMATION`
        // (136 bytes) and puts the extension after them, so the records are
        // found at that stride rather than the 80 of a class-5 snapshot.
        let records = [
            extend(thread_record(11, 1234)),
            extend(thread_record(12, 1234)),
        ];
        let snap = make_snapshot_with_threads(&records, 2, SnapshotKind::Full);
        let entry = snap.iter().next().expect("one entry");

        let got = entry.threads();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].tid, 11);
        assert_eq!(got[1].tid, 12);
        assert_eq!(got[0].pid, 1234);
    }

    /// `IMAGE_FILE_MACHINE_*` of the architecture this test was built for.
    #[cfg(target_arch = "x86_64")]
    const OWN_MACHINE: u16 = 0x8664;
    #[cfg(target_arch = "x86")]
    const OWN_MACHINE: u16 = 0x014c;
    #[cfg(target_arch = "aarch64")]
    const OWN_MACHINE: u16 = 0xaa64;

    /// Queries a real process, so it also pins down the buffer size the
    /// kernel insists on: a wrong `SECTION_IMAGE_INFORMATION` layout answers
    /// `STATUS_INFO_LENGTH_MISMATCH` and the query returns `None`.
    #[test]
    fn image_machine_of_this_process() {
        use windows_sys::Win32::System::Threading::GetCurrentProcess;

        // SAFETY: the pseudo-handle needs no closing.
        let handle = unsafe { GetCurrentProcess() };
        assert_eq!(process_image_machine(handle), Some(OWN_MACHINE));
    }
}
