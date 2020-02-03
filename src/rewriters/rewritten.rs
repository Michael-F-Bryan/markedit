use crate::Rewriter;
use pulldown_cmark::Event;
use std::collections::VecDeque;

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
    Rewritten {
        rewriter,
        events: events.into_iter(),
        writer: Writer::new(),
    }
}

struct Rewritten<'src, E, R>
where
    E: Iterator<Item = Event<'src>>,
{
    events: E,
    rewriter: R,
    writer: Writer<'src>,
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
        self.rewriter.rewrite(event, &mut self.writer);

        self.writer.buffer.pop_front()
    }
}

#[derive(Debug)]
pub struct Writer<'a> {
    buffer: VecDeque<Event<'a>>,
}

impl<'a> Writer<'a> {
    fn new() -> Writer<'a> {
        Writer {
            buffer: VecDeque::new(),
        }
    }

    pub fn push(&mut self, event: Event<'a>) { self.buffer.push_back(event); }
}

impl<'a> Extend<Event<'a>> for Writer<'a> {
    fn extend<I: IntoIterator<Item = Event<'a>>>(&mut self, iter: I) {
        self.buffer.extend(iter);
    }
}
