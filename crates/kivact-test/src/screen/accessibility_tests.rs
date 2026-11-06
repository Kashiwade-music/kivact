//! Role: Real-window rootless publication through the normal engine Writer.
//! Rootless hosts cannot be constructed through the public App API.
use kivact_engine::api::Client;
use kivact_engine::contract::{NodeKind, TurnOutput};
use kivact_engine::testing::{ArvaneTestHost, MirrorMode, TestHostOptions};
use kivact_types::{NodeId, Role};
use std::cell::RefCell;
use std::rc::Rc;

fn settle(host: &mut ArvaneTestHost) {
    host.wake();
    while host.redraw() {}
    #[cfg(debug_assertions)]
    assert!(host.self_check().is_empty());
}

#[test]
fn empty_real_empty_and_replaced_roots_publish_without_stale_nodes() {
    let slot = Rc::new(RefCell::new(None));
    let setup = slot.clone();
    let mut host = ArvaneTestHost::new(
        Client::new(
            move |_, _, writer| {
                *setup.borrow_mut() = Some(writer);
            },
            |_, _, _| TurnOutput::default(),
        ),
        TestHostOptions {
            width: 320.0,
            height: 200.0,
            mirror: MirrorMode::Required,
            ..Default::default()
        },
    )
    .expect("real host");
    let writer = slot.borrow_mut().take().expect("writer");
    settle(&mut host);
    let empty = host.semantics().expect("empty root published");
    assert_eq!(empty.nodes.len(), 1);
    assert_eq!(empty.nodes[0].semantics.role, Some(Role::Generic));
    assert_eq!(empty.nodes[0].bounds.width, 320.0);
    assert!(empty.nodes[0].children.is_empty());
    assert!(empty.focus.is_none());
    host.resize(400.0, 240.0);
    settle(&mut host);
    let resized = host.semantics().unwrap();
    assert_eq!(resized.root, empty.root);
    assert_eq!(resized.nodes[0].bounds.width, 400.0);
    for root in [NodeId(1), NodeId(3)] {
        writer.create(root, NodeKind::Root);
        let child = NodeId(root.0 + 1);
        writer.create(child, NodeKind::Box(Default::default()));
        writer.set_role(child, Role::Group);
        writer.set_name(child, format!("root {}", root.0));
        writer.insert(root, child, None);
        writer.commit();
        settle(&mut host);
        let tree = host.semantics().unwrap();
        assert_eq!(tree.root.0, root.0);
        assert_eq!(tree.nodes.len(), 2);
        writer.remove(root);
        writer.commit();
        settle(&mut host);
        let cleared = host.semantics().unwrap();
        assert_eq!(cleared.root, empty.root);
        assert_eq!(cleared.nodes.len(), 1);
        assert!(cleared.nodes[0].children.is_empty());
    }
}
