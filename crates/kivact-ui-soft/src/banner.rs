//! Role: Neutral feedback banners with optional details and dismissal.

use crate::feedback::icon_box;
use crate::{Elevation, tokens};
use kivact::control::{Node, NodeBuilder, column, icon, row, text};
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::primitive::{NodeExt, Owner};
use kivact::style::{Align, Direction, Justify, TransitionProp, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, button, tooltip};
use kivact::view::{
    HandlerResult, IntoText, IntoView, ReadingDirection, View, callback, current_direction,
    node_ref, show,
};
use kivact::{Binding, Callback, Value, effect, signal};

/// Semantic tone of a feedback banner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BannerStatus {
    Info,
    Warning,
    Error,
    Success,
}

/// The banner's container treatment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BannerContainer {
    #[default]
    Card,
    Section,
}

/// A status message with optional controls and details.
pub struct Banner {
    root: Node,
    status: BannerStatus,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    end: Option<View>,
    content: Option<Box<dyn Fn() -> View>>,
    container: BannerContainer,
    elevation: Elevation,
    collapsible: bool,
    open: Option<Binding<bool>>,
    dismissable: bool,
    dismiss: Option<Callback<()>>,
    dismiss_label: Option<Value<String>>,
}

#[track_caller]
pub fn banner(status: BannerStatus, title: impl IntoText) -> Banner {
    Banner {
        root: column(()),
        status,
        title: title.into_text_value(),
        description: None,
        icon: None,
        end: None,
        content: None,
        container: BannerContainer::Card,
        elevation: Elevation::None,
        collapsible: true,
        open: None,
        dismissable: false,
        dismiss: None,
        dismiss_label: None,
    }
}

impl Banner {
    pub fn description(mut self, description: impl IntoText) -> Self {
        self.description = Some(description.into_text_value());
        self
    }

    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    pub fn end<V: IntoView>(mut self, end: impl FnOnce() -> V) -> Self {
        self.end = Some(end().into_view());
        self
    }

    pub fn content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Box::new(move || content().into_view()));
        self
    }

    pub fn container(mut self, container: BannerContainer) -> Self {
        self.container = container;
        self
    }

    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn open(mut self, open: impl Into<Binding<bool>>) -> Self {
        self.open = Some(open.into());
        self
    }

    pub fn dismissable(mut self, dismissable: bool) -> Self {
        self.dismissable = dismissable;
        self
    }

    pub fn on_dismiss<R: HandlerResult>(mut self, dismiss: impl Fn() -> R + 'static) -> Self {
        self.dismiss = Some(callback(move |_| dismiss()));
        self
    }

    pub fn dismiss_label(mut self, label: impl IntoText) -> Self {
        self.dismiss_label = Some(label.into_text_value());
        self
    }
}

impl NodeBuilder for Banner {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Banner {
    fn into_view(self) -> View {
        let dismissed = signal(false);
        let dismiss_focus = self.dismissable.then(|| {
            let reference = node_ref();
            let within = reference.focus_within();
            let owner = Owner::current("banner");
            let previous = signal(None);
            effect(move || {
                let focused = owner.focused_node();
                if !dismissed.get() && !within.get() && focused.is_some() {
                    previous.set(focused);
                }
            });
            (reference, previous)
        });
        let card = self.container == BannerContainer::Card;
        let status = self.status;
        let has_details = self.content.is_some();
        let disclosure = (has_details && self.collapsible)
            .then(|| use_disclosure(self.open.unwrap_or_else(|| signal(false).into())));
        let open = move || has_details && disclosure.is_none_or(|details| details.shown().get());
        let title = self.title;
        let elevation = self.elevation;
        let has_end = self.end.is_some();
        let has_actions = has_end || self.dismissable || disclosure.is_some();
        let description = self.description;
        let has_description = description.is_some();
        let direction = if current_direction() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        };

        let foreground = move || {
            let t = tokens();
            match status {
                BannerStatus::Info => t.colors.on_info_muted,
                BannerStatus::Warning => t.colors.on_warning_muted,
                BannerStatus::Error => t.colors.on_error_muted,
                BannerStatus::Success => t.colors.on_success_muted,
            }
        };
        let toggle = disclosure.map(|details| {
            let expand = kivact::i18n::message("soft.banner.expand", "Expand");
            let collapse = kivact::i18n::message("soft.banner.collapse", "Collapse");
            let label = move || {
                if details.shown().get() {
                    collapse.get()
                } else {
                    expand.get()
                }
            };
            let chevron = icon("chevron-down").style(move || {
                let t = tokens();
                style()
                    .color(foreground())
                    .rotate(if details.shown().get() { 180 } else { 0 })
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                    )
            });
            let control = button(label)
                .size(ControlSize::Small)
                .variant(ButtonVariant::Ghost)
                .hide_label(true)
                .icon(chevron)
                .part(details.trigger());
            tooltip(label).attach(control)
        });
        let dismiss = self.dismissable.then(|| {
            let label = self
                .dismiss_label
                .unwrap_or_else(|| kivact::i18n::message("soft.banner.dismiss", "Dismiss"));
            let name = self.dismiss_label.unwrap_or_else(|| {
                kivact::i18n::message_with(
                    "soft.banner.dismiss_titled",
                    "{dismiss} {title}",
                    move || [("dismiss", label.get()), ("title", title.get())],
                )
            });
            let callback = self.dismiss;
            let control = button(name)
                .size(ControlSize::Small)
                .variant(ButtonVariant::Ghost)
                .hide_label(true)
                .icon(icon("x").style(move || style().color(foreground())))
                .on_press(move || {
                    dismissed.set(true);
                    if let Some((_, previous)) = dismiss_focus
                        && let Some(previous) = previous.get_untracked()
                    {
                        previous.restore_focus();
                    }
                    if let Some(callback) = callback {
                        callback.call(());
                    }
                });
            tooltip(label).attach(control)
        });
        let action_area = has_actions.then(|| {
            row((self.end, toggle, dismiss)).base_style(move || {
                style()
                    .direction(direction)
                    .wrap(true)
                    .align_items(Align::Center)
                    .justify_content(Justify::End)
                    .gap(8)
                    .shrink(0)
                    .max_width(pct(100))
                    .margin_y(-4)
                    .color(tokens().colors.text_primary)
            })
        });
        let label = column((
            text(title).style(style().font_size(14).line_height(20.0 / 14.0).weight(600)),
            description.map(|description| {
                text(description).style(style().font_size(12).line_height(20.0 / 12.0).weight(400))
            }),
        ))
        .base_style(if has_end {
            style().grow(1).basis(128).min_width(0)
        } else {
            style().grow(1).min_width(0)
        });
        let leading_icon = self.icon.unwrap_or_else(|| {
            icon(match status {
                BannerStatus::Info => "info",
                BannerStatus::Warning => "triangle-alert",
                BannerStatus::Error => "circle-x",
                BannerStatus::Success => "circle-check",
            })
            .into_view()
        });
        let header =
            row((icon_box(leading_icon, 20.0), label, action_area)).base_style(move || {
                let t = tokens();
                let background = match status {
                    BannerStatus::Info => t.colors.info_muted,
                    BannerStatus::Warning => t.colors.warning_muted,
                    BannerStatus::Error => t.colors.error_muted,
                    BannerStatus::Success => t.colors.success_muted,
                };
                let radius = if card { t.radius.container } else { 0.0 };
                let bottom = if open() { 0.0 } else { radius };
                style()
                    .direction(direction)
                    .wrap(true)
                    .column_gap(8)
                    .row_gap(12)
                    .padding_y(12)
                    .padding_x(16)
                    .align_items(if has_actions && !has_description {
                        Align::Center
                    } else {
                        Align::Start
                    })
                    .radius_corners(radius, radius, bottom, bottom)
                    .background(background)
                    .color(foreground())
            });
        let content = self.content.map(|content| {
            show(open, move || {
                let mut area = column(content()).base_style(move || {
                    let t = tokens();
                    let radius = if card { t.radius.container } else { 0.0 };
                    style()
                        .padding_y(12)
                        .padding_x(16)
                        .background(t.colors.bg_card)
                        .radius_corners(0, 0, radius, radius)
                        .border_left(1, t.colors.border)
                        .border_right(1, t.colors.border)
                        .border_bottom(1, t.colors.border)
                });
                if let Some(details) = disclosure {
                    area = area.part(details.content());
                }
                area
            })
        });
        let mut root = self.root;
        if let Some((reference, _)) = dismiss_focus {
            root = root.node_ref(reference);
        }
        root.hidden(move || dismissed.get())
            .base_style(move || {
                let t = tokens();
                elevation.apply(
                    style().radius(if card { t.radius.container } else { 0.0 }),
                    &t,
                )
            })
            .child(header)
            .child(content)
            .into_view()
    }
}
