//! Role: Graph lane extension contracts and assistive menu rollback.
use super::editor_extensions::{GRAPH_EDITORS, fixture};
use super::*;
use kivact::ui::EditPhase;
#[test]
fn fixed_cells_are_reactive_and_isolate_pointer_and_keys() {
    for editor in GRAPH_EDITORS {
        super::editor_extensions::cells(editor);
    }
}
#[test]
fn enter_and_stationary_double_click_activate_each_domain_target_once() {
    for editor in GRAPH_EDITORS {
        super::editor_extensions::activation(editor);
    }
}
#[test]
fn double_click_activates_nothing_when_the_lane_is_removed_before_release() {
    for editor in GRAPH_EDITORS {
        super::editor_extensions::activation_needs_a_live_target(editor);
    }
}
#[test]
fn context_requests_snapshot_domain_and_standard_menu_returns_focus() {
    for editor in GRAPH_EDITORS {
        super::editor_extensions::context(editor);
    }
}
#[test]
fn inherited_constraints_block_cells_and_menu() {
    for editor in GRAPH_EDITORS {
        super::editor_extensions::constraints(editor);
    }
}
#[test]
fn failed_context_command_keeps_the_standard_menu_open_and_reports_once() {
    for editor in GRAPH_EDITORS {
        super::editor_extensions::failure(editor);
    }
}
#[test]
fn assistive_graph_menu_cancels_drag_before_taking_the_context_snapshot() {
    for editor in GRAPH_EDITORS {
        let (screen, m) = fixture(editor);
        let p = screen
            .get_by_role(Role::Button, "Alpha key 2")
            .rect()
            .expect("graph key geometry")
            .center();
        screen.mouse_move(p.x, p.y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(p.x + 20.0, p.y);
        screen
            .get_by_role(Role::Canvas, "Graph")
            .a11y(A11yAction::ShowMenu);
        screen.get_by_role(Role::MenuItem, "Inspect Some(11) Some(1) 2");
        assert_eq!(m.value(), 2.0);
        assert_eq!(
            m.phases.get_untracked(),
            [EditPhase::Begin, EditPhase::Cancel]
        );
        screen.press_key(Key::Escape);
        screen.mouse_up(MouseButton::Primary);
        clean(&screen);
    }
}
