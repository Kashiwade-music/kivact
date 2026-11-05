//! Role: Render the Neutral card surface and propagate its content insets.
use crate::Elevation;
use crate::layout::LayoutInset;
use crate::stack::{ContainerInset, Space};
use kivact::custom::PartState;
use kivact::draw::Shadow;
use kivact::interaction::use_button;
use kivact::interaction::use_checkbox;
use kivact::motion::Transition;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::provide;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Length;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::style;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ScrollAxes;
use kivact::view::View;
use kivact::view::column;
use kivact::view::node;
use kivact::view::scroll;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVariant {
    #[default]
    Default,
    Transparent,
    Muted,
    Blue,
    Cyan,
    Gray,
    Green,
    Orange,
    Pink,
    Purple,
    Red,
    Teal,
    Yellow,
}

pub struct Card {
    root: Node,
    content: View,
    padding: Option<Space>,
    variant: Value<CardVariant>,
    elevation: Value<Elevation>,
    height: Option<Value<Length>>,
    action: CardAction,
    disabled: Value<bool>,
}

pub fn card(children: impl IntoView) -> Card {
    Card {
        root: node(),
        content: children.into_view(),
        padding: None,
        variant: (CardVariant::Default).into_value(),
        elevation: (Elevation::None).into_value(),
        height: None,
        action: CardAction::Plain,
        disabled: false.into_value(),
    }
}

enum CardAction {
    Plain,
    Button,
    Checkbox(Binding<bool>),
}

pub fn clickable_card(label: impl IntoText, children: impl IntoView) -> Card {
    let mut card = card(children).name(label);
    card.action = CardAction::Button;
    card
}

pub fn selectable_card(
    label: impl IntoText,
    selected: impl IntoBinding<bool>,
    children: impl IntoView,
) -> Card {
    let mut card = card(children).name(label);
    card.action = CardAction::Checkbox(selected.into_binding());
    card
}

impl Card {
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn padding(mut self, padding: Space) -> Self {
        self.padding = Some(padding);
        self
    }
    pub fn variant(mut self, variant: impl IntoValue<CardVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    pub fn elevation(mut self, elevation: impl IntoValue<Elevation>) -> Self {
        self.elevation = elevation.into_value();
        self
    }
    pub fn height(mut self, height: impl IntoValue<Length>) -> Self {
        self.height = Some(height.into_value());
        self
    }
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        let width = width.into_value();
        self.root = self.root.style(move || style().width(width.get()));
        self
    }
}

impl NodeBuilder for Card {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Card {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            padding,
            variant,
            elevation,
            height,
            action,
            disabled,
        } = self;
        provide((), move || {
            let token = kivact::style::use_tokens::<crate::Tokens>();
            let bordered = variant.get() == CardVariant::Default;
            let logical_padding = padding.map_or(token.spacing.card_padding, Space::px);
            let physical_padding = (logical_padding
                - if bordered {
                    token.sizes.border_width
                } else {
                    0.0
                })
            .max(0.0);
            let actual_inset = ContainerInset::uniform(physical_padding);
            let layout_inset = LayoutInset::from_padding(ContainerInset::uniform(logical_padding));
            let content = provide(actual_inset, move || provide(layout_inset, move || content));
            let content = column(content)
                .with_node(move |node| node.base_style(style().padding(physical_padding)));
            let interactive = !matches!(action, CardAction::Plain);
            let clickable = matches!(action, CardAction::Button);
            let checked = match action {
                CardAction::Checkbox(selected) => Some(selected),
                _ => None,
            };
            let (root, input) = match action {
                CardAction::Plain => (root, None),
                CardAction::Button => {
                    let part = use_button().disabled(disabled).control();
                    let state = part.state();
                    (root.with_node(move |node| node.part(part)), Some(state))
                }
                CardAction::Checkbox(selected) => {
                    let part = use_checkbox(selected).disabled(disabled).control();
                    let state = part.state();
                    let input = kivact::state::memo(move || state.get().input);
                    let root = root.with_node(move |node| node.part(part)).on_key(
                        move |event: KeyEvent| {
                            if event.key() == Key::Enter
                                && input.get().focused
                                && !event.repeat()
                                && !disabled.get()
                                && !event.modifiers().alt
                                && !event.modifiers().ctrl
                                && !event.modifiers().meta
                            {
                                selected.set(!selected.get());
                                Handled::Yes
                            } else {
                                Handled::No
                            }
                        },
                    );
                    (root, Some(input))
                }
            };
            let mut root = root.with_node(move |node| {
                node.base_style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let c = t.colors;
                    let bg = match variant.get() {
                        CardVariant::Default => c.bg_card,
                        CardVariant::Transparent => Color::TRANSPARENT,
                        CardVariant::Muted => c.bg_muted,
                        CardVariant::Blue => c.hue.blue.bg,
                        CardVariant::Cyan => c.hue.cyan.bg,
                        CardVariant::Gray => c.hue.gray.bg,
                        CardVariant::Green => c.hue.green.bg,
                        CardVariant::Orange => c.hue.orange.bg,
                        CardVariant::Pink => c.hue.pink.bg,
                        CardVariant::Purple => c.hue.purple.bg,
                        CardVariant::Red => c.hue.red.bg,
                        CardVariant::Teal => c.hue.teal.bg,
                        CardVariant::Yellow => c.hue.yellow.bg,
                    };
                    let mut base = style()
                        .direction(Direction::Column)
                        .position(Position::Relative)
                        .min_width(0)
                        .radius(t.radius.container)
                        .clip(true)
                        .background(bg);
                    let selected = checked.is_some_and(|checked| checked.get());
                    let ring = match variant.get() {
                        CardVariant::Default | CardVariant::Transparent | CardVariant::Muted => {
                            c.accent
                        }
                        CardVariant::Blue => c.hue.blue.border,
                        CardVariant::Cyan => c.hue.cyan.border,
                        CardVariant::Gray => c.hue.gray.border,
                        CardVariant::Green => c.hue.green.border,
                        CardVariant::Orange => c.hue.orange.border,
                        CardVariant::Pink => c.hue.pink.border,
                        CardVariant::Purple => c.hue.purple.border,
                        CardVariant::Red => c.hue.red.border,
                        CardVariant::Teal => c.hue.teal.border,
                        CardVariant::Yellow => c.hue.yellow.border,
                    };
                    if bordered {
                        base = base
                            .border(t.sizes.border_width, if selected { ring } else { c.border });
                    }
                    if checked.is_some() {
                        let color = if selected { ring } else { Color::TRANSPARENT };
                        base = base.shadow(Shadow::new(0, 0, 0, color).spread(2).inset());
                    }
                    if interactive {
                        base = base
                            .focus_visible(crate::recipe_style::ring(&t, c.accent))
                            .transition(
                                TransitionProp::Paint,
                                Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                            );
                        if disabled.get() {
                            base = base.opacity(t.sizes.disabled_opacity);
                        } else if clickable && bordered {
                            base = base.hover(style().border_color(c.border_emphasized));
                        }
                    }
                    base = elevation.get().apply(base, &t);
                    if let Some(height) = height {
                        base = base.height(height.get());
                    }
                    base
                })
            });
            if height.is_some() {
                root = root.child(
                    scroll(content)
                        .axes(ScrollAxes::Vertical)
                        .style(style().grow(1).min_height(0)),
                );
            } else {
                root = root.child(content);
            }
            root.child(input.map(card_overlay))
        })
    }
}

fn card_overlay(input: Value<PartState>) -> Node {
    node().hit_testable(false).style(move || {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let state = input.get();
        let color = if state.disabled {
            Color::TRANSPARENT
        } else if state.pressed {
            t.colors.overlay_pressed
        } else if state.hovered {
            t.colors.overlay_hover
        } else {
            Color::TRANSPARENT
        };
        style()
            .position(Position::Absolute)
            .top(0)
            .bottom(0)
            .left(0)
            .right(0)
            .radius(t.radius.container)
            .background(color)
            .transition(
                TransitionProp::Paint,
                Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
            )
    })
}

kivact::state::constant_value!(CardVariant);
