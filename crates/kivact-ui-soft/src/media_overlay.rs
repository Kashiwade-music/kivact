//! Role: Layer actions or labels within the bounds of their media.
use crate::recipes_dialog::{focus as focus_ring, motion};

use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::Overflow;
use kivact::style::Position;
use kivact::style::pct;
use kivact::style::rgba;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::OverlayAlign;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::node_ref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShowOn {
    #[default]
    Always,
    Hover,
    Focus,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayScrim {
    #[default]
    Dark,
    Light,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayPosition {
    #[default]
    Fill,
    Bottom,
    Top,
}

pub struct MediaOverlay {
    content: ViewFn,
    show_on: ShowOn,
    open: Option<Value<bool>>,
    scrim: OverlayScrim,
    position: OverlayPosition,
    align: OverlayAlign,
}

pub fn media_overlay<V: IntoView>(content: impl IntoViewFn<V>) -> MediaOverlay {
    MediaOverlay {
        content: content.into_view_fn(),
        show_on: ShowOn::Always,
        open: None,
        scrim: OverlayScrim::Dark,
        position: OverlayPosition::Fill,
        align: OverlayAlign::End,
    }
}

impl MediaOverlay {
    pub fn show_on(mut self, value: ShowOn) -> Self {
        self.show_on = value;
        self
    }

    pub fn open(mut self, value: impl IntoValue<bool>) -> Self {
        self.open = Some(value.into_value());
        self
    }

    pub fn scrim(mut self, value: OverlayScrim) -> Self {
        self.scrim = value;
        self
    }

    pub fn position(mut self, value: OverlayPosition) -> Self {
        self.position = value;
        self
    }

    pub fn align(mut self, value: OverlayAlign) -> Self {
        self.align = value;
        self
    }

    pub fn attach(self, base: impl IntoView) -> Node {
        let root = node_ref();
        let hover = root.hovered();
        let focus = root.focus_within();
        let mode = self.show_on;
        let open = self.open;
        let visible = kivact::state::memo(move || {
            open.map_or_else(
                || match mode {
                    ShowOn::Always => true,
                    ShowOn::Hover => hover.get() || focus.get(),
                    ShowOn::Focus => focus.get(),
                },
                |open| open.get(),
            )
        });
        let reveal_from_keyboard = open.is_none() && mode != ShowOn::Always;
        let content = column(self.content.call(()))
            .hidden(move || !visible.get())
            .style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                let background = match self.scrim {
                    OverlayScrim::Dark => p.colors.overlay_scrim,
                    OverlayScrim::Light => rgba(0xffffff, 0.6),
                    OverlayScrim::None => rgba(0, 0.0),
                };
                let (align, justify) = match self.align {
                    OverlayAlign::Start => (Align::Start, Justify::Start),
                    OverlayAlign::Center => (Align::Center, Justify::Center),
                    OverlayAlign::End => (Align::End, Justify::Start),
                };
                let layer = style()
                    .position(Position::Absolute)
                    .left(0)
                    .right(0)
                    .gap(8)
                    .padding(12)
                    .background(background)
                    .align_items(align)
                    .justify_content(justify);
                let (layer, offset) = match self.position {
                    OverlayPosition::Fill => (layer.top(0).bottom(0), 0),
                    OverlayPosition::Top => (layer.top(0), -100),
                    OverlayPosition::Bottom => (layer.bottom(0), 100),
                };
                let hidden = style().opacity(0.0).translate_y(pct(offset));
                let layer = motion(layer, &p, p.motion.fast).enter(hidden.clone());
                if visible.get() {
                    layer
                } else {
                    layer.merge(&hidden)
                }
            });
        column((base, content))
            .node_ref(root)
            .focusable(reveal_from_keyboard)
            .style(|| {
                style()
                    .overflow(Overflow::Hidden)
                    .focus_visible(focus_ring(&kivact::style::use_tokens::<crate::Tokens>()))
            })
    }
}
