use anyhow::{anyhow, Error};
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::CommandFactory;
use clap::{Parser, ValueEnum};
use clap_complete::Shell;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------------------------------------------------
// Opt
// ---------------------------------------------------------------------------------------------------------------------

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ArgColorMode {
    Auto,
    Always,
    Disable,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ArgThemeMode {
    Auto,
    Dark,
    Light,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ArgPagerMode {
    Auto,
    Always,
    Disable,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum BuiltinConfig {
    Default,
    Large,
}

#[derive(Debug, Parser)]
#[clap(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[clap(
    styles(Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Cyan.on_default())
    )
)]
/// A modern replacement for ps
///
/// please see https://github.com/dalance/procs#configuration to configure columns
pub struct Opt {
    /// Keywords for search
    #[clap(action, name = "KEYWORD")]
    pub keyword: Vec<String>,

    /// AND  logic for multi-keyword
    #[clap(
        short = 'a',
        long = "and",
        conflicts_with_all(&["or", "nand", "nor"])
    )]
    pub and: bool,

    /// OR   logic for multi-keyword
    #[clap(
        short = 'o',
        long = "or",
        conflicts_with_all(&["and", "nand", "nor"])
    )]
    pub or: bool,

    /// NAND logic for multi-keyword
    #[clap(
        short = 'd',
        long = "nand",
        conflicts_with_all(&["and", "or", "nor"])
    )]
    pub nand: bool,

    /// NOR  logic for multi-keyword
    #[clap(
        short = 'r',
        long = "nor",
        conflicts_with_all(&["and", "or", "nand"])
    )]
    pub nor: bool,

    /// Show list of kind
    #[clap(short = 'l', long = "list")]
    pub list: bool,

    /// Show thread
    #[clap(long = "thread")]
    pub thread: bool,

    /// Tree view
    #[clap(short = 't', long = "tree")]
    pub tree: bool,

    /// Watch mode with default interval (1s)
    #[clap(short = 'w', long = "watch")]
    pub watch: bool,

    /// Watch mode with custom interval
    #[clap(short = 'W', long = "watch-interval", value_name = "second")]
    pub watch_interval: Option<f64>,

    #[clap(skip)]
    pub watch_mode: bool,

    /// Insert column to slot
    #[clap(value_name = "kind", short = 'i', long = "insert", number_of_values(1))]
    pub insert: Vec<String>,

    /// Specified column only
    #[clap(value_name = "kind", long = "only")]
    pub only: Option<String>,

    /// Sort column by ascending
    #[clap(
        value_name = "kind",
        long = "sorta",
        conflicts_with_all(&["sortd", "tree"])
    )]
    pub sorta: Option<String>,

    /// Sort column by descending
    #[clap(
        value_name = "kind",
        long = "sortd",
        conflicts_with_all(&["sorta", "tree"])
    )]
    pub sortd: Option<String>,

    /// Color mode
    #[clap(short = 'c', long = "color")]
    pub color: Option<ArgColorMode>,

    /// Theme mode
    #[clap(long = "theme")]
    pub theme: Option<ArgThemeMode>,

    /// Pager mode
    #[clap(short = 'p', long = "pager")]
    pub pager: Option<ArgPagerMode>,

    /// JSON output
    #[clap(long = "json")]
    pub json: bool,

    /// Interval to calculate throughput
    #[clap(long = "interval", default_value = "100", value_name = "millisec")]
    pub interval: u64,

    /// Use built-in configuration
    #[clap(long = "use-config", value_name = "name")]
    pub use_config: Option<BuiltinConfig>,

    /// Load configuration from file
    #[clap(long = "load-config", value_name = "path")]
    pub load_config: Option<PathBuf>,

    /// Generate configuration sample file
    #[clap(long = "gen-config")]
    pub gen_config: bool,

    /// Generate shell completion file
    #[clap(long = "gen-completion", value_name = "shell")]
    pub gen_completion: Option<Shell>,

    /// Generate shell completion file and write to stdout
    #[clap(long = "gen-completion-out", value_name = "shell")]
    pub gen_completion_out: Option<Shell>,

    /// Suppress header
    #[clap(long = "no-header")]
    pub no_header: bool,

    /// Path to procfs
    #[clap(long = "procfs")]
    pub procfs: Option<PathBuf>,

    /// Show debug message
    #[clap(long = "debug", hide = true)]
    pub debug: bool,
}

pub fn gen_completion<P: AsRef<Path>>(shell: Shell, path: P) -> Result<(), Error> {
    let path_str = path.as_ref().as_os_str();
    //Opt::clap().gen_completions("procs", shell, path_str);
    clap_complete::generate_to(shell, &mut Opt::command(), "procs", path_str)?;
    let filename = match shell {
        Shell::Bash => "procs.bash",
        Shell::Elvish => "procs.elv",
        Shell::Fish => "procs.fish",
        Shell::PowerShell => "_procs.ps1",
        Shell::Zsh => "_procs",
        x => return Err(anyhow!("unknown shell type: {}", x)),
    };
    let path = path.as_ref().join(filename);
    println!("completion file is generated: {}", path.display());
    Ok(())
}
