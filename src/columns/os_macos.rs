pub mod command;
pub mod context_sw;
pub mod cpu_time;
#[cfg(feature = "docker")]
pub mod docker;
pub mod elapsed_time;
pub mod empty;
pub mod gid;
pub mod gid_real;
pub mod gid_saved;
pub mod group;
pub mod group_real;
pub mod group_saved;
pub mod maj_flt;
pub mod min_flt;
pub mod nice;
pub mod pgid;
pub mod pid;
pub mod policy;
pub mod ppid;
pub mod priority;
pub mod read_bytes;
pub mod separator;
pub mod slot;
pub mod start_time;
pub mod state;
pub mod tcp_port;
pub mod threads;
pub mod tree;
pub mod tty;
pub mod udp_port;
pub mod uid;
pub mod uid_real;
pub mod uid_saved;
pub mod usage_cpu;
pub mod usage_mem;
pub mod user;
pub mod user_real;
pub mod user_saved;
pub mod vm_rss;
pub mod vm_size;
pub mod write_bytes;

pub use self::command::Command;
pub use self::context_sw::ContextSw;
pub use self::cpu_time::CpuTime;
#[cfg(feature = "docker")]
pub use self::docker::Docker;
pub use self::elapsed_time::ElapsedTime;
pub use self::empty::Empty;
pub use self::gid::Gid;
pub use self::gid_real::GidReal;
pub use self::gid_saved::GidSaved;
pub use self::group::Group;
pub use self::group_real::GroupReal;
pub use self::group_saved::GroupSaved;
pub use self::maj_flt::MajFlt;
pub use self::min_flt::MinFlt;
pub use self::nice::Nice;
pub use self::pgid::Pgid;
pub use self::pid::Pid;
pub use self::policy::Policy;
pub use self::ppid::Ppid;
pub use self::priority::Priority;
pub use self::read_bytes::ReadBytes;
pub use self::separator::Separator;
pub use self::slot::Slot;
pub use self::start_time::StartTime;
pub use self::state::State;
pub use self::tcp_port::TcpPort;
pub use self::threads::Threads;
pub use self::tree::Tree;
pub use self::tty::Tty;
pub use self::udp_port::UdpPort;
pub use self::uid::Uid;
pub use self::uid_real::UidReal;
pub use self::uid_saved::UidSaved;
pub use self::usage_cpu::UsageCpu;
pub use self::usage_mem::UsageMem;
pub use self::user::User;
pub use self::user_real::UserReal;
pub use self::user_saved::UserSaved;
pub use self::vm_rss::VmRss;
pub use self::vm_size::VmSize;
pub use self::write_bytes::WriteBytes;

use crate::column::Column;
use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------------------------------------------------
// ConfigColumnKind
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ConfigColumnKind {
    Command,
    ContextSw,
    CpuTime,
    Docker,
    ElapsedTime,
    Empty,
    Gid,
    GidReal,
    GidSaved,
    Group,
    GroupReal,
    GroupSaved,
    MajFlt,
    MinFlt,
    Nice,
    Pgid,
    Pid,
    Policy,
    Ppid,
    Priority,
    ReadBytes,
    Separator,
    Slot,
    StartTime,
    State,
    TcpPort,
    Threads,
    Tree,
    Tty,
    UdpPort,
    Uid,
    UidReal,
    UidSaved,
    UsageCpu,
    UsageMem,
    User,
    UserReal,
    UserSaved,
    Username,
    VmRss,
    VmSize,
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
) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new(header)),
        ConfigColumnKind::ContextSw => Box::new(ContextSw::new(header)),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new(header)),
        #[cfg(feature = "docker")]
        ConfigColumnKind::Docker => Box::new(Docker::new(header, _docker_path)),
        #[cfg(not(feature = "docker"))]
        ConfigColumnKind::Docker => Box::new(Empty::new()),
        ConfigColumnKind::ElapsedTime => Box::new(ElapsedTime::new(header)),
        ConfigColumnKind::Empty => Box::new(Empty::new()),
        ConfigColumnKind::Gid => Box::new(Gid::new(header, abbr_sid)),
        ConfigColumnKind::GidReal => Box::new(GidReal::new(header)),
        ConfigColumnKind::GidSaved => Box::new(GidSaved::new(header)),
        ConfigColumnKind::Group => Box::new(Group::new(header, abbr_sid)),
        ConfigColumnKind::GroupReal => Box::new(GroupReal::new(header)),
        ConfigColumnKind::GroupSaved => Box::new(GroupSaved::new(header)),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new(header)),
        ConfigColumnKind::MinFlt => Box::new(MinFlt::new(header)),
        ConfigColumnKind::Nice => Box::new(Nice::new(header)),
        ConfigColumnKind::Pgid => Box::new(Pgid::new(header)),
        ConfigColumnKind::Pid => Box::new(Pid::new(header)),
        ConfigColumnKind::Policy => Box::new(Policy::new(header)),
        ConfigColumnKind::Ppid => Box::new(Ppid::new(header)),
        ConfigColumnKind::Priority => Box::new(Priority::new(header)),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new(header)),
        ConfigColumnKind::Separator => Box::new(Separator::new(separator)),
        ConfigColumnKind::Slot => Box::new(Slot::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new(header)),
        ConfigColumnKind::State => Box::new(State::new(header)),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new(header)),
        ConfigColumnKind::Threads => Box::new(Threads::new(header)),
        ConfigColumnKind::Tree => Box::new(Tree::new(tree_symbols)),
        ConfigColumnKind::Tty => Box::new(Tty::new(header)),
        ConfigColumnKind::UdpPort => Box::new(UdpPort::new(header)),
        ConfigColumnKind::Uid => Box::new(Uid::new(header, abbr_sid)),
        ConfigColumnKind::UidReal => Box::new(UidReal::new(header)),
        ConfigColumnKind::UidSaved => Box::new(UidSaved::new(header)),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new(header)),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new(header)),
        ConfigColumnKind::User => Box::new(User::new(header, abbr_sid)),
        ConfigColumnKind::UserReal => Box::new(UserReal::new(header)),
        ConfigColumnKind::UserSaved => Box::new(UserSaved::new(header)),
        ConfigColumnKind::Username => Box::new(User::new(header, abbr_sid)),
        ConfigColumnKind::VmRss => Box::new(VmRss::new(header)),
        ConfigColumnKind::VmSize => Box::new(VmSize::new(header)),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new(header)),
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
            ConfigColumnKind::ContextSw,
            ("ContextSw", "Context switch count")
        ),
        (
            ConfigColumnKind::CpuTime,
            ("CpuTime", "Cumulative CPU time")
        ),
        (
            ConfigColumnKind::Docker,
            ("Docker", "Docker container name")
        ),
        (
            ConfigColumnKind::ElapsedTime,
            ("ElapsedTime", "Elapsed time")
        ),
        (ConfigColumnKind::Empty, ("Empty", "Empty")),
        (ConfigColumnKind::Gid, ("Gid", "Group ID")),
        (ConfigColumnKind::GidReal, ("GidReal", "Real group ID")),
        (ConfigColumnKind::GidSaved, ("GidSaved", "Saved group ID")),
        (ConfigColumnKind::Group, ("Group", "Group name")),
        (
            ConfigColumnKind::GroupReal,
            ("GroupReal", "Real group name")
        ),
        (
            ConfigColumnKind::GroupSaved,
            ("GroupSaved", "Saved group name")
        ),
        (
            ConfigColumnKind::MajFlt,
            ("MajFlt", "Major page fault count")
        ),
        (
            ConfigColumnKind::MinFlt,
            ("MinFlt", "Minor page fault count")
        ),
        (ConfigColumnKind::Nice, ("Nice", "Nice value")),
        (ConfigColumnKind::Pgid, ("Pgid", "Process group ID")),
        (ConfigColumnKind::Pid, ("Pid", "Process ID")),
        (ConfigColumnKind::Policy, ("Policy", "Scheduling policy")),
        (ConfigColumnKind::Ppid, ("Ppid", "Parent process ID")),
        (ConfigColumnKind::Priority, ("Priority", "Priority")),
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
        (ConfigColumnKind::State, ("State", "Process state")),
        (ConfigColumnKind::TcpPort, ("TcpPort", "Bound TCP ports")),
        (ConfigColumnKind::Threads, ("Threads", "Thread count")),
        (ConfigColumnKind::Tty, ("Tty", "Controlling TTY")),
        (ConfigColumnKind::UdpPort, ("UdpPort", "Bound UDP ports")),
        (ConfigColumnKind::Uid, ("Uid", "User ID")),
        (ConfigColumnKind::UidReal, ("UidReal", "Real user ID")),
        (ConfigColumnKind::UidSaved, ("UidSaved", "Saved user ID")),
        (ConfigColumnKind::UsageCpu, ("UsageCpu", "CPU utilization")),
        (
            ConfigColumnKind::UsageMem,
            ("UsageMem", "Memory utilization")
        ),
        (ConfigColumnKind::User, ("User", "User name")),
        (ConfigColumnKind::UserReal, ("UserReal", "Real user name")),
        (
            ConfigColumnKind::UserSaved,
            ("UserSaved", "Saved user name")
        ),
        (ConfigColumnKind::VmRss, ("VmRss", "Resident set size")),
        (ConfigColumnKind::VmSize, ("VmSize", "Physical page size")),
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
kind = "Tty"
style = "BrightWhite|Black"
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
// CONFIG_ALL
// ---------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
pub static CONFIG_ALL: &str = r#"
[[columns]]
kind = "Command"
style = "BrightRed"
align = "Left"
[[columns]]
kind = "ContextSw"
style = "BrightRed"
align = "Right"
[[columns]]
kind = "CpuTime"
style = "BrightGreen"
align = "Center"
[[columns]]
kind = "Docker"
style = "BrightMagenta"
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
kind = "GidReal"
style = "White"
[[columns]]
kind = "GidSaved"
style = "White"
[[columns]]
kind = "Group"
style = "White"
[[columns]]
kind = "GroupReal"
style = "White"
[[columns]]
kind = "GroupSaved"
style = "White"
[[columns]]
kind = "MajFlt"
style = "BrightCyan"
[[columns]]
kind = "MinFlt"
style = "BrightWhite"
[[columns]]
kind = "Nice"
style = "Red"
[[columns]]
kind = "Pgid"
style = "Yellow"
[[columns]]
kind = "Pid"
style = "Green"
[[columns]]
kind = "Policy"
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
kind = "State"
style = "White"
[[columns]]
kind = "TcpPort"
style = "White"
[[columns]]
kind = "Threads"
style = "White"
[[columns]]
kind = "Tty"
style = "White"
[[columns]]
kind = "UdpPort"
style = "White"
[[columns]]
kind = "Uid"
style = "White"
[[columns]]
kind = "UidReal"
style = "White"
[[columns]]
kind = "UidSaved"
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
kind = "UserReal"
style = "White"
[[columns]]
kind = "UserSaved"
style = "White"
[[columns]]
kind = "VmRss"
style = "ByUnit"
[[columns]]
kind = "VmSize"
style = "ByUnit"
[[columns]]
kind = "WriteBytes"
style = "White"
"#;
