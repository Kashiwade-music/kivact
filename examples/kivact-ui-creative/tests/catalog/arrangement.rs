//! Role: Catalog-owned arrangement decisions through public pointer and key input.
use super::{chapter, clean, evidence, local, mount};
use kivact::draw::Point;
use kivact::view::{A11yAction, Key, MouseButton, Role};

#[test]
fn catalog_arrangement_applies_domain_requests_and_cancels_point_creation() {
    let screen = mount();
    chapter(&screen, "timeline");
    let canvas = screen.get_by_test_id("creative-timeline");
    let clip = canvas.get_by_role(Role::Button, "Vox_Take3");
    let initial = clip.rect().expect("clip");
    let p = local(&screen, "creative-timeline", clip);
    clip.drag(p, Point::new(p.x + 26.0, p.y));
    assert!((clip.rect().expect("moved").x - initial.x - 26.0).abs() < 0.1);
    let trim = canvas.get_by_role(Role::Button, "Vox_Take3 end trim");
    let p = local(&screen, "creative-timeline", trim);
    let width = clip.rect().expect("clip").width;
    trim.drag(p, Point::new(p.x + 26.0, p.y));
    assert!((clip.rect().expect("trimmed").width - width - 26.0).abs() < 0.1);
    let mute = canvas.get_by_role(Role::Button, "Vox Mute");
    mute.a11y(A11yAction::Activate);
    assert_eq!(mute.is_checked(), Some(true));
    mute.a11y(A11yAction::Activate);
    assert_eq!(mute.is_checked(), Some(false));
    screen.get_by_role(Role::Button, "Timeline Mute").click();
    canvas.focus();
    clip.hover();
    let before = screen.screenshot().hash();
    clip.click();
    assert_ne!(
        screen.screenshot().hash(),
        before,
        "application applied clip mute"
    );
    screen.get_by_role(Role::Button, "Timeline Split").click();
    clip.click();
    let right = canvas.get_by_role(Role::Button, "Vox_Take3 (2)");
    assert!(right.rect().expect("split right").width < width);
    screen.get_by_role(Role::Button, "Timeline Erase").click();
    right.click();
    assert!(
        screen
            .query_by_role(Role::Button, "Vox_Take3 (2)")
            .is_none()
    );
    screen.get_by_role(Role::Button, "Timeline Select").click();
    let p = local(&screen, "creative-timeline", clip);
    clip.click();
    canvas.press_key(Key::Delete);
    assert!(screen.query_by_role(Role::Button, "Vox_Take3").is_none());
    assert!(p.y > 0.0);
    let point = canvas.get_by_role(Role::Slider, "Vox automation point 1");
    point.a11y_with_value(A11yAction::SetValue, "0.5");
    assert_eq!(point.value_range().2, Some(0.5));
    let root = canvas.rect().expect("canvas");
    let point_rect = point.rect().expect("point");
    screen.mouse_move(root.x + 220.0 + 10.0 * 26.0, point_rect.center().y);
    screen.mouse_down(MouseButton::Primary);
    let created = canvas.get_by_role(Role::Slider, "Vox automation point 8");
    let id_rect = created.rect().expect("created");
    screen.mouse_move(root.x + 220.0 + 11.0 * 26.0, point_rect.center().y - 5.0);
    screen.mouse_move(root.x + 220.0 + 12.0 * 26.0, point_rect.center().y - 8.0);
    assert!(created.rect().expect("updated same point").x > id_rect.x + 40.0);
    assert!(
        screen
            .query_by_role(Role::Slider, "Vox automation point 9")
            .is_none()
    );
    evidence(&screen, "creative-arrangement-request-apply");
    screen.press_key(Key::Escape);
    assert!(
        screen
            .query_by_role(Role::Slider, "Vox automation point 8")
            .is_none()
    );
    assert_eq!(point.value_range().2, Some(0.5));
    evidence(&screen, "creative-arrangement-point-cancel");
    clean(&screen);
}
