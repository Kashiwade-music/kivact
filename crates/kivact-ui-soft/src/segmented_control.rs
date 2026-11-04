//! Role: The Neutral segmented radio presentation.
use kivact::state::Value;

use kivact::state::IntoBinding;
use kivact::state::ItemKey;

use crate::Tokens;
use crate::recipe_style::{fast, ring};
use kivact::interaction::ListItem;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::effect;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Cursor;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::CheckableState;
use kivact::ui::ControlSize;
use kivact::ui::Look;
use kivact::ui::RadioGroup;
use kivact::ui::Variant;
use kivact::ui::radio_group;
use kivact::ui::tooltip;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::Orientation;
use kivact::view::View;
use kivact::view::icon;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::show;
use kivact::view::text;

/// Apply the segmented appearance to a horizontal standard radio group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Segmented;

/// Whether a segmented control sizes to content or its container.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentedLayout {
    #[default]
    Hug,
    Fill,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SegmentedIconOnly;

impl Variant for Segmented {}
impl Variant for SegmentedLayout {}
impl Variant for SegmentedIconOnly {}

/// Neutral segmented radio group.
pub struct SegmentedControl<K: ItemKey> {
    radio: RadioGroup<K>,
    size: Value<ControlSize>,
    layout: Value<SegmentedLayout>,
    icon_only: Value<bool>,
    disabled: Signal<bool>,
    disabled_reason: Option<kivact::state::Value<String>>,
}

#[track_caller]
pub fn segmented_control<K: ItemKey>(
    label: impl IntoText,
    options: impl IntoValue<Vec<ListItem<K>>>,
    value: impl IntoBinding<Option<K>>,
) -> SegmentedControl<K> {
    let disabled = signal(false);
    let options = options.into_value();
    let options = move || {
        let items = options.get();
        if disabled.get() {
            items.into_iter().map(|item| item.disabled(true)).collect()
        } else {
            items
        }
    };
    SegmentedControl {
        radio: radio_group(label, options, value)
            .orientation(Orientation::Horizontal)
            .variant(Segmented),
        size: (ControlSize::Medium).into_value(),
        layout: (SegmentedLayout::Hug).into_value(),
        icon_only: (false).into_value(),
        disabled,
        disabled_reason: None,
    }
}

impl<K: ItemKey> SegmentedControl<K> {
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    pub fn layout(mut self, layout: impl IntoValue<SegmentedLayout>) -> Self {
        self.layout = layout.into_value();
        self
    }

    pub fn icon_only(mut self, hide: impl IntoValue<bool>) -> Self {
        self.icon_only = hide.into_value();
        self
    }

    pub fn disabled(self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled.set(disabled);
        self
    }

    pub fn disabled_reason(mut self, reason: impl IntoText) -> Self {
        self.disabled_reason = Some(reason.into_text_value());
        self
    }
}

impl<K: ItemKey> NodeBuilder for SegmentedControl<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        self.radio.node_mut()
    }
}

impl<K: ItemKey> IntoView for SegmentedControl<K> {
    fn into_view(self) -> View {
        let icon_only = self.icon_only.get();
        let size = self.size.get();
        let disabled = self.disabled.get_untracked();
        let focused = signal(None);
        let (hint_anchor, hint) = crate::toolbar::keyboard_hint(false, move || focused.get());
        let radio = self
            .radio
            .size(size)
            .variant(self.layout.get())
            .item(move |item, state| {
                let reference = node_ref();
                effect(move || {
                    if !disabled && state.get().input.focus_visible {
                        focused.set(Some(reference));
                    } else if focused.get_untracked() == Some(reference) {
                        focused.set(None);
                    }
                });
                let glyph = show(
                    move || item.get().icon().is_some(),
                    move || {
                        let side = match size {
                            ControlSize::ExtraSmall | ControlSize::Small => 14,
                            ControlSize::Medium => 16,
                            ControlSize::Large => 18,
                        };
                        icon(move || item.icon().cloned().unwrap_or_else(|| "".into()))
                            .style(style().size(side, side).shrink(0))
                    },
                );
                let label = (!icon_only).then(|| {
                    text(move || item.get().label())
                        .style(style().min_width(0).text_wrap(false).line_clamp(1))
                });
                row((glyph, label))
                    .node_ref(reference)
                    .style(style().gap(4).min_width(0).align_items(Align::Center))
            });
        let radio = if icon_only {
            radio.variant(SegmentedIconOnly)
        } else {
            radio
        };
        if disabled {
            let radio = radio
                .disabled(true)
                .style(|| style().opacity(crate::tokens().sizes.disabled_opacity));
            if let Some(reason) = self.disabled_reason {
                tooltip(reason).attach(radio)
            } else {
                radio.into_view()
            }
        } else {
            (
                radio.on_key(move |event| {
                    if matches!(
                        event.key(),
                        kivact::view::Key::ArrowUp
                            | kivact::view::Key::ArrowDown
                            | kivact::view::Key::ArrowLeft
                            | kivact::view::Key::ArrowRight
                    ) {
                        hint_anchor.set(None);
                    }
                }),
                hint,
            )
                .into_view()
        }
    }
}

pub(crate) fn track(theme: &Theme, look: &Look<Orientation>) -> Style {
    let t = theme.ext::<Tokens>();
    let mut base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(2)
        .padding(4)
        .radius(t.radius.element)
        .background(t.colors.neutral);
    if look.variant::<SegmentedLayout>() == Some(&SegmentedLayout::Fill) {
        base = base.width(pct(100));
    }
    if look.variant::<SegmentedIconOnly>().is_some() {
        base.shrink(0)
    } else {
        base.max_width(pct(100)).min_width(0)
    }
}

pub(crate) fn segment(theme: &Theme, look: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let c = &t.colors;
    let (height, padding, font) = match look.size {
        ControlSize::ExtraSmall | ControlSize::Small => (20, 8, 12),
        ControlSize::Medium => (24, 12, 14),
        ControlSize::Large => (28, 12, 14),
    };
    let selected = look.checked.is_checked();
    let (fill, ink) = if selected {
        (c.bg_surface, c.text_primary)
    } else {
        (Color::TRANSPARENT, c.text_secondary)
    };
    let mut base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(height)
        .min_width(0)
        .radius(8)
        .padding_x(padding)
        .background(fill)
        .color(ink)
        .font_size(font)
        .line_height(20.0 / font as f64)
        .weight(if selected { 600 } else { 500 })
        .cursor(Cursor::Pointer)
        .focus_visible(ring(&t, c.focus_ring))
        .disabled(style().color(c.text_disabled).cursor(Cursor::Default));
    if !selected && !look.input.disabled {
        base = base
            .hover(style().background(c.overlay_hover))
            .pressed(style().background(c.overlay_pressed));
    }
    if look.variant::<SegmentedLayout>() == Some(&SegmentedLayout::Fill) {
        base = base.grow(1).basis(0);
    }
    fast(&t, base)
}

kivact::state::constant_value!(SegmentedLayout);
