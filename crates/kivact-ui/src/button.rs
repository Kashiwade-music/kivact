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
use super::layer::with_recipe_style;
use super::values::{ButtonState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::motion::{Animated, Easing, FrameRate, Transition, frame_loop};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::state::{Value, on_cleanup, signal};
use kivact_core::style::{style, use_theme};
use kivact_core::task::{sleep, spawn};
use kivact_core::view::{IntoView, View, show};
use kivact_core::view::{Node, NodeBuilder, node, row};
use kivact_interaction::DisabledButtonFocus;
use kivact_interaction::use_button;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

/// Timing for the standard button's busy indicator, configured through
/// [`crate::UiTokens`]; the default turns every 700 ms and reveals immediately.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonBusyMotion {
    /// Milliseconds per revolution. A zero period is treated as 1 ms.
    pub spin_ms: u64,
    /// Milliseconds per revolution when reduced motion is active.
    pub reduced_spin_ms: u64,
    /// Milliseconds to wait before revealing. During this time the content
    /// recipe still sees the idle look. Skipped under reduced motion.
    pub reveal_delay_ms: u64,
    /// Milliseconds for the opacity fade. Zero reveals at once after the delay.
    pub reveal_fade_ms: u64,
}

impl Default for ButtonBusyMotion {
    fn default() -> Self {
        Self {
            spin_ms: 700,
            reduced_spin_ms: 700,
            reveal_delay_ms: 0,
            reveal_fade_ms: 0,
        }
    }
}

pub struct Button {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    trailing: Option<View>,
    children: Vec<View>,
    settings: Rc<RefCell<Settings>>,
    variants: Variants,
    busy: Rc<Cell<Value<bool>>>,
    expanded: Rc<Cell<Option<Value<bool>>>>,
    state: Value<ButtonState>,
    hide_label: Rc<Cell<Option<Value<bool>>>>,
}
/// What the builder knows of the button once it is configured.
#[derive(Clone)]
struct Settings {
    size: Value<ControlSize>,
    size_explicit: bool,
    leading_icon: bool,
    trailing_icon: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            size: super::values::default_control_size().into_value(),
            size_explicit: false,
            leading_icon: false,
            trailing_icon: false,
        }
    }
}
/// A themed button activated through the shared button part ([`use_button`]).
pub fn button(label: impl IntoText) -> Button {
    let label = label.into_text_value();
    let settings = Rc::new(RefCell::new(Settings {
        size: super::values::default_control_size().into_value(),
        ..Settings::default()
    }));
    let expanded = Rc::new(Cell::new(None::<Value<bool>>));
    let read_expanded = expanded.clone();
    let busy = Rc::new(Cell::new(false.into_value()));
    let read_loading = busy.clone();
    let hide_label = Rc::new(Cell::new(None::<Value<bool>>));
    let read_hide = hide_label.clone();
    let busy_value = (move || read_loading.get().get()).into_value();
    let part = use_button().disabled(busy_value).control();
    let input = part.state();
    let read = settings.clone();
    let state = kivact_core::state::memo(move || {
        let settings = read.borrow().clone();
        ButtonState {
            input: input.get(),
            size: settings.size.get(),
            busy: busy_value.get(),
            expanded: read_expanded.get().map(|open| open.get()),
            leading_icon: settings.leading_icon,
            trailing_icon: settings.trailing_icon,
            icon_only: settings.leading_icon && read_hide.get().is_some_and(|hide| hide.get()),
        }
    });
    let variants = Variants::default();
    let root = with_recipe_style(
        node()
            .style(super::helpers::single_line_text())
            .part(part)
            .focusable_when_disabled(matches!(
                use_theme()
                    .get()
                    .ext::<kivact_interaction::InteractionPolicy>()
                    .button
                    .disabled_focus,
                DisabledButtonFocus::Retain
            ))
            .name(label),
        |r| r.button_recipe(),
        &variants,
        move || state.get(),
    );
    Button {
        root,
        label,
        icon: None,
        trailing: None,
        children: Vec::new(),
        settings,
        variants,
        busy,
        expanded,
        state,
        hide_label,
    }
}
impl Button {
    fn configure(self, f: impl FnOnce(&mut Settings)) -> Self {
        f(&mut self.settings.borrow_mut());
        self
    }
    /// Set the size explicitly, overriding a size provider or group default.
    pub fn size(self, size: impl IntoValue<ControlSize>) -> Self {
        self.configure(|settings| {
            settings.size = size.into_value();
            settings.size_explicit = true;
        })
    }

    /// Set the default for this button when composing it into a group.
    pub fn default_size(self, size: ControlSize) -> Self {
        self.configure(|settings| {
            if !settings.size_explicit {
                settings.size = size.into_value();
            }
        })
    }
    /// Show the busy indicator and ignore presses while true (constant or
    /// reactive). Its spin and reveal use the theme's [`ButtonBusyMotion`].
    pub fn busy(mut self, busy: impl IntoValue<bool>) -> Self {
        let busy = busy.into_value();
        self.root = self.root.busy(busy);
        self.busy.set(busy);
        self
    }
    /// A view before the label, in a box the theme's
    /// [`UiRecipes::button_icon`](super::UiRecipes) styles (a kit sizes
    /// the icon there by the button's size).
    /// The recipe knows it from [`ButtonState::leading_icon`].
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self.configure(|s| s.leading_icon = true)
    }
    /// A view after the label, in a box styled like [`Self::icon`]'s; the
    /// recipe knows it from [`ButtonState::trailing_icon`].
    pub fn trailing(mut self, icon: impl IntoView) -> Self {
        self.trailing = Some(icon.into_view());
        self.configure(|s| s.trailing_icon = true)
    }
    /// Keep the accessible name while omitting the visible label (while
    /// true; constant or reactive). A button with a leading
    /// [`icon`](Self::icon) and no visible label is an icon button: its
    /// recipe knows it from [`ButtonState::icon_only`] and draws a square.
    #[track_caller]
    pub fn hide_label(self, hide: impl IntoValue<bool>) -> Self {
        self.hide_label.set(Some(hide.into_value()));
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
    pub fn variant(self, v: impl Variant) -> Self {
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
    fn finish(mut self) -> Node {
        // A popup or disclosure attaches its trigger before the button is
        // finished; the recipe draws the button as that trigger.
        if let Some(open) = self.root.accessible_expanded() {
            self.expanded.set(Some((move || open.get()).into_value()));
        }
        let state = self.state;
        let variants = self.variants;
        let slot = |icon: View| {
            with_recipe_style(
                node().child(icon),
                |r| r.button_icon_recipe(),
                &variants,
                move || state.get(),
            )
        };
        let reveal_pending = signal(false);
        let content = with_recipe_style(
            row((
                self.icon.map(slot),
                super::helpers::visible_label(self.label, self.hide_label.get()),
                self.trailing.map(slot),
                self.children,
            )),
            |r| r.button_content_recipe(),
            &variants,
            move || {
                let mut state = state.get();
                state.busy &= !reveal_pending.get();
                state
            },
        );
        let busy = show(
            move || state.get().busy,
            move || {
                let phase = signal(0.0f64);
                let period = kivact_core::state::memo(|| {
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
                kivact_core::state::effect(move || {
                    if kivact_core::platform::is_surface_active().get() {
                        clock.start(FrameRate::Hz(60.0));
                    } else {
                        clock.stop();
                    }
                });
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
                if delay != 0 {
                    reveal_pending.set(true);
                    on_cleanup(move || reveal_pending.set(false));
                    spawn(async move {
                        sleep(Duration::from_millis(delay)).await;
                        reveal_pending.set(false);
                    });
                }
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
        );
        self.root.child(content).child(busy)
    }
}
impl IntoView for Button {
    fn into_view(self) -> View {
        self.finish().into_view()
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ButtonTokens {
    pub busy_motion: ButtonBusyMotion,
}
