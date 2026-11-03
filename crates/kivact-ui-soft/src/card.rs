//! Role: Render the Neutral card surface and propagate its content insets.
use crate::layout::LayoutInset;
use crate::stack::{ContainerInset, Space};
use crate::{Elevation, tokens};
use kivact::control::{Node, NodeBuilder, ScrollAxes, column, node, scroll};
use kivact::draw::Shadow;
use kivact::interaction::{use_button, use_checkbox};
use kivact::motion::Transition;
use kivact::primitive::NodeExt;
use kivact::style::{Color, Direction, Length, TransitionProp, current_theme, style};
use kivact::view::{Handled, IntoText, IntoValue, IntoView, Key, KeyEvent, View, provide};
use kivact::{Binding, Value};

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
    variant: CardVariant,
    elevation: Elevation,
    height: Option<Length>,
    action: CardAction,
    disabled: Value<bool>,
}

pub fn card(children: impl IntoView) -> Card {
    Card {
        root: node(),
        content: children.into_view(),
        padding: None,
        variant: CardVariant::Default,
        elevation: Elevation::None,
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
    selected: impl Into<Binding<bool>>,
    children: impl IntoView,
) -> Card {
    let mut card = card(children).name(label);
    card.action = CardAction::Checkbox(selected.into());
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
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = Some(height.into());
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.root = self.root.style(style().width(width));
        self
    }
}

impl NodeBuilder for Card {
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
            let token = tokens();
            let bordered = variant == CardVariant::Default;
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
            let content = column(content).base_style(style().padding(physical_padding));
            let interactive = !matches!(action, CardAction::Plain);
            let clickable = matches!(action, CardAction::Button);
            let checked = match action {
                CardAction::Checkbox(selected) => Some(selected),
                _ => None,
            };
            let root = match action {
                CardAction::Plain => root,
                CardAction::Button => root.part(use_button().disabled(disabled).control()),
                CardAction::Checkbox(selected) => root
                    .part(use_checkbox(selected).disabled(disabled).control())
                    .on_key(move |event: KeyEvent| {
                        if event.key == Key::Enter
                            && !event.repeat
                            && !disabled.get()
                            && !event.modifiers.alt
                            && !event.modifiers.ctrl
                            && !event.modifiers.meta
                        {
                            selected.set(!selected.get());
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    }),
            };
            let mut root = root.base_style(move || {
                let t = tokens();
                let c = t.colors;
                let bg = match variant {
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
                    .min_width(0)
                    .radius(t.radius.container)
                    .clip(true)
                    .background(bg);
                let selected = checked.is_some_and(|checked| checked.get());
                let ring = match variant {
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
                    base =
                        base.border(t.sizes.border_width, if selected { ring } else { c.border });
                }
                if selected {
                    base = base.shadow(Shadow::new(0, 0, 0, ring).spread(2).inset());
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
                    } else {
                        let theme = current_theme();
                        let mut hover = style().background(crate::recipe_style::overlay(
                            &theme,
                            bg,
                            c.overlay_hover,
                        ));
                        if clickable && bordered {
                            hover = hover.border_color(c.border_emphasized);
                        }
                        base = base.hover(hover).pressed(style().background(
                            crate::recipe_style::overlay(&theme, bg, c.overlay_pressed),
                        ));
                    }
                }
                base = elevation.apply(base, &t);
                if let Some(height) = height {
                    base = base.height(height);
                }
                base
            });
            if height.is_some_and(|height| height != Length::Auto) {
                root = root.child(
                    scroll(content)
                        .axes(ScrollAxes::Vertical)
                        .style(style().grow(1).min_height(0)),
                );
            } else {
                root = root.child(content);
            }
            root
        })
    }
}
