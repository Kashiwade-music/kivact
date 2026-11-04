//! Role: Attach a delayed nonmodal preview to a trigger.
use crate::recipes_dialog::{anchored, body_text, shadows};
use crate::theme_access::tokens;
use kivact::control::{NodeBuilder, column};
use kivact::interaction::{Delays, use_delayed};
use kivact::primitive::NodeExt;
use kivact::style::{Direction, pct};
use kivact::view::IntoValue;
use kivact::view::{IntoView, OverlayAlign, Placement, Role, View, node_ref, overlay, show};
use kivact::{Binding, Value, memo, signal};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusTrigger {
    #[default]
    Auto,
    Always,
    Never,
}

pub struct HoverCard {
    content: Rc<dyn Fn() -> View>,
    placement: Placement,
    align: OverlayAlign,
    delays: Delays,
    label: Option<String>,
    enabled: Value<bool>,
    focus: FocusTrigger,
    open: Option<Binding<bool>>,
    default_open: bool,
    on_open_change: Option<Rc<dyn Fn(bool)>>,
}

pub fn hover_card<V: IntoView>(content: impl Fn() -> V + 'static) -> HoverCard {
    HoverCard {
        content: Rc::new(move || content().into_view()),
        placement: Placement::Above,
        align: OverlayAlign::Center,
        delays: Delays {
            open: Duration::from_millis(300),
            close: Duration::from_millis(200),
        },
        label: None,
        enabled: true.into_value(),
        focus: FocusTrigger::Auto,
        open: None,
        default_open: false,
        on_open_change: None,
    }
}

impl HoverCard {
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.align = align;
        self
    }

    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.delays.open = delay;
        self
    }

    pub fn close_delay(mut self, delay: Duration) -> Self {
        self.delays.close = delay;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn enabled(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.enabled = enabled.into_value();
        self
    }

    pub fn focus_trigger(mut self, focus: FocusTrigger) -> Self {
        self.focus = focus;
        self
    }

    pub fn open(mut self, open: impl Into<Binding<bool>>) -> Self {
        self.open = Some(open.into());
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_open_change(mut self, callback: impl Fn(bool) + 'static) -> Self {
        self.on_open_change = Some(Rc::new(callback));
        self
    }

    pub fn attach(self, trigger: impl NodeBuilder + IntoView) -> View {
        let anchor = node_ref();
        let card = node_ref();
        let over_anchor = anchor.hovered();
        let over_card = card.hovered();
        let anchor_focus = anchor.focus_within();
        let card_focus = card.focus_within();
        let enabled = self.enabled;
        let focus = self.focus;
        let dismissed = signal(false);
        let initially_open = signal(self.default_open);
        let controlled = self.open;
        let visited = signal(false);
        kivact::effect(move || {
            if over_anchor.get() || over_card.get() || anchor_focus.get() || card_focus.get() {
                visited.set(true);
            } else if visited.get() {
                initially_open.set(false);
            }
        });
        let pointer = use_delayed(
            memo(move || enabled.get() && (over_anchor.get() || over_card.get())),
            self.delays,
        );
        let focus_delay = use_delayed(
            memo(move || {
                enabled.get()
                    && focus != FocusTrigger::Never
                    && (anchor_focus.get() || card_focus.get())
            }),
            Delays {
                open: Duration::ZERO,
                close: self.delays.close,
            },
        );
        let trigger = trigger.node_ref(anchor);
        let trigger = if focus == FocusTrigger::Always {
            trigger.focusable(true)
        } else {
            trigger
        };
        let open = memo(move || {
            enabled.get()
                && (controlled.map_or_else(
                    || initially_open.get() || pointer.shown().get(),
                    |value| value.get(),
                ) || (!dismissed.get() && focus_delay.shown().get()))
        });
        if let Some(callback) = self.on_open_change.clone() {
            let previous = kivact::var(open.get());
            kivact::effect(move || {
                let current = open.get();
                if current != previous.get() {
                    previous.set(current);
                    callback(current);
                }
            });
        }
        let panel = show(open, move || {
            let panel = column((self.content)()).style(move || {
                let p = tokens();
                let base = shadows(body_text(&p), &p.shadows.med)
                    .direction(Direction::Column)
                    .background(p.colors.bg_surface)
                    .radius(p.radius.container)
                    .max_width(pct(100) - 32.0)
                    .padding(12);
                let (x, y) = match self.placement {
                    Placement::Above => (0.0, 8.0),
                    Placement::Below => (0.0, -8.0),
                    Placement::Start => (8.0, 0.0),
                    Placement::End => (-8.0, 0.0),
                    _ => (0.0, 0.0),
                };
                anchored(base, &p, x, y)
            });
            let layer = overlay(anchor, self.placement, panel)
                .align(self.align)
                .offset(4)
                .viewport_margin(16.0)
                .node_ref(card)
                .on_dismiss(move |reason| {
                    if reason == kivact::view::DismissReason::Escape && card_focus.get() {
                        anchor.focus();
                    }
                    dismissed.set(anchor_focus.get() || card_focus.get());
                    initially_open.set(false);
                    if let Some(controlled) = controlled {
                        controlled.set(false);
                    }
                    pointer.dismiss();
                    focus_delay.dismiss();
                });
            match &self.label {
                Some(label) => layer.role(Role::Dialog).name(label.clone()),
                None => layer.role(Role::Group),
            }
        });
        kivact::effect(move || {
            if !anchor_focus.get() {
                dismissed.set(false);
            }
        });
        (trigger, panel).into_view()
    }
}

/// A keyboard-reachable text trigger with the native solid underline fallback.
pub fn hover_card_text(label: impl kivact::view::IntoText) -> kivact::control::Node {
    kivact::control::text(label).focusable(true).style(|| {
        kivact::style::style()
            .text_decoration(kivact::style::TextDecorationLine::UNDERLINE)
            .text_decoration_color(tokens().colors.border_emphasized)
            .text_underline_offset(2)
    })
}
