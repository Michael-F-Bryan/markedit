use crate::Matcher;
use pulldown_cmark::Event;

/// Get a new stream of [`Event`]s where the things between `start` and `end`
/// are automatically rewritten.
pub fn rewrite_between<'src, S, E, I, F>(
    events: I,
    start: S,
    end: E,
    rewrite: F,
) -> impl Iterator<Item = Event<'src>>
where
    S: Matcher,
    E: Matcher,
    I: IntoIterator<Item = Event<'src>>,
    I::IntoIter: 'src,
    F: FnMut(&mut Vec<Event<'src>>),
{
    RewriteBetween {
        events: events.into_iter(),
        start,
        end,
        rewrite,
        state: State::default(),
        buffer: Vec::new(),
    }
}

/// The iterator returned by [`rewrite_between()`].
pub struct RewriteBetween<'src, S, E, I, F> {
    events: I,
    start: S,
    end: E,
    rewrite: F,
    state: State<'src>,
    buffer: Vec<Event<'src>>,
}

impl<'src, S, E, I, F> Iterator for RewriteBetween<'src, S, E, I, F>
where
    S: Matcher,
    E: Matcher,
    I: Iterator<Item = Event<'src>>,
    F: FnMut(&mut Vec<Event<'src>>),
{
    type Item = Event<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.buffer.is_empty() {
            while let Some(event) = self.events.next() {
                // temporarily swap out the current state with a dummy value so
                // we can update it
                let current_state =
                    std::mem::replace(&mut self.state, State::default());
                handle_event(
                    current_state,
                    &mut self.start,
                    &mut self.end,
                    event,
                    &mut self.buffer,
                    &mut self.rewrite,
                );
                unimplemented!()
            }
        }

        if self.buffer.is_empty() {
            None
        } else {
            Some(self.buffer.remove(0))
        }
    }
}

impl<'src, S, E, I, F> RewriteBetween<'src, S, E, I, F> {}

enum State<'src> {
    WaitingForCodeblocks,
    ReadingCodeblock { buffer: Vec<Event<'src>> },
}

impl<'src> Default for State<'src> {
    fn default() -> State<'src> { State::WaitingForCodeblocks }
}

fn handle_event<'src, S, E, F>(
    current_state: State<'src>,
    start: &mut S,
    end: &mut E,
    event: Event<'src>,
    processed: &mut Vec<Event<'src>>,
    rewrite: &mut F,
) -> State<'src>
where
    F: FnMut(&mut Vec<Event<'src>>),
    S: Matcher,
    E: Matcher,
{
    match current_state {
        State::WaitingForCodeblocks => {
            handle_waiting_for_start(event, start, processed)
        },
        State::ReadingCodeblock { buffer } => {
            handle_reading_codeblock(buffer, event, end, processed, rewrite)
        },
    }
}

fn handle_waiting_for_start<'src, S>(
    event: Event<'src>,
    start: &mut S,
    processed: &mut Vec<Event<'src>>,
) -> State<'src>
where
    S: Matcher,
{
    if start.matches_event(&event) {
        State::ReadingCodeblock {
            buffer: vec![event],
        }
    } else {
        processed.push(event);
        State::WaitingForCodeblocks
    }
}

fn handle_reading_codeblock<'src, E, F>(
    mut buffer: Vec<Event<'src>>,
    event: Event<'src>,
    end: &mut E,
    processed: &mut Vec<Event<'src>>,
    transform: &mut F,
) -> State<'src>
where
    F: FnMut(&mut Vec<Event<'src>>),
    E: Matcher,
{
    if end.matches_event(&event) {
        transform(&mut buffer);
        processed.extend(buffer.drain(..));
        State::WaitingForCodeblocks
    } else {
        buffer.push(event);
        State::ReadingCodeblock { buffer }
    }
}
