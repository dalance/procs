mod column;
mod columns;
mod config;
mod style;
mod util;

use column::Column;
use columns::*;
use config::*;
use console::Term;
use failure::{Error, ResultExt};
use pager::Pager;
use procfs::Process;
use std::fs;
use std::io::Read;
use std::process;
use std::thread;
use std::time::{Duration, Instant};
use structopt::{clap, StructOpt};
use style::{apply_color, apply_style};
use util::KeywordClass;

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
// ColumnInfo
// ---------------------------------------------------------------------------------------------------------------------

struct ColumnInfo {
    column: Box<dyn Column>,
    style: ConfigColumnStyle,
    nonnumeric_search: bool,
    numeric_search: bool,
}

fn gen_column(kind: &ConfigColumnKind, docker_path: &str) -> Box<dyn Column> {
    match kind {
        ConfigColumnKind::Command => Box::new(Command::new()),
        ConfigColumnKind::CpuTime => Box::new(CpuTime::new()),
        ConfigColumnKind::Docker => Box::new(Docker::new(docker_path)),
        ConfigColumnKind::Eip => Box::new(Eip::new()),
        ConfigColumnKind::Esp => Box::new(Esp::new()),
        ConfigColumnKind::MajFlt => Box::new(MajFlt::new()),
        ConfigColumnKind::MinFlt => Box::new(MinFlt::new()),
        ConfigColumnKind::Nice => Box::new(Nice::new()),
        ConfigColumnKind::Pid => Box::new(Pid::new()),
        ConfigColumnKind::Ppid => Box::new(Ppid::new()),
        ConfigColumnKind::Priority => Box::new(Priority::new()),
        ConfigColumnKind::Processor => Box::new(Processor::new()),
        ConfigColumnKind::ReadBytes => Box::new(ReadBytes::new()),
        ConfigColumnKind::RtPriority => Box::new(RtPriority::new()),
        ConfigColumnKind::Separator => Box::new(Separator::new()),
        ConfigColumnKind::StartTime => Box::new(StartTime::new()),
        ConfigColumnKind::State => Box::new(State::new()),
        ConfigColumnKind::TcpPort => Box::new(TcpPort::new()),
        ConfigColumnKind::Threads => Box::new(Threads::new()),
        ConfigColumnKind::Tty => Box::new(Tty::new()),
        ConfigColumnKind::UdpPort => Box::new(UdpPort::new()),
        ConfigColumnKind::UsageCpu => Box::new(UsageCpu::new()),
        ConfigColumnKind::UsageMem => Box::new(UsageMem::new()),
        ConfigColumnKind::Username => Box::new(Username::new()),
        ConfigColumnKind::VmRss => Box::new(VmRss::new()),
        ConfigColumnKind::VmSize => Box::new(VmSize::new()),
        ConfigColumnKind::WriteBytes => Box::new(WriteBytes::new()),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------------------------------------------------

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

fn collect_proc(cols: &mut Vec<ColumnInfo>, opt: &Opt) {
    let mut base_procs = Vec::new();

    for proc in procfs::all_processes() {
        let io = proc.io();
        let time = Instant::now();
        base_procs.push((proc.pid(), proc, io, time));
    }

    thread::sleep(Duration::from_millis(opt.interval));

    for (pid, prev_proc, prev_io, prev_time) in base_procs {
        let curr_proc = if let Ok(proc) = Process::new(pid) {
            proc
        } else {
            prev_proc.clone()
        };
        let curr_io = curr_proc.io();
        let curr_time = Instant::now();
        let interval = curr_time - prev_time;

        for c in cols.iter_mut() {
            c.column
                .add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        }
    }
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

        process::exit(1);
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

    collect_proc(&mut cols, &opt);

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

    let self_pid = process::id() as i32;

    let mut visible_pids = Vec::new();
    for pid in pids {
        let mut visible = true;
        if !opt.keyword.is_empty() {
            visible = match config.search.nonnumeric_search {
                ConfigSearchKind::Partial => {
                    util::find_partial(cols_nonnumeric.as_slice(), pid, &keyword_nonnumeric)
                }
                ConfigSearchKind::Exact => {
                    util::find_exact(cols_nonnumeric.as_slice(), pid, &keyword_nonnumeric)
                }
            };
            visible |= match config.search.numeric_search {
                ConfigSearchKind::Partial => {
                    util::find_partial(cols_numeric.as_slice(), pid, &keyword_numeric)
                }
                ConfigSearchKind::Exact => {
                    util::find_exact(cols_numeric.as_slice(), pid, &keyword_numeric)
                }
            };
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

    let user_attended = console::user_attended();

    if !user_attended {
        term_w = std::u16::MAX;
    }

    let use_pager = match config.pager.mode {
        // +3 means header/unit line and next prompt
        ConfigPagerMode::Auto => term_h < visible_pids.len() as u16 + 3,
        ConfigPagerMode::Always => true,
        ConfigPagerMode::Disable => false,
    };

    if use_pager {
        if let Some(ref pager) = config.pager.command {
            Pager::with_pager(&pager).setup();
        }
        if quale::which("less").is_some() {
            Pager::with_default_pager("less").setup();
        } else {
            Pager::with_default_pager("more -f").setup();
        }
    }

    match opt.color.as_ref() {
        "always" => console::set_colors_enabled(true),
        "disable" => console::set_colors_enabled(false),
        _ => {
            if use_pager && user_attended {
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

    #[test]
    fn test_run_with_nonnumeric() {
        let args = vec!["procs", "root"];
        let opt = Opt::from_iter(args.iter());
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        let ret = run_opt_config(opt, config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_with_numeric() {
        let args = vec!["procs", "1"];
        let opt = Opt::from_iter(args.iter());
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        let ret = run_opt_config(opt, config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_config() {
        let args = vec!["procs", "--config"];
        let opt = Opt::from_iter(args.iter());
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        let ret = run_opt_config(opt, config);
        assert!(ret.is_ok());
    }
}
