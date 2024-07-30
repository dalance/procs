use anyhow::Error;
use console::Term;
use minus::input::{self, HashedEventRegister, InputEvent};
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

impl TermInfo {
    pub fn new(clear_by_line: bool, use_pager: bool) -> Result<Self, Error> {
        let term = Term::stdout();
        let pager = RefCell::new(Some(gen_pager()?));
        let (term_h, term_w) = term.size();
        let height = term_h as usize;
        let width = term_w as usize;

        Ok(TermInfo {
            term,
            pager,
            height,
            width,
            clear_by_line,
            use_pager,
        })
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

fn gen_pager() -> Result<Pager, Error> {
    let pager = Pager::new();
    let mut input_register = HashedEventRegister::default();
    input::generate_default_bindings(&mut input_register);

    // less compatible keybindings
    input_register.add_key_events(&["y", "c-y", "c-p", "c-k"], |_, ps| {
        let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(position))
    });

    input_register.add_key_events(&["c-n", "e", "c-e", "c-j"], |_, ps| {
        let position = ps.prefix_num.parse::<usize>().unwrap_or(1);
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(position))
    });
    input_register.add_key_events(&["b", "c-b"], |_, ps| {
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_sub(ps.rows - 1))
    });
    input_register.add_key_events(&["c-v", "f", "c-f"], |_, ps| {
        InputEvent::UpdateUpperMark(ps.upper_mark.saturating_add(ps.rows - 1))
    });
    input_register.add_key_events(&["<"], |_, _| InputEvent::UpdateUpperMark(0));
    input_register.add_key_events(&[">"], |_, ps| {
        let mut position = ps
            .prefix_num
            .parse::<usize>()
            .unwrap_or(usize::MAX)
            // Reduce 1 here, because line numbering starts from 1
            // while upper_mark starts from 0
            .saturating_sub(1);

        if position == 0 {
            position = usize::MAX;
        }
        InputEvent::UpdateUpperMark(position)
    });

    pager.set_input_classifier(Box::new(input_register))?;
    Ok(pager)
}
