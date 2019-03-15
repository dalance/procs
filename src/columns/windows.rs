pub mod command;
pub mod cpu_time;
pub mod gid;
pub mod group;
pub mod maj_flt;
pub mod pid;
pub mod ppid;
pub mod read_bytes;
pub mod separator;
pub mod slot;
pub mod start_time;
pub mod uid;
pub mod usage_cpu;
pub mod usage_mem;
pub mod user;
pub mod vm_hwm;
pub mod vm_peak;
pub mod vm_pin;
pub mod vm_rss;
pub mod vm_size;
pub mod vm_swap;
pub mod write_bytes;

pub use self::command::Command;
pub use self::cpu_time::CpuTime;
pub use self::gid::Gid;
pub use self::group::Group;
pub use self::maj_flt::MajFlt;
pub use self::pid::Pid;
pub use self::ppid::Ppid;
pub use self::read_bytes::ReadBytes;
pub use self::separator::Separator;
pub use self::slot::Slot;
pub use self::start_time::StartTime;
pub use self::uid::Uid;
pub use self::usage_cpu::UsageCpu;
pub use self::usage_mem::UsageMem;
pub use self::user::User;
pub use self::vm_hwm::VmHwm;
pub use self::vm_peak::VmPeak;
pub use self::vm_pin::VmPin;
pub use self::vm_rss::VmRss;
pub use self::vm_size::VmSize;
pub use self::vm_swap::VmSwap;
pub use self::write_bytes::WriteBytes;

use crate::column::Column;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------------------------------
// ConfigColumnKind
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigColumnKind {
    Command,
    CpuTime,
    Gid,
    Group,
    MajFlt,
    Pid,
    Ppid,
    ReadBytes,
    Separator,
    Slot,
    StartTime,
    Uid,
    UsageCpu,
    UsageMem,
    User,
    VmHwm,
    VmPeak,
    VmPin,
    VmRss,
    VmSize,
    VmSwap,
    WriteBytes,
}

// ---------------------------------------------------------------------------------------------------------------------
// gen_column
// ---------------------------------------------------------------------------------------------------------------------

pub fn gen_column(
    kind: &ConfigColumnKind,
    _docker_path: &str,
    separator: &str,
    abbr_sid: bool,
) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new()),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new()),
        ConfigColumnKind::Gid => Box::new(Gid::new(abbr_sid)),
        ConfigColumnKind::Group => Box::new(Group::new(abbr_sid)),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::Ppid => Box::new(Ppid::new()),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new(separator)),
        ConfigColumnKind::Slot => Box::new(Slot::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new()),
        ConfigColumnKind::Uid => Box::new(Uid::new(abbr_sid)),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new()),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new()),
        ConfigColumnKind::User => Box::new(User::new(abbr_sid)),
        ConfigColumnKind::VmHwm => Box::new(VmHwm::new()),
        ConfigColumnKind::VmPeak => Box::new(VmPeak::new()),
        ConfigColumnKind::VmPin => Box::new(VmPin::new()),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
        ConfigColumnKind::VmSwap => Box::new(VmSwap::new()),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new()),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// KIND_LIST
// ---------------------------------------------------------------------------------------------------------------------

lazy_static! {
    pub static ref KIND_LIST: HashMap<ConfigColumnKind, (&'static str, &'static str)> = [
        (
            ConfigColumnKind::Command,
            ("Command", "Command with all arguments")
        ),
        (
            ConfigColumnKind::CpuTime,
            ("CpuTime", "Cumulative CPU time")
        ),
        (ConfigColumnKind::Gid, ("Gid", "Group ID")),
        (ConfigColumnKind::Group, ("Group", "Group name")),
        (
            ConfigColumnKind::MajFlt,
            ("MajFlt", "Major page fault count")
        ),
        (ConfigColumnKind::Pid, ("Pid", "Process ID")),
        (ConfigColumnKind::Ppid, ("Ppid", "Parent process ID")),
        (
            ConfigColumnKind::ReadBytes,
            ("ReadBytes", "Read bytes from storage")
        ),
        (
            ConfigColumnKind::Separator,
            ("Separator", "Show | for column separation")
        ),
        (
            ConfigColumnKind::Slot,
            ("Slot", "Slot for `--insert` option")
        ),
        (ConfigColumnKind::StartTime, ("StartTime", "Starting time")),
        (ConfigColumnKind::Uid, ("Uid", "User ID")),
        (ConfigColumnKind::UsageCpu, ("UsageCpu", "CPU utilization")),
        (
            ConfigColumnKind::UsageMem,
            ("UsageMem", "Memory utilization")
        ),
        (ConfigColumnKind::User, ("User", "User name")),
        (ConfigColumnKind::VmHwm, ("VmHwm", "Peak resident set size")),
        (
            ConfigColumnKind::VmPeak,
            ("VmPeak", "Peak virtual memory size")
        ),
        (ConfigColumnKind::VmPin, ("VmPin", "Pinned memory size")),
        (ConfigColumnKind::VmRss, ("VmRss", "Resident set size")),
        (ConfigColumnKind::VmSize, ("VmSize", "Physical page size")),
        (
            ConfigColumnKind::VmSwap,
            ("VmSwap", "Swapped-out virtual memory size")
        ),
        (
            ConfigColumnKind::WriteBytes,
            ("WriteBytes", "Write bytes to storage")
        ),
    ]
    .iter()
    .cloned()
    .collect();
}

// ---------------------------------------------------------------------------------------------------------------------
// CONFIG_DEFAULT
// ---------------------------------------------------------------------------------------------------------------------

pub static CONFIG_DEFAULT: &'static str = r#"
[[columns]]
kind = "Pid"
style = "BrightYellow"
numeric_search = true
nonnumeric_search = false
[[columns]]
kind = "User"
style = "BrightGreen"
numeric_search = false
nonnumeric_search = true
[[columns]]
kind = "Separator"
style = "White"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "UsageCpu"
style = "ByPercentage"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "UsageMem"
style = "ByPercentage"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "VmSize"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "VmRss"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "ReadBytes"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "WriteBytes"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "Slot"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "Separator"
style = "White"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "CpuTime"
style = "BrightCyan"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "StartTime"
style = "BrightMagenta"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "Separator"
style = "White"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "Command"
style = "BrightWhite"
numeric_search = false
nonnumeric_search = true
"#;
