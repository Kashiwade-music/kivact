//! Role: The toast region builder, its toast cards and their dismiss controls.

use crate::button::ButtonVariant;
use crate::layer::with_recipe_style;
use crate::toast::look::ToastGroupState;
use crate::toast::look::ToastTheme;
use crate::toast::swipe;
use kivact_core::custom::{Caller, part};
use kivact_core::geometry::Edge;
use kivact_core::motion::Transition;
use kivact_core::state::{IntoValue, Value};
use kivact_core::style::use_theme;
use kivact_core::task::spawn;
use kivact_core::view::Element;
use kivact_core::view::Variants;
use kivact_core::view::{IntoView, LiveRegion, View, for_each};
use kivact_core::view::{column, node, text};
use kivact_interaction::ToastSwipe;
use kivact_interaction::{SwipeDismissApi, SwipeDismissPolicy, use_swipe_dismiss};
use kivact_interaction::{ToastId, ToasterApi, Tone};
use kivact_macros::component;

/// Where the toasts of `toaster` show: two live regions (polite and
/// assertive) that the region recipe places, by default at the window's
/// bottom right.
///
/// A toast shows its icon, title and body, its action as a button (which
/// dismisses the toast and runs the action) and a close button named by the
/// message `toast.close` ("Close"). Its time, Escape, and pausing under the
/// pointer and the focus are [`ToasterApi::item`]'s, and
/// `hold_all_on_hover` stops every
/// toast's time while the region is hovered or focused, so a look that
/// stacks the toasts opens the stack then (each toast's
/// [`ToastState`](kivact_interaction::ToastState) says how deep it is and
/// whether it is held). How many show at once is the application's
/// [`ToasterApi::max_visible`]; mounting the region changes nothing else in the
/// toaster. Mount one region near the root of the App.
///
/// ```no_run
/// use kivact::interaction::{Toast, use_toaster};
/// use kivact::prelude::*;
/// use kivact::ui::toast_region;
///
/// fn app() -> impl IntoView {
///     let toaster = use_toaster();
///     column((
///         button("Save").on_press(move || {
///             toaster.push(Toast::new("Saved"));
///         }),
///         toast_region(toaster),
///     ))
/// }
/// ```
#[component]
pub fn toast_region(
    toaster: ToasterApi,
    hold_all_on_hover: Option<Value<bool>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let hold_all = hold_all_on_hover.unwrap_or_else(|| false.into_value());
    let polite = toaster.visible(false);
    let (rows, groups) = (variants.clone(), variants.clone());
    let group = move |assertive: bool, live: LiveRegion| {
        let rows = rows.clone();
        let shown = toaster.visible(assertive);
        let row_variants = rows.clone();
        let build = move |id| toast(toaster, id, &rows);
        let items = for_each(
            shown,
            |id| *id,
            move |id| {
                let (card, swiping) = build(id.get_untracked());
                let row = with_recipe_style(
                    column(card),
                    |r: &ToastTheme| r.row_recipe(),
                    &row_variants,
                    kivact_core::custom::PartState::default,
                );
                match swiping {
                    Some((gesture, _, exit)) => {
                        row.style(move || swipe::row_style(gesture.phase().get(), &exit))
                    }
                    None => row,
                }
            },
        );
        let part = toaster.region(live);
        let input = part.state();
        with_recipe_style(
            column(items).part(part),
            |r: &ToastTheme| r.group_recipe(),
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
    let anchor = kivact_core::view::node_ref();
    let state = region.state();
    {
        let hovered = anchor.hovered();
        let focused = anchor.focus_within();
        toaster.hold_while(kivact_core::state::memo(move || {
            hold_all.get() && (hovered.get() || focused.get())
        }));
    }
    with_recipe_style(
        column((
            group(false, LiveRegion::Polite),
            group(true, LiveRegion::Assertive),
        ))
        .node_ref(anchor)
        .part(region),
        |r: &ToastTheme| r.region_recipe(),
        &variants,
        move || state.get(),
    )
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
    kivact_core::style::use_theme()
        .get()
        .icons()
        .get(name)
        .map(|_| name)
}

/// The icon of a toast card: the toast's own icon, or the one its tone draws.
fn card_icon(
    content: &kivact_interaction::Toast,
    state: Value<kivact_interaction::ToastState>,
    variants: &Variants,
) -> Option<View> {
    content
        .icon_name
        .or_else(|| tone_icon(content.tone))
        .map(|name| {
            with_recipe_style(
                kivact_core::view::icon(name),
                |r: &ToastTheme| r.icon_recipe(),
                variants,
                move || state.get(),
            )
            .into_view()
        })
}

/// One toast's card, with its swipe gesture and the gesture's exit
/// transition when the theme enables swiping.
fn toast(
    toaster: ToasterApi,
    id: ToastId,
    variants: &Variants,
) -> (View, Option<(SwipeDismissApi, Edge, Transition)>) {
    let Some(content) = toaster.toast(id) else {
        return (().into_view(), None);
    };
    let card_ref = kivact_core::view::node_ref();
    let policy = use_theme()
        .get()
        .ext::<kivact_interaction::InteractionPolicy>()
        .toast
        .clone();
    let (part, swiping) = match &policy.swipe {
        ToastSwipe::Disabled => (toaster.item(id), None),
        ToastSwipe::Enabled { edge, transition } => {
            // A task runs in the same turn once the effects have settled:
            // the card's style has committed its released look (the swipe
            // exit and the theme's transition) before the card is removed,
            // so the exit starts from the drag offset instead of jumping
            // through the drag's zero-length transition.
            let gesture = use_swipe_dismiss(
                SwipeDismissPolicy::toast(*edge, policy.swipe_slop),
                card_ref.rect(),
                move || {
                    spawn(async move { toaster.dismiss(id) });
                },
            );
            let part = toaster.item_while(id, gesture.is_held());
            (part, Some((gesture, *edge, transition.clone())))
        }
    };
    let gesture = swiping.as_ref().map(|(gesture, _, _)| *gesture);
    let state = part.state();
    let title = with_recipe_style(
        text(content.title.clone()),
        |r: &ToastTheme| r.title_recipe(),
        variants,
        move || state.get(),
    );
    let body = content.body.clone().map(|body| {
        with_recipe_style(
            text(body),
            |r: &ToastTheme| r.body_recipe(),
            variants,
            move || state.get(),
        )
    });
    let icon = card_icon(&content, state, variants);
    let action = action_button(&content, id, gesture);
    let close = close_button(toaster, id, gesture, state, variants);
    let card = with_recipe_style(
        node()
            .node_ref(card_ref)
            .part(part)
            .child(icon)
            .child(
                column((title, body)).style(kivact_core::style::style().flex_grow(1).min_width(0)),
            )
            .child(action)
            .child(close),
        |r: &ToastTheme| r.toast_recipe(),
        variants,
        move || state.get(),
    );
    if let Some((gesture, edge, transition)) = swiping {
        let exit = transition.clone();
        let card = card
            .part(gesture.control())
            .style(move || swipe::card_style(edge, gesture.phase().get(), &transition))
            .into_view();
        (card, Some((gesture, edge, exit)))
    } else {
        (card.into_view(), None)
    }
}

/// The toast's action button, protected from swipe activation.
fn action_button(
    content: &kivact_interaction::Toast,
    id: ToastId,
    gesture: Option<SwipeDismissApi>,
) -> Option<crate::Button> {
    content.action.as_ref().map(|(label, _)| {
        let activate = content.clone();
        let action = crate::button(label.clone())
            .variant(ButtonVariant::Outline)
            .size(crate::ControlSize::Small)
            .on_press(move || activate.activate(id));
        if let Some(gesture) = gesture {
            action.with_node(|node| node.part(gesture.protect()))
        } else {
            action
        }
    })
}

/// Render a toast dismiss control protected from swipe gesture activation.
fn close_button(
    toaster: ToasterApi,
    id: ToastId,
    gesture: Option<SwipeDismissApi>,
    state: Value<kivact_interaction::ToastState>,
    variants: &Variants,
) -> kivact_core::view::Node {
    let close = node()
        .part(kivact_interaction::use_button().control())
        .name(kivact_core::i18n::message("toast.close", "Close"))
        .on_press(move || toaster.dismiss(id))
        .child(kivact_core::view::icon("x"));
    let close = if let Some(gesture) = gesture {
        close.part(gesture.protect())
    } else {
        close
    };
    with_recipe_style(
        close,
        |r: &ToastTheme| r.close_recipe(),
        variants,
        move || state.get(),
    )
}
