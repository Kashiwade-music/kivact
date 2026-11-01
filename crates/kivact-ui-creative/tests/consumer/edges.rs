//! Role: Canvas gesture edge geometry and sticky movement regressions.
use super::*;
use kivact::draw::Rect;

const DOMAINS: [Kind; 5] = [
    Kind::Timeline,
    Kind::DopeSheet,
    Kind::Graph,
    Kind::Nodes,
    Kind::Arrangement,
];

#[test]
fn reversed_marquee_and_a_touching_edge_select_the_same_domain_item() {
    for kind in DOMAINS {
        for touching in [false, true] {
            let (screen, model) = fixture(kind, false);
            let bounds = item(&screen, model, 1).rect().expect("target item bounds");
            let (from, to) = if touching {
                (
                    Point::new(bounds.x - 12., bounds.center().y),
                    Point::new(bounds.x, bounds.center().y),
                )
            } else {
                (
                    Point::new(bounds.right() + 2., bounds.bottom() + 2.),
                    Point::new(bounds.x - 2., bounds.y - 2.),
                )
            };
            let rectangle = Rect::from_points(from, to);
            assert_eq!(
                Rect::from_points(from, from),
                Rect::new(from.x, from.y, 0., 0.)
            );
            assert_eq!(rectangle.x, from.x.min(to.x));
            assert_eq!(rectangle.y, from.y.min(to.y));
            assert!(rectangle.intersects(bounds));
            screen
                .get_by_role(Role::Canvas, "Gesture editor")
                .drag(from, to);
            assert_eq!(
                model.selection.get_untracked(),
                [2],
                "{kind:?}, touching={touching}"
            );
            clean(&screen);
        }
    }
}

#[test]
fn a_second_click_that_moves_out_and_back_never_activates() {
    for kind in KINDS {
        let (screen, model) = fixture(kind, true);
        if kind == Kind::Nodes {
            model.nodes.set(super::super::node_edit::nodes());
            screen.run_until_idle();
        }
        let target = if kind == Kind::Nodes {
            screen.get_by_role(Role::SpinButton, "Source Amount")
        } else {
            item(&screen, model, 0)
        };
        target.click();
        let point = target.rect().expect("target bounds").center();
        screen.mouse_move(point.x, point.y);
        screen.mouse_down(MouseButton::Primary);
        screen.mouse_move(point.x + 1., point.y);
        screen.mouse_move(point.x, point.y);
        screen.mouse_up(MouseButton::Primary);
        assert!(model.activated.get_untracked().is_empty(), "{kind:?}");
        if kind == Kind::Nodes {
            assert!(screen.query_by_role(Role::SpinButton, "Amount").is_none());
        }
        clean(&screen);
    }
}

#[test]
fn cancelling_a_stationary_press_keeps_multiple_selection_and_discards_pending_replace() {
    for kind in DOMAINS {
        let (screen, model) = fixture(kind, false);
        model.selection.set(vec![1, 2]);
        screen.run_until_idle();
        let point = item(&screen, model, 0)
            .rect()
            .expect("target bounds")
            .center();
        screen.mouse_move(point.x, point.y);
        screen.mouse_down(MouseButton::Primary);
        screen.press_key(Key::Escape);
        screen.mouse_up(MouseButton::Primary);
        assert_eq!(model.selection.get_untracked(), [1, 2], "{kind:?}");
        item(&screen, model, 1).click();
        assert_eq!(
            model.selection.get_untracked(),
            [2],
            "{kind:?}: the next press has its own pending selection"
        );
        clean(&screen);
    }
}
