use anyhow::Error;

pub enum SearchRegex {
    Fast(regex::Regex),
    Fancy(fancy_regex::Regex),
}

impl SearchRegex {
    pub fn new(pattern: &str, ignore_case: bool) -> Result<Self, Error> {
        if let Ok(regex) = regex::RegexBuilder::new(pattern)
            .case_insensitive(ignore_case)
            .build()
        {
            return Ok(Self::Fast(regex));
        }

        let regex = fancy_regex::RegexBuilder::new(pattern)
            .case_insensitive(ignore_case)
            .build()?;
        Ok(Self::Fancy(regex))
    }

    pub fn is_match(&self, text: &str) -> Result<bool, Error> {
        match self {
            Self::Fast(regex) => Ok(regex.is_match(text)),
            Self::Fancy(regex) => Ok(regex.is_match(text)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SearchRegex;

    #[test]
    fn uses_fast_engine_for_standard_regex() {
        let regex = SearchRegex::new("proc.*", false).unwrap();

        assert!(matches!(regex, SearchRegex::Fast(_)));
        assert!(regex.is_match("procs").unwrap());
    }

    #[test]
    fn falls_back_to_fancy_engine_for_lookahead() {
        let regex = SearchRegex::new("proc(?=s)", false).unwrap();

        assert!(matches!(regex, SearchRegex::Fancy(_)));
        assert!(regex.is_match("procs").unwrap());
        assert!(!regex.is_match("procx").unwrap());
    }

    #[test]
    fn falls_back_to_fancy_engine_for_negative_lookahead() {
        let regex = SearchRegex::new("proc(?!x)", false).unwrap();

        assert!(matches!(regex, SearchRegex::Fancy(_)));
        assert!(regex.is_match("procs").unwrap());
        assert!(!regex.is_match("procx").unwrap());
    }
}
