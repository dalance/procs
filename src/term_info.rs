use console::Term;

pub struct TermInfo {
    pub term: Term,
    pub height: usize,
    pub width: usize,
}

impl TermInfo {
    pub fn new() -> Self {
        let term = Term::stdout();
        let (term_h, term_w) = term.size();
        let height = term_h as usize;
        let width = term_w as usize;

        TermInfo {
            term,
            height,
            width,
        }
    }
}
