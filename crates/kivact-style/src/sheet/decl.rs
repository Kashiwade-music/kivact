//! Role: The style declaration list: how declarations are pushed, merged, named and classed.

use kivact_types::StyleState;

pub(crate) use super::props::Decl;

/// Whether a declaration can be a host-evaluated paint delta of a state
/// (`Delta`) or must be compiled into the node's style (`Rest`, ST-05).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PropClass {
    Delta,
    Rest,
}

/// The group a declaration belongs to (diagnostics order, text detection).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PropGroup {
    Size,
    Spacing,
    Flow,
    Paint,
    Transform,
    Text,
    States,
    Motion,
}

impl Decl {
    pub(super) fn extra_name(&self) -> &'static str {
        match self {
            Decl::BoxShadow(_) => "shadow",
            Decl::TextShadow(_) => "text_shadow",
            Decl::Transition(..) => "transition",
            Decl::State(state, _) => state.name(),
            Decl::Group => "group",
            _ => unreachable!("invariant: the table names every other declaration"),
        }
    }

    pub(super) fn extra_class(&self) -> PropClass {
        match self {
            Decl::BoxShadow(_) => PropClass::Delta,
            _ => PropClass::Rest,
        }
    }

    pub(super) fn extra_value_debug(&self) -> String {
        match self {
            Decl::BoxShadow(list) | Decl::TextShadow(list) => format!("{list:?}"),
            Decl::Transition(prop, t) => format!("{prop:?} {t:?}"),
            _ => String::new(),
        }
    }

    pub(super) fn extra_group(&self) -> PropGroup {
        match self {
            Decl::BoxShadow(_) => PropGroup::Paint,
            Decl::TextShadow(_) => PropGroup::Text,
            Decl::Transition(..) => PropGroup::Motion,
            _ => PropGroup::States,
        }
    }

    /// Whether setting it twice in one style is a duplicate (states nest,
    /// shadows stack, transitions are per property group, a group flag
    /// repeats harmlessly).
    fn counts_twice(&self) -> bool {
        !matches!(
            self,
            Decl::State(..)
                | Decl::BoxShadow(_)
                | Decl::TextShadow(_)
                | Decl::Transition(..)
                | Decl::Group
        )
    }
}

/// A style value: an ordered list of declarations, the later winning.
/// Build with [`style()`](crate::style).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    pub(crate) decls: Vec<Decl>,
}

const _: () = assert!(
    std::mem::size_of::<Decl>() <= 64,
    "a declaration stays small: large values are boxed"
);

kivact_runtime::constant_value!(Style);

impl Style {
    /// Append `decl` (constant time: a setter never scans the list, PF-01).
    pub(crate) fn push(mut self, decl: Decl) -> Self {
        self.decls.push(decl);
        self
    }

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
