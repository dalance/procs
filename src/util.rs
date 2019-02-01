use crate::column::Column;

pub enum KeywordClass {
    Numeric,
    NonNumeric,
}

pub fn find_partial<T: AsRef<str>>(columns: &[&Box<Column>], pid: i32, keyword: &[T]) -> bool {
    for w in keyword {
        for c in columns {
            if c.find_partial(pid, w.as_ref()) {
                return true;
            }
        }
    }
    false
}

pub fn find_exact<T: AsRef<str>>(columns: &[&Box<Column>], pid: i32, keyword: &[T]) -> bool {
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
