//! Role: Build the themed button from a momentary toggle part.
//!
//! The root, the content row, the icon boxes and the busy indicator all get
//! one [`ButtonState`]. Its `expanded` is read from the finished root's
//! semantics (`NodeExt::accessible_expanded`): `Some(open)` when a popup,
//! menu or disclosure trigger part was attached or the caller set
//! `.expanded`, `None` otherwise. The `.icon` and `.trailing_icon` views go
//! into `button_icon` boxes, where a kit sizes the icon by the button's size.
//! While loading, the busy part turns once every 0.7 s, also under reduced
//! motion: it says that work is going on (essential motion, as the
//! platforms' own spinners), not decoration. A [`ButtonVariant`] is only a meaning,
//! given with `.variant(..)` like a kit's tone or slanted shape and read by
//! the recipe with `look.variant::<ButtonVariant>()` (a button without one is
//! the default, `Primary`). A kit drawing its own part as a button calls the
//! recipe with
//! `Look::new(ButtonState::new(input, size)).with_variant(variant)`.
//! The caller's view in `.icon`, `.trailing_icon` and `.child` is composed
//! in, without reimplementing activation.
use super::layer::with_recipe_style;
use super::values::{ButtonState, ControlSize};
use super::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, node, row};
use kivact_core::motion::{FrameRate, frame_loop};
use kivact_core::primitive::NodeExt;
use kivact_core::style::style;
use kivact_core::view::{IntoText, IntoValue, IntoView, View, show};
use kivact_core::{Value, signal};
use kivact_interaction::use_button;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
pub struct Button {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    trailing: Option<View>,
    children: Vec<View>,
    settings: Rc<RefCell<Settings>>,
    variants: Variants,
    loading: Rc<Cell<Value<bool>>>,
    expanded: Rc<Cell<Option<Value<bool>>>>,
    state: Value<ButtonState>,
    phase: kivact_core::Signal<f32>,
    hide_label: Rc<Cell<Option<Value<bool>>>>,
}
/// What the builder knows of the button once it is configured.
#[derive(Clone, Default)]
struct Settings {
    size: ControlSize,
    leading_icon: bool,
    trailing_icon: bool,
}
/// A themed button activated through the shared button part ([`use_button`]).
pub fn button(label: impl IntoText) -> Button {
    let label = label.into_text_value();
    let settings = Rc::new(RefCell::new(Settings::default()));
    let expanded = Rc::new(Cell::new(None::<Value<bool>>));
    let read_expanded = expanded.clone();
    let loading = Rc::new(Cell::new(false.into_value()));
    let read_loading = loading.clone();
    let hide_label = Rc::new(Cell::new(None::<Value<bool>>));
    let read_hide = hide_label.clone();
    let busy = (move || read_loading.get().get()).into_value();
    let part = use_button().control().disabled(busy);
    let input = part.state();
    let phase = signal(0.0f32);
    let read = settings.clone();
    let state = kivact_core::memo(move || {
        let settings = read.borrow().clone();
        ButtonState {
            input: input.get(),
            size: settings.size,
            loading: busy.get(),
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
            .name(label),
        |r| r.button_recipe(),
        &variants,
        move || state.get(),
    );
    // The clock is mounted only while busy; idle buttons have no loop. The
    // turn is essential motion, so reduced motion does not stop it.
    let animation = show(busy, move || {
        let clock = frame_loop(move |frame| phase.set((frame.now.as_secs_f32() / 0.7).fract()));
        kivact_core::effect(move || {
            if kivact_core::platform::surface_active() {
                clock.start(FrameRate::Hz(60.0));
            } else {
                clock.stop();
            }
        });
    });
    Button {
        root: root.child(animation),
        label,
        icon: None,
        trailing: None,
        children: Vec::new(),
        settings,
        variants,
        loading,
        expanded,
        state,
        phase,
        hide_label,
    }
}
impl Button {
    fn configure(self, f: impl FnOnce(&mut Settings)) -> Self {
        f(&mut self.settings.borrow_mut());
        self
    }
    /// Selected state of a toggle-like button (semantics and the `selected`
    /// style).
    ///
    /// Same as [`NodeExt::selected`](kivact_core::primitive::NodeExt::selected).
    pub fn selected(mut self, v: impl IntoValue<bool>) -> Self {
        self.root = self.root.selected(v);
        self
    }

    /// Expanded state of a button that opens or closes something (a
    /// disclosure trigger).
    ///
    /// Same as [`NodeExt::expanded`](kivact_core::primitive::NodeExt::expanded).
    pub fn expanded(mut self, v: impl IntoValue<bool>) -> Self {
        self.root = self.root.expanded(v);
        self
    }

    pub fn size(self, size: ControlSize) -> Self {
        self.configure(|s| s.size = size)
    }
    /// Show the busy indicator and ignore presses while true (constant or
    /// reactive). The indicator turns once every 0.7 s, also under reduced
    /// motion (it is essential motion: it shows that work goes on).
    pub fn loading(self, loading: impl IntoValue<bool>) -> Self {
        self.loading.set(loading.into_value());
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
    pub fn trailing_icon(mut self, icon: impl IntoView) -> Self {
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
    /// Finish the public root, including themed content and busy indicator.
    pub fn into_node(mut self) -> Node {
        // A popup or disclosure attaches its trigger before the button is
        // finished; the recipe draws the button as that trigger.
        if let Some(open) = self.root.accessible_expanded() {
            self.expanded.set(Some((move || open()).into_value()));
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
        let content = with_recipe_style(
            row((
                self.icon.map(slot),
                super::helpers::visible_label(self.label, self.hide_label.get()),
                self.trailing.map(slot),
                self.children,
            )),
            |r| r.button_content_recipe(),
            &variants,
            move || state.get(),
        );
        let phase = self.phase;
        let busy = show(
            move || state.get().loading,
            move || {
                with_recipe_style(
                    node(),
                    |r| r.button_busy_recipe(),
                    &variants,
                    move || state.get(),
                )
                .style(move || style().rotate(phase.get() * 360.0))
            },
        );
        self.root.child(content).child(busy)
    }
}
impl IntoView for Button {
    fn into_view(self) -> View {
        self.into_node().into_view()
    }
}
