#![cfg(any(target_os = "linux", target_os = "android"))]

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("procs-low-fd-{}-{unique}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_stat(path: &Path, pid: i32, ppid: i32, kernel_thread: bool) {
    let mut fields = vec!["0".to_string(); 49];
    fields[0] = ppid.to_string(); // ppid, field 4
    if kernel_thread {
        fields[5] = "2097152".to_string(); // PF_KTHREAD, field 9
    }
    fields[16] = "2".to_string(); // num_threads, field 20
    fields[18] = "1".to_string(); // starttime, field 22
    fs::write(path, format!("{pid} (comm-{pid}) S {}\n", fields.join(" "))).unwrap();
}

fn run_procs(root: &Path, args: &[&str], fd_limit: libc::rlim_t) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_procs"));
    command
        .arg("--load-config")
        .arg(root.join("config.toml"))
        .arg("--procfs")
        .arg(root.join("proc"))
        .args([
            "--color",
            "disable",
            "--theme",
            "dark",
            "--pager",
            "disable",
            "--no-header",
            "--interval",
            "0",
        ])
        .args(args);

    // Only the child running procs gets a lower limit. The pre-exec callback
    // uses resource-limit syscalls without allocation or locks after fork.
    unsafe {
        command.pre_exec(move || {
            let mut limit = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) != 0 {
                return Err(io::Error::last_os_error());
            }
            limit.rlim_cur = fd_limit.min(limit.rlim_max);
            if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn lists_all_processes_with_a_low_file_descriptor_limit() {
    let fixture = Fixture::new();
    let procfs = fixture.0.join("proc");
    fs::create_dir(&procfs).unwrap();
    let pids = 900001..900097;
    for pid in pids.clone() {
        let process = procfs.join(pid.to_string());
        fs::create_dir(&process).unwrap();
        let ppid = if pid == pids.start { 1 } else { pids.start };
        write_stat(&process.join("stat"), pid, ppid, pid >= 900089);
        fs::write(
            process.join("cmdline"),
            format!("fixture-{pid}\0--from-cmdline\0"),
        )
        .unwrap();
        for tid in [pid, pid + 1000000] {
            let task = process.join("task").join(tid.to_string());
            fs::create_dir_all(&task).unwrap();
            write_stat(&task.join("stat"), tid, ppid, false);
        }
    }

    for (args, with_threads, show_kthreads) in [
        (vec![], false, true),
        (vec!["--thread"], true, true),
        (vec!["--tree"], true, true),
        (vec!["--thread"], true, false),
    ] {
        fs::write(
            fixture.0.join("config.toml"),
            format!(
                r#"[[columns]]
kind = "Pid"
[[columns]]
kind = "TreeSlot"
[[columns]]
kind = "Command"
[display]
show_self = true
show_self_parents = true
show_kthreads = {show_kthreads}
"#,
            ),
        )
        .unwrap();
        let expected: BTreeSet<_> = pids
            .clone()
            .filter(|&pid| show_kthreads || pid < 900089)
            .flat_map(|pid| {
                let mut ids = vec![pid];
                if with_threads {
                    ids.push(pid + 1000000);
                }
                ids
            })
            .collect();
        let control = run_procs(&fixture.0, &args, 1024);
        let output = run_procs(&fixture.0, &args, 32);
        let actual: BTreeSet<i32> = output
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .find_map(|word| word.trim_matches(['[', ']']).parse().ok())
                    .unwrap_or_else(|| panic!("no PID in {line:?}, args={args:?}"))
            })
            .collect();
        assert_eq!(
            actual, expected,
            "args={args:?}, show_kthreads={show_kthreads}"
        );
        for pid in &expected {
            let command = if pids.contains(pid) {
                format!("fixture-{pid} --from-cmdline")
            } else {
                format!("comm-{pid}")
            };
            assert!(
                output.contains(&command),
                "missing command for {pid}, args={args:?}"
            );
        }
        assert_eq!(
            output, control,
            "args={args:?}, show_kthreads={show_kthreads}"
        );
    }
}
