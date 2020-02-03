mod one_shot;
mod start_of_next_line;

pub use one_shot::OneShot;
pub use start_of_next_line::StartOfNextLine;

use pulldown_cmark::{Event, Tag};

/// A predicate which can be fed a stream of [`Event`]s and tell you whether
/// they match a desired condition.
///
/// Individual [`Matcher`]s may choose to return `true` more than once.
///
/// Any function which accepts a [`Event`] reference and returns a `bool` can be
/// used as a [`Matcher`].
///
/// ```rust
/// # use markedit::Matcher;
/// # use pulldown_cmark::Event;
/// fn assert_is_matcher(_: impl Matcher) {}
///
/// assert_is_matcher(|ev: &Event<'_>| true);
/// ```
pub trait Matcher {
    fn process_next(&mut self, event: &Event<'_>) -> bool;

    /// Find the index of the first [`Event`] which is matched by this
    /// predicate.
    fn first_match(&mut self, events: &[Event<'_>]) -> Option<usize> {
        events.iter().position(|ev| self.process_next(ev))
    }

    /// Returns a [`Matcher`] which will wait until `self` matches, then return
    /// `true` at the start of the next top-level element.
    fn then_start_of_next_line(self) -> StartOfNextLine<Self>
    where
        Self: Sized,
    {
        StartOfNextLine::new(self)
    }

    /// Wraps `self` in a [`Matcher`] which will only ever return `true` once.
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
    fn process_next(&mut self, event: &Event<'_>) -> bool { self(event) }
}

/// Get an iterator over the indices of matching events.
///
/// # Examples
///
/// ```rust
/// use pulldown_cmark::Event;
///
/// let matcher = markedit::text("Header");
/// let src = "# Header\nsome text\n# Header";
/// let events: Vec<_> = markedit::parse_events(src).collect();
///
/// let indices: Vec<_> = markedit::match_indices(matcher, &events).collect();
///
/// assert_eq!(indices.len(), 2);
///
/// for ix in indices {
///     assert_eq!(events[ix], Event::Text("Header".into()));
/// }
/// ```
pub fn match_indices<'ev, M>(
    mut matcher: M,
    events: &'ev [Event<'ev>],
) -> impl Iterator<Item = usize> + 'ev
where
    M: Matcher + 'ev,
{
    events.iter().enumerate().filter_map(move |(i, event)| {
        if matcher.process_next(event) {
            Some(i)
        } else {
            None
        }
    })
}

/// Gets all [`Event`]s between (inclusive) two matchers.
///
/// # Examples
///
/// ```rust
/// use pulldown_cmark::Event;
///
/// let src = "# Header\nnormal text\n# End";
///
/// let events: Vec<_> = markedit::parse_events(src).collect();
/// let start = markedit::text("Header");
/// let end = markedit::text("End");
///
/// let got = markedit::between(start, end, &events).unwrap();
///
/// assert_eq!(got.first().unwrap(), &Event::Text("Header".into()));
/// assert_eq!(got.last().unwrap(), &Event::Text("End".into()));
/// assert_eq!(got.len(), 7);
/// ```
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

pub fn text_containing<S: AsRef<str>>(needle: S) -> impl Matcher {
    move |ev: &Event<'_>| match ev {
        Event::Text(haystack) => haystack.as_ref().contains(needle.as_ref()),
        _ => false,
    }
}

/// Matches the start of a link who's URL contains a certain string.
///
/// # Examples
///
/// ```rust
/// # use markedit::Matcher;
/// use pulldown_cmark::{Event, Tag};
///
/// let src = "Some text containing [a link to google](https://google.com/).";
/// let mut matcher = markedit::link_with_url_containing("google.com");
///
/// let events: Vec<_> = markedit::parse_events(src).collect();
///
/// let ix = matcher.first_match(&events).unwrap();
///
/// match &events[ix] {
///     Event::Start(Tag::Link(_, url, _)) => assert_eq!(url.as_ref(), "https://google.com/"),
///     _ => unreachable!(),
/// }
/// ```
pub fn link_with_url_containing<S: AsRef<str>>(needle: S) -> impl Matcher {
    move |ev: &Event<'_>| match ev {
        Event::Start(Tag::Link(_, link, _)) => {
            link.as_ref().contains(needle.as_ref())
        },
        _ => false,
    }
}

pub fn text<S: AsRef<str>>(needle: S) -> impl Matcher {
    move |ev: &Event<'_>| match ev {
        Event::Text(text) => text.as_ref() == needle.as_ref(),
        _ => false,
    }
}
