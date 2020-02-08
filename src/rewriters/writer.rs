use pulldown_cmark::Event;
use std::collections::VecDeque;

#[allow(unused_imports)] // for rustdoc
use crate::Rewriter;

/// The output buffer given to [`Rewriter::rewrite_event()`].
#[derive(Debug)]
pub struct Writer<'a> {
    pub(crate) buffer: VecDeque<Event<'a>>,
}

impl<'a> Writer<'a> {
    pub(crate) fn new() -> Writer<'a> {
        Writer {
            buffer: VecDeque::new(),
        }
    }

    /// Queue an [`Event`] to be emitted.
    pub fn push(&mut self, event: Event<'a>) { self.buffer.push_back(event); }
}

impl<'a> Extend<Event<'a>> for Writer<'a> {
    fn extend<I: IntoIterator<Item = Event<'a>>>(&mut self, iter: I) {
        self.buffer.extend(iter);
    }
}
