//! Role: Build the themed disclosure, a trigger over content it shows and hides.
//!
//! The trigger is the shared disclosure trigger part (a button publishing
//! `expanded` and `controls`), the content its content part. All four
//! recipes (the root, the trigger, the indicator and the content) get the
//! interaction part's [`DisclosureState`](kivact_interaction::DisclosureState).
//! An accordion is a column of disclosures whose `open` bindings read and
//! write one `Binding<Vec<K>>`.
use super::layer::{with_base_recipe_style, with_recipe_style};
use super::variants::{Variant, Variants};
use kivact_core::motion::{Easing, Transition};
use kivact_core::state::IntoText;
use kivact_core::state::{Binding, Value};
use kivact_core::style::{TransitionProp, style, use_theme};
use kivact_core::view::{IntoView, Orientation, View, show};
use kivact_core::view::{Node, NodeBuilder, icon, node, text};
use kivact_interaction::use_disclosure;

/// A trigger and the content it opens, created by [`disclosure`].
pub struct Disclosure {
    root: Node,
    title: Value<String>,
    open: Binding<bool>,
    content: kivact_core::view::ViewFn,
    motion: DisclosureMotion,
    variants: Variants,
}

/// A section titled `title` that `open` opens and closes, showing what
/// `content` builds while it is open.
///
/// The trigger is a button named and labelled by `title`, with the expanded
/// state; click, Enter, Space and assistive technology's Expand, Collapse and
/// Toggle write `open`. The content is built in a fresh owner each time the
/// section opens and dropped when it closes. Engine presence retains its
/// exit paint until the closing motion finishes. The look
/// is the theme's [`UiRecipes::disclosure`](super::UiRecipes). A caller's
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
pub fn disclosure(
    title: impl IntoText,
    open: impl kivact_core::state::IntoBinding<bool>,
) -> Disclosure {
    Disclosure {
        root: node(),
        title: title.into_text_value(),
        open: open.into_binding(),
        content: kivact_core::view::ViewFn::from(|()| ()),
        motion: use_theme()
            .get()
            .ext::<crate::UiTokens>()
            .disclosure
            .motion
            .clone(),
        variants: Variants::default(),
    }
}

impl Disclosure {
    /// Build this section each time it opens.
    pub fn content<V: IntoView>(mut self, content: impl kivact_core::view::IntoViewFn<V>) -> Self {
        self.content = content.into_view_fn();
        self
    }
    /// Open and close by moving the content's height over 200 ms, a
    /// decorative motion that reduced motion skips (default: the content
    /// appears and disappears at once). A constant.
    pub fn animated(mut self, animated: bool) -> Self {
        self.motion.animate = animated;
        self
    }

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

impl NodeBuilder for Disclosure {
    /// The column of the trigger over the content.
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Disclosure {
    fn into_view(mut self) -> View {
        let mut details = use_disclosure(self.open);
        // `.disabled(..)` ignores input and assistive technology.
        if let Some(disabled) = self.root.accessible_disabled() {
            details = details.disabled(move || disabled.get());
        }
        let shown = details.is_open();
        let motion = self.motion;
        let variants = self.variants;
        let part = details.trigger();
        let state = part.state();
        let title = self.title;
        let indicator = with_recipe_style(
            node().child(icon("chevron-down")),
            |recipes| recipes.disclosure_recipes().indicator_recipe(),
            &variants,
            move || state.get(),
        );
        let trigger = with_recipe_style(
            node().part(part).name(title),
            |recipes| recipes.disclosure_recipes().trigger_recipe(),
            &variants,
            move || state.get(),
        )
        .child((
            text(title).style(super::helpers::label_style().grow(1)),
            indicator,
        ));
        let build = self.content;
        let region = {
            let variants = variants.clone();
            move || {
                let part = details.content();
                let content_state = part.state();
                let region = with_recipe_style(
                    node().part(part),
                    |recipes| recipes.disclosure_recipes().content_recipe(),
                    &variants,
                    move || content_state.get(),
                );
                let region = if motion.animate {
                    region.style(
                        style()
                            .enter_size(Orientation::Vertical)
                            .exit_size(Orientation::Vertical)
                            .transition(
                                TransitionProp::Size,
                                Transition::tween(
                                    motion.duration.as_secs_f64() * 1000.0,
                                    Easing::EaseOut,
                                ),
                            ),
                    )
                } else {
                    region
                };
                region.child(build.call(()))
            }
        };
        let body = show(shown, region);
        with_base_recipe_style(
            self.root,
            |recipes| recipes.disclosure_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .child(trigger)
        .child(body)
        .into_view()
    }
}

pub use crate::disclosure_tokens::{DisclosureMotion, DisclosureTokens};
