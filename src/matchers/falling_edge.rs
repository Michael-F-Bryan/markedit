use crate::matchers::Matcher;
use pulldown_cmark::Event;

/// A [`Matcher`] which will detect the falling edge of another.
///
/// # Examples
///
/// ```rust
/// # use markedit::{FallingEdge, Matcher, pulldown_cmark::{Tag, Event}};
///
/// let matches_something = markedit::exact_text("Something");
/// let mut matcher = FallingEdge::new(matches_something);
///
/// // enter the paragraph
/// let got = matcher.matches_event(&Event::Start(Tag::Paragraph));
/// assert_eq!(got, false);
/// // then encounter some text. matches_something should have gone from false -> true
/// let got = matcher.matches_event(&Event::Text("Something".into()));
/// assert_eq!(got, false);
/// // then leave the paragraph. `matches_something` should go from true -> false
/// let got = matcher.matches_event(&Event::End(Tag::Paragraph));
/// assert_eq!(got, true, "We've entered a paragraph");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FallingEdge<M> {
    inner: M,
    previous_was_matched: bool,
}

impl<M> FallingEdge<M> {
    pub const fn new(inner: M) -> Self {
        FallingEdge {
            inner,
            previous_was_matched: false,
        }
    }
}

impl<M: Matcher> Matcher for FallingEdge<M> {
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        let current_is_matched = self.inner.matches_event(event);
        let is_falling_edge = self.previous_was_matched && !current_is_matched;
        self.previous_was_matched = current_is_matched;
        is_falling_edge
    }
}
