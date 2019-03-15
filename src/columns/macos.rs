pub mod command;
pub mod context_sw;
pub mod cpu_time;
pub mod docker;
pub mod gid;
pub mod gid_real;
pub mod gid_saved;
pub mod group;
pub mod group_real;
pub mod group_saved;
pub mod maj_flt;
pub mod min_flt;
pub mod nice;
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
pub use self::docker::Docker;
pub use self::gid::Gid;
pub use self::gid_real::GidReal;
pub use self::gid_saved::GidSaved;
pub use self::group::Group;
pub use self::group_real::GroupReal;
pub use self::group_saved::GroupSaved;
pub use self::maj_flt::MajFlt;
pub use self::min_flt::MinFlt;
pub use self::nice::Nice;
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfigColumnKind {
    Command,
    ContextSw,
    CpuTime,
    Docker,
    Gid,
    GidReal,
    GidSaved,
    Group,
    GroupReal,
    GroupSaved,
    MajFlt,
    MinFlt,
    Nice,
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
    docker_path: &str,
    separator: &str,
    abbr_sid: bool,
) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new()),
        ConfigColumnKind::ContextSw => Box::new(ContextSw::new()),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new()),
        ConfigColumnKind::Docker => Box::new(Docker::new(docker_path)),
        ConfigColumnKind::Gid => Box::new(Gid::new(abbr_sid)),
        ConfigColumnKind::GidReal => Box::new(GidReal::new()),
        ConfigColumnKind::GidSaved => Box::new(GidSaved::new()),
        ConfigColumnKind::Group => Box::new(Group::new(abbr_sid)),
        ConfigColumnKind::GroupReal => Box::new(GroupReal::new()),
        ConfigColumnKind::GroupSaved => Box::new(GroupSaved::new()),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new()),
        ConfigColumnKind::MinFlt => Box::new(MinFlt::new()),
        ConfigColumnKind::Nice => Box::new(Nice::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::Policy => Box::new(Policy::new()),
        ConfigColumnKind::Ppid => Box::new(Ppid::new()),
        ConfigColumnKind::Priority => Box::new(Priority::new()),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new(separator)),
        ConfigColumnKind::Slot => Box::new(Slot::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new()),
        ConfigColumnKind::State => Box::new(State::new()),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new()),
        ConfigColumnKind::Threads => Box::new(Threads::new()),
        ConfigColumnKind::Tty => Box::new(Tty::new()),
        ConfigColumnKind::UdpPort => Box::new(UdpPort::new()),
        ConfigColumnKind::Uid => Box::new(Uid::new(abbr_sid)),
        ConfigColumnKind::UidReal => Box::new(UidReal::new()),
        ConfigColumnKind::UidSaved => Box::new(UidSaved::new()),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new()),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new()),
        ConfigColumnKind::User => Box::new(User::new(abbr_sid)),
        ConfigColumnKind::UserReal => Box::new(UserReal::new()),
        ConfigColumnKind::UserSaved => Box::new(UserSaved::new()),
        ConfigColumnKind::Username => Box::new(User::new(abbr_sid)),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
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
        (ConfigColumnKind::Username, ("Username", "User name")),
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
kind = "State"
style = "ByState"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "Nice"
style = "BrightMagenta"
numeric_search = false
nonnumeric_search = false
align = "Right"
[[columns]]
kind = "Tty"
style = "BrightWhite"
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
kind = "TcpPort"
style = "BrightCyan"
numeric_search = true
nonnumeric_search = false
[[columns]]
kind = "UdpPort"
style = "BrightCyan"
numeric_search = true
nonnumeric_search = false
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
kind = "Docker"
style = "BrightGreen"
numeric_search = false
nonnumeric_search = true
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
