use crate::Opt;
use crate::column::Column;
use crate::columns::*;
use crate::config::*;
use crate::opt::{ArgColorMode, ArgPagerMode};
use crate::process::{ShowFilter, collect_proc};
use crate::search_regex::SearchRegex;
use crate::style::{apply_color, apply_style, color_to_column_style};
use crate::term_info::TermInfo;
use crate::util::{
    KeywordClass, ansi_trim_end, classify, find_column_kind, find_exact, find_partial,
    has_regex_syntax, truncate,
};
use anyhow::{Error, bail};
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
    pub visible_pids: Vec<i64>,
    pub auxiliary_pids: Vec<i64>,
    pub parent_pids: HashMap<i64, i64>,
    pub child_pids: HashMap<i64, Vec<i64>>,
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
            (_, Some(col)) | (Some(col), _) if !opt.insert.contains(col) => {
                opt.insert.push(col.clone());
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
                    if kind == ConfigColumnKind::Tree {
                        true
                    } else {
                        let kind_name = KIND_LIST[&kind].0.to_lowercase();
                        if !kind_name.contains(&only.to_lowercase()) {
                            false
                        } else {
                            only_kind_found = true;
                            true
                        }
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
            bail!(
                "There is not enough slot for inserting columns {:?}.\nPlease add \"Slot\" or \"MultiSlot\" to your config.\nhttps://github.com/dalance/procs#insert-column",
                opt.insert
            );
        }

        if let Some(only_kind) = &opt.only
            && !only_kind_found
        {
            bail!("kind \"{}\" is not found in columns", only_kind);
        }

        let show_thread = if opt.thread {
            true
        } else if opt.tree {
            config.display.show_thread_in_tree
        } else {
            config.display.show_thread
        };

        let filter = ShowFilter {
            other_users: config.display.show_other_users,
            kthread: config.display.show_kthreads,
        };

        let proc = collect_proc(
            Duration::from_millis(opt.interval),
            show_thread,
            &opt.procfs,
            filter,
        );

        for c in columns.iter_mut() {
            for p in &proc {
                c.column.add(p);
            }
        }

        let mut parent_pids = HashMap::new();
        let mut child_pids = HashMap::<i64, Vec<i64>>::new();
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

    pub fn filter(&mut self, opt: &Opt, config: &Config, header_lines: usize) -> Result<(), Error> {
        let mut cols_nonnumeric = Vec::new();
        let mut cols_numeric = Vec::new();
        let mut cols_searchable = Vec::new();
        for c in &self.columns {
            if c.nonnumeric_search {
                cols_nonnumeric.push(c.column.as_ref());
                cols_searchable.push(c.column.as_ref());
            }
            if c.numeric_search {
                cols_numeric.push(c.column.as_ref());
                if !c.nonnumeric_search {
                    cols_searchable.push(c.column.as_ref());
                }
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

        let regex_mode = if opt.regex {
            true
        } else if opt.smart {
            opt.keyword.len() == 1 && has_regex_syntax(&opt.keyword[0])
        } else {
            false
        };

        let regex = if regex_mode && !opt.keyword.is_empty() {
            let pattern = &opt.keyword[0];
            let ignore_case = match config.search.case {
                ConfigSearchCase::Smart => pattern == &pattern.to_ascii_lowercase(),
                ConfigSearchCase::Insensitive => true,
                ConfigSearchCase::Sensitive => false,
            };
            let regex = SearchRegex::new(pattern, ignore_case)?;
            Some(regex)
        } else {
            None
        };

        let pids = self.columns[self.sort_info.idx]
            .column
            .sorted_pid(&self.sort_info.order);

        let self_pid = std::process::id() as i64;

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
            } else if let Some(regex) = &regex {
                View::search_regex(*pid, cols_searchable.as_slice(), regex)?
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
        Ok(())
    }

    fn get_parent_pids(&self, pid: i64, parent_pids: &mut Vec<i64>) {
        if let Some(x) = self.parent_pids.get(&pid)
            && !parent_pids.contains(x)
        {
            parent_pids.push(*x);
            self.get_parent_pids(*x, parent_pids);
        }
    }

    fn get_child_pids(&self, pid: i64, child_pids: &mut Vec<i64>) {
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

        // On Windows, `[pager] command` is what selects the external pager, so
        // it wins over the built-in one that is otherwise always available -
        // unless `use_builtin` is set, which is an explicit request for the
        // built-in pager.
        #[cfg(target_os = "windows")]
        let use_builtin_pager = config.pager.use_builtin || pager_command(config).is_none();
        #[cfg(not(target_os = "windows"))]
        let use_builtin_pager = config.pager.use_builtin;

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

        // Minus support of horizontal scroll seems broken with ANSI escape code
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
                self.pager(config)?;
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
        } else {
            self.term_info.finish_external_pager()?;
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
        row = ansi_trim_end(&row);
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
        row = ansi_trim_end(&row);
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_content(
        &self,
        config: &Config,
        pid: i64,
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
        row = ansi_trim_end(&row);
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_json(&self) -> Result<(), Error> {
        self.term_info.write_line("[")?;

        let len_pid = self.visible_pids.len();
        for (i, pid) in self.visible_pids.iter().enumerate() {
            let fields: Vec<String> = self
                .columns
                .iter()
                .filter(|c| c.visible && c.kind != ConfigColumnKind::Separator)
                .map(|c| c.column.display_json(*pid))
                .collect();
            let mut line = json_object(&fields);
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
        pid: i64,
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

    fn search_regex(pid: i64, cols: &[&dyn Column], regex: &SearchRegex) -> Result<bool, Error> {
        for c in cols {
            if regex.is_match(&c.display_json(pid))? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[cfg(not(any(target_os = "windows", any(target_os = "linux", target_os = "android"))))]
    fn pager(&mut self, config: &Config) -> Result<(), Error> {
        if let Some(ref pager) = config.pager.command {
            Pager::with_pager(pager).setup();
        } else if which::which("less").is_ok() {
            Pager::with_pager("less -SR").setup();
        } else {
            Pager::with_pager("more -f").setup();
        }
        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn pager(&mut self, config: &Config) -> Result<(), Error> {
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
        Ok(())
    }

    /// Spawns the pager given by `[pager] command` and feeds the output to its stdin.
    /// If it is not set, or the command cannot be parsed or spawned,
    /// the built-in pager is used.
    #[cfg(target_os = "windows")]
    fn pager(&mut self, config: &Config) -> Result<(), Error> {
        let Some(command) = pager_command(config) else {
            // `[pager] command` is not set: the built-in pager is the normal choice
            self.term_info.use_pager = true;
            return Ok(());
        };

        let Some((program, args)) = split_pager_command(command) else {
            let _ = console::Term::stderr().write_line(&format!(
                "warning: failed to parse pager command \"{command}\". falling back to the built-in pager"
            ));
            self.term_info.use_pager = true;
            return Ok(());
        };

        match std::process::Command::new(&program)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                self.term_info.external_pager.replace(Some(child));
            }
            Err(x) => {
                let _ = console::Term::stderr().write_line(&format!(
                    "warning: failed to launch pager \"{program}\" ({x}). falling back to the built-in pager"
                ));
                self.term_info.use_pager = true;
            }
        }
        Ok(())
    }

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

/// Builds one JSON object row. Columns without a JSON representation
/// (e.g. Tree) yield an empty string and must not produce a separator.
fn json_object(fields: &[String]) -> String {
    let fields: Vec<&str> = fields
        .iter()
        .map(String::as_str)
        .filter(|x| !x.is_empty())
        .collect();
    format!("{{{}}}", fields.join(", "))
}

/// Returns the pager command given by `command` of `[pager]` section.
/// An empty value is treated as unset.
#[cfg(target_os = "windows")]
fn pager_command(config: &Config) -> Option<&str> {
    config
        .pager
        .command
        .as_deref()
        .map(str::trim)
        .filter(|x| !x.is_empty())
}

/// Splits a command line into the program and its arguments.
/// A part surrounded by `"` or `'` is kept as a single argument
/// so that a path containing spaces (ex. `"C:\Program Files\Git\usr\bin\less.exe" -SR`) works.
/// Returns `None` if the command is empty or a quote is not closed,
/// so that the caller can fall back to the built-in pager.
#[cfg(target_os = "windows")]
fn split_pager_command(command: &str) -> Option<(String, Vec<String>)> {
    let mut args = Vec::new();
    let mut arg = String::new();
    let mut started = false;
    let mut quote = None;

    for c in command.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else {
                    arg.push(c);
                }
            }
            None => match c {
                q @ ('"' | '\'') => {
                    started = true;
                    quote = Some(q);
                }
                c if c.is_whitespace() => {
                    if started {
                        args.push(std::mem::take(&mut arg));
                        started = false;
                    }
                }
                _ => {
                    arg.push(c);
                    started = true;
                }
            },
        }
    }
    if quote.is_some() {
        return None;
    }
    if started {
        args.push(arg);
    }

    let mut args = args.into_iter();
    let program = args.next()?;
    if program.is_empty() {
        return None;
    }
    Some((program, args.collect()))
}

#[cfg(test)]
mod tests {
    use super::json_object;
    #[cfg(target_os = "windows")]
    use super::{pager_command, split_pager_command};
    #[cfg(target_os = "windows")]
    use crate::{CONFIG_DEFAULT, Config};

    #[test]
    fn json_object_skips_columns_without_json() {
        assert_eq!(json_object(&[]), "{}");
        assert_eq!(json_object(&[r#""PID": 1"#.into()]), r#"{"PID": 1}"#);
        // a hidden (--only) or Tree column contributes nothing, not a comma
        assert_eq!(
            json_object(&["".into(), r#""PID": 1"#.into(), "".into()]),
            r#"{"PID": 1}"#
        );
        assert_eq!(
            json_object(&[r#""PID": 1"#.into(), r#""CPU": 0"#.into()]),
            r#"{"PID": 1, "CPU": 0}"#
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn split_pager_command_parses_program_and_args() {
        assert_eq!(split_pager_command(""), None);
        assert_eq!(split_pager_command("   "), None);
        assert_eq!(split_pager_command("less"), Some(("less".into(), vec![])));
        assert_eq!(
            split_pager_command("less -SR"),
            Some(("less".into(), vec!["-SR".into()]))
        );
        assert_eq!(
            split_pager_command("  less   -S   -R  "),
            Some(("less".into(), vec!["-S".into(), "-R".into()]))
        );
        assert_eq!(
            split_pager_command(r#""C:\Program Files\Git\usr\bin\less.exe" -SR"#),
            Some((
                r"C:\Program Files\Git\usr\bin\less.exe".into(),
                vec!["-SR".into()]
            ))
        );
        // single quotes work as well
        assert_eq!(
            split_pager_command(r"'C:\Program Files\Git\usr\bin\less.exe' -S -R"),
            Some((
                r"C:\Program Files\Git\usr\bin\less.exe".into(),
                vec!["-S".into(), "-R".into()]
            ))
        );
        // a quoted argument keeps the whitespaces in it
        assert_eq!(
            split_pager_command(r#"less "-S -R""#),
            Some(("less".into(), vec!["-S -R".into()]))
        );
        // unbalanced quote is rejected instead of building a broken command line
        assert_eq!(
            split_pager_command(r#""C:\Program Files\Git\usr\bin\less.exe -SR"#),
            None
        );
        assert_eq!(split_pager_command(r"less '-SR"), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn pager_command_uses_config_command() {
        let mut config: Config = toml::from_str(CONFIG_DEFAULT).unwrap();
        assert_eq!(pager_command(&config), None);

        config.pager.command = Some("less -SR".into());
        assert_eq!(pager_command(&config), Some("less -SR"));

        config.pager.command = Some("  less -SR  ".into());
        assert_eq!(pager_command(&config), Some("less -SR"));

        // an empty `command` is treated as unset
        config.pager.command = Some("   ".into());
        assert_eq!(pager_command(&config), None);

        config.pager.command = Some(String::new());
        assert_eq!(pager_command(&config), None);
    }
}
