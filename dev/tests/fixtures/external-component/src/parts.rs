//! Role: Icon button and related parts for external components.
//!
//! Small parts shared by the components of this crate: the icon button and
//! its icons. The public `node` and monochrome `image` primitives supply
//! semantics and drawing; the application supplies its own appearance.
//!
//! [`IconButton`] has the shape of a builder with its own method:
//! it holds its root `Node` and implements `NodeBuilder`, so every
//! standard modifier (`test_id`, `disabled`, `style`, `at`, ...) comes from
//! `NodeBuilder`. The constructor is `#[track_caller]` and creates the root
//! and its icon, so each instance is located at its own call.
//!
//! The face is a decorative monochrome `image` painted in the
//! text color the button's state styles set (muted, the text color on
//! hover, the accent while pressed), like a React icon in `currentColor`.
//! Only the button's name reaches assistive technology.

use kivact::Asset;
use kivact::prelude::*;
use kivact::style::current_theme;
use kivact::style::{Align, Justify};
use kivact::view::HandlerResult;
use kivact::view::View;

use crate::skin;

/// An "×" icon for clear and close buttons (an original 24×24 SVG). A
/// `static`, not a `const`: embedded bytes are cached by address, and only
/// a `static` has one address, so every use is decoded once.
pub static CROSS_ICON: &[u8] = include_bytes!("../assets/icons/cross.svg");

/// A magnifier icon for search fields (an original 24×24 SVG).
pub static SEARCH_ICON: &[u8] = include_bytes!("../assets/icons/search.svg");

/// A square button whose face is an icon (no visible text). Built by
/// [`icon_button`].
pub struct IconButton {
    root: Node,
}

/// An icon button named `name` (its accessible name) whose face is the
/// icon `src` (embedded bytes such as [`CROSS_ICON`], a path, or a closure
/// returning an [`Asset`]), painted in the button's text color.
#[track_caller]
pub fn icon_button(name: impl IntoText, src: impl IntoValue<Asset>) -> IconButton {
    IconButton {
        root: node()
            .role(Role::Button)
            .focusable(true)
            .name(name)
            .style(move || {
                let t = &current_theme();
                let h = t.control_height() - 8.0;
                style()
                    .size(h, h)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .radius(t.radius().sm)
                    .color(ColorToken::TextMuted)
                    .hover(
                        style()
                            .background(ColorToken::SurfaceAlt)
                            .color(ColorToken::Text),
                    )
                    .pressed(style().color(ColorToken::Accent))
                    .disabled(style().opacity(0.45))
            })
            .style(move || skin::focus_ring(&current_theme()))
            .child(
                image(src)
                    .monochrome(true)
                    .role(Role::Generic)
                    .style(style().size(16, 16).shrink(0)),
            ),
    }
}

impl IconButton {
    /// Click, Enter, Space or UIA Invoke.
    pub fn on_press<F, R>(self, f: F) -> Self
    where
        F: Fn() -> R + 'static,
        R: HandlerResult,
    {
        NodeBuilder::on_press(self, f)
    }
}

impl NodeBuilder for IconButton {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for IconButton {
    fn into_view(self) -> View {
        self.root.into_view()
    }
}
