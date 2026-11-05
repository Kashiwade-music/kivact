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
            soft::toolbar("Document tools")
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
        let page = signal(10);
        let tab = signal("one");
        let viewport = node_ref();
        let target = node_ref();
        let resize = signal(250.0);
        column((
            soft::card(
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
                    }),
            ),
            soft::selectable_card("Select option", selected, text("Selectable")),
            soft::collapsible("Disclosure", open).content(|| text("Disclosure body")),
            soft::pagination(page).total_pages(20),
            soft::tab_list("Test tabs", tab)
                .tabs([soft::tab("one", "One"), soft::tab("two", "Two")])
                .menu(soft::tab_menu(
                    "More",
                    vec![soft::TabMenuOption::new("three", "Three")],
                )),
            soft::stepper(1, [soft::step(0, "One"), soft::step(1, "Two")]),
            soft::carousel(
                "Pictures",
                (0..3).map(|i| {
                    node()
                        .style(style().size(100, 32))
                        .child(text(i.to_string()))
                }),
            )
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
            text(move || format!("Page state {}", page.get())),
            text(move || format!("Tab state {}", tab.get())),
        ))
    });
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    screen.get_by_role(Role::CheckBox, "Select option").click();
    assert!(screen.query_by_text("selected").is_some());
    screen.get_by_role(Role::Button, "Disclosure").click();
    assert!(screen.query_by_text("Disclosure body").is_some());
    screen.get_by_role(Role::Button, "Go to page 11").click();
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
