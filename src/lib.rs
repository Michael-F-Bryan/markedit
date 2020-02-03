#![forbid(unsafe_code)]

//! Utilities for editing unstructured markdown documents.

pub extern crate pulldown_cmark;

mod matchers;

pub use matchers::*;

use pulldown_cmark::{Event, Parser};

pub fn parse_events(text: &str) -> impl Iterator<Item = Event<'_>> + '_ {
    Parser::new(text)
}

/// Something which can be used to transform a parsed markdown document.
pub trait Rewriter<'src> {
    fn rewrite(&mut self, events: &mut Vec<Event<'src>>);
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateLink<M> {
    link_content_matcher: M,
}
