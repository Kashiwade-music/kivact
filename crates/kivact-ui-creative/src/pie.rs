//! Role: Pie menu of the Creative kit with pointer and keyboard selection.
use kivact::ui::button;
use kivact::view::HandlerResult;

use kivact::draw::{Point, Rect};
use kivact::i18n::message;
use kivact::interaction::use_popup;
use kivact::prelude::{
    ColorToken, IntoValue, IntoView, Key, Node, NodeBuilder, Role, column, node, style, text,
};
use kivact::primitive::NodeExt;
use kivact::style::{Icon, Position};
use kivact::view::Handled;
use kivact::view::{ButtonPress, MouseButton, Placement, PointerPhase, callback, overlay_at};
use kivact::{Binding, binding};

/// A directional command, in clockwise degrees from the positive horizontal axis.
#[derive(Clone, Debug, PartialEq)]
pub struct PieEntry {
    pub label: String,
    pub angle: f32,
    /// The icon of the command: a name in the theme's icon set or an image.
    pub icon: Icon,
}
impl PieEntry {
    pub fn new(label: impl Into<String>, angle: f32, icon: impl Into<Icon>) -> Self {
        Self {
            label: label.into(),
            angle,
            icon: icon.into(),
        }
    }
}
/// Public state permits a surrounding application to open the same menu from a hovered editor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PieState {
    pub open: bool,
    pub held: bool,
    pub center: Point,
    pub hot: Option<usize>,
}
impl PieState {
    pub fn open_at(center: Point, held: bool) -> Self {
        Self {
            open: true,
            held,
            center,
            hot: None,
        }
    }
    /// Keep the center dead zone and select the closest offered direction.
    pub fn pointer(&mut self, pointer: Point, entries: &[PieEntry]) {
        let delta = Point::new(pointer.x - self.center.x, pointer.y - self.center.y);
        self.hot = if delta.x.hypot(delta.y) < 22.0 {
            None
        } else {
            let angle = delta.y.atan2(delta.x).to_degrees();
            entries
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let distance = |entry: &PieEntry| {
                        ((angle - entry.angle + 180.0).rem_euclid(360.0) - 180.0).abs()
                    };
                    distance(a).total_cmp(&distance(b))
                })
                .map(|(index, _)| index)
        };
    }
}

/// A directional popup with pointer selection and keyboard numbers/arrows/Enter/Escape.
/// To implement a held shortcut, open `PieState::open_at(point, true)` on key-down and
/// commit its `hot` choice on key-up using the same application command callback.
#[track_caller]
pub fn pie_menu<R: HandlerResult>(
    entries: impl IntoValue<Vec<PieEntry>>,
    state: impl Into<Binding<PieState>>,
    choose: impl Fn(usize) -> R + 'static,
) -> Node {
    let choose = callback(choose);
    let entries = entries.into_value();
    let state = state.into();
    let current = state.into_value();
    let visible = binding(
        (move || current.get().open).into_value(),
        move |open: bool| {
            if !open {
                let mut next = current.get();
                next.open = false;
                next.held = false;
                state.set(next);
            }
        },
    );
    let popup = use_popup(visible);
    let choose = move |index| {
        choose.call(index);
        let mut value = current.get_untracked();
        value.open = false;
        value.held = false;
        state.set(value);
    };
    let layer = kivact::view::show(popup.shown(), move || {
        let pointer_entries = entries;
        let key_entries = entries;
        let buttons = kivact::view::for_index(entries, move |index, entry| {
            let button_entries = entries;
            let angle = move || entry.with(|entry| entry.angle.to_radians());
            let button = button(move || entry.get().label)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .icon(kivact::control::icon(move || entry.get().icon))
                .selected(move || current.get().hot == Some(index));
            button
                .style(move || {
                    style()
                        .position(Position::Absolute)
                        .left(160.0 + angle().cos() * 104.0 - 48.0)
                        .top(160.0 + angle().sin() * 104.0 - 18.0)
                        .size(96, 36)
                })
                .on_pointer_default(move |event| {
                    if matches!(event.phase, PointerPhase::Move | PointerPhase::Enter) {
                        let mut next = current.get();
                        next.pointer(event.window_pos, &button_entries.get());
                        state.set(next);
                    }
                    if matches!(event.phase, PointerPhase::Down(_)) {
                        Handled::Yes
                    } else {
                        Handled::No
                    }
                })
                .on_press(move || choose(index))
                .into_view()
        });
        overlay_at(
            move || current.get().center,
            Placement::Center,
            node()
                .child(buttons)
                .child(
                    text(message("creative.pie.label", "Shading")).style(
                        style()
                            .position(Position::Absolute)
                            .left(130)
                            .top(152)
                            .width(60)
                            .text_align(kivact::style::TextAlign::Center),
                    ),
                )
                .style(style().size(320, 320).position(Position::Relative))
                .draw(|draw, _| {
                    draw.rounded_rect(
                        Rect::new(92.0, 92.0, 136.0, 136.0),
                        68.0,
                        ColorToken::Raised,
                    )
                })
                .on_pointer_default(move |event| {
                    if matches!(event.phase, PointerPhase::Move | PointerPhase::Enter) {
                        let mut next = current.get();
                        next.pointer(event.window_pos, &pointer_entries.get());
                        state.set(next);
                    } else if matches!(
                        event.phase,
                        PointerPhase::Down(ButtonPress {
                            button: MouseButton::Primary,
                            ..
                        })
                    ) {
                        if let Some(index) = current.get().hot {
                            choose(index);
                        }
                        return Handled::Yes;
                    }
                    Handled::No
                }),
        )
        .part(popup.content().role(Role::Menu))
        .name(message("creative.pie.name", "Shading pie"))
        .focusable(true)
        .on_key_default(move |event| {
            let mut next = current.get();
            match event.key {
                Key::Char(key) if ('1'..='9').contains(&key) => {
                    let index = (key as u8 - b'1') as usize;
                    if index < key_entries.with(Vec::len) {
                        choose(index);
                    }
                }
                Key::ArrowRight | Key::ArrowDown | Key::ArrowLeft | Key::ArrowUp => {
                    if key_entries.with(Vec::is_empty) {
                        return Handled::No;
                    }
                    let index = next.hot.unwrap_or(0).min(key_entries.with(Vec::len) - 1);
                    next.hot = Some(if matches!(event.key, Key::ArrowRight | Key::ArrowDown) {
                        (index + 1) % key_entries.with(Vec::len)
                    } else {
                        (index + key_entries.with(Vec::len) - 1) % key_entries.with(Vec::len)
                    });
                    state.set(next);
                }
                Key::Enter => {
                    if let Some(index) =
                        next.hot.filter(|index| *index < key_entries.with(Vec::len))
                    {
                        choose(index);
                    }
                }
                _ => return Handled::No,
            }
            Handled::Yes
        })
    });
    column((layer,)).style(style().height(0).gap(0))
}
