use crate::Opt;
use crate::config::*;
use crate::search_regex::SearchRegex;
use crate::term_info::TermInfo;
use crate::util::{get_theme, has_regex_syntax};
use crate::view::View;
use anyhow::Error;
use chrono::offset::Local;
use getch::Getch;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

const KEY_ENTER_LF: u8 = 10;
const KEY_ENTER_CR: u8 = 13;
const KEY_ESC: u8 = 27;
const KEY_BACKSPACE: u8 = 8;
const KEY_DELETE: u8 = 127;

enum Command {
    Wake,
    Sleep,
    Key(u8),
    Quit,
}

pub struct Watcher;

impl Watcher {
    fn spawn_cmd(tx: Sender<Command>) {
        let _ = thread::spawn(move || {
            let getch = Getch::new();
            loop {
                match getch.getch() {
                    // On windows, _getch return EXT(0x3) by Ctrl-C
                    #[cfg(target_os = "windows")]
                    Ok(x) if x == 3 => {
                        let _ = tx.send(Command::Quit);
                        break;
                    }
                    Ok(x) => {
                        let _ = tx.send(Command::Key(x));
                    }
                    _ => (),
                }
            }
        });
    }

    fn spawn_sleep(rx: Receiver<Command>, tx: Sender<Command>, interval: u64) {
        let _ = thread::spawn(move || {
            loop {
                if let Ok(Command::Quit) = rx.recv() {
                    break;
                }
                thread::sleep(Duration::from_millis(interval));
                let _ = tx.send(Command::Wake);
            }
        });
    }

    fn display_header(
        term_info: &TermInfo,
        opt: &Opt,
        interval: u64,
        regex_editing: bool,
        regex_buffer: &str,
        regex_error: &Option<String>,
    ) -> Result<usize, Error> {
        let header = if opt.tree {
            format!(
                " Interval: {}ms, Last Updated: {} ( Quit: q or Ctrl-C )",
                interval,
                Local::now().format("%Y/%m/%d %H:%M:%S"),
            )
        } else {
            format!(
                " Interval: {}ms, Last Updated: {} ( Next: n, Prev: p, Ascending: a, Descending: d, Quit: q or Ctrl-C )",
                interval,
                Local::now().format("%Y/%m/%d %H:%M:%S"),
            )
        };
        let result = header.len();
        term_info.write_line(&format!(
            "{}",
            console::style(header).white().bold().underlined()
        ))?;

        if opt.regex || opt.smart {
            let active = if regex_editing {
                format!(" Editing regex: {}", regex_buffer)
            } else {
                let current = opt.keyword.first().cloned().unwrap_or_default();
                format!(
                    " Regex filter: {} ( Edit: /, Apply: Enter, Cancel: Esc )",
                    current
                )
            };
            term_info.write_line(&active)?;
            if let Some(err) = regex_error {
                term_info.write_line(&format!(" Regex error: {err}"))?;
            }
        }

        term_info.write_line("")?;
        let mut lines = result.div_ceil(term_info.width) + 1;
        if opt.regex || opt.smart {
            lines += 1;
            if regex_error.is_some() {
                lines += 1;
            }
        }
        Ok(lines)
    }

    pub fn start(opt: &mut Opt, config: &Config, interval: u64) -> Result<(), Error> {
        let theme = get_theme(opt, config);

        let (tx_cmd, rx_cmd) = channel();
        Watcher::spawn_cmd(tx_cmd.clone());

        let (tx_sleep, rx_sleep) = channel();
        Watcher::spawn_sleep(rx_sleep, tx_cmd, interval);

        let term_info = TermInfo::new(false, false)?;
        term_info.clear_screen()?;

        let mut sort_idx = None;
        let mut sort_order = None;
        let mut min_widths = HashMap::new();
        let mut prev_term_width = 0;
        let mut prev_term_height = 0;
        let mut regex_editing = false;
        let mut regex_buffer = String::new();
        let mut regex_error: Option<String> = None;
        'outer: loop {
            let mut view = View::new(opt, config, true)?;

            // Override sort_info by key
            if !opt.tree {
                view.sort_info.idx = sort_idx.unwrap_or(view.sort_info.idx);
                view.sort_info.order = sort_order.clone().unwrap_or(view.sort_info.order);
            }

            let resized = prev_term_width != view.term_info.width
                || prev_term_height != view.term_info.height;
            if resized {
                term_info.clear_screen()?;
            }
            let header_lines = Watcher::display_header(
                &view.term_info,
                opt,
                interval,
                regex_editing,
                &regex_buffer,
                &regex_error,
            )?;

            view.filter(opt, config, header_lines)?;
            view.adjust(config, &min_widths);
            for (i, c) in view.columns.iter().enumerate() {
                min_widths.insert(i, c.column.get_width());
            }

            view.display(opt, config, &theme)?;

            view.term_info.clear_rest_lines()?;
            view.term_info.move_cursor_to(0, 0)?;

            tx_sleep.send(Command::Sleep)?;
            let mut cmds = Vec::new();
            if let Ok(cmd) = rx_cmd.recv() {
                cmds.push(cmd);
                for c in rx_cmd.try_iter() {
                    cmds.push(c);
                }
            }

            for cmd in cmds {
                match cmd {
                    Command::Quit => {
                        tx_sleep.send(Command::Quit)?;
                        view.term_info.clear_screen()?;
                        break 'outer;
                    }
                    Command::Key(x) => {
                        if !regex_editing {
                            match char::from(x) {
                                'q' => {
                                    tx_sleep.send(Command::Quit)?;
                                    view.term_info.clear_screen()?;
                                    break 'outer;
                                }
                                'n' => sort_idx = Some(view.inc_sort_column()),
                                'p' => sort_idx = Some(view.dec_sort_column()),
                                'a' => sort_order = Some(ConfigSortOrder::Ascending),
                                'd' => sort_order = Some(ConfigSortOrder::Descending),
                                '/' if opt.regex || opt.smart => {
                                    regex_editing = true;
                                    regex_buffer = opt.keyword.first().cloned().unwrap_or_default();
                                }
                                _ => (),
                            }
                        } else {
                            match x {
                                KEY_ENTER_LF | KEY_ENTER_CR => {
                                    let candidate = regex_buffer.clone();
                                    if candidate.is_empty() {
                                        opt.keyword.clear();
                                        regex_error = None;
                                        regex_editing = false;
                                    } else {
                                        let use_regex = opt.regex
                                            || (opt.smart && has_regex_syntax(&candidate));
                                        if use_regex {
                                            let ignore_case = match config.search.case {
                                                ConfigSearchCase::Smart => {
                                                    candidate == candidate.to_ascii_lowercase()
                                                }
                                                ConfigSearchCase::Insensitive => true,
                                                ConfigSearchCase::Sensitive => false,
                                            };
                                            match SearchRegex::new(&candidate, ignore_case) {
                                                Ok(_) => {
                                                    opt.keyword = vec![candidate];
                                                    regex_error = None;
                                                    regex_editing = false;
                                                }
                                                Err(e) => {
                                                    regex_error = Some(e.to_string());
                                                }
                                            }
                                        } else {
                                            opt.keyword = vec![candidate];
                                            regex_error = None;
                                            regex_editing = false;
                                        }
                                    }
                                }
                                KEY_ESC => {
                                    regex_editing = false;
                                    regex_error = None;
                                }
                                KEY_BACKSPACE | KEY_DELETE => {
                                    regex_buffer.pop();
                                }
                                _ => {
                                    let c = char::from(x);
                                    if !c.is_control() {
                                        regex_buffer.push(c);
                                    }
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }

            prev_term_width = view.term_info.width;
            prev_term_height = view.term_info.height;
        }
        Ok(())
    }
}
