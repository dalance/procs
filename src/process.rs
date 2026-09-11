use std::time::Duration;

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

/// What every platform's `ProcessInfo` is made of before its own fields are
/// added: the row key, the parent, and the sampling window.
///
/// The rest of a row - the CPU times, the memory, the command line - is
/// whatever that platform's kernel hands out and none of it is shared, so each
/// platform keeps a `ProcessInfo` of its own with this one in its `base` field
/// and dereferences to it. That is what lets a column write `proc.pid`
/// without knowing which platform it was compiled for, and it is why the
/// fields here are only the ones no platform can do without.
pub struct ProcessInfoBase {
    /// The row key: a process id when it is positive, and the negated thread
    /// id of a thread row when it is negative.
    ///
    /// Threads and processes share this one field because every column keys
    /// its contents by it. Thread ids are handed out of the same pool as
    /// process ids, so a thread row could otherwise be taken for a process -
    /// or for a thread of another process - while a recycled id is in use
    /// elsewhere. Negating the id cannot collide with a process id, and the
    /// Pid column turns it back into the thread id.
    ///
    /// A row is a process when the key is not negative - `pid >= 0` - and a
    /// thread otherwise. The sign is read where it is needed rather than
    /// behind a method, so the encoding has no second spelling to keep in
    /// step.
    pub pid: i64,
    /// The key of the process this row hangs under: a parent pid for a
    /// process, and the pid of the process it belongs to for a thread.
    pub ppid: i64,
    /// How long the two samples behind this row are apart.
    ///
    /// Every rate column divides by it - bytes per second, CPU percent - so
    /// it is the measured distance between the snapshots rather than the
    /// interval that was asked for.
    pub interval: Duration,
}

impl ProcessInfoBase {
    pub fn new(pid: i64, ppid: i64, interval: Duration) -> Self {
        Self {
            pid,
            ppid,
            interval,
        }
    }
}

/// Dereferences a platform's `ProcessInfo` to its [`ProcessInfoBase`].
///
/// Written once so all four platforms read the same: a column says `proc.pid`,
/// not `proc.base.pid`, and the platform structs stay free to lay the rest of
/// the row out as their kernel reports it.
macro_rules! process_info_deref {
    () => {
        impl ::std::ops::Deref for ProcessInfo {
            type Target = $crate::process::ProcessInfoBase;

            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }
    };
}

/// The row key of a thread: its thread id, negated.
///
/// Every column keys its contents by `pid`, the tree keys parent and child
/// rows by it, and so does the filter - so a thread needs a key of its own
/// that can be told apart from the process it belongs to and from every
/// other thread. Process ids are never negative, so negating the thread id
/// gives one, and the key still says which thread it is: negating it back
/// gives the id.
pub fn thread_key(tid: u64) -> i64 {
    // An id that does not fit has nothing to be negated into; it is folded
    // onto the largest one rather than skipped, and no kernel hands one out.
    -i64::try_from(tid).unwrap_or(i64::MAX)
}

/// The key a row is ordered by: the id it stands for, and whether that id
/// belongs to a process or to the thread negated into the key.
///
/// Sorting rows by their key would sink every thread below every process, so
/// the Pid column and the tree order by this instead: a thread then lands
/// next to the process whose id it was handed - the first thread of a process
/// is given the process' own id - and on such a tie the process comes first.
pub fn row_sort_key(pid: i64) -> (u64, bool) {
    // `unsigned_abs` and not `abs`: `i64::MIN` has no positive counterpart to
    // be negated into, and no kernel hands out such an id anyway.
    (pid.unsigned_abs(), pid < 0)
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

#[cfg(test)]
mod tests {
    use super::{row_sort_key, thread_key};

    /// A thread row is keyed by its negated id, and negating the key back
    /// gives the id - which is what the Pid column reports. Pid 0 is a
    /// process - Idle on Windows, `kernel_task` on macOS - so its key is
    /// left at 0 and read back as a process; no kernel hands out thread 0.
    #[test]
    fn thread_key_negates_the_id() {
        assert_eq!(thread_key(1), -1);
        assert_eq!(-thread_key(1), 1);
        assert_eq!(thread_key(0), 0);
        assert_eq!(thread_key(u64::MAX), -i64::MAX);
        assert!(thread_key(u64::from(u32::MAX)) < 0);
    }

    /// A thread row sorts by the id it stands for, next to - and just after -
    /// the process that was handed the same number.
    #[test]
    fn row_sort_key_orders_by_the_id_behind_the_key() {
        let mut pids = [-1i64, 2, -2, 0, 1];
        pids.sort_by_key(|pid| row_sort_key(*pid));
        assert_eq!(pids, [0, 1, -1, 2, -2]);
    }
}
