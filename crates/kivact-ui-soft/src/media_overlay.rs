//! Role: Layer actions or labels within the bounds of their media.
use crate::recipes_dialog::motion;
use crate::theme_access::tokens;
use kivact::Value;
use kivact::control::NodeBuilder;
use kivact::control::{Node, column, stack};
use kivact::style::{Align, Justify, Overflow, Position, pct, rgba, style};
use kivact::view::IntoValue;
use kivact::view::{IntoView, OverlayAlign, node_ref};
use std::rc::Rc;

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
    content: Rc<dyn Fn() -> kivact::view::View>,
    show_on: ShowOn,
    open: Option<Value<bool>>,
    scrim: OverlayScrim,
    position: OverlayPosition,
    align: OverlayAlign,
}

pub fn media_overlay<V: IntoView>(content: impl Fn() -> V + 'static) -> MediaOverlay {
    MediaOverlay {
        content: Rc::new(move || content().into_view()),
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
        let content = column((self.content)()).style(move || {
            let p = tokens();
            let visible = self.open.map_or_else(
                || match self.show_on {
                    ShowOn::Always => true,
                    ShowOn::Hover => hover.get() || focus.get(),
                    ShowOn::Focus => focus.get(),
                },
                |open| open.get(),
            );
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
            if visible { layer } else { layer.merge(&hidden) }
        });
        stack((base, content))
            .node_ref(root)
            .style(style().overflow(Overflow::Hidden))
    }
}
