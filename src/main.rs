mod column;
mod columns;
mod config;
mod opt;
mod process;
mod style;
mod term_info;
mod util;
mod view;
mod watcher;

use crate::column::Column;
use crate::columns::*;
use crate::config::*;
use crate::opt::*;
use crate::util::{adjust, get_theme, lap};
use crate::view::View;
use crate::watcher::Watcher;
use anyhow::{Context, Error};
use clap::{CommandFactory, Parser};
use console::Term;
use std::cmp;
use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Read};
use std::path::PathBuf;
use std::time::Instant;
use unicode_width::UnicodeWidthStr;

// ---------------------------------------------------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------------------------------------------------

fn get_config(opt: &Opt) -> Result<Config, Error> {
    let dot_cfg_path = directories::BaseDirs::new()
        .map(|base| base.home_dir().join(".procs.toml"))
        .filter(|path| path.exists());
    let app_cfg_path = directories::ProjectDirs::from("com.github", "dalance", "procs")
        .map(|proj| proj.preference_dir().join("config.toml"))
        .filter(|path| path.exists());
    let xdg_cfg_path = directories::BaseDirs::new()
        .map(|base| {
            base.home_dir()
                .join(".config")
                .join("procs")
                .join("config.toml")
        })
        .filter(|path| path.exists());
    let etc_path = PathBuf::from("/etc/procs/procs.toml");
    let etc_cfg_path = etc_path.exists().then_some(etc_path);
    let cfg_path = opt
        .load_config
        .clone()
        .or(dot_cfg_path)
        .or(app_cfg_path)
        .or(xdg_cfg_path)
        .or(etc_cfg_path);

    let config: Config = if let Some(path) = cfg_path {
        let mut f = fs::File::open(&path).context(format!("failed to open file ({path:?})"))?;
        let mut s = String::new();
        f.read_to_string(&mut s)
            .context(format!("failed to read file ({path:?})"))?;
        let c = toml::from_str(&s);
        check_old_config(&s, c).context(format!("failed to parse toml ({path:?})"))?
    } else {
        toml::from_str(CONFIG_DEFAULT).unwrap()
    };

    match opt.use_config {
        Some(BuiltinConfig::Default) => Ok(toml::from_str(CONFIG_DEFAULT).unwrap()),
        Some(BuiltinConfig::Large) => Ok(toml::from_str(CONFIG_LARGE).unwrap()),
        None => Ok(config),
    }
}

fn check_old_config(s: &str, config: Result<Config, toml::de::Error>) -> Result<Config, Error> {
    match config {
        Ok(x) => Ok(x),
        Err(x) => {
            if s.contains("Color256") {
                let err: Error = x.into();
                let err = err.context("\"Color256\" keyword for 8bit color is obsolete. Please see https://github.com/dalance/procs#color-list");
                Err(err)
            } else {
                Err(x.into())
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------------------------------------------

fn main() {
    let err = Term::stderr();

    if let Err(x) = run() {
        let mut cause = x.chain();
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

fn run() -> Result<(), Error> {
    let mut opt: Opt = Parser::parse();
    opt.watch_mode = opt.watch || opt.watch_interval.is_some();

    if opt.gen_config {
        run_gen_config()
    } else if opt.list {
        run_list();
        Ok(())
    } else if let Some(shell) = opt.gen_completion {
        gen_completion(shell, "./")
    } else if let Some(shell) = opt.gen_completion_out {
        //Opt::clap().gen_completions_to("procs", shell, &mut stdout());
        clap_complete::generate(shell, &mut Opt::command(), "procs", &mut stdout());
        Ok(())
    } else {
        let config = get_config(&opt)?;
        if opt.watch_mode {
            let interval = match opt.watch_interval {
                Some(n) => (n * 1000.0).round() as u64,
                None => 1000,
            };
            run_watch(&mut opt, &config, interval)
        } else {
            run_default(&mut opt, &config)
        }
    }
}

fn run_gen_config() -> Result<(), Error> {
    let config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
    let toml = toml::to_string(&config)?;
    println!("{toml}");
    Ok(())
}

fn run_list() {
    let mut width = 0;
    let mut list = Vec::new();
    let mut desc = HashMap::new();
    for (_, (v, d)) in KIND_LIST.iter() {
        list.push(v);
        desc.insert(v, d);
        width = cmp::max(width, UnicodeWidthStr::width(*v));
    }

    println!("Column kind list:");
    for l in list {
        println!(
            "  {}: {}",
            adjust(l, width, &ConfigColumnAlign::Left),
            desc[l]
        );
    }
}

fn run_watch(opt: &mut Opt, config: &Config, interval: u64) -> Result<(), Error> {
    Watcher::start(opt, config, interval)
}

fn run_default(opt: &mut Opt, config: &Config) -> Result<(), Error> {
    let mut time = Instant::now();

    let theme = get_theme(opt, config);

    let mut view = View::new(opt, config, false)?;

    if opt.debug {
        lap(&mut time, "Info: View::new");
    }

    view.filter(opt, config, 1);

    if opt.debug {
        lap(&mut time, "Info: view.filter");
    }

    view.adjust(config, &HashMap::new());

    if opt.debug {
        lap(&mut time, "Info: view.adjust");
    }

    view.display(opt, config, &theme)?;

    if opt.debug {
        lap(&mut time, "Info: view.display");
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
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_search() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs", "root"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "1"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--or", "root", "1"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--and", "root", "1"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--nor", "root", "1"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--nand", "root", "1"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        config.search.nonnumeric_search = ConfigSearchKind::Exact;
        config.search.numeric_search = ConfigSearchKind::Partial;
        let args = vec!["procs", "root", "1"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_gen_config() {
        let ret = run_gen_config();
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_without_truncate() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.display.cut_to_terminal = false;
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs"];
        let mut opt = Opt::parse_from(args.iter());
        config.pager.mode = ConfigPagerMode::Disable;
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_insert() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs", "--insert", "ppid"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_sort() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs", "--sorta", "cpu"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());

        let args = vec!["procs", "--sortd", "cpu"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_tree() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs", "--tree"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_all() {
        let mut config: Config = toml::from_str(CONFIG_ALL).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        config.display.theme = ConfigTheme::Dark;

        let _tcp = std::net::TcpListener::bind("127.0.0.1:10000");
        let _udp = std::net::UdpSocket::bind("127.0.0.1:10000");

        let args = vec!["procs"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }

    #[test]
    fn test_run_use_config() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        config.pager.mode = ConfigPagerMode::Disable;
        config.display.theme = ConfigTheme::Dark;

        let args = vec!["procs", "--use-config", "large"];
        let mut opt = Opt::parse_from(args.iter());
        let ret = run_default(&mut opt, &config);
        assert!(ret.is_ok());
    }
}
