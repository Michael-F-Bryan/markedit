#![forbid(unsafe_code)]

//! Utilities for editing unstructured markdown documents.

pub extern crate pulldown_cmark;

mod matchers;
mod rewriters;

pub use matchers::*;
pub use rewriters::*;

use pulldown_cmark::{Event, Parser};

pub fn parse_events(text: &str) -> impl Iterator<Item = Event<'_>> + '_ {
    Parser::new(text)
}
