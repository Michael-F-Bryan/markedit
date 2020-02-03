use crate::matchers::{MatchOutcome, Matcher};
use pulldown_cmark::Event;
use std::borrow::Cow;

/// A [`Matcher`] that matches an *exact* string of text.
#[derive(Debug, Clone, PartialEq)]
pub struct Text<'a>(Cow<'a, str>);

impl Text<'static> {
    pub const fn literal(text: &'static str) -> Self {
        Text(Cow::Borrowed(text))
    }
}

impl<'a> Text<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(text: S) -> Self { Text(text.into()) }
}

impl<'a> Matcher for Text<'a> {
    fn process_next(&mut self, event: &Event<'_>) -> MatchOutcome {
        match event {
            Event::Text(text) if text.as_ref() == self.0.as_ref() => {
                MatchOutcome::Match
            },
            _ => MatchOutcome::NotFound,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn match_exact_text() {
        let src = "Hello, World!";
        let events: Vec<_> = Parser::new(src).collect();
        let mut matcher = Text::literal(src);

        assert_eq!(matcher.process_next(&events[0]), MatchOutcome::NotFound);
        assert_eq!(matcher.process_next(&events[1]), MatchOutcome::Match);
        assert_eq!(matcher.process_next(&events[2]), MatchOutcome::NotFound);
    }
}
