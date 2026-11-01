//! Role: The source toolbar centers the six-tool island beside Undo, Snap and Beat.
use super::{evidence, mount};
use kivact::test::Screen;
use kivact::view::Role;

fn aligned(screen: &Screen) {
    let toolbar = screen
        .get_by_role(Role::Toolbar, "Creative toolbar")
        .rect()
        .expect("toolbar");
    let tools = screen.get_by_test_id("creative-toolbar-tools");
    let island = tools.rect().expect("tool island");
    assert!((island.center().y - toolbar.center().y).abs() <= 0.5);
    assert!(island.y >= toolbar.y);
    assert!(island.y + island.height <= toolbar.y + toolbar.height);
    let undo = screen
        .get_by_role(Role::Button, "Undo")
        .rect()
        .expect("Undo");
    for name in [
        "Tool Pointer",
        "Tool Move",
        "Tool Rotate",
        "Tool Scale",
        "Tool Scissors",
        "Tool Pencil",
    ] {
        let tool = tools.get_by_role(Role::Button, name);
        let rect = tool.rect().expect("tool button");
        assert!(
            (rect.center().y - undo.center().y).abs() <= 0.5,
            "{name}: {rect:?}, Undo: {undo:?}"
        );
        assert!(rect.y >= island.y);
        assert!(rect.y + rect.height <= island.y + island.height);
    }
    tools.get_by_role(Role::Button, "Tool Move").click();
    assert!(tools.get_by_role(Role::Button, "Tool Move").is_selected());
    assert!(
        !tools
            .get_by_role(Role::Button, "Tool Pointer")
            .is_selected()
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn creative_toolbar_tool_island_aligns_at_both_densities() {
    let screen = mount();
    aligned(&screen);
    evidence(&screen, "creative-toolbar-aligned");
    screen.get_by_role(Role::Button, "Compact density").click();
    aligned(&screen);
    evidence(&screen, "creative-toolbar-aligned-compact");
    screen.get_by_role(Role::Button, "Accent Lime").click();
    aligned(&screen);
    evidence(&screen, "creative-toolbar-aligned-lime-compact");
}
