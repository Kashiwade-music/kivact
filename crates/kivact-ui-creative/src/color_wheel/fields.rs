//! Role: Model fields, hexadecimal drafts and optional editable swatches.
use super::ColorModel;
use super::color_to_hsv;
use super::geometry::Channel;
use super::state::{ColorOwner, State};
use crate::number;
use kivact::i18n::message;
use kivact::interaction::ListItem;
use kivact::state::IntoValue;
use kivact::state::callback;
use kivact::state::{Binding, Value, binding, effect, memo, signal};
use kivact::style::{Color, pct, style};
use kivact::view::EditPhase;
use kivact::view::IntoView;
use kivact::view::NodeBuilder;
use kivact::view::{View, dynamic, for_each, show};
use kivact::view::{column, row};

pub(super) fn fields(state: State, model: Binding<ColorModel>, alpha: Value<bool>) -> View {
    let current = model.into_value();
    let choices = vec![
        ListItem::new(ColorModel::Rgb, "RGB"),
        ListItem::new(ColorModel::Hsv, "HSV"),
        ListItem::new(ColorModel::Hex, "HEX"),
    ];
    let selected = model.optional();
    column((
        kivact::ui::select(
            message("creative.color.model", "Color model"),
            choices,
            selected,
        )
        .with_node(move |node| node.disabled(state.blocked)),
        dynamic(current, move |model| match model {
            ColorModel::Hex => hex(state, alpha),
            ColorModel::Rgb | ColorModel::Hsv => {
                let hsv = model == ColorModel::Hsv;
                row((
                    numeric(state, hsv, 0),
                    numeric(state, hsv, 1),
                    numeric(state, hsv, 2),
                ))
                .style(style().gap(4))
                .into_view()
            }
        }),
        show(alpha, move || {
            numeric(state, false, 3).style(style().grow(0).width(pct(100)))
        }),
    ))
    .style(style().gap(4).min_width(0).width(pct(100)))
    .into_view()
}
fn numeric(state: State, hsv: bool, index: usize) -> crate::Numeric {
    let owner = ColorOwner::Field { hsv, index };
    kivact::state::on_cleanup(move || {
        state.edit.cancel_owned(owner);
    });
    let value = state.color.into_value();
    let hue = state.hue;
    let (key, label) = match (hsv, index) {
        (_, 3) => ("creative.color.alpha", "Color Alpha"),
        (true, 0) => ("creative.color.h", "Color H"),
        (true, 1) => ("creative.color.s", "Color S"),
        (true, 2) => ("creative.color.v", "Color V"),
        (_, 0) => ("creative.color.red", "Color R"),
        (_, 1) => ("creative.color.green", "Color G"),
        _ => ("creative.color.blue", "Color B"),
    };
    let read = move || {
        let color = value.get();
        if hsv {
            let (_, s, v) = color_to_hsv(color);
            match index {
                0 => hue.get(),
                1 => s,
                _ => v,
            }
        } else {
            color.components()[index]
        }
    };
    let write = callback(move |next| {
        if state.edit.owner() != Some(owner) {
            return;
        }
        if hsv {
            let (_, s, v, _) = state.hsv();
            match index {
                0 => state.set(Channel::Hue, next, v),
                1 => state.set(Channel::Sv, next, v),
                _ => state.set(Channel::Sv, s, next),
            };
        } else {
            let mut c = value.get().components();
            c[index] = next;
            state
                .color
                .set(Color::from_rgba_f64(c[0], c[1], c[2], c[3]));
        }
    });
    number(
        message(key, label),
        binding(read.into_value(), move |value| write.try_call(value)),
    )
    .with_node(move |node| node.disabled(state.blocked))
    .range(
        kivact::interaction::ScalarRange::new(0.0, if hsv && index == 0 { 360.0 } else { 1.0 })
            .expect("invariant: finite ordered control range"),
    )
    .step(if hsv && index == 0 { 1.0 } else { 0.001 })
    .precision(3)
    .on_edit(move |phase| state.phase(owner, phase))
    .with_node(move |node| node.style(style().grow(1).min_width(0).width(60)))
}
fn encode(color: Color, alpha: bool) -> String {
    let c = color
        .components()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
    if alpha {
        format!("#{:02X}{:02X}{:02X}{:02X}", c[0], c[1], c[2], c[3])
    } else {
        format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
    }
}
fn parse(text: &str, current: Color, alpha: bool) -> Option<Color> {
    let text = text.strip_prefix('#')?;
    if !matches!(text.len(), 6 | 8) || !text.is_ascii() {
        return None;
    }
    let mut c = current.components();
    for (i, channel) in c.iter_mut().enumerate().take(text.len() / 2) {
        *channel = f64::from(u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).ok()?) / 255.0;
    }
    if !alpha {
        c[3] = current.components()[3];
    }
    Some(Color::from_rgba_f64(c[0], c[1], c[2], c[3]))
}
/// What the hexadecimal field shows: the color, or the text typed since.
#[derive(Clone, PartialEq)]
enum HexField {
    Following,
    Typed(String),
}
fn hex(state: State, alpha: Value<bool>) -> View {
    let value = state.color.into_value();
    let field = signal(HexField::Following);
    let text = memo(move || match field.get() {
        HexField::Following => encode(value.get(), alpha.get()),
        HexField::Typed(text) => text,
    });
    let input = binding(text, move |typed: String| field.set(HexField::Typed(typed)));
    let invalid = memo(move || parse(&text.get(), value.get(), alpha.get()).is_none());
    // Enter, or leaving the field with a changed text, writes a valid draft;
    // either way the field follows the color again.
    let commit = move |typed: String| {
        field.set(HexField::Following);
        if state.blocked.get_untracked() {
            return;
        }
        if let Some(color) = parse(&typed, value.get_untracked(), alpha.get_untracked()) {
            state.once(ColorOwner::Hex, || state.color.set(color));
        }
    };
    kivact::ui::text_input(message("creative.color.hex", "Color HEX"), input)
        .draft(commit)
        // Escape discards the draft: the field follows the color again.
        .on_edit(move |phase| {
            if phase == EditPhase::Cancel {
                field.set(HexField::Following);
            }
        })
        .disabled(state.blocked)
        .invalid(invalid)
        .into_view()
}
pub(super) fn swatches(state: State, alpha: Value<bool>) -> Option<View> {
    let palette = state.palette?;
    let colors = palette.into_value();
    let selected = signal(None::<usize>);
    effect(move || {
        let len = colors.get().len();
        if let Some(index) = selected.get_untracked() {
            selected.set(if len == 0 {
                None
            } else {
                Some(index.min(len - 1))
            });
        }
    });
    let items = memo(move || colors.get().into_iter().enumerate().collect::<Vec<_>>());
    let swatch_label = message("creative.color.swatch", "Swatch");
    let buttons = for_each(
        items,
        |(index, _)| *index,
        move |index, item: Value<(usize, Color)>| {
            kivact::ui::button(move || format!("{} {}", swatch_label.get(), index + 1))
                .with_node(move |node| node.disabled(state.blocked))
                .on_press(move || {
                    selected.set(Some(index));
                    state.once(ColorOwner::Swatch, || {
                        let color = item.get().1;
                        state.color.set(if alpha.get() {
                            color
                        } else {
                            color.with_alpha(state.color.into_value().get().components()[3])
                        });
                    });
                })
                .with_node(move |node| {
                    node.style(move || {
                        style().size(28, 28).background(item.get().1).border(
                            if selected.get() == Some(index) { 2 } else { 0 },
                            Color::WHITE,
                        )
                    })
                })
        },
    );
    Some(
        column((
            row(buttons).style(style().gap(4)),
            row((
                kivact::ui::button(message("creative.color.add_swatch", "Add swatch"))
                    .with_node(move |node| node.disabled(state.blocked))
                    .on_press(move || {
                        state.once(ColorOwner::Swatch, || {
                            let mut colors = colors.get();
                            colors.push(state.color.into_value().get());
                            selected.set(Some(colors.len() - 1));
                            palette.set(colors);
                        })
                    }),
                kivact::ui::button(message("creative.color.remove_swatch", "Remove swatch"))
                    .with_node(move |node| {
                        node.disabled(move || state.blocked.get() || selected.get().is_none())
                    })
                    .on_press(move || {
                        if let Some(index) = selected.get() {
                            state.once(ColorOwner::Swatch, || {
                                let mut colors = colors.get();
                                if index < colors.len() {
                                    colors.remove(index);
                                    palette.set(colors);
                                }
                            });
                        }
                    }),
            ))
            .style(style().gap(4)),
        ))
        .style(style().gap(4))
        .into_view(),
    )
}
