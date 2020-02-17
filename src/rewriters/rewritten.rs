use crate::{Rewriter, Writer};
use pulldown_cmark::Event;

/// The whole point.
///
/// This function takes a stream of [`Event`]s and a [`Rewriter`], and gives
/// you a stream of rewritten [`Event`]s.
pub fn rewrite<'src, E, R>(
    events: E,
    rewriter: R,
) -> impl Iterator<Item = Event<'src>> + 'src
where
    E: IntoIterator<Item = Event<'src>>,
    E::IntoIter: 'src,
    R: Rewriter<'src> + 'src,
{
    Rewritten::new(events.into_iter(), rewriter)
}

/// A stream of [`Event`]s that have been modified by a [`Rewriter`].
#[derive(Debug)]
pub struct Rewritten<'src, E, R> {
    events: E,
    rewriter: R,
    writer: Writer<'src>,
}

impl<'src, E, R> Rewritten<'src, E, R> {
    /// Create a new [`Rewritten`] iterator.
    pub fn new(events: E, rewriter: R) -> Self {
        Rewritten {
            rewriter,
            events,
            writer: Writer::new(),
        }
    }
}

impl<'src, E, R> Iterator for Rewritten<'src, E, R>
where
    E: Iterator<Item = Event<'src>>,
    R: Rewriter<'src>,
{
    type Item = Event<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        // we're still working through items buffered by the rewriter
        if let Some(ev) = self.writer.buffer.pop_front() {
            return Some(ev);
        }

        // we need to pop another event and process it
        let event = self.events.next()?;
        self.rewriter.rewrite_event(event, &mut self.writer);

        self.writer.buffer.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pulldown_cmark::Tag;

    #[test]
    fn ignore_some_events() {
        let events = vec![
            Event::Start(Tag::Paragraph),
            Event::Text("This is some text.".into()),
            Event::Start(Tag::Heading(2)),
            Event::Text("This is some more text.".into()),
        ];

        let rewritten: Vec<Event<'static>> = rewrite(
            events,
            |event: Event<'static>, writer: &mut Writer<'static>| {
                if let event @ Event::Text(_) = event {
                    writer.push(event);
                }
            },
        )
        .collect();

        assert_eq!(
            rewritten,
            vec![
                Event::Text("This is some text.".into()),
                Event::Text("This is some more text.".into()),
            ]
        );
    }
}
