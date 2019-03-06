use crate::config::{ConfigColor, ConfigColumnStyle, ConfigStyle};
use console::{Style, StyledObject};
use lazy_static::lazy_static;

lazy_static! {
    static ref BRIGHT_RED: Style = Style::new().red().bold();
    static ref BRIGHT_GREEN: Style = Style::new().green().bold();
    static ref BRIGHT_YELLOW: Style = Style::new().yellow().bold();
    static ref BRIGHT_BLUE: Style = Style::new().blue().bold();
    static ref BRIGHT_MAGENTA: Style = Style::new().magenta().bold();
    static ref BRIGHT_CYAN: Style = Style::new().cyan().bold();
    static ref BRIGHT_WHITE: Style = Style::new().white().bold();
    static ref RED: Style = Style::new().red();
    static ref GREEN: Style = Style::new().green();
    static ref YELLOW: Style = Style::new().yellow();
    static ref BLUE: Style = Style::new().blue();
    static ref MAGENTA: Style = Style::new().magenta();
    static ref CYAN: Style = Style::new().cyan();
    static ref WHITE: Style = Style::new().white();
}

fn apply_style_by_state(x: String, s: &ConfigStyle) -> StyledObject<String> {
    match x {
        ref x if x.contains('D') => apply_color(x.to_string(), &s.by_state.color_d),
        ref x if x.contains('R') => apply_color(x.to_string(), &s.by_state.color_r),
        ref x if x.contains('S') => apply_color(x.to_string(), &s.by_state.color_s),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_state.color_t),
        ref x if x.contains('t') => apply_color(x.to_string(), &s.by_state.color_t),
        ref x if x.contains('Z') => apply_color(x.to_string(), &s.by_state.color_z),
        ref x if x.contains('X') => apply_color(x.to_string(), &s.by_state.color_x),
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_state.color_k),
        ref x if x.contains('W') => apply_color(x.to_string(), &s.by_state.color_w),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_state.color_p),
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
    let value: f64 = x.trim().parse().unwrap_or(0.0);
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
        ConfigColor::BrightRed => BRIGHT_RED.apply_to(x),
        ConfigColor::BrightGreen => BRIGHT_GREEN.apply_to(x),
        ConfigColor::BrightYellow => BRIGHT_YELLOW.apply_to(x),
        ConfigColor::BrightBlue => BRIGHT_BLUE.apply_to(x),
        ConfigColor::BrightMagenta => BRIGHT_MAGENTA.apply_to(x),
        ConfigColor::BrightCyan => BRIGHT_CYAN.apply_to(x),
        ConfigColor::BrightWhite => BRIGHT_WHITE.apply_to(x),
        ConfigColor::Red => RED.apply_to(x),
        ConfigColor::Green => GREEN.apply_to(x),
        ConfigColor::Yellow => YELLOW.apply_to(x),
        ConfigColor::Blue => BLUE.apply_to(x),
        ConfigColor::Magenta => MAGENTA.apply_to(x),
        ConfigColor::Cyan => CYAN.apply_to(x),
        ConfigColor::White => WHITE.apply_to(x),
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
