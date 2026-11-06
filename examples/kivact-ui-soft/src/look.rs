//! Role: Style the Soft catalog shell from the active Neutral tokens.
use kivact::style::{Style, pct, style};

pub(super) fn canvas() -> Style {
    let p = kivact::style::use_tokens::<kivact_ui_soft::Tokens>();
    style()
        .size(pct(100), pct(100))
        .background(p.colors.bg_body)
        .color(p.colors.text_primary)
}
