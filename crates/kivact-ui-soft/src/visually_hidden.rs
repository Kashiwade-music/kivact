//! Role: Expose noninteractive text to assistive technology without painting it.

use kivact::component;
use kivact::state::Value;
use kivact::style::Color;
use kivact::style::Position;
use kivact::style::style;
use kivact::view::LiveRegion;
use kivact::view::text;

/// Text available to assistive technology without a visible layout box.
#[component]
pub fn visually_hidden(value: Value<String>, live: Option<LiveRegion>) -> impl Element {
    let root = text(value).hit_testable(false).style(
        style()
            .position(Position::Absolute)
            .size(1, 1)
            .clip(true)
            .text_wrap(false)
            .color(Color::TRANSPARENT),
    );
    match live {
        Some(priority) => root.live_region(priority),
        None => root,
    }
}
