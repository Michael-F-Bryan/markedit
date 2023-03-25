mod and;
mod falling_edge;
mod heading;
mod one_shot;
mod start_of_next_line;

pub use and::And;
pub use falling_edge::FallingEdge;
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
    /// Evaluate this predicate against an event from an [`Event`] stream.
    fn matches_event(&mut self, event: &Event<'_>) -> bool;

    /// Find the index of the first [`Event`] which is matched by this
    /// predicate.
    fn first_match<'src, I, E>(mut self, events: I) -> Option<usize>
    where
        Self: Sized,
        I: IntoIterator<Item = E> + 'src,
        E: Borrow<Event<'src>> + 'src,
    {
        events
            .into_iter()
            .position(|ev| self.matches_event(ev.borrow()))
    }

    /// Checks whether this [`Matcher`] matches anything in a stream of
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
        events.into_iter().any(|ev| self.matches_event(ev.borrow()))
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

    /// Get a [`Matcher`] which returns `true` when `self` goes from `true` to
    /// `false`.
    fn falling_edge(self) -> FallingEdge<Self>
    where
        Self: Sized,
    {
        FallingEdge::new(self)
    }

    /// Get a [`Matcher`] which matches when `self` and `other` both match.
    fn and<M>(self, other: M) -> And<Self, M>
    where
        Self: Sized,
        M: Matcher,
    {
        And::new(self, other)
    }

    /// Borrows the [`Matcher`] , rather than consuming it.
    ///
    /// This allows you to apply [`Matcher`] adaptors while retaining ownership
    /// of the original [`Matcher`].
    fn by_ref(&mut self) -> Ref<'_, Self>
    where
        Self: Sized,
    {
        Ref(self)
    }
}

impl<F> Matcher for F
where
    F: FnMut(&Event<'_>) -> bool,
{
    fn matches_event(&mut self, event: &Event<'_>) -> bool { self(event) }
}

/// A [`Matcher`] which matches everything.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Always;

impl Matcher for Always {
    fn matches_event(&mut self, _event: &Event<'_>) -> bool { true }
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
pub fn match_indices<'ev, M, I>(
    mut matcher: M,
    events: I,
) -> impl Iterator<Item = usize> + 'ev
where
    M: Matcher + 'ev,
    I: IntoIterator + 'ev,
    I::Item: Borrow<Event<'ev>> + 'ev,
{
    events
        .into_iter()
        .enumerate()
        .filter_map(move |(i, event)| {
            if matcher.matches_event(event.borrow()) {
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
    end: E,
    events: &'ev [Event<'ev>],
) -> Option<&'ev [Event<'ev>]>
where
    S: Matcher,
    E: Matcher,
{
    if let Some(start_ix) = match_indices(start, events).next() {
        let rest = &events[start_ix..];

        return Some(
            end.first_match(rest)
                .map_or(rest, |end_ix| &rest[..=end_ix]),
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

/// A glorified `&mut Matcher`.
///
/// This is the return value for [`Matcher::by_ref()`], you won't normally use
/// it directly.
#[derive(Debug)]
pub struct Ref<'a, M>(&'a mut M);

impl<'a, M> Matcher for Ref<'a, M>
where
    M: Matcher,
{
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        self.0.matches_event(event)
    }
}
