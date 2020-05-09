use crate::Matcher;
use pulldown_cmark::Event;

/// Get a new stream of [`Event`]s where the things between `start` and `end`
/// are automatically rewritten.
///
/// # Note
///
/// Unlike most APIs in Rust this function matches an **inclusive** range.
///
/// # Examples
///
/// ```rust
/// use pulldown_cmark::{Event, Parser, Tag};
///
/// fn uppercase_all_text<'src>(events: &mut Vec<Event<'src>>) {
///     for event in events {
///         if let Event::Text(ref mut text) = event {
///             *text = text.to_uppercase().into();
///         }
///     }
/// }
///
/// let src = "# This is a heading\nand a normal line";
///
/// let got: Vec<_> = markedit::rewrite_between(
///     Parser::new(src),
///     |ev: &Event<'_>| match ev { Event::Start(Tag::Heading(_)) => true, _ => false },
///     |ev: &Event<'_>| match ev { Event::End(Tag::Heading(_)) => true, _ => false },
///     uppercase_all_text,
///     )
///     .collect();
///
/// let should_be: Vec<_> = Parser::new("# THIS IS A HEADING\nand a normal line").collect();
///
/// assert_eq!(got, should_be);
/// ```
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
#[derive(Debug)]
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
        loop {
            if !self.buffer.is_empty() {
                return Some(self.buffer.remove(0));
            }

            match self.events.next() {
                Some(event) => {
                    // temporarily swap out the current state with a dummy value
                    // so we can update it
                    let current_state =
                        std::mem::replace(&mut self.state, State::default());
                    self.state = handle_event(
                        current_state,
                        &mut self.start,
                        &mut self.end,
                        event,
                        &mut self.buffer,
                        &mut self.rewrite,
                    );
                },
                None => {
                    if self.buffer.is_empty() {
                        return None;
                    }
                },
            }
        }
    }
}

impl<'src, S, E, I, F> RewriteBetween<'src, S, E, I, F> {}

#[derive(Debug, PartialEq)]
enum State<'src> {
    Waiting,
    Reading { buffer: Vec<Event<'src>> },
}

impl<'src> Default for State<'src> {
    fn default() -> State<'src> { State::Waiting }
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
        State::Waiting => {
            handle_waiting_for_start(event, start, end, processed)
        },
        State::Reading { buffer } => handle_reading_codeblock(
            buffer, event, start, end, processed, rewrite,
        ),
    }
}

fn handle_waiting_for_start<'src, S, E>(
    event: Event<'src>,
    start: &mut S,
    end: &mut E,
    processed: &mut Vec<Event<'src>>,
) -> State<'src>
where
    S: Matcher,
    E: Matcher,
{
    let start_matched = start.matches_event(&event);
    let _ = end.matches_event(&event);

    if start_matched {
        State::Reading {
            buffer: vec![event],
        }
    } else {
        processed.push(event);
        State::Waiting
    }
}

fn handle_reading_codeblock<'src, E, F, S>(
    mut buffer: Vec<Event<'src>>,
    event: Event<'src>,
    start: &mut S,
    end: &mut E,
    processed: &mut Vec<Event<'src>>,
    transform: &mut F,
) -> State<'src>
where
    F: FnMut(&mut Vec<Event<'src>>),
    E: Matcher,
    S: Matcher,
{
    let _ = start.matches_event(&event);
    let end_matches = end.matches_event(&event);

    if end_matches {
        buffer.push(event);
        transform(&mut buffer);
        processed.extend(buffer.drain(..));
        State::Waiting
    } else {
        buffer.push(event);
        State::Reading { buffer }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Parser, Tag};

    fn uppercase_all_text<'src>(events: &mut Vec<Event<'src>>) {
        dbg!(&events);

        for event in events {
            if let Event::Text(ref mut text) = event {
                *text = text.to_uppercase().into();
            }
        }
    }

    #[test]
    fn uppercase_text_in_a_heading() {
        let src = "# This is a heading\nand a normal line";

        let got: Vec<_> = rewrite_between(
            Parser::new(src),
            |ev: &Event<'_>| match ev {
                Event::Start(Tag::Heading(_)) => true,
                _ => false,
            },
            |ev: &Event<'_>| match ev {
                Event::End(Tag::Heading(_)) => true,
                _ => false,
            },
            uppercase_all_text,
        )
        .collect();

        let should_be: Vec<_> =
            Parser::new("# THIS IS A HEADING\nand a normal line").collect();

        assert_eq!(got, should_be);
    }
}
