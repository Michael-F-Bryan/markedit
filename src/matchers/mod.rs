mod heading;
mod one_shot;
mod start_of_next_line;

pub use heading::Heading;
pub use one_shot::OneShot;
pub use start_of_next_line::StartOfNextLine;

use pulldown_cmark::{Event, Tag};
use std::borrow::Borrow;

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

    /// Checks whether this [`Matcher`] would match anything in a stream of
    /// [`Event`]s.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use markedit::Matcher;
    /// let src = "# Heading\nsome text";
    /// let matcher = markedit::exact_text("some text");
    ///
    /// assert!(matcher.is_in(markedit::parse(src)));
    /// ```
    fn is_in<'src, I, E>(mut self, events: I) -> bool
    where
        I: IntoIterator<Item = E> + 'src,
        E: Borrow<Event<'src>>,
        Self: Sized,
    {
        events.into_iter().any(|ev| self.process_next(ev.borrow()))
    }

    /// Returns a [`Matcher`] which will wait until `self` matches, then return
    /// `true` at the start of the next top-level element.
    fn then_start_of_next_line(self) -> StartOfNextLine<Self>
    where
        Self: Sized,
    {
        StartOfNextLine::new(self)
    }

    /// Matches any header where the inner [`Matcher`] matches.
    fn inside_any_header(self) -> Heading<Self>
    where
        Self: Sized,
    {
        Heading::any_matching(self)
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

/// A [`Matcher`] which matches everything.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Always;

impl Matcher for Always {
    fn process_next(&mut self, _event: &Event<'_>) -> bool { true }
}

/// Get an iterator over the indices of matching events.
///
/// # Examples
///
/// ```rust
/// use pulldown_cmark::Event;
///
/// let matcher = markedit::exact_text("Header");
/// let src = "# Header\nsome text\n# Header";
/// let events: Vec<_> = markedit::parse(src).collect();
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
/// let events: Vec<_> = markedit::parse(src).collect();
/// let start = markedit::exact_text("Header");
/// let end = markedit::exact_text("End");
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

/// Match an [`Event::Text`] event with this *exact* text.
///
/// Not to be confused with [`text_containing()`].
///
/// ```rust
/// use markedit::Matcher;
/// use pulldown_cmark::Event;
///
/// assert_eq!(
///     markedit::exact_text("Something").is_in(markedit::parse("Something")),
///     true,
/// );
/// assert_eq!(
///     markedit::exact_text("Something").is_in(markedit::parse("Something Else")),
///     false,
/// );
/// ```
pub fn exact_text<S: AsRef<str>>(needle: S) -> impl Matcher {
    text(move |text| AsRef::<str>::as_ref(text) == needle.as_ref())
}

/// Match an [`Event::Text`] event which *contains* the provided string.
///
/// Not to be confused with [`exact_text()`].
///
/// ```rust
/// use markedit::Matcher;
/// use pulldown_cmark::Event;
///
/// assert_eq!(
///     markedit::text_containing("Something").is_in(markedit::parse("Something")),
///     true,
/// );
/// assert_eq!(
///     markedit::text_containing("Something").is_in(markedit::parse("Something Else")),
///     true,
/// );
/// ```
pub fn text_containing<S: AsRef<str>>(needle: S) -> impl Matcher {
    text(move |text| text.contains(needle.as_ref()))
}

/// Match a [`Event::Text`] node using an arbitrary predicate.
pub fn text<P>(mut predicate: P) -> impl Matcher
where
    P: FnMut(&str) -> bool,
{
    move |ev: &Event<'_>| match ev {
        Event::Text(text) => predicate(text.as_ref()),
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
/// let events: Vec<_> = markedit::parse(src).collect();
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
