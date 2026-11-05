//! Role: Neutral feedback banners with optional details and dismissal.
use kivact::state::IntoValue;
use kivact::view::IntoViewFn;
use kivact::view::ViewFn;

use crate::Elevation;
use crate::feedback::icon_box;
use kivact::custom::Owner;
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::effect;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ReadingDirection;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::show;
use kivact::view::text;
use kivact::view::use_direction;

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
    content: Option<ViewFn>,
    container: Value<BannerContainer>,
    elevation: Value<Elevation>,
    collapsible: Value<bool>,
    open: Option<Binding<bool>>,
    dismissable: Value<bool>,
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
        container: (BannerContainer::Card).into_value(),
        elevation: (Elevation::None).into_value(),
        collapsible: (true).into_value(),
        open: None,
        dismissable: (false).into_value(),
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

    pub fn end(mut self, end: impl IntoView) -> Self {
        self.end = Some(end.into_view());
        self
    }

    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }

    pub fn container(mut self, container: impl IntoValue<BannerContainer>) -> Self {
        self.container = container.into_value();
        self
    }

    pub fn elevation(mut self, elevation: impl IntoValue<Elevation>) -> Self {
        self.elevation = elevation.into_value();
        self
    }

    pub fn collapsible(mut self, collapsible: impl IntoValue<bool>) -> Self {
        self.collapsible = collapsible.into_value();
        self
    }

    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = Some(open.into_binding());
        self
    }

    pub fn dismissable(mut self, dismissable: impl IntoValue<bool>) -> Self {
        self.dismissable = dismissable.into_value();
        self
    }

    pub fn on_dismiss(mut self, dismiss: impl Into<Callback<()>>) -> Self {
        let dismiss: Callback<()> = dismiss.into();
        self.dismiss = Some(callback(move |_| dismiss.call(())));
        self
    }

    pub fn dismiss_label(mut self, label: impl IntoText) -> Self {
        self.dismiss_label = Some(label.into_text_value());
        self
    }
}

impl NodeBuilder for Banner {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Banner {
    fn into_view(self) -> View {
        let dismissed = signal(false);
        let dismiss_focus = self.dismissable.get().then(|| {
            let reference = node_ref();
            let within = reference.focus_within();
            let owner = Owner::current();
            let previous = signal(None);
            effect(move || {
                let focused = owner.focused_node();
                if !dismissed.get() && !within.get() && focused.is_some() {
                    previous.set(focused);
                }
            });
            (reference, previous)
        });
        let card = self.container.get() == BannerContainer::Card;
        let status = self.status;
        let has_details = self.content.is_some();
        let disclosure = (has_details && self.collapsible.get())
            .then(|| use_disclosure(self.open.unwrap_or_else(|| signal(false).into_binding())));
        let open = move || has_details && disclosure.is_none_or(|details| details.is_open().get());
        let title = self.title;
        let elevation = self.elevation.get();
        let has_end = self.end.is_some();
        let has_actions = has_end || self.dismissable.get() || disclosure.is_some();
        let description = self.description;
        let has_description = description.is_some();
        let direction = if use_direction().get() == ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        };

        let foreground = move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
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
                if details.is_open().get() {
                    collapse.get()
                } else {
                    expand.get()
                }
            };
            let chevron = icon("chevron-down").style(move || {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                style()
                    .color(foreground())
                    .rotate(if details.is_open().get() { 180 } else { 0 })
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
                .with_node(move |node| node.part(details.trigger()));
            tooltip(label).attach(control)
        });
        let dismiss = self.dismissable.get().then(|| {
            let label = self
                .dismiss_label
                .unwrap_or_else(|| kivact::i18n::message("soft.banner.dismiss", "DismissOn"));
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
            row((self.end, toggle, dismiss)).with_node(move |node| {
                node.base_style(move || {
                    style()
                        .direction(direction)
                        .wrap(true)
                        .align_items(Align::Center)
                        .justify_content(Justify::End)
                        .gap(8)
                        .shrink(0)
                        .max_width(pct(100))
                        .margin_y(-4)
                        .color(
                            kivact::style::use_tokens::<crate::Tokens>()
                                .colors
                                .text_primary,
                        )
                })
            })
        });
        let label = column((
            text(title).style(style().font_size(14).line_height(20.0 / 14.0).weight(600)),
            description.map(|description| {
                text(description).style(style().font_size(12).line_height(20.0 / 12.0).weight(400))
            }),
        ))
        .with_node(move |node| {
            node.base_style(if has_end {
                style().grow(1).basis(128).min_width(0)
            } else {
                style().grow(1).min_width(0)
            })
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
            row((icon_box(leading_icon, 20.0), label, action_area)).with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
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
                })
            });
        let content = self.content.map(|content| {
            show(open, move || {
                let mut area = column(content.call(())).with_node(move |node| {
                    node.base_style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        let radius = if card { t.radius.container } else { 0.0 };
                        style()
                            .padding_y(12)
                            .padding_x(16)
                            .background(t.colors.bg_card)
                            .radius_corners(0, 0, radius, radius)
                            .border_left(1, t.colors.border)
                            .border_right(1, t.colors.border)
                            .border_bottom(1, t.colors.border)
                    })
                });
                if let Some(details) = disclosure {
                    area = area.with_node(move |node| node.part(details.content()));
                }
                area
            })
        });
        let mut root = self.root;
        if let Some((reference, _)) = dismiss_focus {
            root = root.node_ref(reference);
        }
        root.hidden(move || dismissed.get())
            .with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    elevation.apply(
                        style().radius(if card { t.radius.container } else { 0.0 }),
                        &t,
                    )
                })
            })
            .child(header)
            .child(content)
            .into_view()
    }
}

kivact::state::constant_value!(BannerContainer);
