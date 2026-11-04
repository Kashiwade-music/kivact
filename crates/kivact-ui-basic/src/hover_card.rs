//! Role: Hover card: a floating preview opened by hovering or focusing its trigger.
//!
//! A popover (popover.tsx) is the standard `kivact::ui::dialog` of kind
//! `DialogKind::Popover`, drawn by the kit's dialog recipes. The hover card
//! is not a dialog: it opens and closes on delays while the pointer or the
//! focus is on its trigger or on itself, and it is a `Role::Group` that
//! takes no focus.

use crate::theme_access::{popover_surface, shadow_md, tokens};
use kivact::control::{NodeBuilder, column};
use kivact::interaction::{Delays, use_delayed};
use kivact::memo;
use kivact::primitive::NodeExt;
use kivact::style::{Direction, Style};
use kivact::view::{IntoView, OverlayAlign, Placement, Role, View, node_ref, overlay, show};
use std::rc::Rc;
use std::time::Duration;

/// The 4 px gap between the trigger and the panel (`sideOffset`).
const SIDE_OFFSET: f64 = 4.0;

/// A card that previews a link while the pointer or focus is on it
/// (hover-card.tsx).
///
/// Created by [`hover_card`]. It opens after [`HoverCard::open_delay`] of
/// hover or keyboard focus on the trigger and closes
/// [`HoverCard::close_delay`] after the pointer and the focus leave both the
/// trigger and the card, so the pointer can cross to the card and read it.
/// Escape and a press outside close it at once.
pub struct HoverCard {
    content: Rc<dyn Fn() -> View>,
    placement: Placement,
    align: OverlayAlign,
    open_delay: Duration,
    close_delay: Duration,
}

/// A hover card whose body `content` builds each time the card opens;
/// [`HoverCard::attach`] it to a trigger.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn mention() -> impl IntoView {
///     basic::hover_card(|| text("The React Framework, created by Vercel."))
///         .attach(button("@nextjs").variant(kivact::ui::ButtonVariant::Link))
/// }
/// ```
#[track_caller]
pub fn hover_card<V: IntoView>(content: impl Fn() -> V + 'static) -> HoverCard {
    HoverCard {
        content: Rc::new(move || content().into_view()),
        placement: Placement::Below,
        align: OverlayAlign::Center,
        open_delay: Duration::from_millis(600),
        close_delay: Duration::from_millis(300),
    }
}

impl HoverCard {
    /// The side of the trigger it opens on (default `Placement::Below`;
    /// `Start` and `End` are the inline start and end). The host flips it
    /// to the opposite side when the card does not fit in the window.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// How it lines up with the trigger (default `OverlayAlign::Center`).
    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.align = align;
        self
    }

    /// How long the trigger has to be hovered or focused before the card
    /// opens. The default is 600 ms.
    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.open_delay = delay;
        self
    }

    /// How long after the pointer and focus left before the card closes. The
    /// default is 300 ms.
    pub fn close_delay(mut self, delay: Duration) -> Self {
        self.close_delay = delay;
        self
    }

    /// Show the card for `trigger` and return both.
    ///
    /// The trigger keeps its own role and handlers; hover and focus within it
    /// open the card.
    pub fn attach<T: NodeBuilder + IntoView>(self, trigger: T) -> View {
        let HoverCard {
            content,
            placement,
            align,
            open_delay,
            close_delay,
        } = self;
        let anchor = node_ref();
        let card = node_ref();
        let over_trigger = anchor.hovered();
        let focus = anchor.focus_within();
        let over_card = card.hovered();
        let engaged = memo(move || over_trigger.get() || focus.get() || over_card.get());
        let card_delay = use_delayed(
            engaged,
            Delays {
                open: open_delay,
                close: close_delay,
            },
        );
        let trigger = trigger.node_ref(anchor);
        let panel = show(card_delay.shown(), move || {
            let body = content();
            let panel = column(body).style(panel_style);
            overlay(anchor, placement, panel)
                .align(align)
                .offset(SIDE_OFFSET)
                .node_ref(card)
                .role(Role::Group)
                .name(kivact::i18n::message("basic.hover_card.label", "Preview"))
                .on_dismiss(move |_| card_delay.dismiss())
        });
        (trigger, panel).into_view()
    }
}

/// `w-64 bg-popover p-2.5 text-xs shadow-md ring-1 ring-foreground/10`, as a column.
fn panel_style() -> Style {
    shadow_md(popover_surface(&tokens()))
        .direction(Direction::Column)
        .line_height(1.625)
        .width(256)
        .max_width(kivact::style::pct(100))
        .padding(10)
}
