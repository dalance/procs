use crate::column::Column;
use crate::columns::{ConfigColumnKind, KIND_LIST};
use crate::config::{Config, ConfigColumnAlign, ConfigSearchCase, ConfigSearchLogic, ConfigTheme};
use crate::opt::ArgThemeMode;
use crate::Opt;
use byte_unit::{Byte, UnitType};
use std::borrow::Cow;
use std::io;
use std::io::IsTerminal;
use std::time::Duration;
use std::time::Instant;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
#[cfg(not(target_os = "windows"))]
use uzers::UsersCache;

impl From<ArgThemeMode> for ConfigTheme {
    fn from(item: ArgThemeMode) -> Self {
        match item {
            ArgThemeMode::Auto => ConfigTheme::Auto,
            ArgThemeMode::Dark => ConfigTheme::Dark,
            ArgThemeMode::Light => ConfigTheme::Light,
        }
    }
}

pub enum KeywordClass {
    Numeric,
    NonNumeric,
}

pub fn find_partial<T: AsRef<str>>(
    columns: &[&dyn Column],
    pid: i32,
    keyword: &[T],
    logic: &ConfigSearchLogic,
    case: &ConfigSearchCase,
) -> bool {
    let mut ret = match logic {
        ConfigSearchLogic::And => true,
        ConfigSearchLogic::Or => false,
        ConfigSearchLogic::Nand => true,
        ConfigSearchLogic::Nor => false,
    };
    for w in keyword {
        let mut hit = false;
        let keyword = w.as_ref();
        let keyword_lowercase = keyword.to_ascii_lowercase();

        let ignore_case = match case {
            ConfigSearchCase::Smart => keyword == keyword.to_ascii_lowercase(),
            ConfigSearchCase::Insensitive => true,
            ConfigSearchCase::Sensitive => false,
        };

        let (keyword, content_to_lowercase) = if ignore_case {
            (keyword_lowercase.as_str(), true)
        } else {
            (keyword, false)
        };

        for c in columns {
            if c.find_partial(pid, keyword, content_to_lowercase) {
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
    case: &ConfigSearchCase,
) -> bool {
    let mut ret = match logic {
        ConfigSearchLogic::And => true,
        ConfigSearchLogic::Or => false,
        ConfigSearchLogic::Nand => true,
        ConfigSearchLogic::Nor => false,
    };
    for w in keyword {
        let mut hit = false;
        let keyword = w.as_ref();
        let keyword_lowercase = keyword.to_ascii_lowercase();

        let ignore_case = match case {
            ConfigSearchCase::Smart => keyword == keyword.to_ascii_lowercase(),
            ConfigSearchCase::Insensitive => true,
            ConfigSearchCase::Sensitive => false,
        };

        let (keyword, content_to_lowercase) = if ignore_case {
            (keyword_lowercase.as_str(), true)
        } else {
            (keyword, false)
        };

        for c in columns {
            if c.find_exact(pid, keyword, content_to_lowercase) {
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
        format!("{year:.1}years")
    } else if day >= 1.0 {
        format!("{day:.1}days")
    } else {
        format!("{hour:02}:{min:02}:{sec:02}")
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
        total_width += UnicodeWidthChar::width(c).unwrap_or_default();
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
    // strict search at first
    for (k, (v, _)) in KIND_LIST.iter() {
        if v.to_lowercase().eq(&pat.to_lowercase()) {
            return Some(k.clone());
        }
    }

    for (k, (v, _)) in KIND_LIST.iter() {
        if v.to_lowercase().contains(&pat.to_lowercase()) {
            return Some(k.clone());
        }
    }
    eprintln!("Can't find column kind: {pat}");
    None
}

#[cfg(target_os = "macos")]
pub fn change_endian(val: u32) -> u32 {
    let mut ret = 0;
    ret |= val >> 24 & 0x000000ff;
    ret |= val >> 8 & 0x0000ff00;
    ret |= val << 8 & 0x00ff0000;
    ret |= val << 24 & 0xff000000;
    ret
}

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
    let byte = Byte::from_u64(x);
    let byte = byte.get_appropriate_unit(UnitType::Binary);
    format!("{:.3}", byte).replace([' ', 'B', 'i'], "")
}

pub fn lap(instant: &mut Instant, msg: &str) {
    let period = instant.elapsed();
    eprintln!(
        "{} [{}.{:03}s]",
        msg,
        period.as_secs(),
        period.subsec_millis()
    );
    instant.clone_from(&Instant::now());
}

pub fn get_theme(opt: &Opt, config: &Config) -> ConfigTheme {
    let theme = match (opt.theme, &config.display.theme) {
        (Some(x), _) => x.into(),
        (_, x) => x.clone(),
    };
    match theme {
        ConfigTheme::Auto => {
            if io::stdout().is_terminal() && io::stderr().is_terminal() && io::stdin().is_terminal()
            {
                let minimum_timeout = Duration::from_millis(100);
                let timeout = if let Ok(latency) = termbg::latency(Duration::from_millis(1000)) {
                    if latency * 2 > minimum_timeout {
                        latency * 2
                    } else {
                        minimum_timeout
                    }
                } else {
                    // If latency detection failed, fallback to dark theme
                    return ConfigTheme::Dark;
                };

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

#[cfg(not(target_os = "windows"))]
thread_local! {
    pub static USERS_CACHE: std::cell::RefCell<UsersCache> = UsersCache::new().into();
}

#[cfg(target_os = "freebsd")]
// std::ffi::FromBytesUntilNulError is missing until Rust 1.73.0
// https://github.com/rust-lang/rust/pull/113701
pub fn ptr_to_cstr(
    x: &[std::os::raw::c_char],
) -> Result<&std::ffi::CStr, core::ffi::FromBytesUntilNulError> {
    let ptr = x.as_ptr() as *const u8;
    let len = x.len();
    let x = unsafe { std::slice::from_raw_parts::<u8>(ptr, len) };
    std::ffi::CStr::from_bytes_until_nul(x)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn process_new(
    pid: i32,
    procfs: &Option<std::path::PathBuf>,
) -> procfs::ProcResult<procfs::process::Process> {
    if let Some(ref x) = procfs {
        let path = x.join(pid.to_string());
        procfs::process::Process::new_with_root(path)
    } else {
        procfs::process::Process::new(pid)
    }
}
