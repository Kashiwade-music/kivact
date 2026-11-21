//! Role: The Button builder: its busy phase, icon slots and rendering.

use crate::button::look::ButtonState;
use crate::button::look::ButtonTheme;
use crate::button::look::ButtonVariant;
use crate::button::look::{
    BUSY_REDUCED_SPIN_MS_VAR, BUSY_REVEAL_DELAY_MS_VAR, BUSY_REVEAL_FADE_MS_VAR, BUSY_SPIN_MS_VAR,
};
use crate::button_group::GroupPosition;
use crate::control_size::ControlSize;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::custom::PartState;
use kivact_core::motion::{Animated, Easing, FrameRate, Transition, frame_loop};
use kivact_core::state::try_use_context;
use kivact_core::state::{IntoText, IntoValue, Machine, Step, Value, machine, memo, signal};
use kivact_core::style::{Style, style, use_theme};
use kivact_core::view::Variants;
use kivact_core::view::{
    Element, IntoChildren, IntoView, Node, Orientation, View, node, row, show,
};
use kivact_interaction::{DisabledButtonFocus, use_button};
use std::time::Duration;

#[derive(Clone, Copy, PartialEq)]
enum BusyPhase {
    Idle,
    Waiting,
    Revealing,
    Shown,
}
enum BusyEvent {
    Changed(bool),
    RevealDue,
    ShownDue,
    SurfaceChanged,
}
struct BusyInput {
    delay: u64,
    fade: u64,
    active: bool,
}
impl Machine for BusyInput {
    type State = BusyPhase;
    type Event = BusyEvent;
    type Effect = bool;
    fn transition(&self, state: &BusyPhase, event: BusyEvent) -> Step<BusyPhase, BusyEvent, bool> {
        match event {
            BusyEvent::Changed(false) => Step::to(BusyPhase::Idle)
                .cancel(BusyEvent::RevealDue)
                .cancel(BusyEvent::ShownDue)
                .with(false),
            BusyEvent::Changed(true) if *state == BusyPhase::Idle => {
                if self.delay != 0 {
                    Step::to(BusyPhase::Waiting)
                        .after(Duration::from_millis(self.delay), BusyEvent::RevealDue)
                        .with(self.active)
                } else if self.fade != 0 {
                    Step::to(BusyPhase::Revealing)
                        .after(Duration::from_millis(self.fade), BusyEvent::ShownDue)
                        .with(self.active)
                } else {
                    Step::to(BusyPhase::Shown).with(self.active)
                }
            }
            BusyEvent::RevealDue if *state == BusyPhase::Waiting => {
                if self.fade != 0 {
                    Step::to(BusyPhase::Revealing)
                        .after(Duration::from_millis(self.fade), BusyEvent::ShownDue)
                } else {
                    Step::to(BusyPhase::Shown)
                }
            }
            BusyEvent::ShownDue if *state == BusyPhase::Revealing => Step::to(BusyPhase::Shown),
            BusyEvent::SurfaceChanged => {
                Step::stay().with(*state != BusyPhase::Idle && self.active)
            }
            _ => Step::stay(),
        }
    }
}

// xtask-allow(state): legacy: icon, trailing, busy, hide_label
pub struct Button {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    trailing: Option<View>,
    children: Vec<View>,
    size: Option<Value<ControlSize>>,
    busy: Option<Value<bool>>,
    input: Value<PartState>,
    hide_label: Option<Value<bool>>,
}
/// A themed button activated through the shared button part ([`use_button`]).
#[track_caller]
pub fn button(label: impl IntoText) -> Button {
    let label = label.into_text_value();
    let part = use_button().control();
    let input = part.state();
    let theme = use_theme();
    let root = node()
        .part(part)
        .focusable_when_disabled(move || {
            theme.with(|theme| {
                matches!(
                    theme
                        .ext::<kivact_interaction::InteractionPolicy>()
                        .button
                        .disabled_focus,
                    DisabledButtonFocus::Retain
                )
            })
        })
        .name(label);
    Button {
        root,
        label,
        icon: None,
        trailing: None,
        children: Vec::new(),
        size: None,
        busy: None,
        input,
        hide_label: None,
    }
}
impl Button {
    /// Set the size explicitly, overriding the inherited control size of a
    /// size provider or a button group.
    #[track_caller]
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = Some(size.into_value());
        self
    }
    /// Show the busy indicator and ignore presses while true (constant or
    /// reactive). Its spin and reveal use the theme's `theme::button::BUSY_*_VAR`.
    #[track_caller]
    pub fn busy(mut self, busy: impl IntoValue<bool>) -> Self {
        let busy = busy.into_value();
        self.root = self.root.busy(busy);
        self.busy = Some(busy);
        self
    }
    /// A view before the label, in a box the theme's
    /// [`ButtonTheme::icon`](crate::theme::ButtonTheme) styles (a kit sizes
    /// the icon there by the button's size).
    /// The recipe knows it from [`ButtonState::leading_icon`].
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    /// A view after the label, in a box styled like [`Self::icon`]'s; the
    /// recipe knows it from [`ButtonState::trailing_icon`].
    pub fn trailing(mut self, icon: impl IntoView) -> Self {
        self.trailing = Some(icon.into_view());
        self
    }
    /// Keep the accessible name while omitting the visible label (while
    /// true; constant or reactive). A button with a leading
    /// [`icon`](Self::icon) and no visible label is an icon button: its
    /// recipe knows it from [`ButtonState::is_icon_only`] and draws a square.
    #[track_caller]
    pub fn hide_label(mut self, hide: impl IntoValue<bool>) -> Self {
        self.hide_label = Some(hide.into_value());
        self
    }
    /// The content after the label, given once as a tuple or a list of views.
    pub fn children(mut self, children: impl IntoChildren) -> Self {
        self.children = children.into_views();
        self
    }
}
impl Button {}
impl Element for Button {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl Button {
    /// Finish the root, including themed content and busy indicator.
    fn finish(self) -> Node {
        let Button {
            mut root,
            label,
            icon,
            trailing,
            children,
            size,
            busy,
            input,
            hide_label,
        } = self;
        let variants = root.authored().variants();
        // A popup or disclosure attaches its trigger before the button is
        // finished; the recipe draws the button as that trigger.
        let expanded = root.accessible_expanded();
        // Busy ignores presses as the caller's `.disabled(..)` does. A button
        // that is neither busy nor disabled gets no disabled computed.
        let disabled = root.accessible_disabled();
        let root = if busy.is_some() || disabled.is_some() {
            root.disabled(move || {
                busy.is_some_and(|busy| busy.get()) || disabled.is_some_and(|d| d.get())
            })
        } else {
            root
        };
        let size = size.unwrap_or_else(|| crate::control_size::default_control_size().into_value());
        let (leading_icon, trailing_icon) = (icon.is_some(), trailing.is_some());
        let state = memo(move || ButtonState {
            input: input.get(),
            size: size.get(),
            busy: busy.is_some_and(|busy| busy.get()),
            expanded: expanded.map(|open| open.get()),
            leading_icon,
            trailing_icon,
            label_hidden: hide_label.is_some_and(|hide| hide.get()),
        });
        // A button in a button group takes the group's corners and dividers.
        let solid = matches!(
            crate::recipe::Look::of((), variants.clone())
                .variant::<ButtonVariant>()
                .unwrap_or_default(),
            ButtonVariant::Primary | ButtonVariant::Danger
        );
        let root = root.style(move || group_style(solid));
        // The text style, then the recipe: base layers below every `.style(..)` of the caller.
        let root = with_base_recipe_style(
            root.base_style(crate::helpers::single_line_text()),
            |r: &ButtonTheme| r.root_recipe(),
            &variants,
            move || state.get(),
        );
        let slot = |icon: View| {
            with_recipe_style(
                node().child(icon),
                |r: &ButtonTheme| r.icon_recipe(),
                &variants,
                move || state.get(),
            )
        };
        // Only a button the caller gave `.busy(..)` creates the busy clock, its
        // machine and its indicator (MEM-5): the others create no constant either.
        let busy_parts = busy.map(|_| busy_animation(state));
        let content = with_recipe_style(
            row((
                icon.map(slot),
                crate::helpers::visible_label(label, hide_label),
                trailing.map(slot),
                children,
            )),
            |r: &ButtonTheme| r.content_recipe(),
            &variants,
            move || {
                let mut state = state.get();
                state.busy &= !busy_parts.is_some_and(|(_, reveal)| reveal.get());
                state
            },
        );
        let indicator = busy_parts.map(|(phase, _)| busy_indicator(state, &variants, phase));
        root.child(content).child(indicator)
    }
}
/// The corners and dividers of a button in a button group, read from the
/// group's position. Outside a group the style is empty.
fn group_style(solid: bool) -> Style {
    let Some(position) = try_use_context::<GroupPosition>().map(|position| position.0.get()) else {
        return style();
    };
    let theme = use_theme().get();
    let divider = if solid {
        theme.get(&crate::button_group::look::SOLID_DIVIDER_VAR)
    } else {
        theme.get(&crate::button_group::look::DIVIDER_VAR)
    };
    let radius = theme.get(&crate::button_group::look::RADIUS_VAR);
    let first = if position.first { radius } else { 0.0 };
    let last = if position.last { radius } else { 0.0 };
    let vertical = position.layout.orientation == Orientation::Vertical;
    let (tl, tr, br, bl) = if vertical {
        (first, first, last, last)
    } else if position.layout.rtl {
        (last, first, first, last)
    } else {
        (first, last, last, first)
    };
    let base = style()
        .active(style().scale(1))
        .border_radius_corners(tl, tr, br, bl);
    if position.first {
        base
    } else if vertical {
        base.border_top(1, divider)
    } else if position.layout.rtl {
        base.border_right(1, divider)
    } else {
        base.border_left(1, divider)
    }
}

/// A button that shows only `icon` and is named `label`, with the label as
/// its tooltip. Its variant is `Ghost` unless the caller gives another with
/// `.variant(..)` on the button before it is wrapped.
#[track_caller]
pub fn icon_button(
    icon: impl IntoValue<kivact_core::style::Icon>,
    label: impl IntoText,
) -> crate::Tooltip<Button> {
    let label = label.into_text_value();
    crate::tooltip(
        label,
        button(label)
            .icon(kivact_core::view::icon(icon))
            .hide_label(true)
            .variant(ButtonVariant::Ghost),
    )
}

/// The busy indicator of `state`: its spin and delayed reveal.
fn busy_indicator(
    state: Value<ButtonState>,
    variants: &Variants,
    phase: Value<f64>,
) -> impl IntoView {
    let variants = variants.clone();
    show(
        move || state.get().busy,
        move || {
            let theme = use_theme().get();
            let delay = if kivact_core::platform::is_motion_reduced().get() {
                0
            } else {
                theme.get(&BUSY_REVEAL_DELAY_MS_VAR) as u64
            };
            let fade_ms = theme.get(&BUSY_REVEAL_FADE_MS_VAR);
            let fade = (delay != 0 || fade_ms != 0.0).then(|| {
                Animated::from(
                    0.0f64,
                    1.0f64,
                    Transition::tween(fade_ms, Easing::CubicBezier(0.25, 0.1, 0.25, 1.0))
                        .delay(delay as f64)
                        .essential(),
                )
            });
            with_recipe_style(
                node(),
                |r: &ButtonTheme| r.busy_recipe(),
                &variants,
                move || state.get(),
            )
            .style(move || {
                let base = style().rotate(phase.get() * 360.0);
                if let Some(fade) = fade {
                    base.opacity(fade.get())
                } else {
                    base
                }
            })
        },
    )
}
impl IntoView for Button {
    fn into_view(self) -> View {
        self.finish().into_view()
    }
}

/// Drive the busy clock and delayed reveal from the actual button state.
fn busy_animation(state: Value<ButtonState>) -> (Value<f64>, Value<bool>) {
    let phase = signal(0.0f64);
    let period = memo(|| {
        let theme = use_theme().get();
        if kivact_core::platform::is_motion_reduced().get() {
            theme.get(&BUSY_REDUCED_SPIN_MS_VAR)
        } else {
            theme.get(&BUSY_SPIN_MS_VAR)
        }
        .max(1.0)
    });
    let clock = frame_loop(move |frame| {
        let period = period.get();
        let elapsed = frame.now.as_secs_f64() * 1000.0;
        phase.set(elapsed.rem_euclid(period) / period);
    });
    let busy_machine = machine(
        BusyPhase::Idle,
        || {
            let theme = use_theme().get();
            BusyInput {
                delay: if kivact_core::platform::is_motion_reduced().get() {
                    0
                } else {
                    theme.get(&BUSY_REVEAL_DELAY_MS_VAR) as u64
                },
                fade: theme.get(&BUSY_REVEAL_FADE_MS_VAR) as u64,
                active: kivact_core::platform::is_surface_active().get(),
            }
        },
        move |_, running| {
            if running {
                clock.start(FrameRate::Hz(60.0));
            } else {
                clock.stop();
            }
        },
    );
    let loading = memo(move || state.get().busy);
    busy_machine.send(BusyEvent::Changed(loading.get_untracked()));
    busy_machine.feed(loading, BusyEvent::Changed);
    busy_machine.feed(kivact_core::platform::is_surface_active(), |_| {
        BusyEvent::SurfaceChanged
    });
    let reveal_pending = memo(move || busy_machine.get() == BusyPhase::Waiting);
    (phase.into_value(), reveal_pending.into_value())
}
