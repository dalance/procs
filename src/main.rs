mod column;
mod columns;
mod util;

use column::Column;
use columns::*;
use console::{Style, StyledObject, Term};
use failure::{Error, ResultExt};
use lazy_static::lazy_static;
use procfs::Process;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};
use structopt::{clap, StructOpt};
use util::KeywordClass;

// ---------------------------------------------------------------------------------------------------------------------
// Opt
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, StructOpt)]
#[structopt(raw(
    long_version = "option_env!(\"LONG_VERSION\").unwrap_or(env!(\"CARGO_PKG_VERSION\"))"
))]
#[structopt(raw(setting = "clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "clap::AppSettings::DeriveDisplayOrder"))]
pub struct Opt {
    /// Keyword for search
    #[structopt(name = "KEYWORD")]
    pub keyword: Vec<String>,

    /// Interval to calculate throughput
    #[structopt(long = "interval", default_value = "100", value_name = "ms")]
    pub interval: u64,

    /// Mask sensitive information
    #[structopt(long = "mask")]
    pub mask: bool,

    /// Generate configuration sample file
    #[structopt(long = "config")]
    pub config: bool,
}

// ---------------------------------------------------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub columns: Vec<ConfigColumn>,
    pub style: ConfigStyle,
    pub search: ConfigSearch,
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
    Pid,
    ReadBytes,
    Separator,
    StartTime,
    State,
    TcpPort,
    Tty,
    UdpPort,
    UsageCpu,
    UsageMem,
    Username,
    VmRss,
    VmSize,
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
    pub numeric_search: bool,
    pub nonnumeric_search: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyle {
    pub header: ConfigColor,
    pub unit: ConfigColor,
    pub by_percentage: ConfigStyleByPercentage,
    pub by_state: ConfigStyleByState,
    pub by_unit: ConfigStyleByUnit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByPercentage {
    pub color_000: ConfigColor,
    pub color_025: ConfigColor,
    pub color_050: ConfigColor,
    pub color_075: ConfigColor,
    pub color_100: ConfigColor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByUnit {
    pub color_k: ConfigColor,
    pub color_m: ConfigColor,
    pub color_g: ConfigColor,
    pub color_t: ConfigColor,
    pub color_p: ConfigColor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigStyleByState {
    pub color_d: ConfigColor,
    pub color_r: ConfigColor,
    pub color_s: ConfigColor,
    pub color_t: ConfigColor,
    pub color_z: ConfigColor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigSearch {
    pub numeric_search: ConfigSearchKind,
    pub nonnumeric_search: ConfigSearchKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConfigSearchKind {
    Exact,
    Partial,
}

static CONFIG_DEFAULT: &'static str = r#"
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
kind = "Separator"
style = "White"
numeric_search = false
nonnumeric_search = false
[[columns]]
kind = "Command"
style = "BrightWhite"
numeric_search = false
nonnumeric_search = true

[style]
header = "BrightWhite"
unit = "BrightWhite"

[style.by_percentage]
color_000 = "BrightBlue"
color_025 = "BrightGreen"
color_050 = "BrightYellow"
color_075 = "BrightRed"
color_100 = "BrightRed"

[style.by_unit]
color_k = "BrightBlue"
color_m = "BrightGreen"
color_g = "BrightYellow"
color_t = "BrightRed"
color_p = "BrightRed"

[style.by_state]
color_d = "BrightRed"
color_r = "BrightGreen"
color_s = "BrightBlue"
color_t = "BrightCyan"
color_z = "BrightMagenta"

[search]
numeric_search = "Exact"
nonnumeric_search = "Partial"
"#;

// ---------------------------------------------------------------------------------------------------------------------
// Column
// ---------------------------------------------------------------------------------------------------------------------

struct ColumnInfo {
    column: Box<dyn Column>,
    style: ConfigColumnStyle,
    nonnumeric_search: bool,
    numeric_search: bool,
}

fn gen_column(kind: ConfigColumnKind, mask: bool) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new(mask)),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new()),
        ConfigColumnKind::State => Box::new(State::new()),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new()),
        ConfigColumnKind::Tty => Box::new(Tty::new()),
        ConfigColumnKind::UdpPort => Box::new(UdpPort::new()),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new()),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new()),
        ConfigColumnKind::Username => Box::new(Username::new(mask)),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new()),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Style
// ---------------------------------------------------------------------------------------------------------------------

lazy_static! {
    static ref bright_red: Style = Style::new().red().bold();
    static ref bright_green: Style = Style::new().green().bold();
    static ref bright_yellow: Style = Style::new().yellow().bold();
    static ref bright_blue: Style = Style::new().blue().bold();
    static ref bright_magenta: Style = Style::new().magenta().bold();
    static ref bright_cyan: Style = Style::new().cyan().bold();
    static ref bright_white: Style = Style::new().white().bold();
    static ref red: Style = Style::new().red();
    static ref green: Style = Style::new().green();
    static ref yellow: Style = Style::new().yellow();
    static ref blue: Style = Style::new().blue();
    static ref magenta: Style = Style::new().magenta();
    static ref cyan: Style = Style::new().cyan();
    static ref white: Style = Style::new().white();
}

fn apply_style_by_state(x: String, s: &ConfigStyle) -> StyledObject<String> {
    match x {
        ref x if x.starts_with('D') => apply_color(x.to_string(), &s.by_state.color_d),
        ref x if x.starts_with('R') => apply_color(x.to_string(), &s.by_state.color_r),
        ref x if x.starts_with('S') => apply_color(x.to_string(), &s.by_state.color_s),
        ref x if x.starts_with('T') => apply_color(x.to_string(), &s.by_state.color_t),
        ref x if x.starts_with('Z') => apply_color(x.to_string(), &s.by_state.color_z),
        _ => bright_white.apply_to(x),
    }
}

fn apply_style_by_unit(x: String, s: &ConfigStyle) -> StyledObject<String> {
    match x {
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_unit.color_k),
        ref x if x.contains('M') => apply_color(x.to_string(), &s.by_unit.color_m),
        ref x if x.contains('G') => apply_color(x.to_string(), &s.by_unit.color_g),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_unit.color_t),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_unit.color_p),
        _ => bright_blue.apply_to(x),
    }
}

fn apply_style_by_percentage(x: String, s: &ConfigStyle) -> StyledObject<String> {
    let value: f64 = x.parse().unwrap_or(0.0);
    if value > 100.0 {
        apply_color(x, &s.by_percentage.color_100)
    } else if value > 75.0 {
        apply_color(x, &s.by_percentage.color_075)
    } else if value > 50.0 {
        apply_color(x, &s.by_percentage.color_050)
    } else if value > 25.0 {
        apply_color(x, &s.by_percentage.color_025)
    } else {
        apply_color(x, &s.by_percentage.color_000)
    }
}

fn apply_color(x: String, c: &ConfigColor) -> StyledObject<String> {
    match c {
        ConfigColor::BrightRed => bright_red.apply_to(x),
        ConfigColor::BrightGreen => bright_green.apply_to(x),
        ConfigColor::BrightYellow => bright_yellow.apply_to(x),
        ConfigColor::BrightBlue => bright_blue.apply_to(x),
        ConfigColor::BrightMagenta => bright_magenta.apply_to(x),
        ConfigColor::BrightCyan => bright_cyan.apply_to(x),
        ConfigColor::BrightWhite => bright_white.apply_to(x),
        ConfigColor::Red => red.apply_to(x),
        ConfigColor::Green => green.apply_to(x),
        ConfigColor::Yellow => yellow.apply_to(x),
        ConfigColor::Blue => blue.apply_to(x),
        ConfigColor::Magenta => magenta.apply_to(x),
        ConfigColor::Cyan => cyan.apply_to(x),
        ConfigColor::White => white.apply_to(x),
    }
}

fn apply_style(x: String, cs: &ConfigColumnStyle, s: &ConfigStyle) -> StyledObject<String> {
    match cs {
        ConfigColumnStyle::BrightRed => apply_color(x, &ConfigColor::BrightRed),
        ConfigColumnStyle::BrightGreen => apply_color(x, &ConfigColor::BrightGreen),
        ConfigColumnStyle::BrightYellow => apply_color(x, &ConfigColor::BrightYellow),
        ConfigColumnStyle::BrightBlue => apply_color(x, &ConfigColor::BrightBlue),
        ConfigColumnStyle::BrightMagenta => apply_color(x, &ConfigColor::BrightMagenta),
        ConfigColumnStyle::BrightCyan => apply_color(x, &ConfigColor::BrightCyan),
        ConfigColumnStyle::BrightWhite => apply_color(x, &ConfigColor::BrightWhite),
        ConfigColumnStyle::Red => apply_color(x, &ConfigColor::Red),
        ConfigColumnStyle::Green => apply_color(x, &ConfigColor::Green),
        ConfigColumnStyle::Yellow => apply_color(x, &ConfigColor::Yellow),
        ConfigColumnStyle::Blue => apply_color(x, &ConfigColor::Blue),
        ConfigColumnStyle::Magenta => apply_color(x, &ConfigColor::Magenta),
        ConfigColumnStyle::Cyan => apply_color(x, &ConfigColor::Cyan),
        ConfigColumnStyle::White => apply_color(x, &ConfigColor::White),
        ConfigColumnStyle::ByPercentage => apply_style_by_percentage(x, s),
        ConfigColumnStyle::ByState => apply_style_by_state(x, s),
        ConfigColumnStyle::ByUnit => apply_style_by_unit(x, s),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------------------------------------------

fn main() {
    match run() {
        Err(x) => {
            eprintln!("{}", x);
        }
        _ => (),
    }
}

fn get_config() -> Result<Config, Error> {
    let cfg_path = match dirs::home_dir() {
        Some(mut path) => {
            path.push(".procs.toml");
            if path.exists() {
                Some(path)
            } else {
                None
            }
        }
        None => None,
    };

    let config: Config = if let Some(path) = cfg_path {
        let mut f = fs::File::open(&path).context(format!("failed to open file ({:?})", path))?;
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        toml::from_str(&s).context(format!("failed to parse toml ({:?})", path))?
    } else {
        toml::from_str(CONFIG_DEFAULT).unwrap()
    };

    Ok(config)
}

fn collect_proc(cols: &mut Vec<ColumnInfo>, opt: &Opt) -> Vec<i32> {
    let mut base_procs = Vec::new();

    for proc in procfs::all_processes() {
        let io = proc.io();
        let time = Instant::now();
        base_procs.push((proc.pid(), proc, io, time));
    }

    thread::sleep(Duration::from_millis(opt.interval));

    let mut pids = Vec::new();
    for (pid, prev_proc, prev_io, prev_time) in base_procs {
        let curr_proc = if let Ok(proc) = Process::new(pid) {
            proc
        } else {
            prev_proc.clone()
        };
        let curr_io = curr_proc.io();
        let curr_time = Instant::now();
        let interval = curr_time - prev_time;

        for c in cols.iter_mut() {
            c.column
                .add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        }

        pids.push(pid);
    }

    pids
}

fn display_header(max_width: usize, cols: &Vec<ColumnInfo>, config: &Config) -> () {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_color(c.column.display_header(), &config.style.header)
        );
    }
    row = row.trim_end().to_string();
    row = console::truncate_str(&row, max_width, "").to_string();
    println!("{}", row);
}

fn display_unit(max_width: usize, cols: &Vec<ColumnInfo>, config: &Config) -> () {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_color(c.column.display_unit(), &config.style.unit)
        );
    }
    row = row.trim_end().to_string();
    row = console::truncate_str(&row, max_width, "").to_string();
    println!("{}", row);
}

fn display_item(pid: i32, max_width: usize, cols: &Vec<ColumnInfo>, config: &Config) -> () {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_style(c.column.display(pid).unwrap(), &c.style, &config.style)
        );
    }
    row = row.trim_end().to_string();
    row = console::truncate_str(&row, max_width, "").to_string();
    println!("{}", row);
}

fn run() -> Result<(), Error> {
    let opt = Opt::from_args();

    if opt.config {
        let config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        let toml = toml::to_string(&config)?;
        println!("{}", toml);
        return Ok(());
    }

    let config = get_config()?;

    let mut cols = Vec::new();
    for c in &config.columns {
        cols.push(ColumnInfo {
            column: gen_column(c.kind.clone(), opt.mask),
            style: c.style.clone(),
            nonnumeric_search: c.nonnumeric_search,
            numeric_search: c.numeric_search,
        });
    }

    let term = Term::stdout();
    let (_term_h, mut term_w) = term.size();
    if !console::user_attended() {
        term_w = std::u16::MAX;
    }

    let pids = collect_proc(&mut cols, &opt);

    display_header(term_w as usize, &cols, &config);
    display_unit(term_w as usize, &cols, &config);

    let mut cols_nonnumeric = Vec::new();
    let mut cols_numeric = Vec::new();
    for c in &cols {
        if c.nonnumeric_search {
            cols_nonnumeric.push(&c.column);
        }
        if c.numeric_search {
            cols_numeric.push(&c.column);
        }
    }

    let mut keyword_nonnumeric = Vec::new();
    let mut keyword_numeric = Vec::new();

    for k in &opt.keyword {
        match util::classify(k) {
            KeywordClass::Numeric => keyword_numeric.push(k),
            KeywordClass::NonNumeric => keyword_nonnumeric.push(k),
        }
    }

    for pid in pids {
        let mut visible = true;
        if !opt.keyword.is_empty() {
            visible = match config.search.nonnumeric_search {
                ConfigSearchKind::Partial => {
                    util::find_partial(cols_nonnumeric.as_slice(), pid, &keyword_nonnumeric)
                }
                ConfigSearchKind::Exact => {
                    util::find_exact(cols_nonnumeric.as_slice(), pid, &keyword_nonnumeric)
                }
            };
            visible |= match config.search.numeric_search {
                ConfigSearchKind::Partial => {
                    util::find_partial(cols_numeric.as_slice(), pid, &keyword_numeric)
                }
                ConfigSearchKind::Exact => {
                    util::find_exact(cols_numeric.as_slice(), pid, &keyword_numeric)
                }
            };
        }

        if visible {
            display_item(pid, term_w as usize, &cols, &config);
        }
    }

    Ok(())
}
