//! Role: The Lyra attachment: a file chip with media, text, state and actions.

use crate::recipe_parts::{pressable, sized_icon};
use crate::variants::{AttachmentMediaVariant, AttachmentSize};
use crate::{BORDER, CARD, CARD_FOREGROUND, DESTRUCTIVE, FOREGROUND, MUTED, MUTED_FOREGROUND};
use kivact::component;
use kivact::custom::Caller;
use kivact::ui::spinner;

use kivact::Prop;
use kivact::state::{Value, memo};
use kivact::style::{
    Align, BorderStyle, Cursor, FlexDirection, Icon, Justify, Overflow, Position, Style, pct,
    style, use_theme,
};
use kivact::view::{
    Element, Node, Orientation, Role, ScrollAxes, View, column, node, row, scroll, show, text,
};

/// The progress of an [`Attachment`] (attachment.tsx `state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Prop)]
pub enum AttachmentState {
    /// Added and complete.
    #[default]
    Done,
    /// Waiting for a file: a dashed border.
    Idle,
    /// Being sent: a spinner replaces the media.
    Uploading,
    /// Failed: destructive colors.
    Error,
}

/// What the root style of an [`Attachment`] reads once it is turned into a view.
#[derive(Clone, Copy)]
struct Cfg {
    size: Value<AttachmentSize>,
    vertical: Value<bool>,
    has_media: bool,
    actionable: bool,
    state: Value<AttachmentState>,
}

/// A file chip (attachment.tsx): a titled frame with optional media,
/// description and actions.
///
/// An attachment titled `title`. Choose the size with `.variant(..)` on
/// [`AttachmentSize`] values, the layout with `.orientation(..)`, the progress
/// with `.state(..)`, and show `.media(..)`, `.description(..)` and
/// `.actions(..)`. Give it a press handler with `.on_press(..)` to make it a
/// button.
///
/// ```no_run
/// use kivact::view::{icon};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn file() -> impl IntoView {
///     basic::attachment("report.pdf")
///         .media(icon("file-text"))
///         .description("2.4 MB")
///         .actions(basic::attachment_action("x", "Remove"))
/// }
/// ```
#[component]
pub fn attachment(
    title: Value<String>,
    #[prop(default = AttachmentSize::Default)] size: Value<AttachmentSize>,
    #[prop(default = Orientation::Horizontal)] orientation: Value<Orientation>,
    #[prop(default = AttachmentState::Done)] state: Value<AttachmentState>,
    media: Option<View>,
    #[prop(default = AttachmentMediaVariant::Icon)] media_variant: AttachmentMediaVariant,
    actions: Option<View>,
    #[caller] caller: Caller,
) -> impl Element {
    let vertical = memo(move || orientation.get() == Orientation::Vertical);
    let has_media = media.is_some();
    let actionable = caller.has_press();
    let description = caller.description();
    let root = node().name(title);
    let (root, _) = pressable(root, Role::Button, None);
    let root = if actionable {
        root.with_node(|root| root.role(Role::Button))
            .focusable(true)
    } else {
        root
    };
    let cfg = Cfg {
        size,
        vertical,
        has_media,
        actionable,
        state,
    };
    let root = root.base_style(move || root_style(cfg));
    let media = media.map(|media| render_media(media, media_variant, cfg));
    let content = column((
        text(title).style(style().font_weight(500).single_line()),
        description.map(|description| {
            text(description).style(move || {
                style().margin_top(2).single_line().color(
                    if state.get() == AttachmentState::Error {
                        DESTRUCTIVE.with_alpha(0.8)
                    } else {
                        MUTED_FOREGROUND
                    },
                )
            })
        }),
    ))
    .style(move || {
        style()
            .flex_grow(1.0)
            .min_width(0)
            .line_height(1.25)
            .padding_x(if vertical.get() { 4 } else { 0 })
    });
    let actions = actions.map(|actions| {
        row(actions).style(move || {
            let s = style().align_items(Align::Center).flex_shrink(0.0);
            if vertical.get() {
                s.position(Position::Absolute).top(12).right(12).gap(4)
            } else {
                s
            }
        })
    });
    root.child(media).child(content).child(actions)
}

fn media_side(size: AttachmentSize) -> f64 {
    match size {
        AttachmentSize::Default => 40.0,
        AttachmentSize::Sm => 32.0,
        AttachmentSize::Xs => 28.0,
    }
}

/// The root style: the frame, its state colors and the focus ring.
fn root_style(cfg: Cfg) -> Style {
    let theme = use_theme().get();
    let (vertical, size) = (cfg.vertical.get(), cfg.size.get());
    let current = cfg.state.get();
    let line = match current {
        AttachmentState::Error => DESTRUCTIVE.with_alpha(0.3),
        _ => BORDER,
    };
    let pad = match (cfg.has_media, size) {
        (true, AttachmentSize::Default) => (6.0, 6.0),
        (true, _) => (4.0, 4.0),
        (false, AttachmentSize::Default) => (8.0, 6.0),
        (false, _) => (6.0, 4.0),
    };
    let gap = match size {
        AttachmentSize::Default => 8.0,
        AttachmentSize::Sm => 10.0,
        AttachmentSize::Xs => 6.0,
    };
    let s = style()
        .flex_direction(if vertical {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        })
        .align_items(if vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .align_self(Align::Start)
        .position(Position::Relative)
        .gap(gap)
        .padding_x(pad.0)
        .padding_y(pad.1)
        .max_width(pct(100))
        .border(1, line)
        .border_radius(0.0)
        .background(CARD)
        .color(CARD_FOREGROUND)
        .font_size(12);
    let s = if vertical {
        s.width(if cfg.has_media { 120 } else { 96 }).min_width(0)
    } else {
        s.min_width(160)
    };
    let s = if current == AttachmentState::Idle {
        s.border_style(BorderStyle::Dashed)
    } else {
        s
    };
    // A pressable attachment is a focus stop: the theme's focus ring shows
    // while the focus is visible.
    if cfg.actionable {
        s.cursor(Cursor::Pointer)
            .hover(style().background(CARD.mix(MUTED, 0.5, &theme)))
    } else {
        s
    }
}

/// An icon-only ghost button for the actions of an [`attachment`] (24 px).
///
/// `icon` is what `kivact::view::icon` draws: a Lucide name of the
/// kit's icon set (`"x"`), an `Icon` or a reactive one.
#[component]
pub fn attachment_action(icon: Value<Icon>, label: Value<String>) -> impl Element {
    kivact::ui::button(label)
        .variant(kivact::ui::ButtonVariant::Ghost)
        .size(kivact::ui::ControlSize::Small)
        .hide_label(true)
        .icon(kivact::view::icon(icon))
        .style(style().size(24, 24).padding(0))
}

/// Attachments in a row that scrolls sideways, 12 px apart.
#[component]
pub fn attachment_group(children: View) -> impl Element {
    scroll(row(children).style(style().gap(12).align_items(Align::Start)))
        .axes(ScrollAxes::Horizontal)
        .style(style().min_width(0).padding_y(4))
}

fn render_media(media: View, media_variant: AttachmentMediaVariant, cfg: Cfg) -> Node {
    let Cfg {
        size,
        vertical,
        state,
        ..
    } = cfg;
    let uploading = move || state.get() == AttachmentState::Uploading;
    let side = move || media_side(size.get());
    let glyph = memo(move || {
        if vertical.get() {
            24.0
        } else if size.get() == AttachmentSize::Xs {
            14.0
        } else {
            16.0
        }
    });

    let picture = match media_variant {
        AttachmentMediaVariant::Icon => sized_icon(media, glyph),
        AttachmentMediaVariant::Image => node().child(media),
    };
    node()
        .style(move || {
            let current = state.get();
            let failed = current == AttachmentState::Error;
            let s = style()
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .flex_shrink(0.0)
                .overflow(Overflow::Hidden)
                .background(if failed {
                    DESTRUCTIVE.with_alpha(0.1)
                } else {
                    MUTED
                })
                .color(if failed { DESTRUCTIVE } else { FOREGROUND });
            let s = if vertical.get() {
                s.width(pct(100)).aspect_ratio(1.0)
            } else {
                s.size(side(), side())
            };
            if media_variant == AttachmentMediaVariant::Image
                && matches!(current, AttachmentState::Uploading | AttachmentState::Error)
            {
                s.opacity(0.6)
            } else {
                s
            }
        })
        .child(show(uploading, move || sized_icon(spinner(), glyph)))
        .child(picture.hidden(uploading))
}
