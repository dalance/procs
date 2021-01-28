use crate::config::{ConfigColor, ConfigColorByTheme, ConfigColumnStyle, ConfigStyle, ConfigTheme};
use console::{Style, StyledObject};
use lazy_static::lazy_static;

lazy_static! {
    static ref BRIGHT_BLACK: Style = Style::new().black().bright();
    static ref BRIGHT_RED: Style = Style::new().red().bright();
    static ref BRIGHT_GREEN: Style = Style::new().green().bright();
    static ref BRIGHT_YELLOW: Style = Style::new().yellow().bright();
    static ref BRIGHT_BLUE: Style = Style::new().blue().bright();
    static ref BRIGHT_MAGENTA: Style = Style::new().magenta().bright();
    static ref BRIGHT_CYAN: Style = Style::new().cyan().bright();
    static ref BRIGHT_WHITE: Style = Style::new().white().bright();
    static ref BLACK: Style = Style::new().black();
    static ref RED: Style = Style::new().red();
    static ref GREEN: Style = Style::new().green();
    static ref YELLOW: Style = Style::new().yellow();
    static ref BLUE: Style = Style::new().blue();
    static ref MAGENTA: Style = Style::new().magenta();
    static ref CYAN: Style = Style::new().cyan();
    static ref WHITE: Style = Style::new().white();
}

fn apply_style_by_state(
    x: String,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> StyledObject<String> {
    match x {
        ref x if x.contains('D') => apply_color(x.to_string(), &s.by_state.color_d, theme, faded),
        ref x if x.contains('R') => apply_color(x.to_string(), &s.by_state.color_r, theme, faded),
        ref x if x.contains('S') => apply_color(x.to_string(), &s.by_state.color_s, theme, faded),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_state.color_t, theme, faded),
        ref x if x.contains('t') => apply_color(x.to_string(), &s.by_state.color_t, theme, faded),
        ref x if x.contains('Z') => apply_color(x.to_string(), &s.by_state.color_z, theme, faded),
        ref x if x.contains('X') => apply_color(x.to_string(), &s.by_state.color_x, theme, faded),
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_state.color_k, theme, faded),
        ref x if x.contains('W') => apply_color(x.to_string(), &s.by_state.color_w, theme, faded),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_state.color_p, theme, faded),
        _ => apply_color(x, &s.by_state.color_x, theme, faded),
    }
}

fn apply_style_by_unit(
    x: String,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> StyledObject<String> {
    match x {
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_unit.color_k, theme, faded),
        ref x if x.contains('M') => apply_color(x.to_string(), &s.by_unit.color_m, theme, faded),
        ref x if x.contains('G') => apply_color(x.to_string(), &s.by_unit.color_g, theme, faded),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_unit.color_t, theme, faded),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_unit.color_p, theme, faded),
        _ => apply_color(x, &s.by_unit.color_x, theme, faded),
    }
}

fn apply_style_by_percentage(
    x: String,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> StyledObject<String> {
    let value: f64 = x.trim().parse().unwrap_or(0.0);
    if value > 100.0 {
        apply_color(x, &s.by_percentage.color_100, theme, faded)
    } else if value > 75.0 {
        apply_color(x, &s.by_percentage.color_075, theme, faded)
    } else if value > 50.0 {
        apply_color(x, &s.by_percentage.color_050, theme, faded)
    } else if value > 25.0 {
        apply_color(x, &s.by_percentage.color_025, theme, faded)
    } else {
        apply_color(x, &s.by_percentage.color_000, theme, faded)
    }
}

pub fn apply_color(
    x: String,
    c: &ConfigColorByTheme,
    theme: &ConfigTheme,
    faded: bool,
) -> StyledObject<String> {
    let c = match theme {
        ConfigTheme::Dark => &c.dark,
        ConfigTheme::Light => &c.light,
        _ => unreachable!(),
    };

    if faded {
        match c {
            ConfigColor::BrightBlack => BLACK.apply_to(x),
            ConfigColor::BrightRed => RED.apply_to(x),
            ConfigColor::BrightGreen => GREEN.apply_to(x),
            ConfigColor::BrightYellow => YELLOW.apply_to(x),
            ConfigColor::BrightBlue => BLUE.apply_to(x),
            ConfigColor::BrightMagenta => MAGENTA.apply_to(x),
            ConfigColor::BrightCyan => CYAN.apply_to(x),
            ConfigColor::BrightWhite => WHITE.apply_to(x),
            ConfigColor::Black => BLACK.apply_to(x),
            ConfigColor::Red => RED.apply_to(x),
            ConfigColor::Green => GREEN.apply_to(x),
            ConfigColor::Yellow => YELLOW.apply_to(x),
            ConfigColor::Blue => BLUE.apply_to(x),
            ConfigColor::Magenta => MAGENTA.apply_to(x),
            ConfigColor::Cyan => CYAN.apply_to(x),
            ConfigColor::White => WHITE.apply_to(x),
            ConfigColor::Color256(c) => Style::new().color256(*c).apply_to(x),
        }
    } else {
        match c {
            ConfigColor::BrightBlack => BRIGHT_BLACK.apply_to(x),
            ConfigColor::BrightRed => BRIGHT_RED.apply_to(x),
            ConfigColor::BrightGreen => BRIGHT_GREEN.apply_to(x),
            ConfigColor::BrightYellow => BRIGHT_YELLOW.apply_to(x),
            ConfigColor::BrightBlue => BRIGHT_BLUE.apply_to(x),
            ConfigColor::BrightMagenta => BRIGHT_MAGENTA.apply_to(x),
            ConfigColor::BrightCyan => BRIGHT_CYAN.apply_to(x),
            ConfigColor::BrightWhite => BRIGHT_WHITE.apply_to(x),
            ConfigColor::Black => BLACK.apply_to(x),
            ConfigColor::Red => RED.apply_to(x),
            ConfigColor::Green => GREEN.apply_to(x),
            ConfigColor::Yellow => YELLOW.apply_to(x),
            ConfigColor::Blue => BLUE.apply_to(x),
            ConfigColor::Magenta => MAGENTA.apply_to(x),
            ConfigColor::Cyan => CYAN.apply_to(x),
            ConfigColor::White => WHITE.apply_to(x),
            ConfigColor::Color256(c) => Style::new().color256(*c).apply_to(x),
        }
    }
}

pub fn apply_style(
    x: String,
    cs: &ConfigColumnStyle,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> StyledObject<String> {
    match cs {
        ConfigColumnStyle::Fixed(c) => apply_color(x, c, theme, faded),
        ConfigColumnStyle::ByPercentage => apply_style_by_percentage(x, s, theme, faded),
        ConfigColumnStyle::ByState => apply_style_by_state(x, s, theme, faded),
        ConfigColumnStyle::ByUnit => apply_style_by_unit(x, s, theme, faded),
    }
}

pub fn color_to_column_style(c: &ConfigColorByTheme) -> ConfigColumnStyle {
    ConfigColumnStyle::Fixed(c.clone())
}
