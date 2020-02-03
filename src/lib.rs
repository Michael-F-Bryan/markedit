#![forbid(unsafe_code)]

//! Utilities for editing unstructured markdown documents.

pub extern crate pulldown_cmark;

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
