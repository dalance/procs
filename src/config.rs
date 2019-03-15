use crate::column::Column;
use crate::columns::ConfigColumnKind;
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

fn default_column_align_left() -> ConfigColumnAlign {
    ConfigColumnAlign::Left
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

fn default_search_logic_and() -> ConfigSearchLogic {
    ConfigSearchLogic::And
}

fn default_sort_order_ascending() -> ConfigSortOrder {
    ConfigSortOrder::Ascending
}

fn default_separator() -> String {
    String::from("│")
}

fn default_ascending() -> String {
    String::from("▲")
}

fn default_descending() -> String {
    String::from("▼")
}

// ---------------------------------------------------------------------------------------------------------------------
// ColumnInfo
// ---------------------------------------------------------------------------------------------------------------------

pub struct ColumnInfo {
    pub column: Box<dyn Column>,
    pub kind: ConfigColumnKind,
    pub style: ConfigColumnStyle,
    pub nonnumeric_search: bool,
    pub numeric_search: bool,
    pub align: ConfigColumnAlign,
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
pub enum ConfigColumnAlign {
    Left,
    Right,
    Center,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigColumn {
    pub kind: ConfigColumnKind,
    pub style: ConfigColumnStyle,
    #[serde(default = "default_false")]
    pub numeric_search: bool,
    #[serde(default = "default_false")]
    pub nonnumeric_search: bool,
    #[serde(default = "default_column_align_left")]
    pub align: ConfigColumnAlign,
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
    #[serde(default = "default_search_logic_and")]
    pub logic: ConfigSearchLogic,
}

impl Default for ConfigSearch {
    fn default() -> Self {
        ConfigSearch {
            numeric_search: ConfigSearchKind::Exact,
            nonnumeric_search: ConfigSearchKind::Partial,
            logic: ConfigSearchLogic::And,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigSearchKind {
    Exact,
    Partial,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigSearchLogic {
    And,
    Or,
    Nand,
    Nor,
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
    #[serde(default = "default_separator")]
    pub separator: String,
    #[serde(default = "default_ascending")]
    pub ascending: String,
    #[serde(default = "default_descending")]
    pub descending: String,
    #[serde(default = "default_true")]
    pub abbr_sid: bool,
}

impl Default for ConfigDisplay {
    fn default() -> Self {
        ConfigDisplay {
            show_self: false,
            cut_to_terminal: true,
            cut_to_pager: false,
            cut_to_pipe: false,
            color_mode: ConfigColorMode::Auto,
            separator: String::from("│"),
            ascending: String::from("▲"),
            descending: String::from("▼"),
            abbr_sid: true,
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
