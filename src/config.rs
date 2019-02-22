use crate::column::Column;
use crate::columns::*;
use serde_derive::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------------------------------
// Functions for serde defalut
// ---------------------------------------------------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_color_mode_auto() -> ConfigColorMode {
    ConfigColorMode::Auto
}

fn default_pager_mode_auto() -> ConfigPagerMode {
    ConfigPagerMode::Auto
}

fn default_search_kind_exact() -> ConfigSearchKind {
    ConfigSearchKind::Exact
}

fn default_search_kind_partial() -> ConfigSearchKind {
    ConfigSearchKind::Partial
}

fn default_sort_order_ascending() -> ConfigSortOrder {
    ConfigSortOrder::Ascending
}

// ---------------------------------------------------------------------------------------------------------------------
// ColumnInfo
// ---------------------------------------------------------------------------------------------------------------------

pub struct ColumnInfo {
    pub column: Box<dyn Column>,
    pub style: ConfigColumnStyle,
    pub nonnumeric_search: bool,
    pub numeric_search: bool,
}

pub fn gen_column(kind: &ConfigColumnKind, docker_path: &str) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new()),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new()),
        ConfigColumnKind::Docker => Box::new(Docker::new(docker_path)),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::Eip => Box::new(Eip::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::Esp => Box::new(Esp::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::Gid => Box::new(Gid::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::GidFs => Box::new(GidFs::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::GidReal => Box::new(GidReal::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::GidSaved => Box::new(GidSaved::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::Group => Box::new(Group::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::GroupFs => Box::new(GroupFs::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::GroupReal => Box::new(GroupReal::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::GroupSaved => Box::new(GroupSaved::new()),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new()),
        ConfigColumnKind::MinFlt => Box::new(MinFlt::new()),
        ConfigColumnKind::Nice => Box::new(Nice::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::Ppid => Box::new(Ppid::new()),
        ConfigColumnKind::Priority => Box::new(Priority::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::Processor => Box::new(Processor::new()),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::RtPriority => Box::new(RtPriority::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new()),
        ConfigColumnKind::State => Box::new(State::new()),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new()),
        ConfigColumnKind::Threads => Box::new(Threads::new()),
        ConfigColumnKind::Tty => Box::new(Tty::new()),
        ConfigColumnKind::UdpPort => Box::new(UdpPort::new()),
        ConfigColumnKind::Uid => Box::new(Uid::new()),
        ConfigColumnKind::UidFs => Box::new(UidFs::new()),
        ConfigColumnKind::UidReal => Box::new(UidReal::new()),
        ConfigColumnKind::UidSaved => Box::new(UidSaved::new()),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new()),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new()),
        ConfigColumnKind::User => Box::new(User::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::UserFs => Box::new(UserFs::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::UserReal => Box::new(UserReal::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::UserSaved => Box::new(UserSaved::new()),
        ConfigColumnKind::Username => Box::new(User::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmData => Box::new(VmData::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmExe => Box::new(VmExe::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmHwm => Box::new(VmHwm::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmLib => Box::new(VmLib::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmLock => Box::new(VmLock::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmPeak => Box::new(VmPeak::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmPin => Box::new(VmPin::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmPte => Box::new(VmPte::new()),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmStack => Box::new(VmStack::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::VmSwap => Box::new(VmSwap::new()),
        #[cfg(target_os = "linux")]
        ConfigColumnKind::Wchan => Box::new(Wchan::new()),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new()),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub columns: Vec<ConfigColumn>,
    #[serde(default)]
    pub style: ConfigStyle,
    #[serde(default)]
    pub search: ConfigSearch,
    #[serde(default)]
    pub display: ConfigDisplay,
    #[serde(default)]
    pub sort: ConfigSort,
    #[serde(default)]
    pub docker: ConfigDocker,
    #[serde(default)]
    pub pager: ConfigPager,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigColor {
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigColumnKind {
    Command,
    CpuTime,
    Docker,
    #[cfg(target_os = "linux")]
    Eip,
    #[cfg(target_os = "linux")]
    Esp,
    #[cfg(target_os = "linux")]
    Gid,
    #[cfg(target_os = "linux")]
    GidFs,
    #[cfg(target_os = "linux")]
    GidReal,
    #[cfg(target_os = "linux")]
    GidSaved,
    #[cfg(target_os = "linux")]
    Group,
    #[cfg(target_os = "linux")]
    GroupFs,
    #[cfg(target_os = "linux")]
    GroupReal,
    #[cfg(target_os = "linux")]
    GroupSaved,
    MajFlt,
    MinFlt,
    Nice,
    Pid,
    Ppid,
    Priority,
    #[cfg(target_os = "linux")]
    Processor,
    ReadBytes,
    #[cfg(target_os = "linux")]
    RtPriority,
    Separator,
    StartTime,
    State,
    TcpPort,
    Threads,
    Tty,
    UdpPort,
    Uid,
    UidFs,
    UidReal,
    UidSaved,
    UsageCpu,
    UsageMem,
    User,
    #[cfg(target_os = "linux")]
    UserFs,
    #[cfg(target_os = "linux")]
    UserReal,
    #[cfg(target_os = "linux")]
    UserSaved,
    Username,
    #[cfg(target_os = "linux")]
    VmData,
    #[cfg(target_os = "linux")]
    VmExe,
    #[cfg(target_os = "linux")]
    VmHwm,
    #[cfg(target_os = "linux")]
    VmLib,
    #[cfg(target_os = "linux")]
    VmLock,
    #[cfg(target_os = "linux")]
    VmPeak,
    #[cfg(target_os = "linux")]
    VmPin,
    #[cfg(target_os = "linux")]
    VmPte,
    VmRss,
    VmSize,
    #[cfg(target_os = "linux")]
    VmStack,
    #[cfg(target_os = "linux")]
    VmSwap,
    #[cfg(target_os = "linux")]
    Wchan,
    WriteBytes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigColumnStyle {
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    ByPercentage,
    ByState,
    ByUnit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigColumn {
    pub kind: ConfigColumnKind,
    pub style: ConfigColumnStyle,
    #[serde(default = "default_false")]
    pub numeric_search: bool,
    #[serde(default = "default_false")]
    pub nonnumeric_search: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyle {
    pub header: ConfigColor,
    pub unit: ConfigColor,
    #[serde(default)]
    pub by_percentage: ConfigStyleByPercentage,
    #[serde(default)]
    pub by_state: ConfigStyleByState,
    #[serde(default)]
    pub by_unit: ConfigStyleByUnit,
}

impl Default for ConfigStyle {
    fn default() -> Self {
        ConfigStyle {
            header: ConfigColor::BrightWhite,
            unit: ConfigColor::BrightWhite,
            by_percentage: Default::default(),
            by_state: Default::default(),
            by_unit: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByPercentage {
    pub color_000: ConfigColor,
    pub color_025: ConfigColor,
    pub color_050: ConfigColor,
    pub color_075: ConfigColor,
    pub color_100: ConfigColor,
}

impl Default for ConfigStyleByPercentage {
    fn default() -> Self {
        ConfigStyleByPercentage {
            color_000: ConfigColor::BrightBlue,
            color_025: ConfigColor::BrightGreen,
            color_050: ConfigColor::BrightYellow,
            color_075: ConfigColor::BrightRed,
            color_100: ConfigColor::BrightRed,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByUnit {
    pub color_k: ConfigColor,
    pub color_m: ConfigColor,
    pub color_g: ConfigColor,
    pub color_t: ConfigColor,
    pub color_p: ConfigColor,
    pub color_x: ConfigColor,
}

impl Default for ConfigStyleByUnit {
    fn default() -> Self {
        ConfigStyleByUnit {
            color_k: ConfigColor::BrightBlue,
            color_m: ConfigColor::BrightGreen,
            color_g: ConfigColor::BrightYellow,
            color_t: ConfigColor::BrightRed,
            color_p: ConfigColor::BrightRed,
            color_x: ConfigColor::BrightBlue,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByState {
    pub color_d: ConfigColor,
    pub color_r: ConfigColor,
    pub color_s: ConfigColor,
    pub color_t: ConfigColor,
    pub color_z: ConfigColor,
    pub color_x: ConfigColor,
    pub color_k: ConfigColor,
    pub color_w: ConfigColor,
    pub color_p: ConfigColor,
}

impl Default for ConfigStyleByState {
    fn default() -> Self {
        ConfigStyleByState {
            color_d: ConfigColor::BrightRed,
            color_r: ConfigColor::BrightGreen,
            color_s: ConfigColor::BrightBlue,
            color_t: ConfigColor::BrightCyan,
            color_z: ConfigColor::BrightMagenta,
            color_x: ConfigColor::BrightMagenta,
            color_k: ConfigColor::BrightYellow,
            color_w: ConfigColor::BrightYellow,
            color_p: ConfigColor::BrightYellow,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSearch {
    #[serde(default = "default_search_kind_exact")]
    pub numeric_search: ConfigSearchKind,
    #[serde(default = "default_search_kind_partial")]
    pub nonnumeric_search: ConfigSearchKind,
}

impl Default for ConfigSearch {
    fn default() -> Self {
        ConfigSearch {
            numeric_search: ConfigSearchKind::Exact,
            nonnumeric_search: ConfigSearchKind::Partial,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigSearchKind {
    Exact,
    Partial,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigDisplay {
    #[serde(default = "default_false")]
    pub show_self: bool,
    #[serde(default = "default_true")]
    pub cut_to_terminal: bool,
    #[serde(default = "default_false")]
    pub cut_to_pager: bool,
    #[serde(default = "default_false")]
    pub cut_to_pipe: bool,
    #[serde(default = "default_color_mode_auto")]
    pub color_mode: ConfigColorMode,
}

impl Default for ConfigDisplay {
    fn default() -> Self {
        ConfigDisplay {
            show_self: false,
            cut_to_terminal: true,
            cut_to_pager: false,
            cut_to_pipe: false,
            color_mode: ConfigColorMode::Auto,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigColorMode {
    Auto,
    Always,
    Disable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSort {
    #[serde(default)]
    pub column: usize,
    #[serde(default = "default_sort_order_ascending")]
    pub order: ConfigSortOrder,
}

impl Default for ConfigSort {
    fn default() -> Self {
        ConfigSort {
            column: 0,
            order: ConfigSortOrder::Ascending,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigSortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigDocker {
    pub path: String,
}

impl Default for ConfigDocker {
    fn default() -> Self {
        ConfigDocker {
            path: String::from("unix:///var/run/docker.sock"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigPager {
    #[serde(default = "default_pager_mode_auto")]
    pub mode: ConfigPagerMode,
    pub command: Option<String>,
}

impl Default for ConfigPager {
    fn default() -> Self {
        ConfigPager {
            mode: ConfigPagerMode::Auto,
            command: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigPagerMode {
    Auto,
    Always,
    Disable,
}

#[cfg(target_os = "linux")]
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
[[columns]]
kind = "UsageMem"
style = "ByPercentage"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "VmPeak"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "VmSize"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "VmRss"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
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
[[columns]]
kind = "WriteBytes"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
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

#[cfg(target_os = "macos")]
pub static CONFIG_DEFAULT: &'static str = r#"
[[columns]]
kind = "Pid"
style = "BrightYellow"
numeric_search = true
nonnumeric_search = false
[[columns]]
kind = "Username"
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
[[columns]]
kind = "UsageMem"
style = "ByPercentage"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "VmSize"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "VmRss"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
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
[[columns]]
kind = "WriteBytes"
style = "ByUnit"
numeric_search = false
nonnumeric_search = false
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
