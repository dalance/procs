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

// ---------------------------------------------------------------------------------------------------------------------
// ColumnInfo
// ---------------------------------------------------------------------------------------------------------------------

pub struct ColumnInfo {
    pub column: Box<dyn Column>,
    pub style: ConfigColumnStyle,
    pub nonnumeric_search: bool,
    pub numeric_search: bool,
}

#[cfg(target_os = "linux")]
pub fn gen_column(kind: &ConfigColumnKind, docker_path: &str) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new()),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new()),
        ConfigColumnKind::Docker => Box::new(Docker::new(docker_path)),
        ConfigColumnKind::Eip => Box::new(Eip::new()),
        ConfigColumnKind::Esp => Box::new(Esp::new()),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new()),
        ConfigColumnKind::MinFlt => Box::new(MinFlt::new()),
        ConfigColumnKind::Nice => Box::new(Nice::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::Ppid => Box::new(Ppid::new()),
        ConfigColumnKind::Priority => Box::new(Priority::new()),
        ConfigColumnKind::Processor => Box::new(Processor::new()),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new()),
        ConfigColumnKind::RtPriority => Box::new(RtPriority::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new()),
        ConfigColumnKind::State => Box::new(State::new()),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new()),
        ConfigColumnKind::Threads => Box::new(Threads::new()),
        ConfigColumnKind::Tty => Box::new(Tty::new()),
        ConfigColumnKind::UdpPort => Box::new(UdpPort::new()),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new()),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new()),
        ConfigColumnKind::Username => Box::new(Username::new()),
        ConfigColumnKind::VmData => Box::new(VmData::new()),
        ConfigColumnKind::VmExe => Box::new(VmExe::new()),
        ConfigColumnKind::VmHwm => Box::new(VmHwm::new()),
        ConfigColumnKind::VmLib => Box::new(VmLib::new()),
        ConfigColumnKind::VmLock => Box::new(VmLock::new()),
        ConfigColumnKind::VmPeak => Box::new(VmPeak::new()),
        ConfigColumnKind::VmPin => Box::new(VmPin::new()),
        ConfigColumnKind::VmPte => Box::new(VmPte::new()),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
        ConfigColumnKind::VmStack => Box::new(VmStack::new()),
        ConfigColumnKind::VmSwap => Box::new(VmSwap::new()),
        ConfigColumnKind::Wchan => Box::new(Wchan::new()),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new()),
    }
}

#[cfg(target_os = "macos")]
pub fn gen_column(kind: &ConfigColumnKind, _docker_path: &str) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new()),
        ConfigColumnKind::Nice => Box::new(Nice::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new()),
        ConfigColumnKind::State => Box::new(State::new()),
        ConfigColumnKind::Username => Box::new(Username::new()),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
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

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigColumnKind {
    Command,
    CpuTime,
    Docker,
    Eip,
    Esp,
    MajFlt,
    MinFlt,
    Nice,
    Pid,
    Ppid,
    Priority,
    Processor,
    ReadBytes,
    RtPriority,
    Separator,
    StartTime,
    State,
    TcpPort,
    Threads,
    Tty,
    UdpPort,
    UsageCpu,
    UsageMem,
    Username,
    VmData,
    VmExe,
    VmHwm,
    VmLib,
    VmLock,
    VmPeak,
    VmPin,
    VmPte,
    VmRss,
    VmSize,
    VmStack,
    VmSwap,
    Wchan,
    WriteBytes,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigColumnKind {
    Command,
    Nice,
    Pid,
    Separator,
    State,
    Username,
    VmRss,
    VmSize,
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
    pub numeric_search: ConfigSearchKind,
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
}

impl Default for ConfigDisplay {
    fn default() -> Self {
        ConfigDisplay {
            show_self: false,
            cut_to_terminal: true,
            cut_to_pager: false,
            cut_to_pipe: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSort {
    pub column: usize,
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
