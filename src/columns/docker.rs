use crate::process::ProcessInfo;
use crate::{column_default, Column};
use dockworker::container::ContainerFilters;
use std::cmp;
use std::collections::HashMap;

pub struct Docker {
    header: String,
    unit: String,
    fmt_contents: HashMap<i32, String>,
    raw_contents: HashMap<i32, String>,
    max_width: usize,
    containers: HashMap<String, String>,
    available: bool,
}

impl Docker {
    pub fn new(path: &str) -> Self {
        let header = String::from("Docker");
        let unit = String::from("");
        let mut containers = HashMap::new();
        let mut available = true;
        if let Ok(docker) = dockworker::Docker::connect_with_unix(path) {
            if let Ok(cont) = docker.list_containers(None, None, None, ContainerFilters::new()) {
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
        Docker {
            fmt_contents: HashMap::new(),
            raw_contents: HashMap::new(),
            max_width: cmp::max(header.len(), unit.len()),
            header,
            unit,
            containers,
            available,
        }
    }
}

impl Column for Docker {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(cgroups) = proc.curr_proc.cgroups() {
            let cgroup_name = cgroups[0].pathname.clone();
            if cgroup_name.starts_with("/docker") {
                let container_id = cgroup_name.replace("/docker/", "");
                if let Some(name) = self.containers.get(&container_id) {
                    name.to_string()
                } else {
                    String::from("?")
                }
            } else {
                String::from("")
            }
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

    column_default!(String);
}
