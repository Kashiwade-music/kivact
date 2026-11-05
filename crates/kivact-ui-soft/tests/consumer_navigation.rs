//! Role: Exercise responsive Soft navigation through the public facade.
use kivact::state::signal;
use kivact::style::ThemeMode;
use kivact::style::style;
use kivact::test::RenderOptions;
use kivact::test::render_with;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::NodeBuilder;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::provide_direction;
use kivact::view::row;
use kivact::view::text;
use kivact_ui_soft as soft;

fn options(width: f64) -> RenderOptions {
    RenderOptions::from_env()
        .window_size(width, 650.0)
        .theme(soft::theme(ThemeMode::Light))
}

#[test]
fn rtl_toolbar_moves_between_unwrapped_slots_in_visual_order() {
    let screen = render_with(options(800.0), || {
        provide_direction(ReadingDirection::Rtl, || {
            kivact::ui::toolbar("Document tools")
                .start((kivact::ui::button("Start"),))
                .center((kivact::ui::button("Center"),))
                .end((kivact::ui::button("End"),))
        })
    });
    let start = screen.get_by_role(Role::Button, "Start");
    let center = screen.get_by_role(Role::Button, "Center");
    let end = screen.get_by_role(Role::Button, "End");
    assert!(start.rect().expect("laid out").x > end.rect().expect("laid out").x);
    screen.press_tab(false);
    assert!(start.is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(center.is_focused());
    screen.press_key(Key::ArrowLeft);
    assert!(end.is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn layout_components_mount_and_interact() {
    let screen = render_with(options(800.0), || {
        let selected = signal(false);
        let open = signal(false);
        let page = signal(9usize);
        let tab = signal("one");
        let viewport = node_ref();
        let target = node_ref();
        let resize = signal(250.0);
        column((
            kivact::ui::card(soft::with_card_insets(Default::default(), move || {
                soft::layout()
                    .height(soft::LayoutHeight::Auto)
                    .header(soft::layout_header(text("Header")).divider(true))
                    .start(
                        soft::layout_panel(text("Panel"))
                            .width(move || kivact::style::Length::Px(resize.get())),
                    )
                    .start_resize_handle(kivact::ui::splitter(
                        "Resize panel",
                        resize,
                        kivact::interaction::ScalarRange::new(50.0, 600.0).expect("valid range"),
                    ))
                    .content({
                        soft::layout_content(soft::section(soft::vstack((
                            soft::grid((text("First"), text("Second")))
                                .columns(soft::GridColumns::Fixed(2)),
                            soft::divider().label_text("or"),
                            soft::form_layout(soft::form_row(text("Label"), text("Value"))),
                        ))))
                        .scrollable(false)
                    })
            })),
            kivact::ui::card(text("Selectable"))
                .name("Select option")
                .selected(selected),
            kivact::ui::disclosure("Disclosure", open).content(|| text("Disclosure body")),
            kivact::ui::pagination(20usize, page)
                .window(kivact::ui::PaginationWindow::Buffered)
                .summary(soft::pagination_status),
            soft::tab_list("Test tabs", tab)
                .tabs([soft::tab("one", "One"), soft::tab("two", "Two")])
                .menu(soft::tab_menu(
                    "More",
                    vec![soft::TabMenuOption::new("three", "Three")],
                )),
            kivact::ui::stepper(
                vec![
                    kivact::ui::StepperItem::new("One"),
                    kivact::ui::StepperItem::new("Two"),
                ],
                1usize,
            ),
            kivact::ui::carousel(
                signal(0usize),
                (0..3)
                    .map(|i| {
                        node()
                            .style(style().size(100, 32))
                            .child(text(i.to_string()))
                    })
                    .collect::<Vec<_>>(),
            )
            .name("Pictures")
            .mode(kivact::ui::CarouselMode::Scroll)
            .snap(kivact::interaction::Snap::None)
            .style(style().width(160)),
            row((
                soft::outline(vec![soft::OutlineItem::new("a", "Outline link", 2, target)])
                    .scroll_container(viewport),
                soft::scrollable_area(
                    "Article",
                    node()
                        .node_ref(target)
                        .style(style().height(100))
                        .child(text("Heading")),
                )
                .node_ref(viewport)
                .style(style().height(24)),
            )),
            text(move || {
                if selected.get() {
                    "selected".to_owned()
                } else {
                    "unselected".to_owned()
                }
            }),
            text(move || format!("Page state {}", page.get() + 1)),
            text(move || format!("Tab state {}", tab.get())),
        ))
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.get_by_role(Role::CheckBox, "Select option").click();
    assert!(screen.query_by_text("selected").is_some());
    screen.get_by_role(Role::Button, "Disclosure").click();
    assert!(screen.query_by_text("Disclosure body").is_some());
    screen.get_by_role(Role::Button, "11").click();
    assert!(screen.query_by_text("Page state 11").is_some());
    screen.get_by_role(Role::Button, "Two").click();
    assert!(screen.query_by_text("Tab state two").is_some());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn shell_adapts_navigation_and_opens_the_mobile_drawer() {
    let screen = render_with(options(850.0), || {
        soft::app_shell(text("Main body"))
            .top_nav(|| {
                soft::top_nav()
                    .heading(|| soft::top_nav_heading("Acme").icon(|| soft::nav_icon(icon("menu"))))
                    .start(|| soft::top_nav_item("Documents").on_press(|| {}))
            })
            .side_nav(|| {
                soft::side_nav(
                    soft::side_nav_item("Files")
                        .icon(icon("menu"))
                        .on_press(|| {}),
                )
                .collapsible(soft::SideNavCollapse::Uncontrolled { default: false })
            })
            .into_view()
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    assert!(screen.query_by_role(Role::Link, "Files").is_some());
    screen.resize(600.0, 600.0);
    assert!(screen.query_by_role(Role::Link, "Files").is_none());
    screen.get_by_role(Role::Button, "Open navigation").click();
    assert!(screen.query_by_role(Role::Dialog, "Navigation").is_some());
    screen.advance(std::time::Duration::from_millis(350));
    screen.get_by_role(Role::Link, "Files").click();
    screen.advance(std::time::Duration::from_millis(350));
    assert!(screen.query_by_role(Role::Dialog, "Navigation").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn key_hint_observes_actual_focus_and_uses_a_single_timed_visit() {
    use kivact::test::render_with_handle;
    use kivact::view::Orientation;
    use std::time::Duration;
    let (screen, axis) = render_with_handle(options(800.0), || {
        let axis = signal(Orientation::Horizontal);
        (
            axis,
            column((
                soft::with_key_hints(axis, || {
                    kivact::ui::toolbar("Hint tools").start((
                        soft::keyboard_hint(kivact::ui::button("Undo")),
                        soft::keyboard_hint(kivact::ui::button("Redo")),
                    ))
                }),
                kivact::ui::button("After tools"),
            )),
        )
    });
    screen.press_tab(false);
    screen.get_by_text("to navigate");
    screen.get_by_role(Role::Image, "Left arrow");
    axis.set(Orientation::Vertical);
    screen.run_until_idle();
    screen.get_by_role(Role::Image, "Up arrow");
    screen.advance(Duration::from_millis(3000));
    screen.advance(Duration::from_millis(200));
    assert!(screen.query_by_text("to navigate").is_none());
    screen.press_tab(false);
    screen.press_tab(true);
    assert!(screen.query_by_text("to navigate").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn segmented_hint_attaches_to_the_canonical_radio_target() {
    use kivact::interaction::ListItem;
    use kivact::test::render_with_handle;
    use kivact::ui::{RadioGroupVariant, radio_group};
    use kivact::view::Orientation;
    let (screen, selected) = render_with_handle(options(800.0), || {
        let selected = signal(Some("summary"));
        (
            selected,
            soft::with_key_hints(Orientation::Horizontal, move || {
                radio_group(
                    "Views",
                    vec![
                        ListItem::new("summary", "Summary"),
                        ListItem::new("activity", "Activity"),
                    ],
                    selected,
                )
                .variant(RadioGroupVariant::Segmented)
                .item_node(|target, _, _| soft::keyboard_hint(target))
            }),
        )
    });
    screen.press_tab(false);
    assert!(
        screen
            .get_by_role(Role::RadioButton, "Summary")
            .is_focused()
    );
    screen.get_by_text("to navigate");
    screen.press_key(Key::ArrowRight);
    assert!(
        screen
            .get_by_role(Role::RadioButton, "Activity")
            .is_focused()
    );
    assert_eq!(selected.get_untracked(), Some("activity"));
    screen.advance(std::time::Duration::from_millis(200));
    assert!(screen.query_by_text("to navigate").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn compact_steps_skip_disabled_items_and_all_keeps_its_structure() {
    let (screen, current) = kivact::test::render_with_handle(options(400.0), || {
        let current = signal(0usize);
        (
            current,
            kivact::ui::stepper(
                vec![
                    kivact::ui::StepperItem::new("First"),
                    kivact::ui::StepperItem::new("Skipped").disabled(true),
                    kivact::ui::StepperItem::new("Last"),
                ],
                current,
            )
            .on_step_press(move |at| current.set(at))
            .style(style().width(170)),
        )
    });
    screen.get_by_role(Role::Button, "Next step").click();
    assert_eq!(current.get_untracked(), 2);
    screen.get_by_role(Role::Button, "Previous step").click();
    assert_eq!(current.get_untracked(), 0);
    assert!(screen.query_by_role(Role::ListItem, "Skipped").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    let all = kivact::test::render_with(options(400.0), || {
        kivact::ui::stepper(
            vec![
                kivact::ui::StepperItem::new("First"),
                kivact::ui::StepperItem::new("Last"),
            ],
            0usize,
        )
        .collapsed_steps(kivact::ui::CollapsedSteps::All)
        .style(style().width(100))
    });
    assert!(all.get_by_role(Role::ListItem, "First").is_visible());
    assert!(all.get_by_role(Role::ListItem, "Last").is_visible());
    assert!(all.errors().is_empty(), "{:?}", all.errors());
}

#[test]
fn canonical_dots_wrap_actual_focus_and_summary_tracks_item_units() {
    let (screen, page) = kivact::test::render_with_handle(options(400.0), || {
        let page = signal(2usize);
        (
            page,
            kivact::ui::pagination(3usize, page).mode(kivact::ui::PaginationMode::Dots),
        )
    });
    let last = screen.get_by_role(Role::Button, "3");
    last.focus();
    last.press_key(Key::ArrowRight);
    assert_eq!(page.get_untracked(), 0);
    assert!(screen.get_by_role(Role::Button, "1").is_focused());
    screen.press_key(Key::ArrowLeft);
    assert_eq!(page.get_untracked(), 2);
    assert!(screen.get_by_role(Role::Button, "3").is_focused());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    let (summary, size) = kivact::test::render_with_handle(options(400.0), || {
        let size = signal(20u32);
        (
            size,
            kivact::ui::pagination(0usize, signal(1usize))
                .total_items(95u64)
                .page_size(size)
                .summary(|facts| {
                    text(move || {
                        let facts = facts.get();
                        let total = facts.total_items.unwrap();
                        let first = (facts.page as u64 * u64::from(facts.page_size) + 1).min(total);
                        let last =
                            ((facts.page as u64 + 1) * u64::from(facts.page_size)).min(total);
                        format!("{first}–{last} of {total}")
                    })
                }),
        )
    });
    assert!(summary.get_by_text("21–40 of 95").exists());
    size.set(40);
    summary.run_until_idle();
    assert!(summary.get_by_text("41–80 of 95").exists());
    assert!(summary.errors().is_empty(), "{:?}", summary.errors());
}

#[test]
fn buffered_page_windows_preserve_all_four_original_edge_and_gap_cases() {
    use kivact::ui::{PageItem, PaginationWindow};
    for (current, total, expected) in [
        (
            1usize,
            20usize,
            vec!["1", "2", "3", "4", "5", "right gap", "20"],
        ),
        (
            10,
            20,
            vec!["1", "left gap", "9", "10", "11", "right gap", "20"],
        ),
        (20, 20, vec!["1", "left gap", "16", "17", "18", "19", "20"]),
        (1, 3, vec!["1", "2", "3"]),
    ] {
        let screen = kivact::test::render_with(options(600.0), move || {
            kivact::ui::pagination(total, signal(current - 1))
                .window(PaginationWindow::Buffered)
                .item(|cell| {
                    text(match cell {
                        PageItem::Page(at) => (at + 1).to_string(),
                        PageItem::Ellipsis => "…".to_owned(),
                        _ => String::new(),
                    })
                })
        });
        let current_x = screen
            .get_by_role(Role::Button, &current.to_string())
            .rect()
            .expect("mounted current page")
            .x;
        let mut actual = Vec::new();
        for page in 1..=total {
            if let Some(button) = screen.query_by_role(Role::Button, &page.to_string()) {
                actual.push((button.rect().expect("mounted page").x, page.to_string()));
            }
        }
        for gap in screen.all_by_text("…") {
            let x = gap.rect().expect("mounted gap").x;
            actual.push((
                x,
                if x < current_x {
                    "left gap"
                } else {
                    "right gap"
                }
                .to_owned(),
            ));
        }
        actual.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert_eq!(
            actual
                .into_iter()
                .map(|(_, label)| label)
                .collect::<Vec<_>>(),
            expected,
            "current {current} / {total}"
        );
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
