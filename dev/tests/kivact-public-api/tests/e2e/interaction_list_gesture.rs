//! Role: Public modifier interpretation drives live list selection and semantics.
use crate::support::mount;
use kivact::interaction::{
    ListCommandModifier, ListGesture, ListPolicy, ListRangePolicy, ListSelection, use_list,
};
use kivact::prelude::*;
use kivact::ui::ListItem;
use kivact::view::Modifiers;

#[test]
fn modifier_gestures_use_each_lists_command_policy_in_a_live_selection() {
    let plain: &[u64] = &[2];
    let toggle: &[u64] = &[3, 2];
    let replace_range: &[u64] = &[0, 1, 2];
    let add_range: &[u64] = &[3, 0, 1, 2];
    let cases = [
        (Modifiers::default(), [plain, plain, plain]),
        (
            Modifiers {
                ctrl: true,
                ..Modifiers::default()
            },
            [toggle, plain, toggle],
        ),
        (
            Modifiers {
                meta: true,
                ..Modifiers::default()
            },
            [plain, toggle, toggle],
        ),
        (
            Modifiers {
                shift: true,
                ..Modifiers::default()
            },
            [replace_range, replace_range, replace_range],
        ),
        (
            Modifiers {
                ctrl: true,
                shift: true,
                ..Modifiers::default()
            },
            [add_range, replace_range, add_range],
        ),
        (
            Modifiers {
                meta: true,
                shift: true,
                ..Modifiers::default()
            },
            [replace_range, add_range, add_range],
        ),
    ];
    for (index, command) in [
        ListCommandModifier::Control,
        ListCommandModifier::Meta,
        ListCommandModifier::ControlOrMeta,
    ]
    .into_iter()
    .enumerate()
    {
        let (screen, (selected, anchor, modifiers)) = mount(move || {
            let selected = signal(vec![3_u64]);
            let anchor = signal(Some(0_u64));
            let modifiers = signal(Modifiers::default());
            let policy = ListPolicy {
                range: ListRangePolicy {
                    command,
                    ..ListRangePolicy::default()
                },
                ..ListPolicy::default()
            };
            let list = use_list(
                (0..4)
                    .map(|key| ListItem::new(key, format!("Item {key}")))
                    .collect::<Vec<_>>(),
                ListSelection::Multiple(selected.into()),
            )
            .anchor(anchor)
            .policy(policy);
            (
                (selected, anchor, modifiers),
                column((
                    button("Choose third").on_press(move || {
                        list.select(
                            2,
                            ListGesture::from_modifiers(modifiers.get_untracked(), &policy),
                        );
                    }),
                    for_each(
                        vec![0_u64, 1, 2, 3],
                        |key| *key,
                        move |key, _| {
                            node()
                                .part(list.item(key))
                                .child(text(format!("Item {key}")))
                        },
                    ),
                )),
            )
        });
        for (keys, expected) in cases {
            selected.set(vec![3]);
            anchor.set(Some(0));
            modifiers.set(keys);
            screen.run_until_idle();
            screen.get_by_role(Role::Button, "Choose third").click();
            assert_eq!(
                selected.get_untracked(),
                expected[index],
                "{command:?}, {keys:?}"
            );
            for key in 0..4 {
                assert_eq!(
                    screen
                        .get_by_role(Role::Option, &format!("Item {key}"))
                        .is_selected(),
                    expected[index].contains(&key)
                );
            }
        }
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
