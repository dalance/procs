use crate::column::Column;

pub enum KeywordClass {
    Integer,
    Other,
}

pub struct Finder;

impl Finder {
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

    pub fn keyword_class(keyword: &str) -> KeywordClass {
        let parsed = keyword.parse::<i64>();
        match parsed {
            Ok(_) => KeywordClass::Integer,
            _ => KeywordClass::Other,
        }
    }
}
