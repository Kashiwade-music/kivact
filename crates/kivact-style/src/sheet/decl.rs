//! Role: Operations on a style's declaration list: merge, duplicates, states.

use kivact_types::StyleState;

use super::kinds::PropGroup;
use super::props::{Decl, Style};

impl Style {
    /// Overlay `o` on `self`: its declarations come after, so they win.
    pub fn merge(mut self, o: &Style) -> Style {
        self.decls.extend(o.decls.iter().cloned());
        self
    }

    /// How many declarations set a property an earlier declaration of this
    /// style sets (debug diagnostics, counted only when asked: core asks
    /// once per style layer in debug builds).
    pub(crate) fn duplicates(&self) -> u32 {
        let mut seen = std::collections::HashSet::new();
        self.decls
            .iter()
            .filter(|d| d.counts_twice())
            .filter(|d| !seen.insert(std::mem::discriminant(*d)))
            .count() as u32
    }

    /// The style of `state`: every `state` style of this style merged in
    /// order; `None` when it sets none.
    pub(crate) fn state(&self, state: StyleState) -> Option<Style> {
        self.decls
            .iter()
            .filter_map(|d| match d {
                Decl::State(s, style) if *s == state => Some(&**style),
                _ => None,
            })
            .fold(None, |acc: Option<Style>, s| {
                Some(acc.unwrap_or_default().merge(s))
            })
    }

    /// This style without its `state` styles; the removed ones merged.
    pub(crate) fn take_state(&mut self, state: StyleState) -> Option<Style> {
        let taken = self.state(state);
        self.decls
            .retain(|d| !matches!(d, Decl::State(s, _) if *s == state));
        taken
    }

    /// Whether it sets a text property.
    pub(crate) fn has_text(&self) -> bool {
        self.decls.iter().any(|d| d.group() == PropGroup::Text)
    }

    /// The retained paint callback the style sets last (`Style::paint`).
    pub(crate) fn local_paint(&self) -> Option<crate::draw::LocalPaint> {
        self.decls.iter().rev().find_map(|d| match d {
            Decl::LocalPaint(p) => Some(p.clone()),
            _ => None,
        })
    }

    /// Remove every paint callback, in its state styles too.
    pub(crate) fn strip_local_paint(&mut self) {
        self.decls.retain(|d| !matches!(d, Decl::LocalPaint(_)));
        for d in &mut self.decls {
            if let Decl::State(_, s) = d {
                s.strip_local_paint();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the declaration list.
    use super::*;
    use crate::sheet::style;

    #[test]
    fn the_later_declaration_wins() {
        let s = style().width(10).width(20);
        assert_eq!(s.fold().size.width, Some(kivact_types::Length::Px(20.0)));
    }

    #[test]
    fn merge_appends_and_the_overlay_wins() {
        let s = style().width(10).merge(&style().width(30));
        assert_eq!(s.fold().size.width, Some(kivact_types::Length::Px(30.0)));
        assert_eq!(s.duplicates(), 1);
    }

    #[test]
    fn a_repeated_setter_counts_a_duplicate() {
        assert_eq!(style().width(1).width(2).duplicates(), 1);
        assert_eq!(style().width(1).height(2).duplicates(), 0);
        assert_eq!(style().hover(style()).hover(style()).duplicates(), 0);
    }

    #[test]
    fn shadows_stack_in_one_style_and_a_later_layer_replaces_them() {
        let one = crate::effect::Shadow::new(0.0, 1.0, 2.0, crate::color::SHADOW);
        let two = crate::effect::Shadow::new(0.0, 2.0, 4.0, crate::color::SHADOW);
        let stacked = style().shadow(one).shadow(two);
        assert_eq!(stacked.fold().paint.shadows.as_ref().map(Vec::len), Some(2));
        let replaced = stacked.merge(&style().shadow(one));
        assert_eq!(
            replaced.fold().paint.shadows.as_ref().map(Vec::len),
            Some(1)
        );
    }

    #[test]
    fn nested_state_styles_merge() {
        let s = style().hover(style().width(1)).hover(style().height(2));
        let hover = s.state(StyleState::Hover).expect("a hover style").fold();
        assert_eq!(hover.size.width, Some(kivact_types::Length::Px(1.0)));
        assert_eq!(hover.size.height, Some(kivact_types::Length::Px(2.0)));
    }
}
