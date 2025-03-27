use crate::process::ProcessInfo;
use crate::{column_default, Column};
use dockworker::container::ContainerFilters;
use std::cmp;
use std::collections::HashMap;
use tokio::runtime::Runtime;

pub struct Docker {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    width: usize,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    containers: HashMap<String, String>,
    #[cfg(target_os = "macos")]
    containers: HashMap<i32, String>,
    available: bool,
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Docker {
    pub fn new(header: Option<String>, path: &str) -> Self {
        let header = header.unwrap_or_else(|| String::from("Docker"));
        let unit = String::new();
        let mut containers = HashMap::new();
        let mut available = true;
        if let Ok(docker) = dockworker::Docker::connect_with_unix(path) {
            let rt = Runtime::new().unwrap();
            if let Ok(cont) =
                rt.block_on(docker.list_containers(None, None, None, ContainerFilters::new()))
            {
                for c in cont {
                    // remove the first letter '/' from container name
                    let name = String::from(&c.Names[0][1..]);
                    containers.insert(c.Id, name);
                }
            } else {
                available = false;
            }
        } else {
            available = false;
        }
        Self {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            containers,
            available,
        }
    }
}

#[cfg(target_os = "macos")]
impl Docker {
    pub fn new(header: Option<String>, path: &str) -> Self {
        let header = header.unwrap_or_else(|| String::from("Docker"));
        let unit = String::from("");
        let mut containers = HashMap::new();
        let mut available = true;
        if let Ok(docker) = dockworker::Docker::connect_with_unix(path) {
            let rt = Runtime::new().unwrap();
            if let Ok(cont) =
                rt.block_on(docker.list_containers(None, None, None, ContainerFilters::new()))
            {
                for c in cont {
                    // remove the first letter '/' from container name
                    let name = String::from(&c.Names[0][1..]);
                    if let Ok(processes) = rt.block_on(docker.processes(c.Id.as_str())) {
                        for p in processes {
                            if let Ok(pid) = p.pid.parse::<i32>() {
                                containers.insert(pid, name.clone());
                            }
                        }
                    }
                }
            } else {
                available = false;
            }
        } else {
            available = false;
        }
        Docker {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            width: 0,
            header,
            unit,
            containers,
            available,
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Docker {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(cgroups) = proc.curr_proc.cgroups() {
            let mut ret = String::new();
            for cgroup in cgroups {
                let cgroup_name = cgroup.pathname.clone();
                if cgroup_name.starts_with("/docker") {
                    let container_id = cgroup_name.replace("/docker/", "");
                    if let Some(name) = self.containers.get(&container_id) {
                        ret = name.to_string();
                        break;
                    } else {
                        ret = String::from("?");
                        break;
                    }
                } else if cgroup_name.starts_with("/system.slice/docker-") {
                    let container_id = cgroup_name
                        .replace("/system.slice/docker-", "")
                        .replace(".scope", "");
                    if let Some(name) = self.containers.get(&container_id) {
                        ret = name.to_string();
                        break;
                    } else {
                        ret = String::from("?");
                        break;
                    }
                }
            }
            ret
        } else {
            String::new()
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    fn available(&self) -> bool {
        self.available
    }

    column_default!(String, false);
}

#[cfg(target_os = "macos")]
impl Column for Docker {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Some(name) = self.containers.get(&proc.pid) {
            name.to_string()
        } else {
            String::from("")
        };
        let raw_content = fmt_content.clone();

        self.fmt_contents.insert(proc.pid, fmt_content);
        self.raw_contents.insert(proc.pid, raw_content);
    }

    fn available(&self) -> bool {
        self.available
    }

    column_default!(String, false);
}
