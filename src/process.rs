/// What the listing shows, decided while the processes are collected.
///
/// This is handed to `collect_proc` instead of being applied to its result:
/// enumerating every process cannot be avoided - that is what tells us which
/// ones to keep - but everything a platform does per process afterwards can be
/// skipped for the ones the filter drops. On Windows that is an `OpenProcess`
/// and a command line read per process, on Linux a handful of `/proc/<pid>`
/// reads, on macOS the descriptor and thread walks.
#[derive(Clone, Copy, Default)]
pub struct ShowFilter {
    /// Whether processes of other users are shown. When `false`, only the
    /// processes of the user `procs` runs as are kept.
    pub other_users: bool,
    /// Whether kernel threads are shown.
    pub kthread: bool,
}

#[cfg(target_os = "freebsd")]
pub mod freebsd;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
mod ntapi;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "freebsd")]
pub use self::freebsd::*;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;
#[cfg(target_os = "macos")]
pub use self::macos::*;
#[cfg(target_os = "windows")]
pub use self::windows::*;
