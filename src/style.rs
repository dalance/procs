use crate::config::{ConfigColor, ConfigColumnStyle, ConfigStyle};
use console::{Style, StyledObject};
use lazy_static::lazy_static;

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
        ref x if x.starts_with('t') => apply_color(x.to_string(), &s.by_state.color_t),
        ref x if x.starts_with('Z') => apply_color(x.to_string(), &s.by_state.color_z),
        ref x if x.starts_with('X') => apply_color(x.to_string(), &s.by_state.color_x),
        ref x if x.starts_with('K') => apply_color(x.to_string(), &s.by_state.color_k),
        ref x if x.starts_with('W') => apply_color(x.to_string(), &s.by_state.color_w),
        ref x if x.starts_with('P') => apply_color(x.to_string(), &s.by_state.color_p),
        _ => apply_color(x.to_string(), &s.by_state.color_x),
    }
}

fn apply_style_by_unit(x: String, s: &ConfigStyle) -> StyledObject<String> {
    match x {
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_unit.color_k),
        ref x if x.contains('M') => apply_color(x.to_string(), &s.by_unit.color_m),
        ref x if x.contains('G') => apply_color(x.to_string(), &s.by_unit.color_g),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_unit.color_t),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_unit.color_p),
        _ => apply_color(x.to_string(), &s.by_unit.color_x),
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

pub fn apply_color(x: String, c: &ConfigColor) -> StyledObject<String> {
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

pub fn apply_style(x: String, cs: &ConfigColumnStyle, s: &ConfigStyle) -> StyledObject<String> {
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
