use crate::column::Column;
use crate::columns::ConfigColumnKind;
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

// ---------------------------------------------------------------------------------------------------------------------
// Functions for serde default
// ---------------------------------------------------------------------------------------------------------------------

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_column_style_by_unit() -> ConfigColumnStyle {
    ConfigColumnStyle::ByUnit
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

fn default_search_case_smart() -> ConfigSearchCase {
    ConfigSearchCase::Smart
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

fn default_tree_symbols() -> [String; 5] {
    [
        String::from("│"),
        String::from("─"),
        String::from("┬"),
        String::from("├"),
        String::from("└"),
    ]
}

fn default_color_by_theme() -> ConfigColorByTheme {
    ConfigColorByTheme {
        dark: ConfigColor::BrightWhite,
        light: ConfigColor::Black,
    }
}

fn default_theme_auto() -> ConfigTheme {
    ConfigTheme::Auto
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
    pub max_width: Option<usize>,
    pub min_width: Option<usize>,
    pub visible: bool,
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
pub enum ConfigTheme {
    Auto,
    Dark,
    Light,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigColor {
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Color256(u8),
}

fn serialize_color(c: &ConfigColor) -> String {
    match c {
        ConfigColor::BrightBlack => "BrightBlack".to_string(),
        ConfigColor::BrightRed => "BrightRed".to_string(),
        ConfigColor::BrightGreen => "BrightGreen".to_string(),
        ConfigColor::BrightYellow => "BrightYellow".to_string(),
        ConfigColor::BrightBlue => "BrightBlue".to_string(),
        ConfigColor::BrightMagenta => "BrightMagenta".to_string(),
        ConfigColor::BrightCyan => "BrightCyan".to_string(),
        ConfigColor::BrightWhite => "BrightWhite".to_string(),
        ConfigColor::Black => "Black".to_string(),
        ConfigColor::Red => "Red".to_string(),
        ConfigColor::Green => "Green".to_string(),
        ConfigColor::Yellow => "Yellow".to_string(),
        ConfigColor::Blue => "Blue".to_string(),
        ConfigColor::Magenta => "Magenta".to_string(),
        ConfigColor::Cyan => "Cyan".to_string(),
        ConfigColor::White => "White".to_string(),
        ConfigColor::Color256(x) => format!("{x}"),
    }
}

fn deserialize_color(s: &str) -> Option<ConfigColor> {
    match s {
        "BrightBlack" => Some(ConfigColor::BrightBlack),
        "BrightRed" => Some(ConfigColor::BrightRed),
        "BrightGreen" => Some(ConfigColor::BrightGreen),
        "BrightYellow" => Some(ConfigColor::BrightYellow),
        "BrightBlue" => Some(ConfigColor::BrightBlue),
        "BrightMagenta" => Some(ConfigColor::BrightMagenta),
        "BrightCyan" => Some(ConfigColor::BrightCyan),
        "BrightWhite" => Some(ConfigColor::BrightWhite),
        "Black" => Some(ConfigColor::Black),
        "Red" => Some(ConfigColor::Red),
        "Green" => Some(ConfigColor::Green),
        "Yellow" => Some(ConfigColor::Yellow),
        "Blue" => Some(ConfigColor::Blue),
        "Magenta" => Some(ConfigColor::Magenta),
        "Cyan" => Some(ConfigColor::Cyan),
        "White" => Some(ConfigColor::White),
        s if u8::from_str(s).is_ok() => Some(ConfigColor::Color256(u8::from_str(s).unwrap())),
        _ => None,
    }
}

fn serialize_color_by_theme(c: &ConfigColorByTheme) -> String {
    let dark = &c.dark;
    let light = &c.light;
    if dark == light {
        serialize_color(dark)
    } else {
        let dark = serialize_color(dark);
        let light = serialize_color(light);
        format!("{dark}|{light}")
    }
}

fn deserialize_color_by_theme(s: &str) -> Option<ConfigColorByTheme> {
    if let Some(i) = s.find('|') {
        let (dark, light) = s.split_at(i);
        let light = &light[1..];
        let dark = deserialize_color(dark)?;
        let light = deserialize_color(light)?;
        Some(ConfigColorByTheme { dark, light })
    } else {
        let c = deserialize_color(s)?;
        Some(ConfigColorByTheme {
            dark: c.clone(),
            light: c,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ConfigColorByTheme {
    pub dark: ConfigColor,
    pub light: ConfigColor,
}

impl serde::Serialize for ConfigColorByTheme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = serialize_color_by_theme(self);
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for ConfigColorByTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        deserialize_color_by_theme(&s).ok_or_else(|| serde::de::Error::custom(""))
    }
}

#[derive(Clone, Debug)]
pub enum ConfigColumnStyle {
    Fixed(ConfigColorByTheme),
    ByPercentage,
    ByState,
    ByUnit,
}

impl serde::Serialize for ConfigColumnStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ConfigColumnStyle::ByPercentage => "ByPercentage".to_string(),
            ConfigColumnStyle::ByState => "ByState".to_string(),
            ConfigColumnStyle::ByUnit => "ByUnit".to_string(),
            ConfigColumnStyle::Fixed(c) => serialize_color_by_theme(c),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for ConfigColumnStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "ByPercentage" => Ok(ConfigColumnStyle::ByPercentage),
            "ByState" => Ok(ConfigColumnStyle::ByState),
            "ByUnit" => Ok(ConfigColumnStyle::ByUnit),
            s => {
                let c =
                    deserialize_color_by_theme(s).ok_or_else(|| serde::de::Error::custom(""))?;
                Ok(ConfigColumnStyle::Fixed(c))
            }
        }
    }
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
    #[serde(default = "default_column_style_by_unit")]
    pub style: ConfigColumnStyle,
    #[serde(default = "default_false")]
    pub numeric_search: bool,
    #[serde(default = "default_false")]
    pub nonnumeric_search: bool,
    #[serde(default = "default_column_align_left")]
    pub align: ConfigColumnAlign,
    pub max_width: Option<usize>,
    pub min_width: Option<usize>,
    pub header: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyle {
    #[serde(default = "default_color_by_theme")]
    pub header: ConfigColorByTheme,
    #[serde(default = "default_color_by_theme")]
    pub unit: ConfigColorByTheme,
    #[serde(default = "default_color_by_theme")]
    pub tree: ConfigColorByTheme,
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
            header: default_color_by_theme(),
            unit: default_color_by_theme(),
            tree: default_color_by_theme(),
            by_percentage: Default::default(),
            by_state: Default::default(),
            by_unit: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByPercentage {
    pub color_000: ConfigColorByTheme,
    pub color_025: ConfigColorByTheme,
    pub color_050: ConfigColorByTheme,
    pub color_075: ConfigColorByTheme,
    pub color_100: ConfigColorByTheme,
}

impl Default for ConfigStyleByPercentage {
    fn default() -> Self {
        ConfigStyleByPercentage {
            color_000: ConfigColorByTheme {
                dark: ConfigColor::BrightBlue,
                light: ConfigColor::Blue,
            },
            color_025: ConfigColorByTheme {
                dark: ConfigColor::BrightGreen,
                light: ConfigColor::Green,
            },
            color_050: ConfigColorByTheme {
                dark: ConfigColor::BrightYellow,
                light: ConfigColor::Yellow,
            },
            color_075: ConfigColorByTheme {
                dark: ConfigColor::BrightRed,
                light: ConfigColor::Red,
            },
            color_100: ConfigColorByTheme {
                dark: ConfigColor::BrightRed,
                light: ConfigColor::Red,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByUnit {
    pub color_k: ConfigColorByTheme,
    pub color_m: ConfigColorByTheme,
    pub color_g: ConfigColorByTheme,
    pub color_t: ConfigColorByTheme,
    pub color_p: ConfigColorByTheme,
    pub color_x: ConfigColorByTheme,
}

impl Default for ConfigStyleByUnit {
    fn default() -> Self {
        ConfigStyleByUnit {
            color_k: ConfigColorByTheme {
                dark: ConfigColor::BrightBlue,
                light: ConfigColor::Blue,
            },
            color_m: ConfigColorByTheme {
                dark: ConfigColor::BrightGreen,
                light: ConfigColor::Green,
            },
            color_g: ConfigColorByTheme {
                dark: ConfigColor::BrightYellow,
                light: ConfigColor::Yellow,
            },
            color_t: ConfigColorByTheme {
                dark: ConfigColor::BrightRed,
                light: ConfigColor::Red,
            },
            color_p: ConfigColorByTheme {
                dark: ConfigColor::BrightRed,
                light: ConfigColor::Red,
            },
            color_x: ConfigColorByTheme {
                dark: ConfigColor::BrightBlue,
                light: ConfigColor::Blue,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByState {
    pub color_d: ConfigColorByTheme,
    pub color_r: ConfigColorByTheme,
    pub color_s: ConfigColorByTheme,
    pub color_t: ConfigColorByTheme,
    pub color_z: ConfigColorByTheme,
    pub color_x: ConfigColorByTheme,
    pub color_k: ConfigColorByTheme,
    pub color_w: ConfigColorByTheme,
    pub color_p: ConfigColorByTheme,
}

impl Default for ConfigStyleByState {
    fn default() -> Self {
        ConfigStyleByState {
            color_d: ConfigColorByTheme {
                dark: ConfigColor::BrightRed,
                light: ConfigColor::Red,
            },
            color_r: ConfigColorByTheme {
                dark: ConfigColor::BrightGreen,
                light: ConfigColor::Green,
            },
            color_s: ConfigColorByTheme {
                dark: ConfigColor::BrightBlue,
                light: ConfigColor::Blue,
            },
            color_t: ConfigColorByTheme {
                dark: ConfigColor::BrightCyan,
                light: ConfigColor::Cyan,
            },
            color_z: ConfigColorByTheme {
                dark: ConfigColor::BrightMagenta,
                light: ConfigColor::Magenta,
            },
            color_x: ConfigColorByTheme {
                dark: ConfigColor::BrightMagenta,
                light: ConfigColor::Magenta,
            },
            color_k: ConfigColorByTheme {
                dark: ConfigColor::BrightYellow,
                light: ConfigColor::Yellow,
            },
            color_w: ConfigColorByTheme {
                dark: ConfigColor::BrightYellow,
                light: ConfigColor::Yellow,
            },
            color_p: ConfigColorByTheme {
                dark: ConfigColor::BrightYellow,
                light: ConfigColor::Yellow,
            },
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
    #[serde(default = "default_search_case_smart")]
    pub case: ConfigSearchCase,
}

impl Default for ConfigSearch {
    fn default() -> Self {
        ConfigSearch {
            numeric_search: ConfigSearchKind::Exact,
            nonnumeric_search: ConfigSearchKind::Partial,
            logic: ConfigSearchLogic::And,
            case: ConfigSearchCase::Smart,
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
pub enum ConfigSearchCase {
    Smart,
    Insensitive,
    Sensitive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigDisplay {
    #[serde(default = "default_false")]
    pub show_self: bool,
    #[serde(default = "default_false")]
    pub show_self_parents: bool,
    #[serde(default = "default_false")]
    pub show_thread: bool,
    #[serde(default = "default_true")]
    pub show_thread_in_tree: bool,
    #[serde(default = "default_true")]
    pub show_parent_in_tree: bool,
    #[serde(default = "default_true")]
    pub show_children_in_tree: bool,
    #[serde(default = "default_true")]
    pub show_header: bool,
    #[serde(default = "default_false")]
    pub show_footer: bool,
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
    #[serde(default = "default_tree_symbols")]
    pub tree_symbols: [String; 5],
    #[serde(default = "default_true")]
    pub abbr_sid: bool,
    #[serde(default = "default_theme_auto")]
    pub theme: ConfigTheme,
    #[serde(default = "default_true")]
    pub show_kthreads: bool,
}

impl Default for ConfigDisplay {
    fn default() -> Self {
        ConfigDisplay {
            show_self: false,
            show_self_parents: false,
            show_thread: false,
            show_thread_in_tree: true,
            show_parent_in_tree: true,
            show_children_in_tree: true,
            show_header: true,
            show_footer: false,
            cut_to_terminal: true,
            cut_to_pager: false,
            cut_to_pipe: false,
            color_mode: ConfigColorMode::Auto,
            separator: String::from("│"),
            ascending: String::from("▲"),
            descending: String::from("▼"),
            tree_symbols: [
                String::from("│"),
                String::from("─"),
                String::from("┬"),
                String::from("├"),
                String::from("└"),
            ],
            abbr_sid: true,
            theme: ConfigTheme::Auto,
            show_kthreads: true,
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
            path: std::env::var("DOCKER_HOST")
                .unwrap_or(String::from("unix:///var/run/docker.sock")),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigPager {
    #[serde(default = "default_pager_mode_auto")]
    pub mode: ConfigPagerMode,
    #[serde(default = "default_false")]
    pub detect_width: bool,
    #[serde(default = "default_false")]
    pub use_builtin: bool,
    pub command: Option<String>,
}

impl Default for ConfigPager {
    fn default() -> Self {
        ConfigPager {
            mode: ConfigPagerMode::Auto,
            detect_width: false,
            use_builtin: false,
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
