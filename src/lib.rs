//! An ergonomic library for manipulating markdown documents.
//!
//! There are two fundamental concepts in `markedit`,
//!
//! - `Matcher` - something which can match an [`Event`] from pulldown-cmark
//!   (typically implemented using state machines or simple functions)
//! - `Rewriter` - something which can rewrite part of a stream of [`Event`]s
//!   (typically just a function)
//!
//! Together we can use these to transform a stream of [`Event`]s on the fly
//! with minimal overhead.
//!
//! # Examples
//!
//! The use case which prompted this entire library was to insert arbitrary
//! markdown after a heading.
//!
//! ```rust
//! use pulldown_cmark::{Event, Tag};
//! use markedit::{Matcher, Heading};
//!
//! let src = "# Heading\n Some text\n some more text \n\n # Another Heading";
//!
//! // first we need to construct our predicate
//! let matcher = Heading::with_level(1).falling_edge();
//!
//! // we also need a rewriting rule
//! let rule = markedit::insert_markdown_before("## Sub-Heading", matcher);
//!
//! // create our stream of events
//! let events = markedit::parse(src);
//! // then mutate them and collect them into a vector so we can inspect the
//! // results
//! let mutated: Vec<_> = markedit::rewrite(events, rule).collect();
//!
//! // the heading before we want to insert
//! assert_eq!(mutated[1], Event::Text("Heading".into()));
//! // our inserted tags
//! assert_eq!(mutated[3], Event::Start(Tag::Heading(2)));
//! assert_eq!(mutated[4], Event::Text("Sub-Heading".into()));
//! assert_eq!(mutated[5], Event::End(Tag::Heading(2)));
//! // "Some text" is the line after
//! assert_eq!(mutated[7], Event::Text("Some text".into()));
//! ```
//!
//! You can also use [`change_text()`] to upper-case text based on a predicate
//! (e.g. the text contains a certain keyword).
//!
//! ```rust
//! use pulldown_cmark::Event;
//!
//! let src = "# Heading\n Some text \n some more text \n\n # Another Heading";
//!
//! // first we construct the rewriting rule
//! let rule = markedit::change_text(
//!     |text| text.contains("Heading"),
//!     |text| text.to_uppercase(),
//! );
//!
//! // let's parse the input text into Events
//! let events_before: Vec<_> = markedit::parse(src).collect();
//!
//! // some sanity checks on the initial input
//! assert_eq!(events_before[1], Event::Text("Heading".into()));
//! assert_eq!(events_before[9], Event::Text("Another Heading".into()));
//!
//! // now rewrite the events using our rewriter rule
//! let events_after: Vec<_> = markedit::rewrite(events_before, rule)
//!     .collect();
//!
//! // and check the results
//! println!("{:?}", events_after);
//! assert_eq!(events_after[1], Event::Text("HEADING".into()));
//! assert_eq!(events_after[9], Event::Text("ANOTHER HEADING".into()));
//! ```
//!
//! Note that everything works with streaming iterators, we only needed to
//! `collect()` the events into a `Vec` for demonstration purposes.

#![forbid(unsafe_code)]
#![deny(missing_docs, missing_debug_implementations, rust_2018_idioms)]

pub use pulldown_cmark;

mod matchers;
mod rewriters;

pub use matchers::*;
pub use rewriters::*;

use pulldown_cmark::{Event, Parser};

/// A convenience function for parsing some text into [`Event`]s without
/// needing to add [`pulldown_cmark`] as an explicit dependency.
pub fn parse(text: &str) -> impl Iterator<Item = Event<'_>> + '_ {
    Parser::new(text)
}
