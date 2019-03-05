use crate::config::{Config, ConfigColumnAlign, ConfigSortOrder};
use crate::process::ProcessInfo;

pub trait Column {
    fn add(&mut self, proc: &ProcessInfo) -> ();

    fn available(&self) -> bool {
        true
    }

    fn display_header(
        &self,
        align: &ConfigColumnAlign,
        order: Option<ConfigSortOrder>,
        config: &Config,
    ) -> String;
    fn display_unit(&self, align: &ConfigColumnAlign) -> String;
    fn display_content(&self, pid: i32, align: &ConfigColumnAlign) -> Option<String>;
    fn find_partial(&self, pid: i32, keyword: &str) -> bool;
    fn find_exact(&self, pid: i32, keyword: &str) -> bool;
    fn sorted_pid(&self, order: &ConfigSortOrder) -> Vec<i32>;
    fn reset_max_width(&mut self, order: Option<ConfigSortOrder>, config: &Config) -> ();
    fn update_max_width(&mut self, pid: i32) -> ();
}

#[macro_export]
macro_rules! column_default_display_header {
    () => {
        fn display_header(&self, align: &crate::config::ConfigColumnAlign, order: Option<crate::config::ConfigSortOrder>, config: &crate::config::Config) -> String {
            if let Some(order) = order {
                let header = match order {
                    crate::config::ConfigSortOrder::Ascending => format!("{} {}", self.header, config.display.ascending),
                    crate::config::ConfigSortOrder::Descending => format!("{} {}", self.header, config.display.descending),
                };
                crate::util::expand(&header, self.max_width, align)
            } else {
                crate::util::expand(&self.header, self.max_width, align)
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_display_unit {
    () => {
        fn display_unit(&self, align: &crate::config::ConfigColumnAlign) -> String {
            crate::util::expand(&self.unit, self.max_width, align)
        }
    };
}

#[macro_export]
macro_rules! column_default_display_content {
    () => {
        fn display_content(&self, pid: i32, align: &crate::config::ConfigColumnAlign) -> Option<String> {
            if let Some(content) = self.fmt_contents.get(&pid) {
                Some(crate::util::expand(content, self.max_width, align))
            } else {
                None
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_find_partial {
    () => {
        fn find_partial(&self, pid: i32, keyword: &str) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                content.find(keyword).is_some()
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_find_exact {
    () => {
        fn find_exact(&self, pid: i32, keyword: &str) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                content == keyword
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_sorted_pid {
    ($x:ty) => {
        fn sorted_pid(&self, order: &crate::config::ConfigSortOrder) -> Vec<i32> {
            let mut contents: Vec<(&i32, &$x)> = self.raw_contents.iter().collect();
            contents.sort_by_key(|&(_x, y)| y);
            if let crate::config::ConfigSortOrder::Descending = order { contents.reverse() }
            contents.iter().map(|(x, _y)| **x).collect()
        }
    };
}

#[macro_export]
macro_rules! column_default_reset_max_width {
    () => {
        fn reset_max_width(&mut self, order: Option<crate::config::ConfigSortOrder>, config: &crate::config::Config) {
            // +1 for spacing between header and sort indicator
            let sorted_space = if let Some(order) = order {
                match order {
                    crate::config::ConfigSortOrder::Ascending => config.display.ascending.chars().count() + 1,
                    crate::config::ConfigSortOrder::Descending => config.display.descending.chars().count() + 1,
                }
            } else {
                0
            };
            self.max_width = std::cmp::max(self.header.chars().count()+sorted_space, self.unit.chars().count());
        }
    };
}

#[macro_export]
macro_rules! column_default_update_max_width {
    () => {
        fn update_max_width(&mut self, pid: i32) {
            if let Some(content) = self.fmt_contents.get(&pid) {
                self.max_width = cmp::max(content.chars().count(), self.max_width);
            }
        }
    };
}

#[macro_export]
macro_rules! column_default {
    ($x:ty) => {
        crate::column_default_display_header!();
        crate::column_default_display_unit!();
        crate::column_default_display_content!();
        crate::column_default_find_partial!();
        crate::column_default_find_exact!();
        crate::column_default_sorted_pid!($x);
        crate::column_default_reset_max_width!();
        crate::column_default_update_max_width!();
    };
}
