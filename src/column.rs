use crate::config::{Config, ConfigColumnAlign, ConfigSortOrder};
use crate::process::ProcessInfo;

pub trait Column {
    fn add(&mut self, proc: &ProcessInfo);

    fn available(&self) -> bool {
        true
    }

    fn sortable(&self) -> bool {
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
    fn display_json(&self, pid: i32) -> String;
    fn find_partial(&self, pid: i32, keyword: &str, content_to_lowercase: bool) -> bool;
    fn find_exact(&self, pid: i32, keyword: &str, content_to_lowercase: bool) -> bool;
    fn sorted_pid(&self, order: &ConfigSortOrder) -> Vec<i32>;
    fn apply_visible(&mut self, visible_pids: &[i32]);
    fn reset_width(
        &mut self,
        order: Option<ConfigSortOrder>,
        config: &Config,
        max_width: Option<usize>,
        min_width: Option<usize>,
    );
    fn update_width(&mut self, pid: i32, max_width: Option<usize>);
    fn get_width(&self) -> usize;
    fn is_numeric(&self) -> bool;
}

#[macro_export]
macro_rules! column_default_display_header {
    () => {
        fn display_header(
            &self,
            align: &$crate::config::ConfigColumnAlign,
            order: Option<$crate::config::ConfigSortOrder>,
            config: &$crate::config::Config,
        ) -> String {
            if let Some(order) = order {
                let header = match order {
                    $crate::config::ConfigSortOrder::Ascending => {
                        format!("{}:{}", self.header, config.display.ascending)
                    }
                    $crate::config::ConfigSortOrder::Descending => {
                        format!("{}:{}", self.header, config.display.descending)
                    }
                };
                $crate::util::adjust(&header, self.width, align)
            } else {
                $crate::util::adjust(&self.header, self.width, align)
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_display_unit {
    () => {
        fn display_unit(&self, align: &$crate::config::ConfigColumnAlign) -> String {
            $crate::util::adjust(&self.unit, self.width, align)
        }
    };
}

#[macro_export]
macro_rules! column_default_display_content {
    () => {
        fn display_content(
            &self,
            pid: i32,
            align: &$crate::config::ConfigColumnAlign,
        ) -> Option<String> {
            self.fmt_contents
                .get(&pid)
                .map(|content| $crate::util::adjust(content, self.width, align))
        }
    };
}

#[macro_export]
macro_rules! column_default_display_json {
    () => {
        fn display_json(&self, pid: i32) -> String {
            let value = if self.is_numeric() {
                self.raw_contents
                    .get(&pid)
                    .map(|x| x.to_string())
                    .unwrap_or("".to_string())
            } else {
                let value = self
                    .fmt_contents
                    .get(&pid)
                    .map(|x| x.clone())
                    .unwrap_or("".to_string());
                let value = value.replace("\\", "\\\\");
                let value = value.replace("\"", "\\\"");
                format!("\"{}\"", value)
            };
            format!("\"{}\": {}", self.header, value)
        }
    };
}

#[macro_export]
macro_rules! column_default_find_partial {
    () => {
        fn find_partial(&self, pid: i32, keyword: &str, content_to_lowercase: bool) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                if content_to_lowercase {
                    content.to_ascii_lowercase().find(keyword).is_some()
                } else {
                    content.find(keyword).is_some()
                }
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_find_exact {
    () => {
        fn find_exact(&self, pid: i32, keyword: &str, content_to_lowercase: bool) -> bool {
            if let Some(content) = self.fmt_contents.get(&pid) {
                if content_to_lowercase {
                    content.to_ascii_lowercase() == keyword
                } else {
                    content == keyword
                }
            } else {
                false
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_sorted_pid {
    ($x:ty) => {
        fn sorted_pid(&self, order: &$crate::config::ConfigSortOrder) -> Vec<i32> {
            let mut contents: Vec<(&i32, &$x)> = self.raw_contents.iter().collect();
            contents.sort_by_key(|&(_x, y)| y);
            if matches!(*order, $crate::config::ConfigSortOrder::Descending) {
                contents.reverse()
            }
            contents.iter().map(|(x, _y)| **x).collect()
        }
    };
}

#[macro_export]
macro_rules! column_default_apply_visible {
    () => {
        fn apply_visible(&mut self, _visible_pids: &[i32]) {}
    };
}

#[macro_export]
macro_rules! column_default_reset_width {
    () => {
        fn reset_width(
            &mut self,
            order: Option<$crate::config::ConfigSortOrder>,
            config: &$crate::config::Config,
            max_width: Option<usize>,
            min_width: Option<usize>,
        ) {
            // +1 for spacing between header and sort indicator
            let sorted_space = if let Some(order) = order {
                match order {
                    $crate::config::ConfigSortOrder::Ascending => {
                        unicode_width::UnicodeWidthStr::width(config.display.ascending.as_str()) + 1
                    }
                    $crate::config::ConfigSortOrder::Descending => {
                        unicode_width::UnicodeWidthStr::width(config.display.descending.as_str())
                            + 1
                    }
                }
            } else {
                0
            };
            let header_len = unicode_width::UnicodeWidthStr::width(self.header.as_str());
            let unit_len = unicode_width::UnicodeWidthStr::width(self.unit.as_str());
            self.width = std::cmp::max(header_len + sorted_space, unit_len);
            if let Some(min_width) = min_width {
                self.width = std::cmp::max(self.width, min_width);
            }
            if let Some(max_width) = max_width {
                self.width = std::cmp::min(self.width, max_width);
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_update_width {
    () => {
        fn update_width(&mut self, pid: i32, max_width: Option<usize>) {
            if let Some(content) = self.fmt_contents.get(&pid) {
                let content_len = unicode_width::UnicodeWidthStr::width(content.as_str());
                self.width = cmp::max(content_len, self.width);
                if let Some(max_width) = max_width {
                    self.width = std::cmp::min(self.width, max_width);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! column_default_get_width {
    () => {
        fn get_width(&self) -> usize {
            self.width
        }
    };
}

#[macro_export]
macro_rules! column_default_is_numeric {
    ($x:expr) => {
        fn is_numeric(&self) -> bool {
            $x
        }
    };
}

#[macro_export]
macro_rules! column_default {
    ($x:ty, $y:expr) => {
        $crate::column_default_display_header!();
        $crate::column_default_display_unit!();
        $crate::column_default_display_content!();
        $crate::column_default_display_json!();
        $crate::column_default_find_partial!();
        $crate::column_default_find_exact!();
        $crate::column_default_sorted_pid!($x);
        $crate::column_default_apply_visible!();
        $crate::column_default_reset_width!();
        $crate::column_default_update_width!();
        $crate::column_default_get_width!();
        $crate::column_default_is_numeric!($y);
    };
}
