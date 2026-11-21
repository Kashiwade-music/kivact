//! Role: The disclosure builder, its trigger and content rendering, and its motion.

use crate::disclosure::look::DisclosureMotion;
use crate::disclosure::look::DisclosureTheme;
use crate::layer::{with_base_recipe_style, with_recipe_style};
use kivact_core::motion::{Easing, Transition};
use kivact_core::state::{IntoText, IntoValue};
use kivact_core::state::{Signal, Value};
use kivact_core::style::{TransitionProp, style, use_theme};
use kivact_core::view::Variants;
use kivact_core::view::{Element, Node, icon, node, text};
use kivact_core::view::{IntoView, Orientation, View, show};
use kivact_interaction::use_disclosure;

// xtask-allow(state): legacy: always_open, keep_mounted, heading_level, animation, group
/// A trigger and the content it opens, created by [`disclosure`].
pub struct Disclosure {
    root: Node,
    title: Value<String>,
    open: Signal<bool>,
    content: kivact_core::view::ViewFn,
    motion: DisclosureMotion,
    inherited: Variants,
    always_open: bool,
    keep_mounted: bool,
    heading_level: Option<u32>,
    animation: Option<Value<bool>>,
    group: Option<crate::theme::DisclosureGroupPosition>,
}

/// A section titled `title` that `open` opens and closes, showing what
/// `content` builds while it is open.
///
/// The trigger is a button named and labelled by `title`, with the expanded
/// state; click, Enter, Space and assistive technology's Expand, Collapse and
/// Toggle write `open`. The content is built in a fresh owner each time the
/// section opens and dropped when it closes (unless
/// [`keep_mounted`](Disclosure::keep_mounted)). Engine presence retains its
/// exit paint until the closing motion finishes. The look
/// is the theme's [`DisclosureTheme`](crate::theme::DisclosureTheme). A caller's
/// `.disabled(..)` on the disclosure disables the trigger.
///
/// An accordion is a column of disclosures over one list of open keys; each
/// section's `open` is a binding that reads and writes its key in the list:
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::disclosure;
///
/// fn faq() -> impl IntoView {
///     let open = signal(vec!["shipping"]);
///     let section = move |key: &'static str, title: &'static str, body: &'static str| {
///         let is_open = binding(
///             move || open.with(|keys| keys.contains(&key)),
///             move |on| {
///                 open.update(|keys| {
///                     keys.retain(|k| *k != key);
///                     if on {
///                         keys.push(key);
///                     }
///                 })
///             },
///         );
///         disclosure(title, is_open).content(move || text(body))
///     };
///     column((
///         section("shipping", "Shipping", "Ships in 2 days."),
///         section("returns", "Returns", "30 day returns."),
///     ))
/// }
/// ```
#[track_caller]
pub fn disclosure(title: impl IntoText, open: kivact_core::state::Signal<bool>) -> Disclosure {
    Disclosure {
        root: node(),
        title: title.into_text_value(),
        open,
        content: kivact_core::view::ViewFn::from(|()| ()),
        motion: use_theme()
            .get()
            .ext::<DisclosureTheme>()
            .motion_value()
            .clone(),
        inherited: Variants::default(),
        always_open: false,
        keep_mounted: false,
        heading_level: None,
        animation: None,
        group: None,
    }
}

impl Disclosure {
    pub(crate) fn group_position(mut self, index: usize, count: usize) -> Self {
        self.group = Some(crate::theme::DisclosureGroupPosition { index, count });
        self
    }
    /// The looks of the group this section is in; the section's own
    /// `.variant(..)` values win over them.
    pub(crate) fn group_variants(mut self, variants: Variants) -> Self {
        self.inherited = variants;
        self
    }

    /// The group's looks with the section's own values over them.
    fn looks(&self) -> Variants {
        let mut looks = self.inherited.clone();
        looks.merge(&self.root.authored().variants());
        looks
    }

    pub(crate) fn heading(mut self, level: u32) -> Self {
        self.heading_level = Some(level);
        self
    }

    /// Keep the content expanded and prevent the trigger from collapsing it
    /// while `always` is true. This structural choice is fixed before mounting.
    pub fn always_open(mut self, always: bool) -> Self {
        self.always_open = always;
        self
    }

    /// Build the content once and keep it mounted, hidden while the section
    /// is closed, so what it holds (the text of an input, a scroll position)
    /// survives closing, while `keep` is true. The hidden content takes no
    /// space, input or assistive technology; the open and close motion of
    /// [`animated`](Self::animated) does not run. This structural choice is
    /// fixed before mounting.
    pub fn keep_mounted(mut self, keep: bool) -> Self {
        self.keep_mounted = keep;
        self
    }

    /// Attach disclosure behavior to a caller-owned trigger.
    pub fn attach<T: Element + IntoView>(mut self, trigger: T) -> impl kivact_core::view::Element {
        let variants = self.looks();
        let open = if self.always_open {
            kivact_core::state::binding(|| true, |_| ())
        } else {
            self.open
        };
        let details = use_disclosure(open);
        let always_open = self.always_open;
        let disabled = match (self.root.accessible_disabled(), always_open) {
            (Some(d), _) => Some((move || always_open || d.get()).into_value()),
            (None, true) => Some(true.into_value()),
            (None, false) => None,
        };
        let trigger = trigger.with_node(|root| {
            let root = match disabled {
                Some(d) => root.disabled(d),
                None => root,
            };
            root.part(details.trigger())
        });
        let content = self.content;
        let motion = self.motion;
        let animation = self.animation;
        let group = self.group;
        let region =
            move || content_region(details, &content, &motion, animation, &variants, group);
        let body = if self.keep_mounted {
            region().into_view()
        } else {
            show(details.is_open(), region).into_view()
        };
        self.root
            .style(style().flex_direction(kivact_core::style::FlexDirection::Column))
            .child((trigger, body))
    }

    /// Build this section each time it opens (once, with
    /// [`keep_mounted`](Self::keep_mounted)).
    pub fn content<V: IntoView>(mut self, content: impl kivact_core::view::IntoViewFn<V>) -> Self {
        self.content = content.into_view_fn();
        self
    }
    /// Open and close by moving the content's height over 200 ms, a
    /// decorative motion that reduced motion skips (default: the content
    /// appears and disappears at once). A constant or reactive value.
    #[track_caller]
    pub fn animated(mut self, animated: impl kivact_core::state::IntoValue<bool>) -> Self {
        self.animation = Some(animated.into_value());
        self
    }
}

impl Element for Disclosure {
    /// The column of the trigger over the content.
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Disclosure {
    fn into_view(mut self) -> View {
        let variants = self.looks();
        let open = if self.always_open {
            kivact_core::state::binding(|| true, |_| ())
        } else {
            self.open
        };
        let details = use_disclosure(open);
        // `.disabled(..)` ignores input and assistive technology.
        let always_open = self.always_open;
        let disabled = match (self.root.accessible_disabled(), always_open) {
            (Some(d), _) => Some((move || always_open || d.get()).into_value()),
            (None, true) => Some(true.into_value()),
            (None, false) => None,
        };
        let shown = details.is_open();
        let motion = self.motion;
        let animation = self.animation;
        let group = self.group;
        let part = details.trigger();
        let state = part.state();
        let title = self.title;
        let indicator = with_recipe_style(
            node().child(icon("chevron-down")),
            |r: &DisclosureTheme| r.indicator_recipe(),
            &variants,
            move || crate::disclosure::look::DisclosureRecipeState::from_part(state.get(), group),
        );
        let root = match disabled {
            Some(d) => node().disabled(d),
            None => node(),
        };
        let trigger = with_recipe_style(
            root.part(part).name(title),
            |r: &DisclosureTheme| r.trigger_recipe(),
            &variants,
            move || crate::disclosure::look::DisclosureRecipeState::from_part(state.get(), group),
        )
        .child((
            text(title).style(crate::helpers::label_style().flex_grow(1)),
            indicator,
        ));
        let trigger = match self.heading_level {
            Some(level) => node()
                .role(kivact_core::view::Role::Heading)
                .level(level)
                .child(trigger)
                .into_view(),
            None => trigger.into_view(),
        };
        let build = self.content;
        let region = {
            let variants = variants.clone();
            move || content_region(details, &build, &motion, animation, &variants, group)
        };
        let body = if self.keep_mounted {
            region().into_view()
        } else {
            show(shown, region).into_view()
        };
        with_base_recipe_style(
            self.root,
            |r: &DisclosureTheme| r.root_recipe(),
            &variants,
            move || crate::disclosure::look::DisclosureRecipeState::from_part(state.get(), group),
        )
        .child(trigger)
        .child(body)
        .into_view()
    }
}

fn content_region(
    details: kivact_interaction::DisclosureApi,
    build: &kivact_core::view::ViewFn,
    motion: &DisclosureMotion,
    animation: Option<Value<bool>>,
    variants: &Variants,
    group: Option<crate::theme::DisclosureGroupPosition>,
) -> Node {
    let part = details.content();
    let content_state = part.state();
    let region = with_recipe_style(
        node().part(part),
        |r: &DisclosureTheme| r.content_recipe(),
        variants,
        move || {
            crate::disclosure::look::DisclosureRecipeState::from_part(content_state.get(), group)
        },
    );
    let duration = motion.duration;
    let default_animate = motion.animate;
    let transition = motion.transition.clone();
    let region = region.style(move || {
        if animation.map_or(default_animate, |value| value.get()) {
            style()
                .enter_size(Orientation::Vertical)
                .exit_size(Orientation::Vertical)
                .transition(
                    TransitionProp::Size,
                    transition.clone().unwrap_or_else(|| {
                        Transition::tween(duration.as_secs_f64() * 1000.0, Easing::EaseOut)
                    }),
                )
        } else {
            style()
        }
    });
    region.child(build.call(()))
}
