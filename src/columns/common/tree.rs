use crate::process::ProcessInfo;
use crate::Column;
use std::cmp;
use std::collections::HashMap;

pub struct Tree {
    header: String,
    unit: String,
    max_width: usize,
    tree: HashMap<i32, Vec<i32>>,
    rev_tree: HashMap<i32, i32>,
    symbols: [String; 5],
}

impl Tree {
    pub fn new(symbols: &[String; 5]) -> Self {
        let header = String::from("");
        let unit = String::from("");
        Tree {
            max_width: 0,
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
            node.sort();
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
        crate::util::expand(&self.header, self.max_width, align)
    }

    fn display_content(
        &self,
        pid: i32,
        align: &crate::config::ConfigColumnAlign,
    ) -> Option<String> {
        fn gen_root(
            tree: &HashMap<i32, Vec<i32>>,
            rev_tree: &HashMap<i32, i32>,
            symbols: &[String; 5],
            pid: i32,
            mut string: String,
        ) -> String {
            if let Some(ppid) = rev_tree.get(&pid) {
                if *ppid != 0 {
                    let pppid = rev_tree.get(ppid).unwrap();
                    let brother = tree.get(pppid).unwrap();
                    let is_last = brother.binary_search(&ppid).unwrap() == brother.len() - 1;

                    if is_last {
                        string.push(' ');
                    } else {
                        //string.push('│');
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
                String::from(""),
            );
            let root: String = root.chars().rev().collect();

            let brother = &self.tree[&ppid];
            let is_last = brother.binary_search(&pid).unwrap() == brother.len() - 1;
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
                self.symbols[1].repeat(self.max_width - root.chars().count() - 2)
            );
            Some(crate::util::expand(&string, self.max_width, align))
        } else {
            None
        }
    }

    fn find_partial(&self, _pid: i32, _keyword: &str) -> bool {
        false
    }

    fn find_exact(&self, _pid: i32, _keyword: &str) -> bool {
        false
    }

    fn sorted_pid(&self, _order: &crate::config::ConfigSortOrder) -> Vec<i32> {
        let pids = push_pid(&self.tree, Vec::new(), 0);

        fn push_pid(tree: &HashMap<i32, Vec<i32>>, mut pids: Vec<i32>, pid: i32) -> Vec<i32> {
            if let Some(leafs) = tree.get(&pid) {
                for p in leafs {
                    pids.push(*p);
                    pids = push_pid(tree, pids, *p);
                }
            }
            pids
        }

        pids
    }

    fn reset_max_width(
        &mut self,
        _order: Option<crate::config::ConfigSortOrder>,
        _config: &crate::config::Config,
    ) {
        self.max_width = 0;
    }

    fn update_max_width(&mut self, pid: i32) {
        fn get_depth(rev_tree: &HashMap<i32, i32>, pid: i32, depth: i32) -> i32 {
            if let Some(ppid) = rev_tree.get(&pid) {
                if *ppid != 0 {
                    get_depth(rev_tree, *ppid, depth + 1)
                } else {
                    depth
                }
            } else {
                depth
            }
        }

        let depth = get_depth(&self.rev_tree, pid, 0) as usize;
        self.max_width = cmp::max(depth + 4, self.max_width);
    }

    crate::column_default_display_unit!();
}
