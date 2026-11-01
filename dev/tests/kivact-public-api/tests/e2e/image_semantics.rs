//! Role: Image primitives preserve sizing, decoration policy and inherited state tint.

use kivact::prelude::*;
use kivact::style::pct;
use kivact::test::{SemanticNode, render};

/// A 24×24 square (an original test shape).
static SQUARE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><rect x="4" y="4" width="16" height="16" fill="black"/></svg>"#;

/// The node with test id `id` under `n`.
fn by_test_id<'a>(n: &'a SemanticNode, id: &str) -> Option<&'a SemanticNode> {
    if n.test_id.as_deref() == Some(id) {
        return Some(n);
    }
    n.children.iter().find_map(|c| by_test_id(c, id))
}

/// The nodes under `n` as `(role, name)`, depth first.
fn roles(n: &SemanticNode, out: &mut Vec<(Option<Role>, String)>) {
    out.push((n.role, n.name.clone()));
    for c in &n.children {
        roles(c, out);
    }
}

/// An icon without a name is an `Image` with no name, which no host
/// exposes (the shared semantics policy, as HTML `<img alt="">`); its
/// image child is `Generic`, never exposed. `.name(..)` names the root, so
/// it is exposed as an `Image` with that name.
#[test]
fn image_container_is_decorative_until_named() {
    let screen = render(|| {
        column((
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .test_id("plain"),
            node()
                .role(Role::Image)
                .style(style().size(16, 16).shrink(0))
                .child(
                    image(SQUARE)
                        .monochrome(true)
                        .role(Role::Generic)
                        .style(style().size(pct(100), pct(100))),
                )
                .name("保存")
                .test_id("named"),
        ))
    });
    let snap = screen.snapshot();
    let mut plain = Vec::new();
    roles(by_test_id(&snap, "plain").expect("plain"), &mut plain);
    assert_eq!(
        plain,
        vec![
            (Some(Role::Image), String::new()),
            (Some(Role::Generic), String::new()),
        ]
    );
    let mut named = Vec::new();
    roles(by_test_id(&snap, "named").expect("named"), &mut named);
    assert_eq!(
        named,
        vec![
            (Some(Role::Image), "保存".to_string()),
            (Some(Role::Generic), String::new()),
        ]
    );
    assert_eq!(
        screen.get_by_role(Role::Image, "保存").rect(),
        screen.get_by_test_id("named").rect()
    );
}
