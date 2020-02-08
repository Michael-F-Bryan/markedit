use crate::matchers::Matcher;
use pulldown_cmark::Event;

/// A [`Matcher`] which will match the start of the next top-level element after
/// some inner [`Matcher`] matches.
#[derive(Debug, Clone, PartialEq)]
pub struct StartOfNextLine<M> {
    inner: M,
    state: State,
    current_nesting_level: usize,
}

impl<M> StartOfNextLine<M> {
    pub const fn new(inner: M) -> Self {
        StartOfNextLine {
            inner,
            state: State::WaitingForFirstMatch,
            current_nesting_level: 0,
        }
    }

    fn update_nesting(&mut self, event: &Event<'_>) {
        match event {
            Event::Start(_) => self.current_nesting_level += 1,
            Event::End(_) => self.current_nesting_level -= 1,
            _ => {},
        }
    }
}

impl<M: Matcher> StartOfNextLine<M> {
    fn process_with_inner(&mut self, event: &Event<'_>) {
        if self.inner.matches_event(event) {
            self.state = State::LookingForLastEndTag;
        }
    }
}

impl<M: Matcher> Matcher for StartOfNextLine<M> {
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        self.update_nesting(event);

        match self.state {
            State::WaitingForFirstMatch => {
                self.process_with_inner(event);
            },
            State::LookingForLastEndTag => {
                if self.current_nesting_level == 0 {
                    self.state = State::FoundLastEndTag;
                }
            },
            State::FoundLastEndTag => {
                self.state = State::WaitingForFirstMatch;
                return true;
            },
        }

        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    WaitingForFirstMatch,
    LookingForLastEndTag,
    FoundLastEndTag,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn match_start_of_line_after_heading() {
        let src = "# Heading \nSome Text";
        let events: Vec<_> = Parser::new(src).collect();
        let mut matcher = StartOfNextLine::new(crate::exact_text("Heading"));

        let got = matcher.first_match(&events).unwrap();

        assert_eq!(got, 3);
        // we're on the first start tag, so at nesting level 1
        assert_eq!(matcher.current_nesting_level, 1);
        assert_eq!(matcher.state, State::WaitingForFirstMatch);
    }
}
