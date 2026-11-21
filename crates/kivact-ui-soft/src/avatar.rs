//! Role: Supply the optional person silhouette of a canonical avatar.
use kivact::style::{pct, style};
use kivact::view::{Asset, image};

const PERSON: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/></svg>"#;

/// Pass this decorative silhouette to standard `avatar(...).fallback_content(...)`.
pub fn avatar_silhouette() -> impl kivact::view::IntoView {
    image(Asset::from(PERSON))
        .monochrome(true)
        .hit_testable(false)
        .style(style().size(pct(60), pct(60)))
}
