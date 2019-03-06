use crate::column::Column;
use crate::config::{ConfigColumnAlign, ConfigSearchLogic};
use std::borrow::Cow;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub enum KeywordClass {
    Numeric,
    NonNumeric,
}

pub fn find_partial<T: AsRef<str>>(
    columns: &[&Column],
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
    columns: &[&Column],
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

pub fn expand(x: &str, len: usize, align: &ConfigColumnAlign) -> String {
    match align {
        ConfigColumnAlign::Left => format!("{}{}", x, " ".repeat(len - UnicodeWidthStr::width(x))),
        ConfigColumnAlign::Right => format!("{}{}", " ".repeat(len - UnicodeWidthStr::width(x)), x),
        ConfigColumnAlign::Center => {
            let space = len - UnicodeWidthStr::width(x);
            let left = space / 2;
            let right = space / 2 + space % 2;
            format!("{}{}{}", " ".repeat(left), x, " ".repeat(right))
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
