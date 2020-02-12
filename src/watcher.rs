use crate::config::*;
use crate::term_info::TermInfo;
use crate::view::View;
use crate::Opt;
use anyhow::Error;
use chrono::offset::Local;
use getch::Getch;
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
            thread::sleep(Duration::from_secs(interval));
            let _ = tx.send(Command::Wake);
        });
    }

    fn display_header(term_info: &TermInfo, opt: &Opt, interval: u64) {
        let header = if opt.tree {
            format!(
                " Interval: {}s, Last Updated: {} ( Quit: q or Ctrl-C )",
                interval,
                Local::now().format("%Y/%m/%d %H:%M:%S"),
            )
        } else {
            format!(
                " Interval: {}s, Last Updated: {} ( Next: n, Prev: p, Ascending: a, Descending: d, Quit: q or Ctrl-C )",
                interval,
                Local::now().format("%Y/%m/%d %H:%M:%S"),
            )
        };
        let _ = term_info.term.clear_screen();
        let _ = term_info.term.write_line(&format!(
            "{}\n",
            console::style(header).white().bold().underlined()
        ));
    }

    pub fn start(opt: &Opt, config: &Config, interval: u64) -> Result<(), Error> {
        let (tx_cmd, rx_cmd) = channel();
        Watcher::spawn_cmd(tx_cmd.clone());

        let (tx_sleep, rx_sleep) = channel();
        Watcher::spawn_sleep(rx_sleep, tx_cmd, interval);

        let term_info = TermInfo::new();
        let _ = term_info.term.clear_screen();

        let mut sort_offset = 0;
        let mut sort_order = None;
        'outer: loop {
            let mut view = View::new(opt, config);

            // Override sort_info by key
            let max_idx = view.columns.len();
            if !opt.tree {
                view.sort_info.idx = (view.sort_info.idx + sort_offset) % max_idx;
                view.sort_info.order = sort_order.clone().unwrap_or(view.sort_info.order);
            }

            view.filter(opt, config);
            view.adjust(config);

            Watcher::display_header(&view.term_info, opt, interval);

            view.display(opt, config);

            let _ = view
                .term_info
                .term
                .move_cursor_up((view.term_info.height - 1) as usize);

            let _ = tx_sleep.send(Command::Sleep);
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
                        let _ = tx_sleep.send(Command::Quit);
                        view.term_info.term.clear_screen()?;
                        break 'outer;
                    }
                    Command::Next => sort_offset = (sort_offset + 1) % max_idx,
                    Command::Prev => sort_offset = (sort_offset + max_idx - 1) % max_idx,
                    Command::Ascending => sort_order = Some(ConfigSortOrder::Ascending),
                    Command::Descending => sort_order = Some(ConfigSortOrder::Descending),
                    _ => (),
                }
            }
        }
        Ok(())
    }
}
