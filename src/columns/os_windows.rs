pub mod command;
pub mod cpu_time;
pub mod elapsed_time;
pub mod empty;
pub mod gid;
pub mod group;
pub mod maj_flt;
pub mod multi_slot;
pub mod pid;
pub mod ppid;
pub mod priority;
pub mod read_bytes;
pub mod separator;
pub mod slot;
pub mod start_time;
pub mod tcp_port;
pub mod threads;
pub mod tree;
pub mod tree_slot;
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
pub use self::elapsed_time::ElapsedTime;
pub use self::empty::Empty;
pub use self::gid::Gid;
pub use self::group::Group;
pub use self::maj_flt::MajFlt;
pub use self::multi_slot::MultiSlot;
pub use self::pid::Pid;
pub use self::ppid::Ppid;
pub use self::priority::Priority;
pub use self::read_bytes::ReadBytes;
pub use self::separator::Separator;
pub use self::slot::Slot;
pub use self::start_time::StartTime;
pub use self::tcp_port::TcpPort;
pub use self::threads::Threads;
pub use self::tree::Tree;
pub use self::tree_slot::TreeSlot;
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
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------------------------------------------------
// ConfigColumnKind
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConfigColumnKind {
    Command,
    CpuTime,
    ElapsedTime,
    Empty,
    Gid,
    Group,
    MajFlt,
    MultiSlot,
    Pid,
    Ppid,
    Priority,
    ReadBytes,
    Separator,
    Slot,
    StartTime,
    TcpPort,
    Threads,
    Tree,
    TreeSlot,
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
    header: Option<String>,
    _docker_path: &str,
    separator: &str,
    abbr_sid: bool,
    tree_symbols: &[String; 5],
    _procfs: Option<PathBuf>,
) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new(header)),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new(header)),
        ConfigColumnKind::ElapsedTime => Box::new(ElapsedTime::new(header)),
        ConfigColumnKind::Empty => Box::new(Empty::new()),
        ConfigColumnKind::Gid => Box::new(Gid::new(header, abbr_sid)),
        ConfigColumnKind::Group => Box::new(Group::new(header, abbr_sid)),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new(header)),
        ConfigColumnKind::MultiSlot => Box::new(MultiSlot::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new(header)),
        ConfigColumnKind::Ppid => Box::new(Ppid::new(header)),
        ConfigColumnKind::Priority => Box::new(Priority::new(header)),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new(header)),
        ConfigColumnKind::Separator => Box::new(Separator::new(separator)),
        ConfigColumnKind::Slot => Box::new(Slot::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new(header)),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new(header)),
        ConfigColumnKind::Threads => Box::new(Threads::new(header)),
        ConfigColumnKind::Tree => Box::new(Tree::new(tree_symbols)),
        ConfigColumnKind::TreeSlot => Box::new(TreeSlot::new()),
        ConfigColumnKind::Uid => Box::new(Uid::new(header, abbr_sid)),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new(header)),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new(header)),
        ConfigColumnKind::User => Box::new(User::new(header, abbr_sid)),
        ConfigColumnKind::VmHwm => Box::new(VmHwm::new(header)),
        ConfigColumnKind::VmPeak => Box::new(VmPeak::new(header)),
        ConfigColumnKind::VmPin => Box::new(VmPin::new(header)),
        ConfigColumnKind::VmRss => Box::new(VmRss::new(header)),
        ConfigColumnKind::VmSize => Box::new(VmSize::new(header)),
        ConfigColumnKind::VmSwap => Box::new(VmSwap::new(header)),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new(header)),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// KIND_LIST
// ---------------------------------------------------------------------------------------------------------------------

pub static KIND_LIST: Lazy<BTreeMap<ConfigColumnKind, (&'static str, &'static str)>> =
    Lazy::new(|| {
        [
            (
                ConfigColumnKind::Command,
                ("Command", "Command with all arguments"),
            ),
            (
                ConfigColumnKind::CpuTime,
                ("CpuTime", "Cumulative CPU time"),
            ),
            (
                ConfigColumnKind::ElapsedTime,
                ("ElapsedTime", "Elapsed time"),
            ),
            (ConfigColumnKind::Empty, ("Empty", "Empty")),
            (ConfigColumnKind::Gid, ("Gid", "Group ID")),
            (ConfigColumnKind::Group, ("Group", "Group name")),
            (
                ConfigColumnKind::MajFlt,
                ("MajFlt", "Major page fault count"),
            ),
            (
                ConfigColumnKind::MultiSlot,
                ("MultiSlot", "Slot for `--insert` option"),
            ),
            (ConfigColumnKind::Pid, ("Pid", "Process ID")),
            (ConfigColumnKind::Ppid, ("Ppid", "Parent process ID")),
            (ConfigColumnKind::Priority, ("Priority", "Priority")),
            (
                ConfigColumnKind::ReadBytes,
                ("ReadBytes", "Read bytes from storage"),
            ),
            (
                ConfigColumnKind::Separator,
                ("Separator", "Show | for column separation"),
            ),
            (
                ConfigColumnKind::Slot,
                ("Slot", "Slot for `--insert` option"),
            ),
            (ConfigColumnKind::StartTime, ("StartTime", "Starting time")),
            (ConfigColumnKind::TcpPort, ("TcpPort", "Bound TCP ports")),
            (ConfigColumnKind::Threads, ("Threads", "Thread count")),
            (
                ConfigColumnKind::TreeSlot,
                ("TreeSlot", "Slot for tree column"),
            ),
            (ConfigColumnKind::Uid, ("Uid", "User ID")),
            (ConfigColumnKind::UsageCpu, ("UsageCpu", "CPU utilization")),
            (
                ConfigColumnKind::UsageMem,
                ("UsageMem", "Memory utilization"),
            ),
            (ConfigColumnKind::User, ("User", "User name")),
            (ConfigColumnKind::VmHwm, ("VmHwm", "Peak resident set size")),
            (
                ConfigColumnKind::VmPeak,
                ("VmPeak", "Peak virtual memory size"),
            ),
            (ConfigColumnKind::VmPin, ("VmPin", "Pinned memory size")),
            (ConfigColumnKind::VmRss, ("VmRss", "Resident set size")),
            (ConfigColumnKind::VmSize, ("VmSize", "Physical page size")),
            (
                ConfigColumnKind::VmSwap,
                ("VmSwap", "Swapped-out virtual memory size"),
            ),
            (
                ConfigColumnKind::WriteBytes,
                ("WriteBytes", "Write bytes to storage"),
            ),
        ]
        .iter()
        .cloned()
        .collect()
    });

// ---------------------------------------------------------------------------------------------------------------------
// CONFIG_DEFAULT
// ---------------------------------------------------------------------------------------------------------------------

pub static CONFIG_DEFAULT: &str = r#"
[[columns]]
kind = "Pid"
style = "BrightYellow|Yellow"
numeric_search = true
nonnumeric_search = false
[[columns]]
kind = "User"
style = "BrightGreen|Green"
numeric_search = false
nonnumeric_search = true
[[columns]]
kind = "Separator"
style = "White|BrightBlack"
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
kind = "CpuTime"
style = "BrightCyan|Cyan"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "MultiSlot"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "Separator"
style = "White|BrightBlack"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "Command"
style = "BrightWhite|Black"
numeric_search = false
nonnumeric_search = true
"#;

// ---------------------------------------------------------------------------------------------------------------------
// CONFIG_LARGE
// ---------------------------------------------------------------------------------------------------------------------

pub static CONFIG_LARGE: &str = r#"
[[columns]]
kind = "Pid"
style = "BrightYellow|Yellow"
numeric_search = true
nonnumeric_search = false
align = "Left"
[[columns]]
kind = "User"
style = "BrightGreen|Green"
numeric_search = false
nonnumeric_search = true
align = "Left"
[[columns]]
kind = "Separator"
style = "White|BrightBlack"
numeric_search = false
nonnumeric_search = false
align = "Left"
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
kind = "TcpPort"
style = "BrightCyan|Cyan"
numeric_search = true
nonnumeric_search = false
align = "Left"
max_width = 20
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
style = "White|BrightBlack"
numeric_search = false
nonnumeric_search = false
align = "Left"
[[columns]]
kind = "CpuTime"
style = "BrightCyan|Cyan"
numeric_search = false
nonnumeric_search = false
align = "Left"
[[columns]]
kind = "StartTime"
style = "BrightMagenta|Magenta"
numeric_search = false
nonnumeric_search = false
align = "Left"
[[columns]]
kind = "Separator"
style = "White|BrightBlack"
numeric_search = false
nonnumeric_search = false
align = "Left"
[[columns]]
kind = "Command"
style = "BrightWhite|Black"
numeric_search = false
nonnumeric_search = true
align = "Left"
"#;

// ---------------------------------------------------------------------------------------------------------------------
// CONFIG_ALL
// ---------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
pub static CONFIG_ALL: &str = r#"
[[columns]]
kind = "Command"
style = "BrightRed"
align = "Left"
[[columns]]
kind = "CpuTime"
style = "BrightGreen"
align = "Center"
[[columns]]
kind = "ElapsedTime"
style = "BrightYellow"
[[columns]]
kind = "Empty"
style = "BrightYellow"
[[columns]]
kind = "Gid"
style = "White"
[[columns]]
kind = "Group"
style = "White"
[[columns]]
kind = "MajFlt"
style = "BrightCyan"
[[columns]]
kind = "MultiSlot"
style = "BrightWhite"
[[columns]]
kind = "Pid"
style = "Green"
[[columns]]
kind = "Ppid"
style = "Yellow"
[[columns]]
kind = "Priority"
style = "Blue"
[[columns]]
kind = "ReadBytes"
style = "Cyan"
[[columns]]
kind = "Separator"
style = "White"
[[columns]]
kind = "StartTime"
style = "White"
[[columns]]
kind = "TcpPort"
style = "White"
[[columns]]
kind = "Threads"
style = "White"
[[columns]]
kind = "TreeSlot"
style = "BrightWhite"
[[columns]]
kind = "Uid"
style = "White"
[[columns]]
kind = "UsageCpu"
style = "White"
[[columns]]
kind = "UsageMem"
style = "White"
[[columns]]
kind = "User"
style = "White"
[[columns]]
kind = "VmHwm"
style = "ByUnit"
[[columns]]
kind = "VmPeak"
style = "ByUnit"
[[columns]]
kind = "VmPin"
style = "ByUnit"
[[columns]]
kind = "VmRss"
style = "ByUnit"
[[columns]]
kind = "VmSize"
style = "ByUnit"
[[columns]]
kind = "VmSwap"
style = "ByUnit"
[[columns]]
kind = "WriteBytes"
style = "White"
"#;
