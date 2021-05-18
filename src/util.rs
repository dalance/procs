use crate::column::Column;
use crate::columns::{ConfigColumnKind, KIND_LIST};
use crate::config::{Config, ConfigColumnAlign, ConfigSearchLogic, ConfigTheme};
use crate::Opt;
use byte_unit::Byte;
use std::borrow::Cow;
use std::time::Duration;
use std::time::Instant;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub enum KeywordClass {
    Numeric,
    NonNumeric,
}

pub fn find_partial<T: AsRef<str>>(
    columns: &[&dyn Column],
    pid: i32,
    keyword: &[T],
    logic: &ConfigSearchLogic,
) -> bool {
    let mut ret = match logic {
        ConfigSearchLogic::And => true,
        ConfigSearchLogic::Or => false,
        ConfigSearchLogic::Nand => true,
        ConfigSearchLogic::Nor => false,
    };
    for w in keyword {
        let mut hit = false;
        for c in columns {
            if c.find_partial(pid, w.as_ref()) {
                hit = true;
                break;
            }
        }
        ret = match logic {
            ConfigSearchLogic::And => ret & hit,
            ConfigSearchLogic::Or => ret | hit,
            ConfigSearchLogic::Nand => ret & hit,
            ConfigSearchLogic::Nor => ret | hit,
        };
    }
    ret
}

pub fn find_exact<T: AsRef<str>>(
    columns: &[&dyn Column],
    pid: i32,
    keyword: &[T],
    logic: &ConfigSearchLogic,
) -> bool {
    let mut ret = match logic {
        ConfigSearchLogic::And => true,
        ConfigSearchLogic::Or => false,
        ConfigSearchLogic::Nand => true,
        ConfigSearchLogic::Nor => false,
    };
    for w in keyword {
        let mut hit = false;
        for c in columns {
            if c.find_exact(pid, w.as_ref()) {
                hit = true;
                break;
            }
        }
        ret = match logic {
            ConfigSearchLogic::And => ret & hit,
            ConfigSearchLogic::Or => ret | hit,
            ConfigSearchLogic::Nand => ret & hit,
            ConfigSearchLogic::Nor => ret | hit,
        };
    }
    ret
}

pub fn classify(keyword: &str) -> KeywordClass {
    let parsed = keyword.parse::<i64>();
    match parsed {
        Ok(_) => KeywordClass::Numeric,
        _ => KeywordClass::NonNumeric,
    }
}

pub fn adjust(x: &str, len: usize, align: &ConfigColumnAlign) -> String {
    if len < UnicodeWidthStr::width(x) {
        String::from(truncate(x, len))
    } else {
        match align {
            ConfigColumnAlign::Left => {
                format!("{}{}", x, " ".repeat(len - UnicodeWidthStr::width(x)))
            }
            ConfigColumnAlign::Right => {
                format!("{}{}", " ".repeat(len - UnicodeWidthStr::width(x)), x)
            }
            ConfigColumnAlign::Center => {
                let space = len - UnicodeWidthStr::width(x);
                let left = space / 2;
                let right = space / 2 + space % 2;
                format!("{}{}{}", " ".repeat(left), x, " ".repeat(right))
            }
        }
    }
}

pub fn parse_time(x: u64) -> String {
    let rest = x;

    let sec = rest % 60;
    let rest = rest / 60;

    let min = rest % 60;
    let rest = rest / 60;

    let hour = rest % 24;

    let day = x as f64 / (60.0 * 60.0 * 24.0);
    let year = x as f64 / (365.0 * 60.0 * 60.0 * 24.0);

    if year >= 1.0 {
        format!("{:.1}years", year)
    } else if day >= 1.0 {
        format!("{:.1}days", day)
    } else {
        format!("{:02}:{:02}:{:02}", hour, min, sec)
    }
}

pub fn truncate(s: &'_ str, width: usize) -> Cow<'_, str> {
    let mut total_width = 0;
    let mut ret = None;
    let mut buf = String::new();
    let mut escape = false;
    for c in s.chars() {
        if c == '\u{1b}' {
            escape = true;
        }
        if escape {
            if c == 'm' {
                escape = false;
            }
            buf.push(c);
            continue;
        }
        total_width += if let Some(x) = UnicodeWidthChar::width(c) {
            x
        } else {
            0
        };
        if total_width > width {
            ret = Some(buf);
            break;
        }
        buf.push(c);
    }
    if let Some(buf) = ret {
        Cow::Owned(buf)
    } else {
        Cow::Borrowed(s)
    }
}

pub fn find_column_kind(pat: &str) -> Option<ConfigColumnKind> {
    for (k, (v, _)) in KIND_LIST.iter() {
        if v.to_lowercase().find(&pat.to_lowercase()).is_some() {
            return Some(k.clone());
        }
    }
    None
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
pub fn change_endian(val: u32) -> u32 {
    let mut ret = 0;
    ret |= val >> 24 & 0x000000ff;
    ret |= val >> 8 & 0x0000ff00;
    ret |= val << 8 & 0x00ff0000;
    ret |= val << 24 & 0xff000000;
    ret
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "macos")]
pub unsafe fn get_sys_value(
    high: u32,
    low: u32,
    mut len: usize,
    value: *mut libc::c_void,
    mib: &mut [i32; 2],
) -> bool {
    mib[0] = high as i32;
    mib[1] = low as i32;
    libc::sysctl(
        mib.as_mut_ptr(),
        2,
        value,
        &mut len as *mut usize,
        ::std::ptr::null_mut(),
        0,
    ) == 0
}

#[cfg_attr(tarpaulin, skip)]
#[cfg(target_os = "windows")]
pub fn format_sid(sid: &[u64], abbr: bool) -> String {
    let mut ret = format!("S-{}-{}-{}", sid[0], sid[1], sid[2]);
    if sid.len() > 3 {
        if abbr {
            ret = format!("{}-...-{}", ret, sid[sid.len() - 1]);
        } else {
            for s in sid.iter().skip(3) {
                ret = format!("{}-{}", ret, s);
            }
        }
    }

    ret
}

pub fn bytify(x: u64) -> String {
    let byte = Byte::from_bytes(x as u128);
    let byte = byte.get_appropriate_unit(true);
    byte.format(3)
        .replace(" ", "")
        .replace("B", "")
        .replace("i", "")
}

pub fn lap(instant: &mut Instant, msg: &str) {
    let period = instant.elapsed();
    eprintln!(
        "{} [{}.{:03}s]",
        msg,
        period.as_secs(),
        period.subsec_nanos() / 1000000
    );
    instant.clone_from(&Instant::now());
}

pub fn get_theme(opt: &Opt, config: &Config) -> ConfigTheme {
    let theme = match (opt.theme.as_ref(), &config.display.theme) {
        (Some(x), _) => match x.as_str() {
            "auto" => ConfigTheme::Auto,
            "dark" => ConfigTheme::Dark,
            "light" => ConfigTheme::Light,
            _ => unreachable!(),
        },
        (_, x) => x.clone(),
    };
    match theme {
        ConfigTheme::Auto => {
            if console::user_attended() {
                let timeout = Duration::from_millis(100);
                if let Ok(theme) = termbg::theme(timeout) {
                    match theme {
                        termbg::Theme::Dark => ConfigTheme::Dark,
                        termbg::Theme::Light => ConfigTheme::Light,
                    }
                } else {
                    // If termbg failed, fallback to dark theme
                    ConfigTheme::Dark
                }
            } else {
                // If piped or redirected, fallback to dark theme
                ConfigTheme::Dark
            }
        }
        x => x,
    }
}
