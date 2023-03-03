use anyhow::Error;
use console::Term;
use minus::Pager;
use std::cell::RefCell;
use std::fmt::Write;

pub struct TermInfo {
    term: Term,
    pub pager: RefCell<Option<Pager>>,
    pub height: usize,
    pub width: usize,
    pub clear_by_line: bool,
    pub use_pager: bool,
}

#[cfg_attr(tarpaulin, skip)]
impl TermInfo {
    pub fn new(clear_by_line: bool, use_pager: bool) -> Self {
        let term = Term::stdout();
        let pager = RefCell::new(Some(Pager::new()));
        let (term_h, term_w) = term.size();
        let height = term_h as usize;
        let width = term_w as usize;

        TermInfo {
            term,
            pager,
            height,
            width,
            clear_by_line,
            use_pager,
        }
    }

    pub fn write_line(&self, s: &str) -> Result<(), Error> {
        if self.clear_by_line {
            self.term.clear_line()?;
        }
        if self.use_pager {
            writeln!(self.pager.borrow_mut().as_mut().unwrap(), "{s}")?;
        } else {
            self.term.write_line(s)?;
        }
        Ok(())
    }

    pub fn clear_screen(&self) -> Result<(), Error> {
        self.term.clear_screen()?;
        Ok(())
    }

    pub fn move_cursor_to(&self, x: usize, y: usize) -> Result<(), Error> {
        self.term.move_cursor_to(x, y)?;
        Ok(())
    }

    pub fn clear_rest_lines(&self) -> Result<(), Error> {
        for _ in 0..self.height {
            self.term.clear_line()?;
            self.term.move_cursor_down(1)?;
        }
        Ok(())
    }
}
