use crate::Matcher;
use pulldown_cmark::Event;

/// A [`Matcher`] which only returns `true` when both inner [`Matcher`]s do.
#[derive(Debug, Clone, PartialEq)]
pub struct And<L, R> {
    left: L,
    right: R,
}

impl<L, R> And<L, R> {
    /// Create a new [`And`] matcher.
    pub const fn new(left: L, right: R) -> Self { And { left, right } }
}

impl<L: Matcher, R: Matcher> Matcher for And<L, R> {
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        // Note: We explicitly *don't* want to use short-circuiting logic here
        // because each inner matcher needs to see the entire event stream
        let left = self.left.matches_event(event);
        let right = self.right.matches_event(event);

        left && right
    }
}
