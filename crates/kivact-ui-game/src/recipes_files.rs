//! Role: Apply console surfaces and diagonal stripes to standard drop targets.
use crate::Tokens;
use kivact::draw::{Layer, Path};
use kivact::geometry::Point;
use kivact::interaction::DropzoneState;
use kivact::style::{Align, BorderStyle, ColorToken, Direction, Style, Theme, style};
use kivact::ui::{DropzoneRecipes, Look};

pub(crate) fn dropzone(r: DropzoneRecipes) -> DropzoneRecipes {
    r.root(|_| root)
        .icon(|_| icon)
        .label(|_| label)
        .hint(|_| hint)
        .status(|_| status)
}
fn root(t: &Theme, s: &Look<DropzoneState>) -> Style {
    let p = t.ext::<Tokens>();
    let stripe = p.dropzone_stripe;
    let mut result = style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .gap(6)
        .padding_y(22)
        .padding_x(16)
        .border(1.5, p.line_strong)
        .border_style(BorderStyle::Dashed)
        .paint(move |d, s| {
            d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                let mut x = -s.height;
                while x < s.width {
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(x, s.height))
                            .line_to(Point::new(x + 10., s.height))
                            .line_to(Point::new(x + s.height + 10., 0.))
                            .line_to(Point::new(x + s.height, 0.))
                            .close(),
                        stripe,
                    );
                    x += 20.;
                }
            });
        });
    if matches!(**s, DropzoneState::DragOver { .. }) {
        result = result
            .background(p.primary_weak)
            .border_color(t.colors().accent);
    }
    result
}
fn icon(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().size(28, 28).color(ColorToken::Accent)
}
fn label(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().weight(700)
}
fn hint(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().font_size(11.5).color(ColorToken::TextMuted)
}
fn status(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().color(ColorToken::Danger)
}
