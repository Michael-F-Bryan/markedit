use crate::matchers::{Matcher};
use pulldown_cmark::Event;

/// A [`Matcher`] which will only ever return a [`MatchOutcome::Match`] once.
#[derive(Debug, Clone, PartialEq)]
pub struct OneShot<M> {
    inner: M,
    already_triggered: bool,
}

impl<M> OneShot<M> {
    pub const fn new(inner: M) -> Self {
        OneShot {
            inner,
            already_triggered: false,
        }
    }
}

impl<M: Matcher> Matcher for OneShot<M> {
    fn process_next(&mut self, event: &Event<'_>) -> bool {
        if self.already_triggered {
            return false;
        }

        let got = self.inner.process_next(event);

        if got {
            self.already_triggered = true;
        }

        got
    }
}
