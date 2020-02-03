mod rewritten;

pub use rewritten::{rewrite, Writer};

use crate::Matcher;
use pulldown_cmark::{Event, Tag};

/// Something which can rewrite events.
pub trait Rewriter<'a> {
    /// Process a single [`Event`].
    ///
    /// This may mean ignoring it, mutating it, or adding new events to the
    /// [`Writer`]'s buffer.
    ///
    /// The [`Writer`] is used as a temporary buffer that will then be streamed
    /// to the user via [`rewrite()`].
    fn rewrite(&mut self, event: Event<'a>, writer: &mut Writer<'a>);
}

impl<'a, F> Rewriter<'a> for F
where
    F: FnMut(Event<'a>, &mut Writer<'a>),
{
    fn rewrite(&mut self, event: Event<'a>, writer: &mut Writer<'a>) {
        self(event, writer);
    }
}

/// Inserts some markdown text after whatever is matched by the [`Matcher`].
///
/// # Examples
///
/// ```rust
/// use markedit::Matcher;
/// let src = "# Heading\nsome text\n";
///
/// let first_line_after_heading = markedit::text("Heading")
///     .then_start_of_next_line();
/// let rewriter = markedit::insert_after("## Second Heading", first_line_after_heading);
///
/// let events = markedit::parse(src);
/// let rewritten: Vec<_> = markedit::rewrite(events, rewriter).collect();
///
/// // if everything went to plan, the output should contain "Second Heading"
/// assert!(markedit::text("Second Heading").first_match(&rewritten).is_some());
/// ```
pub fn insert_after<'src, M, S>(
    markdown_text: S,
    matcher: M,
) -> impl Rewriter<'src> + 'src
where
    M: Matcher + 'src,
    S: AsRef<str> + 'src,
{
    let mut matcher = matcher;
    let inserted_events: Vec<Event<'static>> =
        crate::parse(markdown_text.as_ref())
            .map(owned_event)
            .collect();

    move |ev: Event<'src>, writer: &mut Writer<'src>| {
        if matcher.process_next(&ev) {
            writer.extend(inserted_events.clone());
        }
        writer.push(ev);
    }
}

fn owned_event(ev: Event<'_>) -> Event<'static> {
    match ev {
        Event::Start(tag) => Event::Start(owned_tag(tag)),
        Event::End(tag) => Event::End(owned_tag(tag)),
        Event::Text(s) => Event::Text(s.into_string().into()),
        Event::Code(s) => Event::Text(s.into_string().into()),
        Event::Html(s) => Event::Text(s.into_string().into()),
        Event::FootnoteReference(s) => {
            Event::FootnoteReference(s.into_string().into())
        },
        Event::SoftBreak => Event::SoftBreak,
        Event::HardBreak => Event::HardBreak,
        Event::Rule => Event::Rule,
        Event::TaskListMarker(t) => Event::TaskListMarker(t),
    }
}

fn owned_tag(tag: Tag<'_>) -> Tag<'static> {
    match tag {
        Tag::Paragraph => Tag::Paragraph,
        Tag::Heading(h) => Tag::Heading(h),
        Tag::BlockQuote => Tag::BlockQuote,
        Tag::CodeBlock(s) => Tag::CodeBlock(s.into_string().into()),
        Tag::List(u) => Tag::List(u),
        Tag::Item => Tag::Item,
        Tag::FootnoteDefinition(s) => Tag::CodeBlock(s.into_string().into()),
        Tag::Table(alignment) => Tag::Table(alignment),
        Tag::TableHead => Tag::TableHead,
        Tag::TableRow => Tag::TableRow,
        Tag::TableCell => Tag::TableCell,
        Tag::Emphasis => Tag::Emphasis,
        Tag::Strong => Tag::Strong,
        Tag::Strikethrough => Tag::Strikethrough,
        Tag::Link(t, url, title) => {
            Tag::Link(t, url.into_string().into(), title.into_string().into())
        },
        Tag::Image(t, url, alt) => {
            Tag::Image(t, url.into_string().into(), alt.into_string().into())
        },
    }
}
