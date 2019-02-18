mod column;
mod columns;
mod config;
mod process;
mod style;
mod util;

use crate::column::Column;
use crate::config::*;
use crate::process::collect_proc;
use crate::style::{apply_color, apply_style};
use crate::util::KeywordClass;
use console::Term;
use failure::{Error, ResultExt};
use pager::Pager;
use std::fs;
use std::io::Read;
use std::time::Duration;
use structopt::{clap, StructOpt};

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
    /// Keyword for search
    #[structopt(name = "KEYWORD")]
    pub keyword: Vec<String>,

    /// Color mode
    #[structopt(
        short = "c",
        long = "color",
        default_value = "auto",
        possible_value = "auto",
        possible_value = "always",
        possible_value = "disable"
    )]
    pub color: String,

    /// Interval to calculate throughput
    #[structopt(long = "interval", default_value = "100", value_name = "ms")]
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
    let cfg_path = match dirs::home_dir() {
        Some(mut path) => {
            path.push(".procs.toml");
            if path.exists() {
                Some(path)
            } else {
                None
            }
        }
        None => None,
    };

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

fn display_header(term: &Term, max_width: usize, cols: &[ColumnInfo], config: &Config) {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_color(c.column.display_header(), &config.style.header)
        );
    }
    row = row.trim_end().to_string();
    row = console::truncate_str(&row, max_width, "").to_string();
    let _ = term.write_line(&row);
}

fn display_unit(term: &Term, max_width: usize, cols: &[ColumnInfo], config: &Config) {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_color(c.column.display_unit(), &config.style.unit)
        );
    }
    row = row.trim_end().to_string();
    row = console::truncate_str(&row, max_width, "").to_string();
    let _ = term.write_line(&row);
}

fn display_content(term: &Term, pid: i32, max_width: usize, cols: &[ColumnInfo], config: &Config) {
    let mut row = String::from("");
    for c in cols.iter() {
        row = format!(
            "{} {}",
            row,
            apply_style(
                c.column.display_content(pid).unwrap(),
                &c.style,
                &config.style
            )
        );
    }
    row = row.trim_end().to_string();
    row = console::truncate_str(&row, max_width, "").to_string();
    let _ = term.write_line(&row);
}

fn search<T: AsRef<str>>(
    pid: i32,
    keyword_numeric: &[T],
    keyword_nonnumeric: &[T],
    cols_numeric: &[&Column],
    cols_nonnumeric: &[&Column],
    config: &Config,
) -> bool {
    let mut ret = match config.search.nonnumeric_search {
        ConfigSearchKind::Partial => util::find_partial(cols_nonnumeric, pid, keyword_nonnumeric),
        ConfigSearchKind::Exact => util::find_exact(cols_nonnumeric, pid, keyword_nonnumeric),
    };
    ret |= match config.search.numeric_search {
        ConfigSearchKind::Partial => util::find_partial(cols_numeric, pid, keyword_numeric),
        ConfigSearchKind::Exact => util::find_exact(cols_numeric, pid, keyword_numeric),
    };
    ret
}

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
        let config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        let toml = toml::to_string(&config)?;
        println!("{}", toml);
        return Ok(());
    }

    let config = get_config()?;

    run_opt_config(opt, config)
}

fn run_opt_config(opt: Opt, config: Config) -> Result<(), Error> {
    // -------------------------------------------------------------------------
    // Generate column
    // -------------------------------------------------------------------------

    let mut cols = Vec::new();
    for c in &config.columns {
        let column = gen_column(&c.kind, &config.docker.path);
        if column.available() {
            cols.push(ColumnInfo {
                column,
                style: c.style.clone(),
                nonnumeric_search: c.nonnumeric_search,
                numeric_search: c.numeric_search,
            });
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

    let pids = cols[config.sort.column]
        .column
        .sorted_pid(&config.sort.order);

    let self_pid = std::process::id() as i32;

    let mut visible_pids = Vec::new();
    for pid in pids {
        let mut visible = true;
        if !opt.keyword.is_empty() {
            visible = search(
                pid,
                &keyword_numeric,
                &keyword_nonnumeric,
                cols_numeric.as_slice(),
                cols_nonnumeric.as_slice(),
                &config,
            );
        }

        if !config.display.show_self && pid == self_pid {
            visible = false;
        }

        if visible {
            visible_pids.push(pid);
        }
    }

    for pid in &visible_pids {
        for c in &mut cols {
            c.column.update_max_width(*pid);
        }
    }

    // -------------------------------------------------------------------------
    // Display
    // -------------------------------------------------------------------------

    let term = Term::stdout();
    let (term_h, mut term_w) = term.size();

    let use_terminal = console::user_attended();

    let use_pager = match config.pager.mode {
        // +3 means header/unit line and next prompt
        ConfigPagerMode::Auto => term_h < visible_pids.len() as u16 + 3,
        ConfigPagerMode::Always => true,
        ConfigPagerMode::Disable => false,
    };

    let mut truncate = use_terminal && use_pager && config.display.cut_to_pager;
    truncate |= use_terminal && !use_pager && config.display.cut_to_terminal;
    truncate |= !use_terminal && config.display.cut_to_pipe;

    if !truncate {
        term_w = std::u16::MAX;
    }

    if use_pager {
        if let Some(ref pager) = config.pager.command {
            Pager::with_pager(&pager).setup();
        }
        if quale::which("less").is_some() {
            Pager::with_pager("less -SR").setup();
        } else {
            Pager::with_pager("more -f").setup();
        }
    }

    match opt.color.as_ref() {
        "always" => console::set_colors_enabled(true),
        "disable" => console::set_colors_enabled(false),
        _ => {
            if use_pager && use_terminal {
                console::set_colors_enabled(true);
            }
        }
    }

    display_header(&term, term_w as usize, &cols, &config);
    display_unit(&term, term_w as usize, &cols, &config);

    for pid in &visible_pids {
        display_content(&term, *pid, term_w as usize, &cols, &config);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let args = vec!["procs"];
        let opt = Opt::from_iter(args.iter());
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        let ret = run_opt_config(opt, config);
        assert!(ret.is_ok());
    }

    //#[test]
    //fn test_run_with_nonnumeric() {
    //    let args = vec!["procs", "root"];
    //    let opt = Opt::from_iter(args.iter());
    //    let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
    //    config.pager.mode = ConfigPagerMode::Disable;
    //    let ret = run_opt_config(opt, config);
    //    assert!(ret.is_ok());
    //}

    //#[test]
    //fn test_run_with_numeric() {
    //    let args = vec!["procs", "1"];
    //    let opt = Opt::from_iter(args.iter());
    //    let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
    //    config.pager.mode = ConfigPagerMode::Disable;
    //    let ret = run_opt_config(opt, config);
    //    assert!(ret.is_ok());
    //}

    //#[test]
    //fn test_run_config() {
    //    let args = vec!["procs", "--config"];
    //    let opt = Opt::from_iter(args.iter());
    //    let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
    //    config.pager.mode = ConfigPagerMode::Disable;
    //    let ret = run_opt_config(opt, config);
    //    assert!(ret.is_ok());
    //}

    #[cfg(target_os = "linux")]
    pub static CONFIG_ALL: &'static str = r#"
[[columns]]
kind = "Command"
style = "BrightRed"
[[columns]]
kind = "CpuTime"
style = "BrightGreen"
[[columns]]
kind = "Eip"
style = "BrightYellow"
[[columns]]
kind = "Esp"
style = "BrightBlue"
[[columns]]
kind = "Docker"
style = "BrightMagenta"
[[columns]]
kind = "MajFlt"
style = "BrightCyan"
[[columns]]
kind = "MinFlt"
style = "BrightWhite"
[[columns]]
kind = "Nice"
style = "Red"
[[columns]]
kind = "Pid"
style = "Green"
[[columns]]
kind = "Ppid"
style = "Yellow"
[[columns]]
kind = "Priority"
style = "Blue"
[[columns]]
kind = "Processor"
style = "Magenta"
[[columns]]
kind = "ReadBytes"
style = "Cyan"
[[columns]]
kind = "RtPriority"
style = "White"
[[columns]]
kind = "Separator"
style = "White"
[[columns]]
kind = "StartTime"
style = "White"
[[columns]]
kind = "State"
style = "White"
[[columns]]
kind = "TcpPort"
style = "White"
[[columns]]
kind = "Threads"
style = "White"
[[columns]]
kind = "Tty"
style = "White"
[[columns]]
kind = "UdpPort"
style = "White"
[[columns]]
kind = "UsageCpu"
style = "White"
[[columns]]
kind = "UsageMem"
style = "White"
[[columns]]
kind = "Username"
style = "White"
[[columns]]
kind = "VmData"
style = "ByUnit"
[[columns]]
kind = "VmExe"
style = "ByUnit"
[[columns]]
kind = "VmHwm"
style = "ByUnit"
[[columns]]
kind = "VmLib"
style = "ByUnit"
[[columns]]
kind = "VmLock"
style = "ByUnit"
[[columns]]
kind = "VmPeak"
style = "ByUnit"
[[columns]]
kind = "VmPin"
style = "ByUnit"
[[columns]]
kind = "VmPte"
style = "ByUnit"
[[columns]]
kind = "VmRss"
style = "ByUnit"
[[columns]]
kind = "VmSize"
style = "ByUnit"
[[columns]]
kind = "VmStack"
style = "ByUnit"
[[columns]]
kind = "VmSwap"
style = "ByUnit"
[[columns]]
kind = "Wchan"
style = "White"
[[columns]]
kind = "WriteBytes"
style = "White"
"#;

    #[cfg(target_os = "macos")]
    pub static CONFIG_ALL: &'static str = r#"
[[columns]]
kind = "Pid"
style = "Green"
[[columns]]
kind = "Username"
style = "White"
"#;

    //#[test]
    //fn test_run_all() {
    //    let args = vec!["procs"];
    //    let opt = Opt::from_iter(args.iter());
    //    let mut config: Config = toml::from_str(CONFIG_ALL).unwrap();
    //    config.pager.mode = ConfigPagerMode::Disable;
    //    let ret = run_opt_config(opt, config);
    //    assert!(ret.is_ok());
    //}

}
