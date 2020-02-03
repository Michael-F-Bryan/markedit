use crate::matchers::{Always, Matcher};
use pulldown_cmark::{Event, Tag};

/// Matches the start of a heading.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading<M> {
    inner: M,
    state: State,
    level: Option<u32>,
}

impl Heading<Always> {
    /// Matches any heading.
    pub const fn any() -> Self { Heading::new(None, Always) }

    /// Matches only headings with the desired level.
    pub const fn with_level(level: u32) -> Self {
        Heading::new(Some(level), Always)
    }
}

impl<M> Heading<M> {
    pub const fn new(level: Option<u32>, inner: M) -> Self {
        Heading {
            inner,
            level,
            state: State::WaitingForHeading,
        }
    }

    fn matches_level(&self, level: u32) -> bool {
        match self.level {
            Some(expected) => level == expected,
            None => true,
        }
    }
}

impl<M: Matcher> Heading<M> {
    /// Matches any header where the inner [`Matcher`] matches.
    pub fn any_matching(inner: M) -> Self { Heading::new(None, inner) }
}

impl<M: Matcher> Matcher for Heading<M> {
    fn process_next(&mut self, event: &Event<'_>) -> bool {
        match (event, &self.state) {
            (Event::Start(Tag::Heading(level)), State::WaitingForHeading) => {
                if self.matches_level(*level) {
                    self.state = State::InsideHeading;
                }
            },
            (_, State::InsideHeading) => {
                return self.inner.process_next(event);
            },
            _ => {},
        }

        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum State {
    WaitingForHeading,
    InsideHeading,
}
