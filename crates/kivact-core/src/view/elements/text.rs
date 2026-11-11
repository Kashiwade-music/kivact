//! Role: The text elements: [`text`] and [`rich_text`], text [`Node`]s.
//! The `IntoView` implementations for strings mount a plain text node.
//!
//! Invariant: a rich text is one text node whose `Content` is its spans'
//! text joined (its accessible name and `Locator::text`, one text for
//! assistive technology) and whose `TextSpans` style their ranges; one
//! render effect writes both, so they always agree.
//!
//! Invariant: a text node records where it was created (`CoreNode::at`): the
//! call of `text(..)` (or the location given by `.located_at_caller()`), or nothing
//! for a plain string child, which diagnostics
//! show with the location of its nearest located ancestor.
//!
//! A text node states no role: the engine exposes a `Text` node without one as
//! `Role::Text`, named by its content (`kivact_engine` `semantic_role`).

use std::panic::Location;

use crate::state::{IntoText, IntoValue};
use crate::view::element_data::{Content, Node};
use crate::view::span::Span;
use crate::view::{IntoView, View};

/// Text from a constant, a signal, a value or a closure.
#[track_caller]
pub fn text(content: impl IntoText) -> Node {
    Node::new(
        Content::Plain(content.into_text()),
        Some(Location::caller()),
    )
}

/// The text of a plain string child (`IntoView for String / &str`): it has
/// no call site of its own.
pub(crate) fn string_child(content: impl IntoText) -> Node {
    Node::new(Content::Plain(content.into_text()), None)
}

/// One paragraph of [`span`]s with their own styles (weight, color, size,
/// italic, family, letter spacing, decoration lines, features, a
/// background band): a search match in bold, a `<mark>`, a `<small>`
/// unit. It wraps, aligns, clamps and ends in an ellipsis like a [`text`],
/// takes the node's style (`.style(..)`) as the paragraph's, and is one
/// text for assistive technology and tests (its spans' text joined). The
/// spans come from a constant, a signal or a closure: a live search
/// highlight is a closure over the query.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::view::{Span, rich_text, span};
/// use kivact::state::{Signal};
/// use kivact::style::{ColorToken};
/// use kivact::view::{IntoView, Element};
///
/// /// `label` with the first match of `query` bold and highlighted.
/// fn highlight(label: &str, query: &str) -> Vec<Span> {
///     match label.to_lowercase().find(&query.to_lowercase()).filter(|_| !query.is_empty()) {
///         Some(i) => vec![
///             span(&label[..i]),
///             span(&label[i..i + query.len()]).weight(700).background(ColorToken::Selection),
///             span(&label[i + query.len()..]),
///         ],
///         None => vec![span(label)],
///     }
/// }
///
/// fn item(query: Signal<String>) -> impl IntoView {
///     rich_text(move || highlight("Open file", &query.get()))
/// }
/// # let _ = item;
/// ```
///
/// [`span`]: crate::view::span
#[track_caller]
pub fn rich_text(spans: impl IntoValue<Vec<Span>>) -> Node {
    Node::new(Content::Rich(spans.into_prop()), Some(Location::caller()))
}

impl IntoView for String {
    fn into_view(self) -> View {
        string_child(self).into_view()
    }
}

impl IntoView for &'static str {
    fn into_view(self) -> View {
        string_child(self).into_view()
    }
}

impl<F, R> IntoView for F
where
    F: Fn() -> R + 'static,
    R: Into<String>,
{
    fn into_view(self) -> View {
        text(self).into_view()
    }
}
