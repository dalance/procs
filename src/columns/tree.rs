use crate::Column;
use crate::process::{ProcessInfo, row_sort_key};
use std::cmp;
use std::collections::HashMap;

pub struct Tree {
    header: String,
    unit: String,
    width: usize,
    tree: HashMap<i64, Vec<i64>>,
    rev_tree: HashMap<i64, i64>,
    symbols: [String; 5],
}

impl Tree {
    pub fn new(symbols: &[String; 5]) -> Self {
        let header = String::new();
        let unit = String::new();
        Self {
            width: 0,
            header,
            unit,
            tree: HashMap::new(),
            rev_tree: HashMap::new(),
            symbols: symbols.clone(),
        }
    }
}

impl Column for Tree {
    fn add(&mut self, proc: &ProcessInfo) {
        if let Some(node) = self.tree.get_mut(&proc.ppid) {
            node.push(proc.pid);
            // Siblings are ordered the way the Pid column orders rows, so a
            // thread sits next to the process whose id it was handed instead
            // of before every process.
            node.sort_unstable_by_key(|pid| row_sort_key(*pid));
        } else {
            self.tree.insert(proc.ppid, vec![proc.pid]);
        }
        self.rev_tree.insert(proc.pid, proc.ppid);
    }

    fn display_header(
        &self,
        align: &crate::config::ConfigColumnAlign,
        _order: Option<crate::config::ConfigSortOrder>,
        _config: &crate::config::Config,
    ) -> String {
        crate::util::adjust(&self.header, self.width, align)
    }

    fn display_content(
        &self,
        pid: i64,
        align: &crate::config::ConfigColumnAlign,
    ) -> Option<String> {
        fn gen_root(
            tree: &HashMap<i64, Vec<i64>>,
            rev_tree: &HashMap<i64, i64>,
            symbols: &[String; 5],
            pid: i64,
            mut string: String,
        ) -> String {
            if let Some(ppid) = rev_tree.get(&pid) {
                if *ppid == pid {
                    string
                } else if let Some(pppid) = rev_tree.get(ppid) {
                    let brother = tree.get(pppid).unwrap();
                    let is_last = brother
                        .binary_search_by_key(&row_sort_key(*ppid), |pid| row_sort_key(*pid))
                        .unwrap()
                        == brother.len() - 1;

                    if is_last {
                        string.push(' ');
                    } else {
                        string.push_str(&symbols[0]);
                    }
                    gen_root(tree, rev_tree, symbols, *ppid, string)
                } else {
                    string
                }
            } else {
                string
            }
        }

        if let Some(ppid) = self.rev_tree.get(&pid) {
            let root = gen_root(
                &self.tree,
                &self.rev_tree,
                &self.symbols,
                pid,
                String::new(),
            );
            let root: String = root.chars().rev().collect();

            let brother = &self.tree[ppid];
            let is_last = brother
                .binary_search_by_key(&row_sort_key(pid), |pid| row_sort_key(*pid))
                .unwrap()
                == brother.len() - 1;
            let has_child = self.tree.contains_key(&pid);

            let parent_connector = if is_last {
                &self.symbols[4]
            } else {
                &self.symbols[3]
            };
            let child_connector = if has_child {
                &self.symbols[2]
            } else {
                &self.symbols[1]
            };

            let string = format!(
                "{}{}{}{}",
                root,
                parent_connector,
                child_connector,
                self.symbols[1].repeat(self.width - root.chars().count() - 2)
            );
            Some(crate::util::adjust(&string, self.width, align))
        } else {
            None
        }
    }

    // Tree doesn't support JSON
    fn display_json(&self, _pid: i64) -> String {
        "".to_string()
    }

    fn find_partial(&self, _pid: i64, _keyword: &str, _content_to_lowercase: bool) -> bool {
        false
    }

    fn find_exact(&self, _pid: i64, _keyword: &str, _content_to_lowercase: bool) -> bool {
        false
    }

    fn sorted_pid(&self, _order: &crate::config::ConfigSortOrder) -> Vec<i64> {
        let mut root_pids = Vec::new();
        for p in self.rev_tree.values() {
            if !self.rev_tree.contains_key(p) {
                root_pids.push(*p);
            } else if let Some(ppid) = self.rev_tree.get(p)
                && *ppid == *p
            {
                root_pids.push(*p);
            }
        }
        // Ordered the way the sibling lists are - see `add` - so that a thread
        // row whose process row `apply_visible` dropped does not sort in front
        // of every process once it becomes a root of its own.
        root_pids.sort_unstable_by_key(|pid| row_sort_key(*pid));
        root_pids.dedup();

        fn push_pid(tree: &HashMap<i64, Vec<i64>>, mut pids: Vec<i64>, pid: i64) -> Vec<i64> {
            if let Some(leafs) = tree.get(&pid) {
                for p in leafs {
                    pids.push(*p);
                    if pid != *p {
                        pids = push_pid(tree, pids, *p);
                    }
                }
            }
            pids
        }

        let mut pids = Vec::new();
        for r in &root_pids {
            pids = push_pid(&self.tree, pids, *r);
        }

        pids
    }

    fn apply_visible(&mut self, visible_pids: &[i64]) {
        let mut remove_pids = Vec::new();
        for k in self.rev_tree.keys() {
            if !visible_pids.contains(k) {
                remove_pids.push(*k);
            }
        }
        for pid in remove_pids {
            self.rev_tree.remove(&pid);
            for x in self.tree.values_mut() {
                // The siblings are ordered by `row_sort_key`, so a thread row
                // has to be looked up the same way: a plain `binary_search`
                // on its negated key misses it and leaves a row the filter
                // dropped behind in the tree.
                if let Ok(i) = x.binary_search_by_key(&row_sort_key(pid), |p| row_sort_key(*p)) {
                    x.remove(i);
                }
            }
        }
    }

    fn reset_width(
        &mut self,
        _order: Option<crate::config::ConfigSortOrder>,
        _config: &crate::config::Config,
        _max_width: Option<usize>,
        _min_width: Option<usize>,
    ) {
        self.width = 0;
    }

    fn update_width(&mut self, pid: i64, _max_width: Option<usize>) {
        fn get_depth(rev_tree: &HashMap<i64, i64>, pid: i64, depth: i64) -> i64 {
            if let Some(ppid) = rev_tree.get(&pid) {
                if *ppid == pid {
                    depth
                } else {
                    get_depth(rev_tree, *ppid, depth + 1)
                }
            } else {
                depth
            }
        }

        let depth = get_depth(&self.rev_tree, pid, 0) as usize;
        self.width = cmp::max(depth + 4, self.width);
    }

    crate::column_default_display_unit!();
    crate::column_default_get_width!();
    crate::column_default_is_numeric!(false);
}

/// Siblings are held in `row_sort_key` order, so every lookup in one of those
/// lists has to use the same key - see `add`.
#[cfg(test)]
mod tests_sorted {
    use super::*;

    #[test]
    fn apply_visible_drops_a_thread_row() {
        let mut tree = Tree::new(&[
            String::from("│"),
            String::from("─"),
            String::from("┬"),
            String::from("├"),
            String::from("└"),
        ]);

        // One parent with four children: two processes with the threads whose
        // keys are the negated ids 8 and 16 in between. Ordering by key puts
        // the last thread after a larger process id, which is exactly where a
        // plain `binary_search` on the key stops looking.
        let mut siblings = vec![4i64, -8, 12, -16];
        siblings.sort_unstable_by_key(|pid| row_sort_key(*pid));
        tree.tree.insert(0, siblings);
        tree.rev_tree.insert(4, 0);
        tree.rev_tree.insert(-8, 0);
        tree.rev_tree.insert(12, 0);
        tree.rev_tree.insert(-16, 0);

        tree.apply_visible(&[0, 4, 12]);

        assert_eq!(tree.tree[&0], vec![4, 12]);
        assert!(!tree.rev_tree.contains_key(&-8));
        assert!(!tree.rev_tree.contains_key(&-16));
    }
}

#[cfg(test)]
#[cfg(any(target_os = "linux", target_os = "android"))]
mod tests {
    use super::*;
    use crate::process::ProcessInfoBase;
    use crate::process::ProcessTask;
    use procfs::process::Process;
    use std::time::Duration;

    #[test]
    fn test_tree() {
        let mut tree = Tree::new(&[
            String::from("│"),
            String::from("─"),
            String::from("┬"),
            String::from("├"),
            String::from("└"),
        ]);

        let curr_proc = ProcessTask::Process {
            stat: Process::myself().unwrap().stat().unwrap(),
            proc: Process::myself().unwrap(),
            owner: Process::myself().unwrap().uid().unwrap(),
        };
        let prev_stat = Process::myself().unwrap().stat().unwrap();

        let p0 = ProcessInfo {
            base: ProcessInfoBase::new(0, 0, Duration::new(0, 0)),
            curr_proc,
            prev_stat,
            curr_io: None,
            prev_io: None,
            curr_status: None,
        };

        let curr_proc = ProcessTask::Process {
            stat: Process::myself().unwrap().stat().unwrap(),
            proc: Process::myself().unwrap(),
            owner: Process::myself().unwrap().uid().unwrap(),
        };
        let prev_stat = Process::myself().unwrap().stat().unwrap();

        let p1 = ProcessInfo {
            base: ProcessInfoBase::new(1, 0, Duration::new(0, 0)),
            curr_proc,
            prev_stat,
            curr_io: None,
            prev_io: None,
            curr_status: None,
        };

        let curr_proc = ProcessTask::Process {
            stat: Process::myself().unwrap().stat().unwrap(),
            proc: Process::myself().unwrap(),
            owner: Process::myself().unwrap().uid().unwrap(),
        };
        let prev_stat = Process::myself().unwrap().stat().unwrap();

        let p2 = ProcessInfo {
            base: ProcessInfoBase::new(2, 1, Duration::new(0, 0)),
            curr_proc,
            prev_stat,
            curr_io: None,
            prev_io: None,
            curr_status: None,
        };

        tree.add(&p0);
        tree.add(&p1);
        tree.add(&p2);
        tree.update_width(0, None);
        tree.update_width(1, None);
        tree.update_width(2, None);
        assert_eq!(
            tree.display_content(0, &crate::config::ConfigColumnAlign::Left)
                .unwrap(),
            String::from("├┬────")
        );
        assert_eq!(
            tree.display_content(1, &crate::config::ConfigColumnAlign::Left)
                .unwrap(),
            String::from("│└┬───")
        );
        assert_eq!(
            tree.display_content(2, &crate::config::ConfigColumnAlign::Left)
                .unwrap(),
            String::from("│ └───")
        );
    }
}
