//! Role: Render the App's toasts in themed live regions.
use super::layer::with_recipe_style;
use super::values::{ButtonVariant, ToastGroupState};
use super::variants::{Variant, Variants};
use kivact_core::control::{NodeBuilder, column, node, text};
use kivact_core::motion::Transition;
use kivact_core::primitive::part;
use kivact_core::style::{Theme, ThemeExtension, current_theme};
use kivact_core::view::{IntoView, LiveRegion, View, for_each};
use kivact_interaction::{ToastId, Toaster, Tone};

/// Where the toasts of `toaster` show: two live regions (polite and
/// assertive) that the region recipe places, by default at the window's
/// bottom right.
///
/// A toast shows its icon, title and body, its action as a button (which
/// dismisses the toast and runs the action) and a close button named by the
/// message `toast.close` ("Close"). Its time, Escape, and pausing under the
/// pointer and the focus are [`Toaster::item`]'s, and
/// [`hold_all_on_hover`](ToastRegion::hold_all_on_hover) stops every
/// toast's time while the region is hovered or focused, so a look that
/// stacks the toasts opens the stack then (each toast's
/// [`ToastState`](kivact_interaction::ToastState) says how deep it is and
/// whether it is held). How many show at once is the application's
/// [`Toaster::max_visible`]; mounting the region changes nothing else in the
/// toaster. Mount one region near the root of the App.
///
/// ```no_run
/// use kivact::ui::{Toast, toaster};
/// use kivact::prelude::*;
/// use kivact::ui::toast_region;
///
/// fn app() -> impl IntoView {
///     let toaster = toaster();
///     column((
///         button("Save").on_press(move || {
///             toaster.push(Toast::new("Saved"));
///         }),
///         toast_region(toaster),
///     ))
/// }
/// ```
#[track_caller]
pub fn toast_region(toaster: Toaster) -> ToastRegion {
    ToastRegion {
        toaster,
        variants: Variants::default(),
        hold_all: false,
    }
}

/// A theme's optional toast row-height animation.
///
/// The region reads this at setup. With a transition, set the group
/// recipe's fixed gap to zero so the space between cards can animate.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToastRowMotion {
    /// A decorative transition for showing and collapsing the row.
    pub transition: Option<Transition>,
    /// Space before each card, included in its animated height.
    pub gap: f32,
}

impl ThemeExtension for ToastRowMotion {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}

/// The live regions of a [`Toaster`], created by [`toast_region`].
pub struct ToastRegion {
    toaster: Toaster,
    variants: Variants,
    hold_all: bool,
}

impl ToastRegion {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
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

impl ToastRegion {
    /// Stop the time of every toast, not only the one under the pointer or
    /// the focus, while the pointer is over the region or the focus is in it
    /// ([`Toaster::hold_while`]): what a stack of toasts that opens under the
    /// pointer does (sonner). A constant: it is the region's behavior.
    pub fn hold_all_on_hover(mut self, on: bool) -> Self {
        self.hold_all = on;
        self
    }
}

impl IntoView for ToastRegion {
    fn into_view(self) -> View {
        let ToastRegion {
            toaster,
            variants,
            hold_all,
        } = self;
        let polite = toaster.visible(false);
        let row_motion = current_theme().get::<ToastRowMotion>();
        let (rows, groups) = (variants.clone(), variants.clone());
        let group = move |assertive: bool, live: LiveRegion| {
            let rows = rows.clone();
            let shown = toaster.visible(assertive);
            let build = move |id| toast(toaster, id, &rows);
            let items = if let Some(transition) = row_motion.transition.clone() {
                super::toast_rows::animate(shown, transition, row_motion.gap, build)
            } else {
                for_each(shown, |id| *id, move |id, _| build(id))
            };
            let part = toaster.region(live);
            let input = part.state();
            with_recipe_style(
                column(items).part(part),
                |r| r.toast_recipes().group_recipe(),
                &groups,
                move || ToastGroupState {
                    input: input.get(),
                    assertive,
                    count: shown.with(Vec::len),
                    above: if assertive { polite.with(Vec::len) } else { 0 },
                },
            )
        };
        let region = part(|input| input);
        let state = region.state();
        if hold_all {
            let anchor = region.node_ref();
            let hovered = anchor.hovered();
            let focused = anchor.focus_within();
            toaster.hold_while(kivact_core::memo(move || hovered.get() || focused.get()));
        }
        with_recipe_style(
            column((
                group(false, LiveRegion::Polite),
                group(true, LiveRegion::Assertive),
            ))
            .part(region),
            |r| r.toast_recipes().region_recipe(),
            &variants,
            move || state.get(),
        )
        .into_view()
    }
}

/// The name in the theme's icon set of the icon `tone` draws, if the theme
/// has one. A neutral toast has none.
fn tone_icon(tone: Tone) -> Option<&'static str> {
    let name = match tone {
        Tone::Neutral => return None,
        Tone::Info => "toast-info",
        Tone::Success => "toast-success",
        Tone::Warning => "toast-warning",
        Tone::Danger => "toast-danger",
        Tone::Accent => "toast-accent",
    };
    kivact_core::style::current_theme()
        .icons()
        .get(name)
        .map(|_| name)
}

/// One toast's row.
fn toast(toaster: Toaster, id: ToastId, variants: &Variants) -> View {
    let Some(content) = toaster.toast(id) else {
        return ().into_view();
    };
    let part = toaster.item(id);
    let state = part.state();
    let title = with_recipe_style(
        text(content.title.clone()),
        |r| r.toast_recipes().title_recipe(),
        variants,
        move || state.get(),
    );
    let body = content.body.clone().map(|body| {
        with_recipe_style(
            text(body),
            |r| r.toast_recipes().body_recipe(),
            variants,
            move || state.get(),
        )
    });
    let icon = match &content.icon {
        Some(icon) => Some(icon()),
        None => tone_icon(content.tone).map(|name| {
            with_recipe_style(
                kivact_core::control::icon(name),
                |r| r.toast_recipes().icon_recipe(),
                variants,
                move || state.get(),
            )
            .into_view()
        }),
    };
    let action = content.action.as_ref().map(|(label, _)| {
        let activate = content.clone();
        super::button(label.clone())
            .variant(ButtonVariant::Outline)
            .size(super::ControlSize::Small)
            .on_press(move || activate.activate(id))
    });
    let close = with_recipe_style(
        node()
            .part(kivact_interaction::use_button().control())
            .name(kivact_core::i18n::message("toast.close", "Close"))
            .on_press(move || toaster.dismiss(id))
            .child(kivact_core::control::icon("x")),
        |r| r.toast_recipes().close_recipe(),
        variants,
        move || state.get(),
    );
    with_recipe_style(
        node()
            .part(part)
            .child(icon)
            .child(column((title, body)).style(kivact_core::style::style().grow(1).min_width(0)))
            .child(action)
            .child(close),
        |r| r.toast_recipes().toast_recipe(),
        variants,
        move || state.get(),
    )
    .into_view()
}
