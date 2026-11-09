//! Role: The Lyra attachment: a file chip with media, text, state and actions.

use crate::parts::{pressable, sized_icon};
use kivact::ui::spinner;

use kivact::view::{Element, Node, Scroll, ScrollAxes, column, node, row, scroll, text};

use kivact::state::{Callback, Value};
use kivact::state::{IntoText, IntoValue};
use kivact::style::{
    Align, BorderStyle, Cursor, Direction, Justify, Overflow, Position, Style, pct, style,
};
use kivact::ui::{Button, ButtonVariant, ControlSize};
use kivact::view::{IntoView, Orientation, Role, View, show};

/// The size of an [`Attachment`] (attachment.tsx `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentSize {
    /// 40 px media, 8 px gap.
    #[default]
    Default,
    /// 32 px media.
    Sm,
    /// 28 px media.
    Xs,
}

/// The progress of an [`Attachment`] (attachment.tsx `state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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

/// The frame of the media of an [`Attachment`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AttachmentMediaVariant {
    /// An icon on a muted square.
    #[default]
    Icon,
    /// A picture that fills the square.
    Image,
}

/// A file chip (attachment.tsx). Created by [`attachment`].
pub struct Attachment {
    root: Node,
    title: Value<String>,
    size: Value<AttachmentSize>,
    orientation: Value<Orientation>,
    state: Value<AttachmentState>,
    media: Option<kivact::view::View>,
    media_variant: AttachmentMediaVariant,
    description: Option<Value<String>>,
    actions: Option<kivact::view::View>,
    on_press: Option<Callback<()>>,
}

kivact::state::constant_value!(AttachmentState);

/// What the root style of an [`Attachment`] reads once it is turned into a view.
#[derive(Clone, Copy)]
struct Cfg {
    size: Value<AttachmentSize>,
    vertical: Value<bool>,
    // xtask-allow(state-fields): independent: whether the chip has a media slot and whether it is pressable are separate facts; every combination is valid
    has_media: bool,
    // xtask-allow(state-fields): independent: whether the chip has a media slot and whether it is pressable are separate facts; every combination is valid
    actionable: bool,
    state: Value<AttachmentState>,
}

/// An attachment titled `title`.
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
#[track_caller]
pub fn attachment(title: impl IntoText) -> Attachment {
    let title = title.into_text_value();
    Attachment {
        root: node().name(title),
        title,
        size: AttachmentSize::Default.into_value(),
        orientation: Orientation::Horizontal.into_value(),
        state: AttachmentState::Done.into_value(),
        media: None,
        media_variant: AttachmentMediaVariant::Icon,
        description: None,
        actions: None,
        on_press: None,
    }
}

impl Attachment {
    /// Choose the size.
    pub fn size(mut self, size: impl IntoValue<AttachmentSize>) -> Self {
        self.size = size.into_value();
        self
    }

    /// Choose the layout: [`Orientation::Horizontal`] puts the media beside
    /// the text, [`Orientation::Vertical`] above it in a narrow column
    /// (`Unspecified` is horizontal). A constant: the sizes of the media and
    /// its glyph are chosen from it when the attachment is built.
    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }

    /// Set the progress; it may follow a signal.
    pub fn state(mut self, state: impl IntoValue<AttachmentState>) -> Self {
        self.state = state.into_value();
        self
    }

    /// Show `media` (an icon or a picture) in a square before the text.
    pub fn media(mut self, media: impl IntoView) -> Self {
        self.media = Some(media.into_view());
        self
    }

    /// Choose the frame of the media.
    pub fn media_variant(mut self, variant: AttachmentMediaVariant) -> Self {
        self.media_variant = variant;
        self
    }

    /// Show `text` under the title (a size or an error message).
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `actions` (see [`attachment_action`]) at the end of the row.
    pub fn actions(mut self, actions: impl IntoView) -> Self {
        self.actions = Some(actions.into_view());
        self
    }

    /// Make the attachment a button that reports presses to `f`.
    pub fn on_press(mut self, f: impl Into<kivact::state::Callback<()>>) -> Self {
        self.on_press = Some(f.into());
        self
    }
}

impl Element for Attachment {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
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
    let p = kivact::style::use_tokens::<crate::Tokens>();
    let (vertical, size) = (cfg.vertical.get(), cfg.size.get());
    let current = cfg.state.get();
    let line = match current {
        AttachmentState::Error => p.destructive.with_alpha(0.3),
        _ => p.border,
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
        .direction(if vertical {
            Direction::Column
        } else {
            Direction::Row
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
        .radius(0)
        .background(p.card)
        .color(p.card_foreground)
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
            .hover(style().background(p.card.mix(p.muted, 0.5)))
    } else {
        s
    }
}

impl IntoView for Attachment {
    fn into_view(self) -> View {
        let (size, orientation, state) = (self.size, self.orientation, self.state);
        let vertical = kivact::state::memo(move || orientation.get() == Orientation::Vertical);
        let has_media = self.media.is_some();
        let media_variant = self.media_variant;
        let actionable = self.on_press.is_some();
        let (root, _) = pressable(self.root, Role::Button, self.on_press);
        let cfg = Cfg {
            size,
            vertical,
            has_media,
            actionable,
            state,
        };
        let root = root.base_style(move || root_style(cfg));
        let media = self
            .media
            .map(|media| render_media(media, media_variant, cfg));
        let content = column((
            text(self.title).style(style().weight(500).single_line()),
            self.description.map(|description| {
                text(description).style(move || {
                    let p = kivact::style::use_tokens::<crate::Tokens>();
                    style().margin_top(2).single_line().color(
                        if state.get() == AttachmentState::Error {
                            p.destructive.with_alpha(0.8)
                        } else {
                            p.muted_foreground
                        },
                    )
                })
            }),
        ))
        .style(move || {
            style()
                .grow(1)
                .min_width(0)
                .line_height(1.25)
                .padding_x(if vertical.get() { 4 } else { 0 })
        });
        let actions = self.actions.map(|actions| {
            row(actions).style(move || {
                let s = style().align_items(Align::Center).shrink(0);
                if vertical.get() {
                    s.position(Position::Absolute).top(12).right(12).gap(4)
                } else {
                    s
                }
            })
        });
        root.child(media).child(content).child(actions).into_view()
    }
}

/// An icon-only ghost button for the actions of an [`attachment`] (24 px).
///
/// `icon` is what `kivact::view::icon` draws: a Lucide name of the
/// kit's icon set (`"x"`), an `Icon` or a reactive one.
#[track_caller]
pub fn attachment_action(
    icon: impl IntoValue<kivact::style::Icon>,
    label: impl IntoText,
) -> Button {
    kivact::ui::button(label)
        .variant(ButtonVariant::Ghost)
        .size(ControlSize::Small)
        .hide_label(true)
        .icon(kivact::view::icon(icon))
        .style(style().size(24, 24).padding(0))
}

/// Attachments in a row that scrolls sideways, 12 px apart.
#[track_caller]
pub fn attachment_group(children: impl IntoView) -> Scroll {
    scroll(row(children).style(style().gap(12).align_items(Align::Start)))
        .axes(ScrollAxes::Horizontal)
        .style(style().min_width(0).padding_y(4))
}

kivact::state::constant_value!(AttachmentSize);

impl kivact::ui::Variant for AttachmentSize {}
impl kivact::ui::Variant for AttachmentMediaVariant {}

fn render_media(media: View, media_variant: AttachmentMediaVariant, cfg: Cfg) -> Node {
    let Cfg {
        size,
        vertical,
        state,
        ..
    } = cfg;
    let uploading = move || state.get() == AttachmentState::Uploading;
    let side = move || media_side(size.get());
    let glyph = kivact::state::memo(move || {
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
            let p = kivact::style::use_tokens::<crate::Tokens>();
            let current = state.get();
            let failed = current == AttachmentState::Error;
            let s = style()
                .direction(Direction::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .shrink(0)
                .overflow(Overflow::Hidden)
                .background(if failed {
                    p.destructive.with_alpha(0.1)
                } else {
                    p.muted
                })
                .color(if failed { p.destructive } else { p.foreground });
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
