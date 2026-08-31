use crate::columns::ConfigColumnKind;
use crate::util::find_column_kind;
use anyhow::{Error, bail};

/// Matching operator of a column predicate.
pub enum MatchOp {
    /// `==`: the column content must equal the value exactly.
    Exact,
    /// `~=`: the column content must contain the value as a substring.
    Partial,
}

impl MatchOp {
    const EXACT: &'static str = "==";
    const PARTIAL: &'static str = "~=";

    fn token(&self) -> &'static str {
        match self {
            MatchOp::Exact => MatchOp::EXACT,
            MatchOp::Partial => MatchOp::PARTIAL,
        }
    }
}

/// A predicate constraining a single column, e.g. `user==root` or
/// `command~=dockerd`.
pub struct ColumnPredicate<'a> {
    pub kind: ConfigColumnKind,
    pub op: MatchOp,
    pub value: &'a str,
}

/// A single parsed search term borrowing from the raw keyword.
pub enum SearchTerm<'a> {
    /// A bare keyword matched against every searchable column.
    Plain(&'a str),
    /// A predicate matched against one named column.
    Column(ColumnPredicate<'a>),
}

impl<'a> SearchTerm<'a> {
    /// Parse a raw keyword into a [`SearchTerm`].
    ///
    /// A keyword containing a [`MatchOp`] operator is interpreted as a column
    /// predicate; the substring before the leftmost operator names the column
    /// and the remainder is the value. Any other keyword is a plain keyword.
    ///
    /// Returns an error if a predicate names a column that does not exist.
    pub fn parse(raw: &'a str) -> Result<Self, Error> {
        let exact = raw.find(MatchOp::EXACT);
        let partial = raw.find(MatchOp::PARTIAL);

        let (op, idx) = match (exact, partial) {
            (Some(i), Some(j)) if i <= j => (MatchOp::Exact, i),
            (Some(_), Some(j)) => (MatchOp::Partial, j),
            (Some(i), None) => (MatchOp::Exact, i),
            (None, Some(j)) => (MatchOp::Partial, j),
            (None, None) => return Ok(SearchTerm::Plain(raw)),
        };

        let column = &raw[..idx];
        let value = &raw[idx + op.token().len()..];

        if column.is_empty() {
            bail!("empty column name in predicate \"{raw}\"");
        }

        let kind = find_column_kind(column)
            .ok_or_else(|| anyhow::anyhow!("unknown column \"{column}\" in predicate \"{raw}\""))?;

        Ok(SearchTerm::Column(ColumnPredicate { kind, op, value }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_keyword() {
        assert!(matches!(
            SearchTerm::parse("firefox").unwrap(),
            SearchTerm::Plain("firefox")
        ));
    }

    #[test]
    fn parses_exact_predicate() {
        let term = SearchTerm::parse("user==root").unwrap();
        let SearchTerm::Column(pred) = term else {
            panic!("expected column predicate");
        };
        assert!(matches!(pred.op, MatchOp::Exact));
        assert_eq!(pred.value, "root");
    }

    #[test]
    fn parses_partial_predicate() {
        let term = SearchTerm::parse("command~=docker").unwrap();
        let SearchTerm::Column(pred) = term else {
            panic!("expected column predicate");
        };
        assert!(matches!(pred.op, MatchOp::Partial));
        assert_eq!(pred.value, "docker");
    }

    #[test]
    fn leftmost_operator_wins_and_value_may_contain_equals() {
        let term = SearchTerm::parse("command~=a==b").unwrap();
        let SearchTerm::Column(pred) = term else {
            panic!("expected column predicate");
        };
        assert!(matches!(pred.op, MatchOp::Partial));
        assert_eq!(pred.value, "a==b");
    }

    #[test]
    fn unknown_column_is_error() {
        assert!(SearchTerm::parse("nosuchcolumn==x").is_err());
    }

    #[test]
    fn empty_column_is_error() {
        assert!(SearchTerm::parse("==x").is_err());
    }
}
