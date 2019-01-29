use crate::column::Column;

pub enum KeywordClass {
    Integer,
    Other,
}

pub struct Util;

impl Util {
    pub fn find<T: AsRef<str>>(columns: &[&Column], pid: i32, keyword: &[T]) -> bool {
        for w in keyword {
            for c in columns {
                if c.find(pid, w.as_ref()) {
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
            Ok(_) => KeywordClass::Integer,
            _ => KeywordClass::Other,
        }
    }

    pub fn mask(x: &str) -> String {
        let mut masked = String::from("");
        for c in x.chars() {
            if c != ' ' {
                masked.push('*')
            } else {
                masked.push(' ')
            }
        }
        masked
    }
}
