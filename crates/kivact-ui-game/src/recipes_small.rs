//! Role: Game recipes for the standard progress bar and disclosure.
//!
//! The console look of `kivact::ui::progress` (a flat bar in the variant's [`Tone`], optionally
//! [`Striped`] with moving stripes) and `disclosure` (a ruled band whose
//! chevron turns).

use crate::display::ToneExt;
use crate::tokens::Tokens;
use kivact::draw::{Layer, Path};
use kivact::geometry::{Point, Rect};
use kivact::interaction::Tone;
use kivact::style::{Align, ColorToken, Direction, Style, Theme, pct, style};
use kivact::ui::DisclosureRecipeState;
use kivact::ui::{ControlSize, DisclosureRecipes, Look, ProgressRecipes, ProgressState, Variant};

/// Draw a standard progress bar with moving diagonal stripes over its fill:
/// `kivact::ui::progress(..).moving(true).variant(Striped)`. Another theme
/// draws the plain bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Striped;

impl Variant for Striped {}

pub(crate) fn progress(r: ProgressRecipes) -> ProgressRecipes {
    r.track(|_| progress_track)
        .fill(|_| progress_fill)
        .label(|_| crate::recipes_progress::label)
}

/// A hard-edged bar, 4 px for a small one and 8 px otherwise.
fn progress_track(t: &Theme, look: &Look<ProgressState>) -> Style {
    if let Some(special) = crate::recipes_progress::special(t, look) {
        return special;
    }
    style()
        .width(pct(100))
        .height(
            if matches!(look.size, ControlSize::ExtraSmall | ControlSize::Small) {
                4
            } else {
                8
            },
        )
        .min_width(0)
        .clip(true)
        .background(t.ext::<Tokens>().surface_3)
}

fn progress_fill(t: &Theme, look: &Look<ProgressState>) -> Style {
    if let Some(special) = crate::recipes_progress::special_fill(look) {
        return special;
    }
    let state = **look;
    let striped = look.variant::<Striped>().is_some();
    let tone = look.variant::<Tone>().copied().unwrap_or(Tone::Info);
    let (fill, stripe) = (tone.color(t), t.ext::<Tokens>().primary_hover);
    style()
        .width(pct(100))
        .height(pct(100))
        .paint(move |d, size| {
            let (x, width) = if state.value.is_some() {
                (0.0, size.width * state.shown.get().clamp(0.0, 1.0))
            } else {
                let (start, length) = state.sweep();
                (size.width * start, size.width * length)
            };
            d.fill_rect(Rect::new(x, 0.0, width, size.height), fill);
            if striped && state.value.is_some() {
                d.layer(
                    Layer::new().clip_rect(Rect::new(x, 0.0, width, size.height)),
                    |d| {
                        let mut x = -24. + state.phase.get() * 17.;
                        while x < width + 24. {
                            d.fill_path(
                                &Path::new()
                                    .move_to(Point::new(x, size.height))
                                    .line_to(Point::new(x + size.height, 0.))
                                    .line_to(Point::new(x + size.height + 6., 0.))
                                    .line_to(Point::new(x + 6., size.height))
                                    .close(),
                                stripe,
                            );
                            x += 12.;
                        }
                    },
                );
            }
        })
}

pub(crate) fn disclosure(r: DisclosureRecipes) -> DisclosureRecipes {
    r.root(|_| disclosure_root)
        .trigger(|_| disclosure_trigger)
        .indicator(|_| disclosure_indicator)
        .content(|_| disclosure_content)
}

fn disclosure_root(_: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style().direction(Direction::Column)
}

/// The chevron is the trigger's last child, drawn first by the reversed row.
fn disclosure_trigger(_: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style()
        .direction(Direction::RowReverse)
        .height(30)
        .padding_x(10)
        .gap(7)
        .align_items(Align::Center)
        .font_size(11.5)
        .weight(700)
        .background(ColorToken::SurfaceAlt)
        .border_top(1, ColorToken::Border)
        .border_bottom(1, ColorToken::Border)
}

fn disclosure_indicator(_: &Theme, state: &Look<DisclosureRecipeState>) -> Style {
    style()
        .size(10, 10)
        .shrink(0)
        .color(ColorToken::TextMuted)
        .rotate(if state.open { 0 } else { -90 })
}

fn disclosure_content(_: &Theme, _: &Look<DisclosureRecipeState>) -> Style {
    style()
        .enter_size(kivact::view::Orientation::Vertical)
        .exit_size(kivact::view::Orientation::Vertical)
        .direction(Direction::Column)
}
