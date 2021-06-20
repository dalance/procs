use crate::config::*;
use crate::term_info::TermInfo;
use crate::util::get_theme;
use crate::view::View;
use crate::Opt;
use anyhow::Error;
use chrono::offset::Local;
use getch::Getch;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

enum Command {
    Wake,
    Sleep,
    Next,
    Prev,
    Ascending,
    Descending,
    Quit,
}

pub struct Watcher;

#[cfg_attr(tarpaulin, skip)]
impl Watcher {
    fn spawn_cmd(tx: Sender<Command>) {
        let _ = thread::spawn(move || {
            let getch = Getch::new();
            loop {
                match getch.getch() {
                    Ok(x) if char::from(x) == 'q' => {
                        let _ = tx.send(Command::Quit);
                        break;
                    }
                    Ok(x) if char::from(x) == 'n' => {
                        let _ = tx.send(Command::Next);
                    }
                    Ok(x) if char::from(x) == 'p' => {
                        let _ = tx.send(Command::Prev);
                    }
                    Ok(x) if char::from(x) == 'a' => {
                        let _ = tx.send(Command::Ascending);
                    }
                    Ok(x) if char::from(x) == 'd' => {
                        let _ = tx.send(Command::Descending);
                    }
                    // On windows, _getch return EXT(0x3) by Ctrl-C
                    #[cfg(target_os = "windows")]
                    Ok(x) if x == 3 => {
                        let _ = tx.send(Command::Quit);
                        break;
                    }
                    _ => (),
                }
            }
        });
    }

    fn spawn_sleep(rx: Receiver<Command>, tx: Sender<Command>, interval: u64) {
        let _ = thread::spawn(move || loop {
            if let Ok(Command::Quit) = rx.recv() {
                break;
            }
            thread::sleep(Duration::from_millis(interval));
            let _ = tx.send(Command::Wake);
        });
    }

    fn display_header(term_info: &mut TermInfo, opt: &Opt, interval: u64) -> Result<(), Error> {
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
        term_info.write_line(&format!(
            "{}",
            console::style(header).white().bold().underlined()
        ))?;

        term_info.write_line("")?;
        Ok(())
    }

    pub fn start(opt: &Opt, config: &Config, interval: u64) -> Result<(), Error> {
        let theme = get_theme(opt, config);

        let (tx_cmd, rx_cmd) = channel();
        Watcher::spawn_cmd(tx_cmd.clone());

        let (tx_sleep, rx_sleep) = channel();
        Watcher::spawn_sleep(rx_sleep, tx_cmd, interval);

        let term_info = TermInfo::new(false);
        term_info.clear_screen()?;

        let mut sort_idx = None;
        let mut sort_order = None;
        let mut min_widths = HashMap::new();
        let mut prev_term_width = 0;
        let mut prev_term_height = 0;
        'outer: loop {
            let mut view = View::new(opt, config, true)?;

            // Override sort_info by key
            if !opt.tree {
                view.sort_info.idx = sort_idx.unwrap_or(view.sort_info.idx);
                view.sort_info.order = sort_order.clone().unwrap_or(view.sort_info.order);
            }

            view.filter(opt, config);
            view.adjust(config, &min_widths);
            for (i, c) in view.columns.iter().enumerate() {
                min_widths.insert(i, c.column.get_width());
            }

            let resized = prev_term_width != view.term_info.width
                || prev_term_height != view.term_info.height;
            if resized {
                term_info.clear_screen()?;
            }
            Watcher::display_header(&mut view.term_info, opt, interval)?;

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
                    Command::Next => sort_idx = Some(view.inc_sort_column()),
                    Command::Prev => sort_idx = Some(view.dec_sort_column()),
                    Command::Ascending => sort_order = Some(ConfigSortOrder::Ascending),
                    Command::Descending => sort_order = Some(ConfigSortOrder::Descending),
                    _ => (),
                }
            }

            prev_term_width = view.term_info.width;
            prev_term_height = view.term_info.height;
        }
        Ok(())
    }
}
