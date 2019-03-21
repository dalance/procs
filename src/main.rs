mod column;
mod columns;
mod config;
#[cfg(target_os = "macos")]
mod libproc;
mod process;
mod style;
mod util;

use crate::column::Column;
use crate::columns::*;
use crate::config::*;
use crate::process::collect_proc;
use crate::style::{apply_color, apply_style};
use crate::util::{expand, find_column_kind, truncate, KeywordClass};
use chrono::offset::Local;
use console::Term;
use failure::{Error, ResultExt};
use getch::Getch;
#[cfg(not(target_os = "windows"))]
use pager::Pager;
use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use structopt::{clap, StructOpt};
use unicode_width::UnicodeWidthStr;

// ---------------------------------------------------------------------------------------------------------------------
// Opt
// ---------------------------------------------------------------------------------------------------------------------

#[derive(Debug, StructOpt)]
#[structopt(raw(
    long_version = "option_env!(\"LONG_VERSION\").unwrap_or(env!(\"CARGO_PKG_VERSION\"))"
))]
#[structopt(raw(setting = "clap::AppSettings::ColoredHelp"))]
#[structopt(raw(setting = "clap::AppSettings::DeriveDisplayOrder"))]
pub struct Opt {
    /// Keywords for search
    #[structopt(name = "KEYWORD")]
    pub keyword: Vec<String>,

    /// AND  logic for multi-keyword
    #[structopt(
        short = "a",
        long = "and",
        raw(conflicts_with_all = "&[\"or\", \"nand\", \"nor\"]")
    )]
    pub and: bool,

    /// OR   logic for multi-keyword
    #[structopt(
        short = "o",
        long = "or",
        raw(conflicts_with_all = "&[\"and\", \"nand\", \"nor\"]")
    )]
    pub or: bool,

    /// NAND logic for multi-keyword
    #[structopt(
        short = "d",
        long = "nand",
        raw(conflicts_with_all = "&[\"and\", \"or\", \"nor\"]")
    )]
    pub nand: bool,

    /// NOR  logic for multi-keyword
    #[structopt(
        short = "r",
        long = "nor",
        raw(conflicts_with_all = "&[\"and\", \"or\", \"nand\"]")
    )]
    pub nor: bool,

    /// Show list of kind
    #[structopt(short = "l", long = "list")]
    pub list: bool,

    /// Watch mode
    #[structopt(short = "w", long = "watch", value_name = "second")]
    pub watch: Option<u64>,

    /// Insert column to slot
    #[structopt(
        value_name = "kind",
        short = "i",
        long = "insert",
        raw(multiple = "true"),
        raw(number_of_values = "1")
    )]
    pub insert: Vec<String>,

    /// Sort column by ascending
    #[structopt(value_name = "kind", long = "sorta", raw(conflicts_with = "\"sortd\""))]
    pub sorta: Option<String>,

    /// Sort column by descending
    #[structopt(value_name = "kind", long = "sortd", raw(conflicts_with = "\"sorta\""))]
    pub sortd: Option<String>,

    /// Color mode
    #[structopt(
        short = "c",
        long = "color",
        possible_value = "auto",
        possible_value = "always",
        possible_value = "disable"
    )]
    pub color: Option<String>,

    /// Pager mode
    #[structopt(
        short = "p",
        long = "pager",
        possible_value = "auto",
        possible_value = "always",
        possible_value = "disable"
    )]
    pub pager: Option<String>,

    /// Interval to calculate throughput
    #[structopt(long = "interval", default_value = "100", value_name = "millisec")]
    pub interval: u64,

    /// Generate configuration sample file
    #[structopt(long = "config")]
    pub config: bool,
}

// ---------------------------------------------------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------------------------------------------------

#[cfg_attr(tarpaulin, skip)]
fn get_config() -> Result<Config, Error> {
    let dot_cfg_path = directories::BaseDirs::new()
        .map(|base| base.home_dir().join(".procs.toml"))
        .filter(|path| path.exists());
    let app_cfg_path = directories::ProjectDirs::from("com.github", "dalance", "procs")
        .map(|proj| proj.config_dir().join("config.toml"))
        .filter(|path| path.exists());
    let cfg_path = dot_cfg_path.or(app_cfg_path);

    let config: Config = if let Some(path) = cfg_path {
        let mut f = fs::File::open(&path).context(format!("failed to open file ({:?})", path))?;
        let mut s = String::new();
        let _ = f.read_to_string(&mut s);
        toml::from_str(&s).context(format!("failed to parse toml ({:?})", path))?
    } else {
        toml::from_str(CONFIG_DEFAULT).unwrap()
    };

    Ok(config)
}

fn display_header(
    term: &Term,
    max_width: usize,
    cols: &[ColumnInfo],
    config: &Config,
    sort_idx: usize,
    sort_order: &ConfigSortOrder,
) {
    let mut row = String::from("");
    for (i, c) in cols.iter().enumerate() {
        let order = if i == sort_idx {
            Some(sort_order.clone())
        } else {
            None
        };
        row = format!(
            "{} {}",
            row,
            apply_color(
                c.column.display_header(&c.align, order, config),
                &config.style.header
            )
        );
    }
    row = row.trim_end().to_string();
    //row = console::truncate_str(&row, max_width, "").to_string();
    row = truncate(&row, max_width).to_string();
    let _ = term.write_line(&row);
}

fn display_unit(term: &Term, max_width: usize, cols: &[ColumnInfo], config: &Config) {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_color(c.column.display_unit(&c.align), &config.style.unit)
        );
    }
    row = row.trim_end().to_string();
    //row = console::truncate_str(&row, max_width, "").to_string();
    row = truncate(&row, max_width).to_string();
    let _ = term.write_line(&row);
}

fn display_content(term: &Term, pid: i32, max_width: usize, cols: &[ColumnInfo], config: &Config) {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_style(
                c.column.display_content(pid, &c.align).unwrap(),
                &c.style,
                &config.style
            )
        );
    }
    row = row.trim_end().to_string();
    //row = console::truncate_str(&row, max_width, "").to_string();
    row = truncate(&row, max_width).to_string();
    let _ = term.write_line(&row);
}

fn search<T: AsRef<str>>(
    pid: i32,
    keyword_numeric: &[T],
    keyword_nonnumeric: &[T],
    cols_numeric: &[&Column],
    cols_nonnumeric: &[&Column],
    config: &Config,
    logic: &ConfigSearchLogic,
) -> bool {
    let ret_nonnumeric = match config.search.nonnumeric_search {
        ConfigSearchKind::Partial => {
            util::find_partial(cols_nonnumeric, pid, keyword_nonnumeric, logic)
        }
        ConfigSearchKind::Exact => {
            util::find_exact(cols_nonnumeric, pid, keyword_nonnumeric, logic)
        }
    };
    let ret_numeric = match config.search.numeric_search {
        ConfigSearchKind::Partial => util::find_partial(cols_numeric, pid, keyword_numeric, logic),
        ConfigSearchKind::Exact => util::find_exact(cols_numeric, pid, keyword_numeric, logic),
    };
    match logic {
        ConfigSearchLogic::And => ret_nonnumeric & ret_numeric,
        ConfigSearchLogic::Or => ret_nonnumeric | ret_numeric,
        ConfigSearchLogic::Nand => !(ret_nonnumeric & ret_numeric),
        ConfigSearchLogic::Nor => !(ret_nonnumeric | ret_numeric),
    }
}

#[cfg(not(target_os = "windows"))]
fn pager(config: &Config) {
    if let Some(ref pager) = config.pager.command {
        Pager::with_pager(&pager).setup();
    } else if quale::which("less").is_some() {
        Pager::with_pager("less -SR").setup();
    } else {
        Pager::with_pager("more -f").setup();
    }
}

#[cfg(target_os = "windows")]
fn pager(_config: &Config) {}

// ---------------------------------------------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------------------------------------------

#[cfg_attr(tarpaulin, skip)]
fn main() {
    let err = Term::stderr();

    if let Err(x) = run() {
        let mut cause = x.iter_chain();
        let _ = err.write_line(&format!(
            "{} {}",
            console::style("error:").red().bold(),
            cause.next().unwrap()
        ));

        for x in cause {
            let _ = err.write_line(&format!("  {} {}", console::style("caused by:").red(), x));
        }

        std::process::exit(1);
    }
}

#[cfg_attr(tarpaulin, skip)]
fn run() -> Result<(), Error> {
    let opt = Opt::from_args();

    if opt.config {
        run_config()?;
    }

    if opt.list {
        run_list()?;
    }

    let config = get_config()?;

    if let Some(interval) = opt.watch {
        run_watch(&opt, &config, interval)?;
    } else {
        run_default(&opt, &config)?;
    }

    Ok(())
}

fn run_config() -> Result<(), Error> {
    let config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
    let toml = toml::to_string(&config)?;
    println!("{}", toml);
    Ok(())
}

fn run_list() -> Result<(), Error> {
    let mut max_width = 0;
    let mut list = Vec::new();
    let mut desc = HashMap::new();
    for (_, (v, d)) in KIND_LIST.iter() {
        list.push(v);
        desc.insert(v, d);
        max_width = cmp::max(max_width, UnicodeWidthStr::width(*v));
    }

    list.sort();

    println!("Column kind list:");
    for l in list {
        println!(
            "  {}: {}",
            expand(l, max_width, &ConfigColumnAlign::Left),
            desc[l]
        );
    }

    Ok(())
}

#[cfg_attr(tarpaulin, skip)]
fn run_watch(opt: &Opt, config: &Config, interval: u64) -> Result<(), Error> {
    let (tx, rx) = channel();
    let _ = thread::spawn(move || {
        let getch = Getch::new();
        loop {
            match getch.getch() {
                Ok(x) if char::from(x) == 'q' => break,
                _ => (),
            }
        }
        let _ = tx.send(());
    });

    let term = Term::stdout();
    let mut term_h_prev = 0;
    let mut term_w_prev = 0;
    'outer: loop {
        let (term_h, term_w) = term.size();
        let term_changed = term_h != term_h_prev || term_w != term_w_prev;
        if term_changed {
            let _ = term.clear_screen();
        }
        term_h_prev = term_h;
        term_w_prev = term_w;

        let _ = term.write_line(&format!(
            "{}\n",
            console::style(format!(
                " Interval: {}s, Last Updated: {} ( Quit: q or Ctrl-C )",
                interval,
                Local::now().format("%Y/%m/%d %H:%M:%S"),
            ))
            .white()
            .bold()
            .underlined()
        ));

        run_default(&opt, &config)?;
        let _ = term.move_cursor_up((term_h - 1) as usize);

        for _ in 0..interval * 10 {
            thread::sleep(Duration::from_millis(100));

            if rx.try_recv().is_ok() {
                break 'outer;
            }
        }
    }
    Ok(())
}

#[allow(clippy::cyclomatic_complexity)]
fn run_default(opt: &Opt, config: &Config) -> Result<(), Error> {
    // -------------------------------------------------------------------------
    // Generate column
    // -------------------------------------------------------------------------

    let mut slot_idx = 0;
    let mut cols = Vec::new();
    for c in &config.columns {
        let kind = match &c.kind {
            ConfigColumnKind::Slot => {
                let kind = if let Some(insert) = opt.insert.get(slot_idx) {
                    find_column_kind(insert)
                } else {
                    None
                };
                slot_idx += 1;
                kind
            }
            x => Some(x.clone()),
        };
        if let Some(kind) = kind {
            let column = gen_column(
                &kind,
                &config.docker.path,
                &config.display.separator,
                config.display.abbr_sid,
            );
            if column.available() {
                cols.push(ColumnInfo {
                    column,
                    kind,
                    style: c.style.clone(),
                    nonnumeric_search: c.nonnumeric_search,
                    numeric_search: c.numeric_search,
                    align: c.align.clone(),
                });
            }
        }
    }

    let proc = collect_proc(Duration::from_millis(opt.interval));
    for c in cols.iter_mut() {
        for p in &proc {
            c.column.add(&p);
        }
    }

    // -------------------------------------------------------------------------
    // Search column
    // -------------------------------------------------------------------------

    let mut cols_nonnumeric = Vec::new();
    let mut cols_numeric = Vec::new();
    for c in &cols {
        if c.nonnumeric_search {
            cols_nonnumeric.push(c.column.as_ref());
        }
        if c.numeric_search {
            cols_numeric.push(c.column.as_ref());
        }
    }

    let mut keyword_nonnumeric = Vec::new();
    let mut keyword_numeric = Vec::new();

    for k in &opt.keyword {
        match util::classify(k) {
            KeywordClass::Numeric => keyword_numeric.push(k),
            KeywordClass::NonNumeric => keyword_nonnumeric.push(k),
        }
    }

    let (sort_idx, sort_order) = match (&opt.sorta, &opt.sortd) {
        (Some(sort), _) | (_, Some(sort)) => {
            let mut idx = config.sort.column;
            let mut order = config.sort.order.clone();
            for (i, c) in cols.iter().enumerate() {
                let (kind, _) = KIND_LIST[&c.kind];
                if kind.to_lowercase().find(&sort.to_lowercase()).is_some() {
                    idx = i;
                    order = if opt.sorta.is_some() {
                        ConfigSortOrder::Ascending
                    } else {
                        ConfigSortOrder::Descending
                    };
                    break;
                }
            }
            (idx, order)
        }
        _ => (config.sort.column, config.sort.order.clone()),
    };

    let pids = cols[sort_idx].column.sorted_pid(&sort_order);

    let self_pid = std::process::id() as i32;

    let logic = if opt.and {
        ConfigSearchLogic::And
    } else if opt.or {
        ConfigSearchLogic::Or
    } else if opt.nand {
        ConfigSearchLogic::Nand
    } else if opt.nor {
        ConfigSearchLogic::Nor
    } else {
        config.search.logic.clone()
    };

    let mut visible_pids = Vec::new();
    for pid in pids {
        let visible = if !config.display.show_self && pid == self_pid {
            false
        } else if opt.keyword.is_empty() {
            true
        } else {
            search(
                pid,
                &keyword_numeric,
                &keyword_nonnumeric,
                cols_numeric.as_slice(),
                cols_nonnumeric.as_slice(),
                &config,
                &logic,
            )
        };

        if visible {
            visible_pids.push(pid);
        }
    }

    for (i, ref mut c) in cols.iter_mut().enumerate() {
        let order = if i == sort_idx {
            Some(sort_order.clone())
        } else {
            None
        };
        c.column.reset_max_width(order, &config);
        for pid in &visible_pids {
            c.column.update_max_width(*pid);
        }
    }

    // -------------------------------------------------------------------------
    // Display
    // -------------------------------------------------------------------------

    let term = Term::stdout();
    let (term_h, term_w) = term.size();
    let term_h = term_h as usize;
    let mut term_w = term_w as usize;

    let use_terminal = console::user_attended();

    // +3 means header/unit line and next prompt
    let pager_threshold = visible_pids.len() + 3;

    let use_pager = if cfg!(target_os = "windows") {
        false
    } else {
        match (opt.watch.as_ref(), opt.pager.as_ref(), &config.pager.mode) {
            (Some(_), _, _) => false,
            (None, Some(x), _) if x == "auto" => term_h < pager_threshold,
            (None, Some(x), _) if x == "always" => true,
            (None, Some(x), _) if x == "disable" => false,
            (None, None, ConfigPagerMode::Auto) => term_h < pager_threshold,
            (None, None, ConfigPagerMode::Always) => true,
            (None, None, ConfigPagerMode::Disable) => false,
            _ => false,
        }
    };

    let mut truncate = use_terminal && use_pager && config.display.cut_to_pager;
    truncate |= use_terminal && !use_pager && config.display.cut_to_terminal;
    truncate |= !use_terminal && config.display.cut_to_pipe;

    if !truncate {
        term_w = std::usize::MAX;
    }

    if use_pager {
        pager(&config);
    }

    match (opt.color.as_ref(), &config.display.color_mode) {
        (Some(x), _) if x == "auto" => {
            if use_pager && use_terminal {
                console::set_colors_enabled(true);
            }
        }
        (Some(x), _) if x == "always" => console::set_colors_enabled(true),
        (Some(x), _) if x == "disable" => console::set_colors_enabled(false),
        (None, ConfigColorMode::Auto) => {
            if use_pager && use_terminal {
                console::set_colors_enabled(true);
            }
        }
        (None, ConfigColorMode::Always) => console::set_colors_enabled(true),
        (None, ConfigColorMode::Disable) => console::set_colors_enabled(false),
        _ => (),
    }

    display_header(&term, term_w, &cols, &config, sort_idx, &sort_order);
    display_unit(&term, term_w, &cols, &config);

    for (i, pid) in visible_pids.iter().enumerate() {
        if opt.watch.is_some() && i >= term_h - 5 {
            break;
        }
        display_content(&term, *pid, term_w, &cols, &config);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;

        let args = vec!["procs"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_search() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;

        let args = vec!["procs", "root"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "1"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--or", "root", "1"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--and", "root", "1"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--nor", "root", "1"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--nand", "root", "1"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        config.search.nonnumeric_search = ConfigSearchKind::Exact;
        config.search.numeric_search = ConfigSearchKind::Partial;
        let args = vec!["procs", "root", "1"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_config() {
        let ret = run_config();
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_list() {
        let ret = run_list();
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_without_truncate() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.display.cut_to_terminal = false;

        let args = vec!["procs"];
        let opt = Opt::from_iter(args.iter());
        config.pager.mode = ConfigPagerMode::Disable;
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_insert() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;

        let args = vec!["procs", "--insert", "ppid"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_sort() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;

        let args = vec!["procs", "--sorta", "cpu"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--sortd", "cpu"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_all() {
        let mut config: Config = toml::from_str(CONFIG_ALL).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;

        let _tcp = std::net::TcpListener::bind("127.0.0.1:10000");
        let _udp = std::net::UdpSocket::bind("127.0.0.1:10000");

        let args = vec!["procs"];
        let opt = Opt::from_iter(args.iter());
        let ret = run_default(&opt, &config);
        assert!(ret.is_ok());
    }

}
