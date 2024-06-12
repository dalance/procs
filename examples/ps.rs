use procs::process::collect_proc;
use std::time::Duration;
use libproc::proc_pid;

fn main() {
    let mut procs = collect_proc(Duration::from_millis(1), true, true);
    procs.sort_by(|a, b| a.ppid.cmp(&b.ppid));

    for proc in procs {
        println!("{}\t{}\t{}\t{}", proc.ppid, proc_pid::name(proc.ppid).unwrap(), proc.pid, proc_pid::name(proc.pid).unwrap());
    }
}
