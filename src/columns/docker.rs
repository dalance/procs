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

/// Extract a Docker container ID from a cgroup path.
///
/// The container's cgroup can be nested at an arbitrary depth: rootful Docker
/// puts it directly under `/system.slice`, while rootless Docker nests it under
/// the invoking user's slice. Matching a path *component* rather than a prefix
/// covers both without enumerating every hierarchy shape.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn container_id_from_cgroup(cgroup_path: &str) -> Option<&str> {
    fn is_container_id(s: &str) -> bool {
        s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
    }

    let mut components = cgroup_path.split('/').filter(|x| !x.is_empty()).peekable();
    while let Some(component) = components.next() {
        // cgroup v1: `.../docker/<id>`
        if component == "docker" {
            if let Some(id) = components.peek().copied().filter(|x| is_container_id(x)) {
                return Some(id);
            }
            continue;
        }
        // cgroup v2 with systemd: `.../docker-<id>.scope`
        if let Some(id) = component
            .strip_prefix("docker-")
            .and_then(|x| x.strip_suffix(".scope"))
            .filter(|x| is_container_id(x))
        {
            return Some(id);
        }
    }
    None
}

#[cfg(any(target_os = "linux", target_os = "android"))]
impl Column for Docker {
    fn add(&mut self, proc: &ProcessInfo) {
        let fmt_content = if let Ok(cgroups) = proc.curr_proc.cgroups() {
            let mut ret = String::new();
            for cgroup in cgroups {
                if let Some(container_id) = container_id_from_cgroup(&cgroup.pathname) {
                    ret = match self.containers.get(container_id) {
                        Some(name) => name.to_string(),
                        None => String::from("?"),
                    };
                    break;
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

#[cfg(all(test, any(target_os = "linux", target_os = "android")))]
mod tests {
    use super::container_id_from_cgroup;

    const ID: &str = "b9fc2a3d3e1c4f5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f809a1b2c3";

    #[test]
    fn cgroup_v1() {
        let path = format!("/docker/{ID}");
        assert_eq!(container_id_from_cgroup(&path), Some(ID));
    }

    #[test]
    fn cgroup_v2_rootful() {
        let path = format!("/system.slice/docker-{ID}.scope");
        assert_eq!(container_id_from_cgroup(&path), Some(ID));
    }

    #[test]
    fn cgroup_v2_rootless() {
        let path = format!(
            "/user.slice/user-1000.slice/user@1000.service/user.slice/docker-{ID}.scope"
        );
        assert_eq!(container_id_from_cgroup(&path), Some(ID));
    }

    #[test]
    fn non_container_cgroups() {
        for path in [
            "/",
            "/init.scope",
            "/system.slice/docker.service",
            "/system.slice/containerd.service",
            "/user.slice/user-1000.slice/user@1000.service/app.slice/docker-desktop.scope",
            "/docker/not-a-container-id",
        ] {
            assert_eq!(container_id_from_cgroup(path), None, "path: {path}");
        }
    }

    #[test]
    fn id_must_be_a_whole_component() {
        let path = format!("/system.slice/prefix-docker-{ID}.scope");
        assert_eq!(container_id_from_cgroup(&path), None);
    }
}
