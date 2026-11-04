//! Role: Attach a delayed nonmodal preview to a trigger.
use crate::recipes_dialog::{anchored, body_text, shadows};
use crate::theme_access::tokens;
use kivact::interaction::Delays;
use kivact::interaction::use_delayed;
use kivact::state::IntoBinding;
use kivact::view::HandlerResult;
use kivact::view::IntoViewFn;
use kivact::view::NodeBuilder;
use kivact::view::ViewFn;
use kivact::view::column;

use kivact::state::Binding;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Direction;
use kivact::style::pct;
use kivact::view::IntoView;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::show;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusTrigger {
    #[default]
    Auto,
    Always,
    Never,
}

pub struct HoverCard {
    content: ViewFn,
    placement: Placement,
    align: OverlayAlign,
    delays: Delays,
    label: Option<String>,
    enabled: Value<bool>,
    focus: FocusTrigger,
    open: Option<Binding<bool>>,
    default_open: bool,
    on_open_change: Option<kivact::state::Callback<bool>>,
}

pub fn hover_card<V: IntoView>(content: impl IntoViewFn<V>) -> HoverCard {
    HoverCard {
        content: content.into_view_fn(),
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

    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = Some(open.into_binding());
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_open_change<R: HandlerResult>(
        mut self,
        callback: impl Fn(bool) -> R + 'static,
    ) -> Self {
        self.on_open_change = Some(kivact::state::callback(callback));
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
        kivact::state::effect(move || {
            if over_anchor.get() || over_card || anchor_focus || card_focus.get() {
                visited.set(true);
            } else if visited.get() {
                initially_open.set(false);
            }
        });
        let pointer = use_delayed(
            memo(move || enabled && (over_anchor || over_card.get())),
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
                    || initially_open || pointer.is_open().get(),
                    |value| value.get(),
                ) || (!dismissed.get() && focus_delay.is_open().get()))
        });
        if let Some(callback) = self.on_open_change.clone() {
            let previous = kivact::var(open.get());
            kivact::state::effect(move || {
                let current = open.get();
                if current != previous.get() {
                    previous.set(current);
                    callback(current);
                }
            });
        }
        let panel = show(open, move || {
            let panel = column(self.content.call(())).style(move || {
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
        kivact::state::effect(move || {
            if !anchor_focus.get() {
                dismissed.set(false);
            }
        });
        (trigger, panel).into_view()
    }
}

/// A keyboard-reachable text trigger with the native solid underline fallback.
pub fn hover_card_text(label: impl kivact::state::IntoText) -> kivact::view::Node {
    kivact::view::text(label).focusable(true).style(|| {
        kivact::style::style()
            .text_decoration(kivact::style::TextDecorationLine::UNDERLINE)
            .text_decoration_color(tokens().colors.border_emphasized)
            .text_underline_offset(2)
    })
}
