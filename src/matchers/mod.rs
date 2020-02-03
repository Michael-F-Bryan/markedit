mod one_shot;
mod start_of_next_line;
mod text;

pub use one_shot::OneShot;
pub use start_of_next_line::StartOfNextLine;
pub use text::Text;

use pulldown_cmark::Event;

/// A predicate which can be fed a stream of [`Event<'_>`]s and tell you whether
/// they match a desired condition.
///
/// Individual [`Matcher`]s may choose to return [`MatchOutcome::Match`] more
/// than once.
pub trait Matcher {
    fn process_next(&mut self, event: &Event<'_>) -> MatchOutcome;

    fn first_match(&mut self, events: &[Event<'_>]) -> Option<usize> {
        events
            .iter()
            .position(|ev| self.process_next(ev) == MatchOutcome::Match)
    }

    /// Returns a [`Matcher`] which will wait until `self` matches, then return
    /// [`MatchOutcome::Match`] at the start of the next top-level element.
    fn then_start_of_next_line(self) -> StartOfNextLine<Self>
    where
        Self: Sized,
    {
        StartOfNextLine::new(self)
    }

    /// Wraps `self` in a [`Matcher`] which will only ever return
    /// [`MatchOutcome::Match`] once.
    fn fuse(self) -> OneShot<Self>
    where
        Self: Sized,
    {
        OneShot::new(self)
    }
}

impl<F> Matcher for F
where
    F: FnMut(&Event<'_>) -> bool,
{
    fn process_next(&mut self, event: &Event<'_>) -> MatchOutcome {
        if self(event) {
            MatchOutcome::Match
        } else {
            MatchOutcome::NotFound
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MatchOutcome {
    Match,
    NotFound,
}

pub fn match_indices<'ev, M>(
    mut matcher: M,
    events: &'ev [Event<'ev>],
) -> impl Iterator<Item = usize> + 'ev
where
    M: Matcher + 'ev,
{
    events.iter().enumerate().filter_map(move |(i, event)| {
        match matcher.process_next(event) {
            MatchOutcome::Match => Some(i),
            MatchOutcome::NotFound => None,
        }
    })
}

/// Gets all [`Event`]s between (inclusive) two matchers.
pub fn between<'ev, S, E>(
    start: S,
    mut end: E,
    events: &'ev [Event<'ev>],
) -> Option<&'ev [Event<'ev>]>
where
    S: Matcher,
    E: Matcher,
{
    for start_ix in match_indices(start, events) {
        let rest = &events[start_ix..];

        return Some(
            end.first_match(rest)
                .map(|end_ix| &rest[..end_ix + 1])
                .unwrap_or(rest),
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::Parser;

    #[test]
    fn get_events_between_headers() {
        let src = "# Header\nnormal text\n# End";
        let events: Vec<_> = Parser::new(src).collect();
        let start = Text::literal("Header");
        let end = Text::literal("End");

        let got = between(start, end, &events).unwrap();

        assert_eq!(got.first().unwrap(), &Event::Text("Header".into()));
        assert_eq!(got.last().unwrap(), &Event::Text("End".into()));
        assert_eq!(got.len(), 7);
    }
}
