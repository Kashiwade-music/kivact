//! Role: Layout measurement and positioning.
//!
//! The layout check of a `Screen`: texts that overflow their box, wrap
//! where a single line is expected, sit off the middle of their parent, or
//! are cut by a clip with no ellipsis, found by the engine from the shaped
//! lines and the laid out boxes (`kivact_engine::testing`'s
//! `layout_findings`), and reported with where the text was created.
//!
//! A [`LayoutViolation`] is `{ rule, role, name, text, test_id, created_at,
//! text_rect, limit, amount }` and displays on one line; `created_at` is
//! the call that made the text, found by walking the core tree up to the
//! nearest located ancestor. `LayoutCheck` and `LayoutRule` are
//! `#[non_exhaustive]`.

use kivact_wire::NodeId;

use super::model::{Element, Screen};
use super::results::{LayoutCheck, LayoutViolation};
use crate::view::state::ViewState;

impl Screen {
    /// The layout defects of every text and text input on the screen, in
    /// tree order, with [`LayoutCheck::default`] (settling first). A hidden
    /// element is left out.
    ///
    /// A text is compared in layout space, before any transform: a line's
    /// advance across and its ink down against its own content box, or the
    /// padding box of its visual parent; a text wrapped where its role
    /// expects one line; a line off the middle of its parent; a line a clip
    /// cuts with no ellipsis. A scrolling axis never overflows.
    pub fn check_layout(&self) -> Vec<LayoutViolation> {
        self.check_layout_with(&LayoutCheck::default())
    }

    /// [`Screen::check_layout`] with explicit options.
    pub fn check_layout_with(&self, check: &LayoutCheck) -> Vec<LayoutViolation> {
        self.layout_violations(check, None)
    }

    /// Fail the test, listing every layout defect ([`Screen::check_layout`])
    /// one per line with where its text was created, and leaving the
    /// failure artifacts (a screenshot in [`super::failure_dir`]).
    #[track_caller]
    pub fn assert_layout(&self) {
        self.assert_layout_with(&LayoutCheck::default());
    }

    /// Fail with layout defects using explicit options, retaining the same
    /// diagnostics and failure artifacts as [`Screen::assert_layout`].
    #[track_caller]
    pub fn assert_layout_with(&self, check: &LayoutCheck) {
        let found = self.check_layout_with(check);
        if found.is_empty() {
            return;
        }
        let mut message = format!("{} layout violation(s):", found.len());
        for v in &found {
            message.push_str("\n  ");
            message.push_str(&v.to_string());
        }
        self.fail(&message);
    }

    fn layout_violations(
        &self,
        check: &LayoutCheck,
        scope: Option<NodeId>,
    ) -> Vec<LayoutViolation> {
        self.run_until_idle();
        let findings = self.host.borrow().layout_findings(&check.to_wire(), scope);
        findings
            .into_iter()
            .map(|f| {
                let element = Element {
                    screen: self,
                    node: f.node,
                    region: None,
                };
                let (role, name) = (element.role(), element.name());
                let tree = ViewState::tree(&self.rt);
                let test_id = tree.test_id(f.node);
                let text = match tree.content(f.node) {
                    Some(c) => c,
                    _ => match tree.input_value(f.node) {
                        Some(v) if !v.is_empty() => v,
                        _ => tree.placeholder(f.node).unwrap_or_default(),
                    },
                };
                // The builder that created it, else the nearest ancestor's.
                let mut created_at = None;
                let mut cur = Some(f.node);
                while let Some(id) = cur {
                    let n = tree.get(id);
                    if let Some(at) = n.and_then(|n| n.at) {
                        created_at = Some(at);
                        break;
                    }
                    cur = n.and_then(|n| n.parent);
                }
                drop(tree);
                LayoutViolation {
                    rule: f.rule.into(),
                    role,
                    name,
                    text,
                    test_id,
                    created_at: created_at.unwrap_or(self.at),
                    text_rect: f.text_rect,
                    limit: f.limit_rect,
                    amount: f.amount,
                }
            })
            .collect()
    }
}

impl Element<'_> {
    /// The layout defects of the texts in this element's subtree
    /// ([`Screen::check_layout`] with the default options, for the subtree
    /// only).
    pub fn check_layout(&self) -> Vec<LayoutViolation> {
        self.screen
            .layout_violations(&LayoutCheck::default(), Some(self.node))
    }
}
