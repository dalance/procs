use crate::column::Column;
use crate::columns::*;
use crate::config::*;
use crate::opt::{ArgColorMode, ArgPagerMode};
use crate::process::collect_proc;
use crate::style::{apply_color, apply_style, color_to_column_style};
use crate::term_info::TermInfo;
use crate::util::{classify, find_column_kind, find_exact, find_partial, truncate, KeywordClass};
use crate::Opt;
use anyhow::{bail, Error};
#[cfg(not(target_os = "windows"))]
use pager::Pager;
use std::collections::HashMap;
use std::time::Duration;

pub struct SortInfo {
    pub idx: usize,
    pub order: ConfigSortOrder,
}

pub struct View {
    pub columns: Vec<ColumnInfo>,
    pub term_info: TermInfo,
    pub sort_info: SortInfo,
    pub visible_pids: Vec<i32>,
    pub auxiliary_pids: Vec<i32>,
    pub parent_pids: HashMap<i32, i32>,
    pub child_pids: HashMap<i32, Vec<i32>>,
}

impl View {
    pub fn new(opt: &mut Opt, config: &Config, clear_by_line: bool) -> Result<Self, Error> {
        let mut slot_idx = 0;
        let mut columns = Vec::new();
        let mut only_kind_found = false;

        // Override style of TreeSlot
        let tree_slot = ConfigColumn {
            kind: ConfigColumnKind::TreeSlot,
            style: color_to_column_style(&config.style.tree),
            numeric_search: false,
            nonnumeric_search: false,
            align: ConfigColumnAlign::Left,
            max_width: None,
            min_width: None,
            header: None,
        };

        // Adding the sort column to inserts if not already present
        match (&opt.sorta, &opt.sortd) {
            (_, Some(col)) | (Some(col), _) => {
                if !opt.insert.contains(col) {
                    opt.insert.push(col.clone());
                }
            }
            _ => {}
        }

        // Add default TreeSlot if there is not TreeSlot in config
        let config_columns = if config
            .columns
            .iter()
            .all(|x| x.kind != ConfigColumnKind::TreeSlot)
            && opt.tree
        {
            let mut ret = vec![tree_slot];
            ret.append(&mut config.columns.clone());
            ret
        } else {
            config
                .columns
                .iter()
                .map(|x| {
                    if x.kind == ConfigColumnKind::TreeSlot {
                        tree_slot.clone()
                    } else {
                        x.clone()
                    }
                })
                .collect()
        };

        for c in &config_columns {
            let kinds = match &c.kind {
                ConfigColumnKind::Slot => {
                    let kinds = if let Some(insert) = opt.insert.get(slot_idx) {
                        find_column_kind(insert).into_iter().collect()
                    } else {
                        vec![]
                    };
                    slot_idx += 1;
                    kinds
                }
                ConfigColumnKind::MultiSlot => {
                    let mut kinds = vec![];
                    while let Some(insert) = opt.insert.get(slot_idx) {
                        if let Some(kind) = find_column_kind(insert) {
                            kinds.push(kind);
                        }
                        slot_idx += 1;
                    }
                    kinds
                }
                ConfigColumnKind::TreeSlot => {
                    if opt.tree {
                        vec![ConfigColumnKind::Tree]
                    } else {
                        vec![]
                    }
                }
                x => vec![x.clone()],
            };

            for kind in kinds {
                let visible = if let Some(only) = &opt.only {
                    let kind_name = KIND_LIST[&kind].0.to_lowercase();
                    if !kind_name.contains(&only.to_lowercase()) {
                        false
                    } else {
                        only_kind_found = true;
                        true
                    }
                } else {
                    true
                };

                let column = gen_column(
                    &kind,
                    c.header.clone(),
                    &config.docker.path,
                    &config.display.separator,
                    config.display.abbr_sid,
                    &config.display.tree_symbols,
                    opt.procfs.clone(),
                );
                if column.available() {
                    columns.push(ColumnInfo {
                        column,
                        kind,
                        style: c.style.clone(),
                        nonnumeric_search: c.nonnumeric_search,
                        numeric_search: c.numeric_search,
                        align: c.align.clone(),
                        max_width: c.max_width,
                        min_width: c.min_width,
                        visible,
                    });
                }
            }
        }

        if slot_idx < opt.insert.len() {
            bail!("There is not enough slot for inserting columns {:?}.\nPlease add \"Slot\" or \"MultiSlot\" to your config.\nhttps://github.com/dalance/procs#insert-column", opt.insert);
        }

        if let Some(only_kind) = &opt.only {
            if !only_kind_found {
                bail!("kind \"{}\" is not found in columns", only_kind);
            }
        }

        let show_thread = if opt.thread {
            true
        } else if opt.tree {
            config.display.show_thread_in_tree
        } else {
            config.display.show_thread
        };

        let proc = collect_proc(
            Duration::from_millis(opt.interval),
            show_thread,
            config.display.show_kthreads,
            &opt.procfs,
        );
        for c in columns.iter_mut() {
            for p in &proc {
                c.column.add(p);
            }
        }

        let mut parent_pids = HashMap::new();
        let mut child_pids = HashMap::<i32, Vec<i32>>::new();
        if opt.tree || !config.display.show_self_parents {
            for p in &proc {
                parent_pids.insert(p.pid, p.ppid);
                if let Some(x) = child_pids.get_mut(&p.ppid) {
                    x.push(p.pid);
                } else {
                    child_pids.insert(p.ppid, vec![p.pid]);
                }
            }
        }

        let term_info = TermInfo::new(clear_by_line, false)?;
        let mut sort_info = View::get_sort_info(opt, config, &columns);

        if opt.only.is_some() {
            sort_info.idx = 0;
        }

        Ok(View {
            columns,
            term_info,
            sort_info,
            visible_pids: vec![],
            auxiliary_pids: vec![],
            parent_pids,
            child_pids,
        })
    }

    pub fn filter(&mut self, opt: &Opt, config: &Config, header_lines: usize) {
        let mut cols_nonnumeric = Vec::new();
        let mut cols_numeric = Vec::new();
        for c in &self.columns {
            if c.nonnumeric_search {
                cols_nonnumeric.push(c.column.as_ref());
            }
            if c.numeric_search {
                cols_numeric.push(c.column.as_ref());
            }
        }

        let mut keyword_nonnumeric = Vec::new();
        let mut keyword_numeric = Vec::new();

        for k in &opt.keyword {
            match classify(k) {
                KeywordClass::Numeric => keyword_numeric.push(k),
                KeywordClass::NonNumeric => keyword_nonnumeric.push(k),
            }
        }

        let pids = self.columns[self.sort_info.idx]
            .column
            .sorted_pid(&self.sort_info.order);

        let self_pid = std::process::id() as i32;

        let self_parents = if !config.display.show_self_parents {
            let mut self_parents = Vec::new();
            self.get_parent_pids(self_pid, &mut self_parents);
            self_parents
                .into_iter()
                .filter(|x| {
                    if let Some(x) = self.child_pids.get(x) {
                        x.len() == 1
                    } else {
                        false
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let logic = if opt.and {
            ConfigSearchLogic::And
        } else if opt.or {
            ConfigSearchLogic::Or
        } else if opt.nand {
            ConfigSearchLogic::Nand
        } else if opt.nor {
            ConfigSearchLogic::Nor
        } else {
            config.search.logic.clone()
        };

        let mut candidate_pids = Vec::new();
        for pid in &pids {
            let hidden_process = (!config.display.show_self && *pid == self_pid)
                || (!config.display.show_self_parents && self_parents.contains(pid));

            let candidate = if hidden_process {
                false
            } else if opt.keyword.is_empty() {
                true
            } else {
                View::search(
                    *pid,
                    &keyword_numeric,
                    &keyword_nonnumeric,
                    cols_numeric.as_slice(),
                    cols_nonnumeric.as_slice(),
                    config,
                    &logic,
                )
            };

            if candidate {
                candidate_pids.push(*pid);
            }
        }

        let mut auxiliary_pids = Vec::new();
        if opt.tree {
            let mut additional_pids = Vec::new();
            for pid in &candidate_pids {
                let mut buf = vec![];
                if config.display.show_parent_in_tree {
                    self.get_parent_pids(*pid, &mut buf);
                }
                if config.display.show_children_in_tree {
                    self.get_child_pids(*pid, &mut buf);
                }
                additional_pids.append(&mut buf);
            }
            let mut additional_pids: Vec<_> = additional_pids
                .iter()
                .filter(|x| !candidate_pids.contains(x))
                .copied()
                .collect();
            candidate_pids.append(&mut additional_pids.clone());
            auxiliary_pids.append(&mut additional_pids);
        }

        let mut visible_pids = Vec::new();
        for pid in &pids {
            if candidate_pids.contains(pid) {
                visible_pids.push(*pid);
            }

            let reserved_rows = 4 + header_lines;
            if opt.watch_mode && visible_pids.len() >= self.term_info.height - reserved_rows {
                break;
            }
        }

        self.visible_pids = visible_pids;
        self.auxiliary_pids = auxiliary_pids;
    }

    fn get_parent_pids(&self, pid: i32, parent_pids: &mut Vec<i32>) {
        if let Some(x) = self.parent_pids.get(&pid) {
            if !parent_pids.contains(x) {
                parent_pids.push(*x);
                self.get_parent_pids(*x, parent_pids);
            }
        }
    }

    fn get_child_pids(&self, pid: i32, child_pids: &mut Vec<i32>) {
        if let Some(pids) = self.child_pids.get(&pid) {
            for x in pids {
                if !child_pids.contains(x) {
                    child_pids.push(*x);
                    self.get_child_pids(*x, child_pids);
                }
            }
        }
    }

    pub fn adjust(&mut self, config: &Config, min_widths: &HashMap<usize, usize>) {
        for (i, ref mut c) in self.columns.iter_mut().enumerate() {
            let order = if i == self.sort_info.idx {
                Some(self.sort_info.order.clone())
            } else {
                None
            };
            c.column.apply_visible(&self.visible_pids);
            let min_width = min_widths.get(&i).map(|x| Some(*x)).unwrap_or(c.min_width);
            c.column.reset_width(order, config, c.max_width, min_width);
            for pid in &self.visible_pids {
                c.column.update_width(*pid, c.max_width);
            }
        }
    }

    pub fn display(
        &mut self,
        opt: &Opt,
        config: &Config,
        theme: &ConfigTheme,
    ) -> Result<(), Error> {
        if opt.json {
            self.term_info.use_pager = false;
            self.display_json()?;
            return Ok(());
        }

        let use_terminal = console::user_attended();

        // +3 means header/unit line and next prompt
        let pager_threshold_height = self.visible_pids.len() + 3;

        // "self.columns.len() - 1" means spacing between columns
        let pager_threshold_width = if config.pager.detect_width {
            self.columns
                .iter()
                .map(|x| x.column.get_width())
                .sum::<usize>()
                + self.columns.len()
                - 1
        } else {
            usize::MIN
        };

        let use_builtin_pager = if cfg!(target_os = "windows") {
            true
        } else {
            config.pager.use_builtin
        };

        let use_pager = match (opt.watch_mode, opt.pager.as_ref(), &config.pager.mode) {
            (true, _, _) => false,
            (false, Some(ArgPagerMode::Auto), _) => {
                self.term_info.height < pager_threshold_height
                    || self.term_info.width < pager_threshold_width
            }
            (false, Some(ArgPagerMode::Always), _) => true,
            (false, Some(ArgPagerMode::Disable), _) => false,
            (false, None, ConfigPagerMode::Auto) => {
                self.term_info.height < pager_threshold_height
                    || self.term_info.width < pager_threshold_width
            }
            (false, None, ConfigPagerMode::Always) => true,
            (false, None, ConfigPagerMode::Disable) => false,
        };

        // Minus doesn't support horizontal scroll yet
        // https://github.com/arijit79/minus/issues/59
        let cut_to_pager = if use_builtin_pager {
            true
        } else {
            config.display.cut_to_pager
        };

        let mut truncate = use_terminal && use_pager && cut_to_pager;
        truncate |= use_terminal && !use_pager && config.display.cut_to_terminal;
        truncate |= !use_terminal && config.display.cut_to_pipe;

        if !truncate {
            self.term_info.width = usize::MAX;
        }

        match (opt.color.as_ref(), &config.display.color_mode) {
            (Some(ArgColorMode::Auto), _) => {
                if use_pager && use_terminal {
                    console::set_colors_enabled(true);
                }
            }
            (Some(ArgColorMode::Always), _) => console::set_colors_enabled(true),
            (Some(ArgColorMode::Disable), _) => console::set_colors_enabled(false),
            (None, ConfigColorMode::Auto) => {
                if use_pager && use_terminal {
                    console::set_colors_enabled(true);
                }
            }
            (None, ConfigColorMode::Always) => console::set_colors_enabled(true),
            (None, ConfigColorMode::Disable) => console::set_colors_enabled(false),
        }

        if use_pager {
            if use_builtin_pager {
                self.term_info.use_pager = true;
            } else {
                View::pager(config);
            }
        }

        if !opt.no_header && config.display.show_header {
            // Ignore display_* error
            //   `Broken pipe` may occur at pager mode. It can be ignored safely.
            let _ = self.display_header(config, theme);
            let _ = self.display_unit(config, theme);
        }

        for pid in &self.visible_pids {
            let auxiliary = self.auxiliary_pids.contains(pid);
            let _ = self.display_content(config, *pid, theme, auxiliary);
        }

        if !opt.no_header && config.display.show_footer {
            let _ = self.display_unit(config, theme);
            let _ = self.display_header(config, theme);
        }

        if self.term_info.use_pager {
            minus::page_all(self.term_info.pager.replace(None).unwrap())?;
        }

        Ok(())
    }

    fn display_header(&self, config: &Config, theme: &ConfigTheme) -> Result<(), Error> {
        let mut row = String::new();
        for (i, c) in self.columns.iter().enumerate() {
            if c.visible {
                let order = if i == self.sort_info.idx {
                    Some(self.sort_info.order.clone())
                } else {
                    None
                };
                row = format!(
                    "{} {}",
                    row,
                    apply_color(
                        c.column.display_header(&c.align, order, config),
                        &config.style.header,
                        theme,
                        false
                    )
                );
            }
        }
        row = row.trim_end().to_string();
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_unit(&self, config: &Config, theme: &ConfigTheme) -> Result<(), Error> {
        let mut row = String::new();
        for c in &self.columns {
            if c.visible {
                row = format!(
                    "{} {}",
                    row,
                    apply_color(
                        c.column.display_unit(&c.align),
                        &config.style.unit,
                        theme,
                        false
                    )
                );
            }
        }
        row = row.trim_end().to_string();
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_content(
        &self,
        config: &Config,
        pid: i32,
        theme: &ConfigTheme,
        auxiliary: bool,
    ) -> Result<(), Error> {
        let mut row = String::new();
        for c in &self.columns {
            if c.visible {
                row = format!(
                    "{} {}",
                    row,
                    apply_style(
                        c.column.display_content(pid, &c.align).unwrap(),
                        &c.style,
                        &config.style,
                        theme,
                        auxiliary
                    )
                );
            }
        }
        row = row.trim_end().to_string();
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_json(&self) -> Result<(), Error> {
        self.term_info.write_line("[")?;

        let len_pid = self.visible_pids.len();
        for (i, pid) in self.visible_pids.iter().enumerate() {
            let mut line = "{".to_string();
            let len_column = self.columns.len();
            for (j, c) in self.columns.iter().enumerate() {
                if c.visible {
                    let text = c.column.display_json(*pid);
                    line.push_str(&text);
                    if j != len_column - 1 {
                        line.push_str(", ");
                    }
                }
            }
            line.push('}');
            if i != len_pid - 1 {
                line.push(',');
            }
            self.term_info.write_line(&line)?;
        }

        self.term_info.write_line("]")?;
        Ok(())
    }

    fn get_sort_info(opt: &Opt, config: &Config, cols: &[ColumnInfo]) -> SortInfo {
        let (mut sort_idx, sort_order) = match (&opt.sorta, &opt.sortd) {
            (Some(sort), _) | (_, Some(sort)) => {
                let mut idx = config.sort.column;
                let mut order = config.sort.order.clone();
                for (i, c) in cols.iter().enumerate() {
                    let (kind, _) = KIND_LIST[&c.kind];
                    if kind.to_lowercase().contains(&sort.to_lowercase()) {
                        idx = i;
                        order = if opt.sorta.is_some() {
                            ConfigSortOrder::Ascending
                        } else {
                            ConfigSortOrder::Descending
                        };
                        break;
                    }
                }
                (idx, order)
            }
            _ => (config.sort.column, config.sort.order.clone()),
        };

        if opt.tree {
            sort_idx = cols
                .iter()
                .position(|x| x.kind == ConfigColumnKind::Tree)
                .unwrap();
        }

        SortInfo {
            idx: sort_idx,
            order: sort_order,
        }
    }

    fn search<T: AsRef<str>>(
        pid: i32,
        keyword_numeric: &[T],
        keyword_nonnumeric: &[T],
        cols_numeric: &[&dyn Column],
        cols_nonnumeric: &[&dyn Column],
        config: &Config,
        logic: &ConfigSearchLogic,
    ) -> bool {
        let ret_nonnumeric = match config.search.nonnumeric_search {
            ConfigSearchKind::Partial => find_partial(
                cols_nonnumeric,
                pid,
                keyword_nonnumeric,
                logic,
                &config.search.case,
            ),
            ConfigSearchKind::Exact => find_exact(
                cols_nonnumeric,
                pid,
                keyword_nonnumeric,
                logic,
                &config.search.case,
            ),
        };
        let ret_numeric = match config.search.numeric_search {
            ConfigSearchKind::Partial => find_partial(
                cols_numeric,
                pid,
                keyword_numeric,
                logic,
                &config.search.case,
            ),
            ConfigSearchKind::Exact => find_exact(
                cols_numeric,
                pid,
                keyword_numeric,
                logic,
                &config.search.case,
            ),
        };
        match logic {
            ConfigSearchLogic::And => ret_nonnumeric & ret_numeric,
            ConfigSearchLogic::Or => ret_nonnumeric | ret_numeric,
            ConfigSearchLogic::Nand => !(ret_nonnumeric & ret_numeric),
            ConfigSearchLogic::Nor => !(ret_nonnumeric | ret_numeric),
        }
    }

    #[cfg(not(any(target_os = "windows", any(target_os = "linux", target_os = "android"))))]
    fn pager(config: &Config) {
        if let Some(ref pager) = config.pager.command {
            Pager::with_pager(pager).setup();
        } else if which::which("less").is_ok() {
            Pager::with_pager("less -SR").setup();
        } else {
            Pager::with_pager("more -f").setup();
        }
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn pager(config: &Config) {
        if let Some(ref pager) = config.pager.command {
            Pager::with_pager(pager)
                // workaround for default less charset is "ascii" on some environments (ex. Ubuntu)
                .pager_envs(["LESSCHARSET=utf-8\0"])
                .setup();
        } else if which::which("less").is_ok() {
            Pager::with_pager("less -SR")
                .pager_envs(["LESSCHARSET=utf-8\0"])
                .setup();
        } else {
            Pager::with_pager("more -f").setup();
        }
    }

    #[cfg(target_os = "windows")]
    fn pager(_config: &Config) {}

    pub fn inc_sort_column(&mut self) -> usize {
        let current = self.sort_info.idx;
        let max_idx = self.columns.len();

        for i in 1..max_idx {
            let idx = (current + i) % max_idx;
            if self.columns[idx].column.sortable() {
                return idx;
            }
        }
        current
    }

    pub fn dec_sort_column(&mut self) -> usize {
        let current = self.sort_info.idx;
        let max_idx = self.columns.len();

        for i in 1..max_idx {
            let idx = (current + max_idx - i) % max_idx;
            if self.columns[idx].column.sortable() {
                return idx;
            }
        }
        current
    }
}
