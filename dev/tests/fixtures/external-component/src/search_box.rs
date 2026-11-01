//! Role: `SearchBox`: a search field with an icon, a clear button and a status line.
//! Native editing and IME use the public headless text_edit attachment;
//! the bound query only changes on committed text, never on preedit.
//!
//! ```
//! use kivact::prelude::*;
//! use kivact_external_component::search_box;
//!
//! use kivact::signal;
//!
//! let screen = kivact::test::render(|| {
//!     let query = signal(String::new());
//!     column((
//!         search_box("Search logs", query).placeholder("error, 42, ..."),
//!         text(move || format!("query={}", query.get())),
//!     ))
//! });
//! screen.get_by_role(Role::TextBox, "Search logs").compose("けんさく");
//! screen.get_by_text("query="); // preedit is not committed
//! screen.get_by_role(Role::TextBox, "Search logs").type_text("検索");
//! screen.get_by_text("query=検索");
//! ```
//!
//! The leading magnifier is a decorative monochrome `image` in the muted
//! text color, the clear button an [`IconButton`](crate::parts::IconButton).
//!
//! The builder holds its root `Node` (a row with role `Group`) and the
//! inner text field: the standard modifiers (`test_id`, `style`, ...) apply
//! to the root, and the settings of the inner field are explicit methods
//! (`placeholder`, `on_submit`). The constructor is `#[track_caller]` and
//! creates the root, the icon and the field, so they are located at the
//! call of `search_box`; the nodes `into_view` creates use `.at(..)` with
//! that location.

use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::view::View;
use std::panic::Location;

use kivact::Callback;
use kivact::control::Node;
use kivact::interaction::{TextEdit, use_text_edit};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::view::{Handled, KeyEvent};

use crate::parts::{CROSS_ICON, SEARCH_ICON, icon_button};
use crate::skin::{self, look};

/// Builder of a search box. Built by [`search_box`].
pub struct SearchBox {
    root: Node,
    field: Node,
    editing: TextEdit,
    query: Signal<String>,
    input: NodeRef,
    status: Option<Node>,
    at: &'static Location<'static>,
}

/// A search field bound to `query`; `label` is the field's accessible name.
/// Escape clears the query; the clear button appears while it is not
/// empty. The field's test id is `"input"`, the clear button's `"clear"`.
#[track_caller]
pub fn search_box(label: impl IntoText, query: Signal<String>) -> SearchBox {
    let look = look();
    let input = node_ref();
    let editing = use_text_edit(query);
    SearchBox {
        root: node()
            .style(style().direction(Direction::Row))
            .role(Role::Group)
            .on_key(move |k: KeyEvent| {
                if k.key == Key::Escape && query.with_untracked(|q| !q.is_empty()) {
                    query.set(String::new());
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
            .style(move || {
                let t = &current_theme();
                skin::field(look(), t)
            })
            .child(
                image(SEARCH_ICON)
                    .monochrome(true)
                    .role(Role::Generic)
                    .style(style().size(16, 16).shrink(0))
                    .style(style().color(ColorToken::TextMuted)),
            ),
        editing,
        field: node()
            .name(label)
            .part(editing.control())
            .node_ref(input)
            .test_id("input")
            .style(
                style()
                    .grow(1)
                    .border(0, ColorToken::Border)
                    .padding_x(2)
                    .background(Color::token(ColorToken::Surface).with_alpha(0.0)),
            ),
        query,
        input,
        status: None,
        at: Location::caller(),
    }
}

impl SearchBox {
    /// Placeholder of the inner field.
    pub fn placeholder(mut self, p: impl IntoText) -> Self {
        self.editing = self.editing.placeholder(p);
        self
    }

    /// A status line after the field ("12 matches"), exposed as
    /// `Role::Status` with the test id `"status"`.
    pub fn status(mut self, v: impl IntoText) -> Self {
        self.status = Some(
            text(v)
                .at(self.at)
                .role(Role::Status)
                .test_id("status")
                .style(style().color(ColorToken::TextMuted)),
        );
        self
    }

    /// Enter in the field, with the current query.
    pub fn on_submit(mut self, cb: Callback<String>) -> Self {
        let query = self.query;
        self.editing = self
            .editing
            .on_submit(move || cb.call(query.get_untracked()));
        self
    }
}

impl NodeBuilder for SearchBox {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SearchBox {
    fn into_view(self) -> View {
        let SearchBox {
            root,
            field,
            editing: _,
            query,
            input,
            status,
            at,
        } = self;
        let has_text = memo(move || query.with(|q| !q.is_empty()));
        let clear = show(has_text, move || {
            icon_button("Clear search", CROSS_ICON)
                .at(at)
                .test_id("clear")
                .on_press(move || {
                    query.set(String::new());
                    input.focus();
                })
        });
        root.child(field).child(clear).child(status).into_view()
    }
}
