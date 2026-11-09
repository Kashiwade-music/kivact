//! Role: Build the themed button from a momentary toggle part.
//!
//! The root, the content row, the icon boxes and the busy indicator all get
//! one [`ButtonState`]. Its `expanded` is read from the finished root's
//! semantics (`Node::accessible_expanded`): `Some(open)` when a popup,
//! menu or disclosure trigger part was attached or the caller set
//! `.expanded`, `None` otherwise. The `.icon` and `.trailing` views go
//! into `button_icon` boxes, where a kit sizes the icon by the button's size.
//! While busy, the busy part uses the theme's [`ButtonBusyMotion`].
//! A [`ButtonVariant`] is only a meaning,
//! given with `.variant(..)` like a kit's tone or slanted shape and read by
//! the recipe with `look.variant::<ButtonVariant>()` (a button without one is
//! the default, `Primary`). A kit drawing its own part as a button calls the
//! recipe with
//! `Look::new(ButtonState::new(input, size)).with_variant(variant)`.
//! The caller's view in `.icon`, `.trailing` and `.child` is composed
//! in, without reimplementing activation.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::values::{ButtonState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::custom::PartState;
use kivact_core::motion::{Animated, Easing, FrameRate, Transition, frame_loop};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::state::{Machine, Step, Value, machine, memo, signal};
use kivact_core::style::{style, use_theme};
use kivact_core::view::{IntoView, View, show};
use kivact_core::view::{Node, NodeBuilder, node, row};
use kivact_interaction::DisabledButtonFocus;
use kivact_interaction::use_button;
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

pub struct Button {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    trailing: Option<View>,
    children: Vec<View>,
    size: ButtonSize,
    variants: Variants,
    busy: Value<bool>,
    input: Value<PartState>,
    hide_label: Option<Value<bool>>,
}
/// Where the button's size comes from: a default a group may replace, or
/// the caller's `.size(..)`, which nothing replaces.
#[derive(Clone, Copy)]
enum ButtonSize {
    Default(ControlSize),
    Explicit(Value<ControlSize>),
}
impl ButtonSize {
    fn value(self) -> Value<ControlSize> {
        match self {
            Self::Default(size) => size.into_value(),
            Self::Explicit(size) => size,
        }
    }
}
/// A themed button activated through the shared button part ([`use_button`]).
pub fn button(label: impl IntoText) -> Button {
    let label = label.into_text_value();
    let part = use_button().control();
    let input = part.state();
    let root = node()
        .part(part)
        .focusable_when_disabled(matches!(
            use_theme()
                .get()
                .ext::<kivact_interaction::InteractionPolicy>()
                .button
                .disabled_focus,
            DisabledButtonFocus::Retain
        ))
        .name(label);
    Button {
        root,
        label,
        icon: None,
        trailing: None,
        children: Vec::new(),
        size: ButtonSize::Default(super::values::default_control_size()),
        variants: Variants::default(),
        busy: false.into_value(),
        input,
        hide_label: None,
    }
}
impl Button {
    /// Set the size explicitly, overriding a size provider or group default.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = ButtonSize::Explicit(size.into_value());
        self
    }

    /// Set the default for this button when composing it into a group.
    pub fn default_size(mut self, size: ControlSize) -> Self {
        if let ButtonSize::Default(_) = self.size {
            self.size = ButtonSize::Default(size);
        }
        self
    }
    /// Show the busy indicator and ignore presses while true (constant or
    /// reactive). Its spin and reveal use the theme's [`ButtonBusyMotion`].
    pub fn busy(mut self, busy: impl IntoValue<bool>) -> Self {
        let busy = busy.into_value();
        self.root = self.root.busy(busy);
        self.busy = busy;
        self
    }
    /// A view before the label, in a box the theme's
    /// [`UiRecipes::button_icon`](super::UiRecipes) styles (a kit sizes
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
    /// recipe knows it from [`ButtonState::icon_only`] and draws a square.
    #[track_caller]
    pub fn hide_label(mut self, hide: impl IntoValue<bool>) -> Self {
        self.hide_label = Some(hide.into_value());
        self
    }
    pub fn child(mut self, child: impl IntoView) -> Self {
        self.children.push(child.into_view());
        self
    }
}
impl Button {
    /// Give the button a [`Variant`]: its [`ButtonVariant`](super::ButtonVariant)
    /// (the default is `Primary`), or a look a kit defines beyond it. Every
    /// recipe of the button reads it with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(mut self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}
impl NodeBuilder for Button {
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
            variants,
            busy,
            input,
            hide_label,
        } = self;
        // A popup or disclosure attaches its trigger before the button is
        // finished; the recipe draws the button as that trigger.
        let expanded = root.accessible_expanded();
        // Busy ignores presses as the caller's `.disabled(..)` does.
        let disabled = root.accessible_disabled();
        let root = root.disabled(move || busy.get() || disabled.is_some_and(|d| d.get()));
        let size = size.value();
        let (leading_icon, trailing_icon) = (icon.is_some(), trailing.is_some());
        let state = memo(move || ButtonState {
            input: input.get(),
            size: size.get(),
            busy: busy.get(),
            expanded: expanded.map(|open| open.get()),
            leading_icon,
            trailing_icon,
            icon_only: leading_icon && hide_label.is_some_and(|hide| hide.get()),
        });
        // The text style, then the recipe: base layers below every `.style(..)` of the caller.
        let root = with_base_recipe_style(
            root.base_style(super::helpers::single_line_text()),
            |r| r.button_recipe(),
            &variants,
            move || state.get(),
        );
        let slot = |icon: View| {
            with_recipe_style(
                node().child(icon),
                |r| r.button_icon_recipe(),
                &variants,
                move || state.get(),
            )
        };
        let (phase, reveal_pending) = busy_animation(state);
        let content = with_recipe_style(
            row((
                icon.map(slot),
                super::helpers::visible_label(label, hide_label),
                trailing.map(slot),
                children,
            )),
            |r| r.button_content_recipe(),
            &variants,
            move || {
                let mut state = state.get();
                state.busy &= !reveal_pending.get();
                state
            },
        );
        let indicator = busy_indicator(state, &variants, phase);
        root.child(content).child(indicator)
    }
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
            let motion = use_theme()
                .get()
                .ext::<crate::UiTokens>()
                .button
                .busy_motion;
            let delay = if kivact_core::platform::is_motion_reduced().get() {
                0
            } else {
                motion.reveal_delay_ms
            };
            let fade = (delay != 0 || motion.reveal_fade_ms != 0).then(|| {
                Animated::from(
                    0.0f64,
                    1.0f64,
                    Transition::tween(
                        motion.reveal_fade_ms as f64,
                        Easing::CubicBezier(0.25, 0.1, 0.25, 1.0),
                    )
                    .delay(delay as f64)
                    .essential(),
                )
            });
            with_recipe_style(
                node(),
                |r| r.button_busy_recipe(),
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

pub use crate::button_tokens::{ButtonBusyMotion, ButtonTokens};

/// Drive the busy clock and delayed reveal from the actual button state.
fn busy_animation(state: Value<ButtonState>) -> (Value<f64>, Value<bool>) {
    let phase = signal(0.0f64);
    let period = memo(|| {
        let motion = use_theme()
            .get()
            .ext::<crate::UiTokens>()
            .button
            .busy_motion;
        if kivact_core::platform::is_motion_reduced().get() {
            motion.reduced_spin_ms
        } else {
            motion.spin_ms
        }
        .max(1) as f64
    });
    let clock = frame_loop(move |frame| {
        let period = period.get();
        let elapsed = frame.now.as_secs_f64() * 1000.0;
        phase.set(elapsed.rem_euclid(period) / period);
    });
    let busy_machine = machine(
        BusyPhase::Idle,
        || {
            let motion = use_theme()
                .get()
                .ext::<crate::UiTokens>()
                .button
                .busy_motion;
            BusyInput {
                delay: if kivact_core::platform::is_motion_reduced().get() {
                    0
                } else {
                    motion.reveal_delay_ms
                },
                fade: motion.reveal_fade_ms,
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
