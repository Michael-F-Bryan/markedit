use crate::matchers::Matcher;
use pulldown_cmark::Event;

/// A [`Matcher`] which will only ever return `true` once.
#[derive(Debug, Clone, PartialEq)]
pub struct OneShot<M> {
    inner: M,
    already_triggered: bool,
}

impl<M> OneShot<M> {
    /// Create a [`OneShot`] matcher.
    pub const fn new(inner: M) -> Self {
        OneShot {
            inner,
            already_triggered: false,
        }
    }
}

impl<M: Matcher> Matcher for OneShot<M> {
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        if self.already_triggered {
            return false;
        }

        let got = self.inner.matches_event(event);

        if got {
            self.already_triggered = true;
        }

        got
    }
}
