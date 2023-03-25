use crate::matchers::Matcher;
use pulldown_cmark::{Event, Tag, HeadingLevel};

/// Matches the items inside a heading tag, including the start and end tags.
#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    inside_heading: bool,
    level: Option<HeadingLevel>,
}

impl Heading {
    /// Create a new [`Heading`].
    const fn new(level: Option<HeadingLevel>) -> Self {
        Heading {
            level,
            inside_heading: false,
        }
    }

    /// Matches any heading.
    pub const fn any_level() -> Self { Heading::new(None) }

    /// Matches only headings with the desired level.
    pub const fn with_level(level: HeadingLevel) -> Self { Heading::new(Some(level)) }

    fn matches_level(&self, level: HeadingLevel) -> bool {
        match self.level {
            Some(expected) => level == expected,
            None => true,
        }
    }
}

impl Matcher for Heading {
    fn matches_event(&mut self, event: &Event<'_>) -> bool {
        match event {
            Event::Start(Tag::Heading(level , _, _)) if self.matches_level(*level) => {
                self.inside_heading = true;
            },
            Event::End(Tag::Heading(level, _, _)) if self.matches_level(*level) => {
                self.inside_heading = false;
                // make sure the end tag is also matched
                return true;
            },
            _ => {},
        }

        self.inside_heading
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::LinkType;

    #[test]
    fn match_everything_inside_a_header() {
        // The original text for these events was:
        //
        // This is some text.
        //
        // ## Then a *header*
        //
        // [And a link](https://example.com)
        let inputs = vec![
            (Event::Start(Tag::Paragraph), false),
            (Event::Text("This is some text.".into()), false),
            (Event::End(Tag::Paragraph), false),
            (Event::Start(Tag::Heading(HeadingLevel::H2, None, vec![])), true),
            (Event::Text("Then a ".into()), true),
            (Event::Start(Tag::Emphasis), true),
            (Event::Text("header".into()), true),
            (Event::End(Tag::Emphasis), true),
            (Event::End(Tag::Heading(HeadingLevel::H2, None, vec![])), true),
            (Event::Start(Tag::Paragraph), false),
            (
                Event::Start(Tag::Link(
                    LinkType::Inline,
                    "https://example.com".into(),
                    "".into(),
                )),
                false,
            ),
            (Event::Text("And a link".into()), false),
            (
                Event::End(Tag::Link(
                    LinkType::Inline,
                    "https://example.com".into(),
                    "".into(),
                )),
                false,
            ),
            (Event::End(Tag::Paragraph), false),
        ];

        let mut matcher = Heading::any_level();

        for (tag, should_be) in inputs {
            let got = matcher.matches_event(&tag);
            assert_eq!(got, should_be, "{:?}", tag);
        }
    }
}
