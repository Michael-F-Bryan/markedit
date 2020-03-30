mod rewritten;
mod writer;

pub use rewritten::{rewrite, Rewritten};
pub use writer::Writer;

use crate::Matcher;
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

/// Something which can rewrite events.
pub trait Rewriter<'src> {
    /// Process a single [`Event`].
    ///
    /// This may mean ignoring it, mutating it, or adding new events to the
    /// [`Writer`]'s buffer.
    ///
    /// The [`Writer`] is used as a temporary buffer that will then be streamed
    /// to the user via [`rewrite()`].
    fn rewrite_event(&mut self, event: Event<'src>, writer: &mut Writer<'src>);

    /// Use this [`Rewriter`] to rewrite a stream of [`Event`]s.
    fn rewrite<E>(self, events: E) -> Rewritten<'src, E, Self>
    where
        Self: Sized,
        E: IntoIterator<Item = Event<'src>>,
    {
        Rewritten::new(events, self)
    }
}

impl<'src, F> Rewriter<'src> for F
where
    F: FnMut(Event<'src>, &mut Writer<'src>),
{
    fn rewrite_event(&mut self, event: Event<'src>, writer: &mut Writer<'src>) {
        self(event, writer);
    }
}

/// Inserts some markdown text before whatever is matched by the [`Matcher`].
///
/// # Examples
///
/// ```rust
/// use markedit::Matcher;
/// let src = "# Heading\nsome text\n";
///
/// let first_line_after_heading = markedit::exact_text("Heading")
///     .falling_edge();
/// let rewriter = markedit::insert_markdown_before(
///     "## Second Heading",
///     first_line_after_heading,
/// );
///
/// let events = markedit::parse(src);
/// let rewritten: Vec<_> = markedit::rewrite(events, rewriter).collect();
///
/// // if everything went to plan, the output should contain "Second Heading"
/// assert!(markedit::exact_text("Second Heading").is_in(&rewritten));
/// ```
pub fn insert_markdown_before<'src, M, S>(
    markdown_text: S,
    matcher: M,
) -> impl Rewriter<'src> + 'src
where
    M: Matcher + 'src,
    S: AsRef<str> + 'src,
{
    let events = crate::parse(markdown_text.as_ref())
        .map(owned_event)
        .collect();
    insert_before(events, matcher)
}

/// Splice some events into the resulting event stream before every match.
pub fn insert_before<'src, M>(
    to_insert: Vec<Event<'src>>,
    mut matcher: M,
) -> impl Rewriter<'src> + 'src
where
    M: Matcher + 'src,
{
    move |ev: Event<'src>, writer: &mut Writer<'src>| {
        if matcher.matches_event(&ev) {
            writer.extend(to_insert.iter().cloned());
        }
        writer.push(ev);
    }
}

/// A [`Rewriter`] which lets you update a [`Event::Text`] node based on some
/// predicate.
pub fn change_text<'src, M, F, S>(
    mut predicate: M,
    mut mutator: F,
) -> impl Rewriter<'src> + 'src
where
    M: FnMut(&str) -> bool + 'src,
    F: FnMut(CowStr<'src>) -> S + 'src,
    S: Into<CowStr<'src>>,
{
    move |ev: Event<'src>, writer: &mut Writer<'src>| match ev {
        Event::Text(text) => {
            let text = if predicate(text.as_ref()) {
                mutator(text).into()
            } else {
                text
            };
            writer.push(Event::Text(text));
        },
        _ => writer.push(ev),
    }
}

fn owned_event(ev: Event<'_>) -> Event<'static> {
    match ev {
        Event::Start(tag) => Event::Start(owned_tag(tag)),
        Event::End(tag) => Event::End(owned_tag(tag)),
        Event::Text(s) => Event::Text(owned_cow_str(s)),
        Event::Code(s) => Event::Code(owned_cow_str(s)),
        Event::Html(s) => Event::Html(owned_cow_str(s)),
        Event::FootnoteReference(s) => {
            Event::FootnoteReference(owned_cow_str(s))
        },
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(t) => Event::TaskListMarker(t),
    }
}

fn owned_cow_str(s: CowStr<'_>) -> CowStr<'static> {
    match s {
        CowStr::Borrowed(_) => CowStr::from(s.into_string()),
        CowStr::Boxed(boxed) => CowStr::Boxed(boxed),
        CowStr::Inlined(inlined) => CowStr::Inlined(inlined),
    }
}

fn owned_tag(tag: Tag<'_>) -> Tag<'static> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading(h) => Tag::Heading(h),
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(CodeBlockKind::Indented) => {
            Tag::CodeBlock(CodeBlockKind::Indented)
        },
        Tag::CodeBlock(CodeBlockKind::Fenced(s)) => {
            Tag::CodeBlock(CodeBlockKind::Fenced(owned_cow_str(s)))
        },
        Tag::List(u) => Tag::List(u),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(s) => Tag::FootnoteDefinition(owned_cow_str(s)),
        Tag::Table(alignment) => Tag::Table(alignment),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link(t, url, title) => {
            Tag::Link(t, owned_cow_str(url), owned_cow_str(title))
        },
        Tag::Image(t, url, alt) => {
            Tag::Image(t, owned_cow_str(url), owned_cow_str(alt))
        },
    }
}
