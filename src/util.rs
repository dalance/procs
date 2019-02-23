use crate::column::Column;

pub enum KeywordClass {
    Numeric,
    NonNumeric,
}

pub fn find_partial<T: AsRef<str>>(columns: &[&Column], pid: i32, keyword: &[T]) -> bool {
    for w in keyword {
        for c in columns {
            if c.find_partial(pid, w.as_ref()) {
                return true;
            }
        }
    }
    false
}

pub fn find_exact<T: AsRef<str>>(columns: &[&Column], pid: i32, keyword: &[T]) -> bool {
    for w in keyword {
        for c in columns {
            if c.find_exact(pid, w.as_ref()) {
                return true;
            }
        }
    }
    false
}

pub fn classify(keyword: &str) -> KeywordClass {
    let parsed = keyword.parse::<i64>();
    match parsed {
        Ok(_) => KeywordClass::Numeric,
        _ => KeywordClass::NonNumeric,
    }
}

pub fn expand(x: &str, len: usize) -> String {
    let ret = format!("{}{}", x, " ".repeat(len - x.len()));
    ret
}

pub fn parse_time(x: u64) -> String {
    let rest = x;

    let sec = rest % 60;
    let rest = rest / 60;

    let min = rest % 60;
    let rest = rest / 60;

    let hour = rest % 24;
    let rest = rest / 24;

    let day = rest % 365;
    let rest = rest / 365;

    let year = rest;

    if year != 0 {
        format!("{}years", year)
    } else if day != 0 {
        format!("{}days", day)
    } else {
        format!("{:02}:{:02}:{:02}", hour, min, sec)
    }
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
