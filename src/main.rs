mod column;
mod columns;

use column::Column;
use columns::*;
use console::{Style, StyledObject, Term};
use lazy_static::lazy_static;
use procfs::Process;
use std::thread;
use std::time::{Duration, Instant};
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
pub struct Opt {}

// ---------------------------------------------------------------------------------------------------------------------
// Style
// ---------------------------------------------------------------------------------------------------------------------

lazy_static! {
    static ref red: Style = Style::new().red().bold();
    static ref green: Style = Style::new().green().bold();
    static ref yellow: Style = Style::new().yellow().bold();
    static ref blue: Style = Style::new().blue().bold();
    static ref magenta: Style = Style::new().magenta().bold();
    static ref cyan: Style = Style::new().cyan().bold();
    static ref white: Style = Style::new().white().bold();
}

fn style_state(x: String) -> StyledObject<String> {
    match x {
        ref x if x.starts_with('D') => red.apply_to(x.to_string()),
        ref x if x.starts_with('R') => green.apply_to(x.to_string()),
        ref x if x.starts_with('S') => blue.apply_to(x.to_string()),
        ref x if x.starts_with('T') => cyan.apply_to(x.to_string()),
        ref x if x.starts_with('Z') => magenta.apply_to(x.to_string()),
        _ => white.apply_to(x),
    }
}

fn style_bytes(x: String) -> StyledObject<String> {
    match x {
        ref x if x.contains('K') => blue.apply_to(x.to_string()),
        ref x if x.contains('M') => green.apply_to(x.to_string()),
        ref x if x.contains('G') => yellow.apply_to(x.to_string()),
        ref x if x.contains('T') => red.apply_to(x.to_string()),
        ref x if x.contains('P') => red.apply_to(x.to_string()),
        _ => blue.apply_to(x),
    }
}

// ---------------------------------------------------------------------------------------------------------------------
// Static
// ---------------------------------------------------------------------------------------------------------------------

static INTERVAL_BY_MS: u64 = 100;

// ---------------------------------------------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------------------------------------------

fn main() {
    let _opt = Opt::from_args();

    let term = Term::stdout();
    let (_term_h, term_w) = term.size();

    let mut col_username = Username::new();
    let mut col_pid = Pid::new();
    let mut col_vmsize = VmSize::new();
    let mut col_vmrss = VmRSS::new();
    let mut col_state = State::new();
    let mut col_command = Command::new();
    let mut col_starttime = StartTime::new();
    let mut col_tcpport = TcpPort::new();
    let mut col_readbytes = ReadBytes::new();
    let mut col_writebytes = WriteBytes::new();

    let mut base_procs = Vec::new();

    for proc in procfs::all_processes() {
        let io = proc.io();
        let time = Instant::now();
        base_procs.push((proc.pid(), proc, io, time));
    }

    thread::sleep(Duration::from_millis(INTERVAL_BY_MS));

    let mut pids = Vec::new();
    for (pid, prev_proc, prev_io, prev_time) in base_procs {
        let curr_proc = if let Ok(proc) = Process::new(pid) {
            proc
        } else {
            prev_proc.clone()
        };
        let curr_io = curr_proc.io();
        let curr_time = Instant::now();
        let interval = curr_time - prev_time;

        let _ = col_username.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_pid.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_vmsize.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_vmrss.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_state.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_command.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_starttime.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_tcpport.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_readbytes.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        let _ = col_writebytes.add(&curr_proc, &prev_proc, &curr_io, &prev_io, &interval);
        pids.push(pid);
    }

    let row = format!(
        "{} {} {} {} {} {} {} {} {} {}",
        white.apply_to(col_pid.display_header()),
        white.apply_to(col_username.display_header()),
        white.apply_to(col_vmsize.display_header()),
        white.apply_to(col_vmrss.display_header()),
        white.apply_to(col_state.display_header()),
        white.apply_to(col_starttime.display_header()),
        white.apply_to(col_tcpport.display_header()),
        white.apply_to(col_readbytes.display_header()),
        white.apply_to(col_writebytes.display_header()),
        white.apply_to(col_command.display_header()),
        //stat.utime,
    );
    let row = console::truncate_str(&row, term_w as usize, "");
    println!("{}", row);

    for pid in pids {
        let row = format!(
            "{} {} {} {} {} {} {} {} {} {}",
            yellow.apply_to(col_pid.display(pid).unwrap()),
            green.apply_to(col_username.display(pid).unwrap()),
            style_bytes(col_vmsize.display(pid).unwrap()),
            style_bytes(col_vmrss.display(pid).unwrap()),
            style_state(col_state.display(pid).unwrap()),
            magenta.apply_to(col_starttime.display(pid).unwrap()),
            cyan.apply_to(col_tcpport.display(pid).unwrap()),
            style_bytes(col_readbytes.display(pid).unwrap()),
            style_bytes(col_writebytes.display(pid).unwrap()),
            white.apply_to(col_command.display(pid).unwrap()),
            //stat.utime,
        );
        let row = console::truncate_str(&row, term_w as usize, "");
        println!("{}", row);
    }
}
