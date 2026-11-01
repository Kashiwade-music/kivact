//! Role: Display and layout component tests: gallery layout and behavior checks.

use super::{CALLER_WIDTH, assert_caller_won, assert_clean, caller_style, distance, pixel_in};
use kivact::prelude::*;
use kivact::style::Align;
use kivact::test::{RenderOptions, Screen};
use kivact::ui::ButtonVariant;
use kivact::view::{Orientation, View};
use kivact_ui_basic as basic;

/// A page painted with the `background` token of the theme in effect.
fn page(child: impl IntoView) -> Node {
    column(child).style(
        style()
            .background(basic::Color::Background)
            .color(basic::Color::Foreground)
            .padding(16)
            .gap(16)
            .grow(1)
            .align_items(Align::Start),
    )
}

fn options(mode: kivact::style::ThemeMode) -> RenderOptions {
    RenderOptions {
        theme: basic::theme(mode),
        width: 900.0,
        height: 3200.0,
        reduced_motion: true,
        ..RenderOptions::from_env()
    }
}

/// A column of gallery sections (a tuple of views holds 16 at most).
fn col(children: impl IntoView) -> Node {
    column(children).style(style().gap(16).align_items(Align::Start))
}

/// `builder` at the gallery column width.
fn wide<B: NodeBuilder>(builder: B) -> B {
    builder.style(style().width(560))
}

/// Every display component once, in a column.
fn gallery() -> impl IntoView {
    let svg: &'static [u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#888"/></svg>"##;
    let open = signal(false);
    let page_index = signal(2usize);
    page((
        col((
            wide(
                basic::alert("Heads up!")
                    .icon(kivact::control::icon("info"))
                    .description("You can add components using the CLI.")
                    .action(move || {
                        kivact::ui::button("Undo").size(kivact::ui::ControlSize::Small)
                    }),
            ),
            wide(
                basic::alert("Error")
                    .variant(basic::AlertVariant::Destructive)
                    .icon(kivact::control::icon("circle-alert"))
                    .description("Your session has expired."),
            ),
            row((
                basic::badge("Default"),
                basic::badge("Secondary").variant(basic::BadgeVariant::Secondary),
                basic::badge("Destructive").variant(basic::BadgeVariant::Destructive),
                basic::badge("Outline").variant(basic::BadgeVariant::Outline),
                basic::badge("Ghost").variant(basic::BadgeVariant::Ghost),
                basic::badge("Link").variant(basic::BadgeVariant::Link),
                basic::badge("Verified")
                    .icon(kivact::control::icon("badge-check"))
                    .trailing_icon(kivact::control::icon("arrow-up-right")),
            ))
            .style(style().gap(8).align_items(Align::Center)),
            row((
                basic::avatar("CN").size(basic::AvatarSize::Sm),
                basic::avatar("CN"),
                basic::avatar("CN")
                    .size(basic::AvatarSize::Lg)
                    .badge(move || node()),
                basic::avatar("IM").image(svg),
                basic::avatar_group((
                    basic::avatar("A"),
                    basic::avatar("B"),
                    basic::avatar_group_count("+3"),
                )),
            ))
            .style(style().gap(12).align_items(Align::Center)),
            basic::breadcrumb_content((
                basic::breadcrumb_link("Home").on_press(|| {}),
                basic::breadcrumb_separator(),
                basic::breadcrumb_ellipsis(),
                basic::breadcrumb_separator(),
                basic::breadcrumb_link("Components"),
                basic::breadcrumb_separator(),
                basic::breadcrumb_page("Breadcrumb"),
            )),
            wide(
                basic::bubble_group((
                    basic::bubble("Hello there").variant(basic::BubbleVariant::Default),
                    basic::bubble("Secondary").variant(basic::BubbleVariant::Secondary),
                    basic::bubble("Muted").variant(basic::BubbleVariant::Muted),
                    basic::bubble("Tinted").variant(basic::BubbleVariant::Tinted),
                    basic::bubble("Outline").variant(basic::BubbleVariant::Outline),
                    basic::bubble("Ghost").variant(basic::BubbleVariant::Ghost),
                    basic::bubble("Destructive")
                        .variant(basic::BubbleVariant::Destructive)
                        .align(basic::BubbleAlign::End)
                        .reactions(move || text("+1")),
                ))
                .style(style().padding_bottom(16)),
            ),
            wide(basic::message_group((
                basic::message(basic::bubble("Hi").variant(basic::BubbleVariant::Secondary))
                    .avatar(move || basic::avatar("AB"))
                    .header(move || text("Ada"))
                    .footer(move || text("10:42")),
                basic::message(basic::bubble("Hello").align(basic::BubbleAlign::End))
                    .align(basic::MessageAlign::End)
                    .header(move || text("You")),
            ))),
        )),
        col((
            basic::button_group((
                kivact::ui::button("Archive").variant(ButtonVariant::Outline),
                kivact::ui::button("Report").variant(ButtonVariant::Outline),
                kivact::ui::button("Snooze").variant(ButtonVariant::Outline),
            )),
            basic::button_group((
                kivact::ui::button("Up").variant(ButtonVariant::Outline),
                kivact::ui::button("Down").variant(ButtonVariant::Outline),
            ))
            .orientation(Orientation::Vertical),
            row((
                basic::button_group_text("https://"),
                basic::button_group_separator(),
                basic::button_group_text("example.com"),
            ))
            .style(style().height(32)),
            basic::card()
                .size(basic::CardSize::Default)
                .header(
                    basic::card_header("Login to your account")
                        .description("Enter your email below to login")
                        .action(move || kivact::ui::button("Sign up").variant(ButtonVariant::Link)),
                )
                .content(move || text("The form goes here"))
                .footer(move || kivact::ui::button("Login"))
                .style(style().width(360)),
            basic::card()
                .size(basic::CardSize::Sm)
                .header(basic::card_header("Small card").bordered(true))
                .content(move || text("Small content"))
                .style(style().width(360)),
            wide(
                basic::empty_state("No Projects Yet")
                    .media(move || kivact::control::icon("folder"))
                    .media_variant(basic::EmptyMediaVariant::Icon)
                    .description("Get started by creating your first project.")
                    .content(move || kivact::ui::button("Create Project"))
                    .style(style().border(1, basic::Color::Border)),
            ),
            wide(basic::item_group((
                basic::item("Basic Item")
                    .description("A simple item with title and description.")
                    .actions(move || kivact::ui::button("Action").variant(ButtonVariant::Outline)),
                basic::item_separator(),
                basic::item("Outline")
                    .variant(basic::ItemVariant::Outline)
                    .media(move || kivact::control::icon("shield"))
                    .media_variant(basic::ItemMediaVariant::Icon)
                    .on_press(|| {}),
                basic::item("Muted")
                    .variant(basic::ItemVariant::Muted)
                    .size(basic::ItemSize::Sm)
                    .header(move || text("Header"))
                    .footer(move || text("Footer")),
            ))),
            wide(basic::marker("Today").variant(basic::MarkerVariant::Separator)),
            wide(basic::marker("Borders").variant(basic::MarkerVariant::Border)),
            wide(basic::marker("With icon").icon(kivact::control::icon("clock"))),
        )),
        col((
            basic::attachment_group((
                basic::attachment("report.pdf")
                    .media(move || kivact::control::icon("file-text"))
                    .description("2.4 MB")
                    .actions(move || basic::attachment_action("x", "Remove report")),
                basic::attachment("upload.png")
                    .size(basic::AttachmentSize::Sm)
                    .media(move || kivact::control::icon("image"))
                    .state(basic::AttachmentState::Uploading),
                basic::attachment("broken.zip")
                    .size(basic::AttachmentSize::Xs)
                    .media(move || kivact::control::icon("archive"))
                    .description("Failed")
                    .state(basic::AttachmentState::Error),
                basic::attachment("drop")
                    .media(move || kivact::control::icon("paperclip"))
                    .state(basic::AttachmentState::Idle),
                basic::attachment("vertical.pdf")
                    .orientation(Orientation::Vertical)
                    .media(move || kivact::control::icon("file"))
                    .description("1 MB")
                    .actions(move || basic::attachment_action("x", "Remove vertical")),
            ))
            .style(style().width(640)),
            basic::pagination(page_index, 12usize),
            basic::pagination_content((
                basic::pagination_previous(),
                basic::pagination_link("1"),
                basic::pagination_link("2").active(true),
                basic::pagination_ellipsis(),
                basic::pagination_next(),
            )),
            wide(kivact::ui::progress(Some(0.66)).name("Upload progress")),
            wide(kivact::ui::progress(Some(0.1))),
            basic::scroll_area(column(
                (0..30)
                    .map(|n| text(format!("v1.2.0-beta.{n}")))
                    .collect::<Vec<_>>(),
            ))
            .style(style().height(120).width(200)),
            wide(basic::table((
                basic::table_header(basic::table_row((
                    basic::table_head("Invoice").width(100.0),
                    basic::table_head("Status"),
                    basic::table_head("Amount").align(basic::TableAlign::End),
                ))),
                basic::table_body((
                    basic::table_row((
                        basic::table_cell("INV001").width(100.0),
                        basic::table_cell("Paid"),
                        basic::table_cell("$250.00").align(basic::TableAlign::End),
                    )),
                    basic::table_row((
                        basic::table_cell("INV002").width(100.0),
                        basic::table_cell("Pending"),
                        basic::table_cell("$150.00").align(basic::TableAlign::End),
                    ))
                    .selected(true),
                )),
                basic::table_footer(basic::table_row((
                    basic::table_cell("Total").width(100.0),
                    basic::table_cell(""),
                    basic::table_cell("$400.00").align(basic::TableAlign::End),
                ))),
                basic::table_caption("A list of your recent invoices."),
            ))),
            wide(basic::trace((
                basic::trace_line("Searched the web")
                    .icon(kivact::control::icon("search"))
                    .description("1.2s"),
                basic::trace_collapsible(
                    open,
                    basic::trace_line("Thought for 4s").icon(kivact::control::icon("sparkles")),
                    "The reasoning goes here.",
                ),
            ))),
        )),
    ))
}

fn shown(screen: &Screen, text: &str) -> bool {
    screen.query_by_text(text).is_some_and(|e| e.is_visible())
}

#[test]
fn gallery_lays_out_cleanly_in_light_and_dark() {
    for mode in [
        kivact::style::ThemeMode::Light,
        kivact::style::ThemeMode::Dark,
    ] {
        let (screen, ()) = kivact::test::render_with_handle(options(mode), || ((), gallery()));
        screen.assert_layout();
        screen.get_by_role(Role::Alert, "Heads up!");
        screen.get_by_role(Role::Alert, "Error");
        screen.get_by_role(Role::Navigation, "Breadcrumb");
        assert_eq!(screen.all_by_role(Role::Navigation, "Pagination").len(), 2);
        screen.get_by_role(Role::ProgressBar, "Upload progress");
        screen.get_by_role(Role::Table, "");
        screen.get_by_role(Role::List, "");
        screen.get_by_role(Role::Status, "Loading");
        assert_clean(&screen);
    }
}

#[test]
fn badge_variants_differ_and_press_reports_to_the_callback() {
    let (screen, count) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let count = signal(0usize);
            (
                count,
                page(
                    row((
                        basic::badge("Default").test_id("default"),
                        basic::badge("Secondary")
                            .variant(basic::BadgeVariant::Secondary)
                            .test_id("secondary"),
                        basic::badge("Open")
                            .variant(basic::BadgeVariant::Outline)
                            .on_press(move || count.update(|n| *n += 1))
                            .test_id("open"),
                    ))
                    .style(style().gap(8)),
                ),
            )
        });
    let fill = |id: &str| pixel_in(&screen, id, 3.0);
    assert!(distance(fill("default"), 0x171717) <= 2);
    assert!(distance(fill("secondary"), 0xf5f5f5) <= 2);
    assert_eq!(
        screen.get_by_test_id("default").rect().unwrap().height,
        20.0
    );
    let open = screen.get_by_role(Role::Button, "Open");
    open.click();
    open.focus();
    open.press_key(Key::Enter);
    open.press_key(Key::Space);
    assert_eq!(count.get_untracked(), 3);
    // A badge without a callback is not a control.
    assert!(screen.query_by_role(Role::Button, "Default").is_none());
    assert_clean(&screen);
}

#[test]
fn item_and_attachment_press_independently_of_their_actions() {
    let (screen, (item_count, file_count, remove_count)) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let item_count = signal(0usize);
            let file_count = signal(0usize);
            let remove_count = signal(0usize);
            (
                (item_count, file_count, remove_count),
                page((
                    basic::item("Open settings")
                        .description("Configure things")
                        .on_press(move || item_count.update(|n| *n += 1))
                        .style(style().width(400)),
                    basic::attachment("report.pdf")
                        .media(move || kivact::control::icon("file-text"))
                        .actions(move || {
                            basic::attachment_action("x", "Remove report")
                                .on_press(move || remove_count.update(|n| *n += 1))
                        })
                        .on_press(move || file_count.update(|n| *n += 1)),
                )),
            )
        });
    let item = screen.get_by_role(Role::Button, "Open settings");
    item.click();
    item.focus();
    item.press_key(Key::Enter);
    assert_eq!(item_count.get_untracked(), 2);
    let file = screen.get_by_role(Role::Button, "report.pdf");
    file.click();
    assert_eq!(file_count.get_untracked(), 1);
    screen.get_by_role(Role::Button, "Remove report").click();
    assert_eq!(remove_count.get_untracked(), 1);
    assert_eq!(
        file_count.get_untracked(),
        1,
        "the action does not press the chip"
    );
    assert_clean(&screen);
}

#[test]
fn pagination_pages_follows_its_binding_and_disables_the_ends() {
    let (screen, page_index) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let index = signal(0usize);
            ((index), page(basic::pagination(index, 10usize)))
        });
    let previous = || screen.get_by_role(Role::Link, "Go to previous page");
    let next = || screen.get_by_role(Role::Link, "Go to next page");
    assert!(previous().is_disabled());
    assert!(!next().is_disabled());
    assert!(screen.get_by_role(Role::Link, "1").is_selected());
    // First, current + 1, last, with one gap between.
    for shown in ["1", "2", "10"] {
        screen.get_by_role(Role::Link, shown);
    }
    assert!(screen.query_by_role(Role::Link, "5").is_none());
    assert_eq!(screen.all_by_role(Role::Text, "More pages").len(), 1);
    next().click();
    assert_eq!(page_index.get_untracked(), 1);
    assert!(!previous().is_disabled());
    assert!(screen.get_by_role(Role::Link, "2").is_selected());
    screen.get_by_role(Role::Link, "10").click();
    assert_eq!(page_index.get_untracked(), 9);
    assert!(next().is_disabled());
    next().force_click();
    assert_eq!(
        page_index.get_untracked(),
        9,
        "a disabled link does not move"
    );
    previous().click();
    assert_eq!(page_index.get_untracked(), 8);
    // Two gaps in the middle; the current page and its neighbours show.
    page_index.set(5);
    screen.run_until_idle();
    for shown in ["1", "5", "6", "7", "10"] {
        screen.get_by_role(Role::Link, shown);
    }
    assert_eq!(screen.all_by_role(Role::Text, "More pages").len(), 2);
    assert!(screen.get_by_role(Role::Link, "6").is_selected());
    assert_clean(&screen);
}

#[test]
fn card_header_border_follows_a_reactive_option() {
    let (screen, bordered) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let bordered = signal(false);
            (
                bordered,
                page(
                    basic::card()
                        .header(basic::card_header("Plain").bordered(bordered))
                        .content(move || text("Body"))
                        .style(style().width(360))
                        .test_id("card"),
                ),
            )
        });
    let height = |screen: &Screen| screen.get_by_test_id("card").rect().unwrap().height;
    let plain = height(&screen);
    bordered.set(true);
    screen.run_until_idle();
    assert!(height(&screen) > plain, "the rule and its padding appear");
    bordered.set(false);
    screen.run_until_idle();
    assert_eq!(height(&screen), plain);
    assert_clean(&screen);
}

#[test]
fn progress_exposes_its_value_range_and_clamps() {
    let (screen, value) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let value = signal(0.4f64);
            (
                value,
                page(
                    kivact::ui::progress(move || Some(value.get()))
                        .name("Upload")
                        .style(style().width(300)),
                ),
            )
        });
    let bar = screen.get_by_role(Role::ProgressBar, "Upload");
    assert_eq!(bar.value_range(), (Some(0.0), Some(1.0), Some(0.4)));
    assert_eq!(bar.value().as_deref(), Some("40%"));
    value.set(2.5);
    screen.run_until_idle();
    assert_eq!(bar.value_range().2, Some(1.0));
    assert_eq!(bar.value().as_deref(), Some("100%"));
    assert_clean(&screen);
}

#[test]
fn trace_collapsible_opens_with_keyboard_and_pointer() {
    let (screen, open) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let open = signal(false);
            (
                open,
                page(basic::trace_collapsible(
                    open,
                    basic::trace_line("Thought for 4s"),
                    "The reasoning goes here.",
                )),
            )
        });
    let trigger = screen.get_by_role(Role::Button, "Thought for 4s");
    assert_eq!(trigger.is_expanded(), Some(false));
    assert!(!shown(&screen, "The reasoning goes here."));
    trigger.click();
    assert!(open.get_untracked());
    assert_eq!(trigger.is_expanded(), Some(true));
    assert!(shown(&screen, "The reasoning goes here."));
    trigger.focus();
    trigger.press_key(Key::Enter);
    assert!(!open.get_untracked());
    assert!(!shown(&screen, "The reasoning goes here."));
    trigger.press_key(Key::Space);
    assert!(open.get_untracked());
    assert!(shown(&screen, "The reasoning goes here."));
    assert_clean(&screen);
}

#[test]
fn avatar_shows_its_fallback_and_names_its_image() {
    let svg: &'static [u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#c00"/></svg>"##;
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), move || {
            (
                (),
                page(
                    row((
                        basic::avatar("SM")
                            .size(basic::AvatarSize::Sm)
                            .test_id("sm"),
                        basic::avatar("CN").test_id("default"),
                        basic::avatar("LG")
                            .size(basic::AvatarSize::Lg)
                            .test_id("lg"),
                        basic::avatar("PIC").image(svg).test_id("picture"),
                    ))
                    .style(style().gap(8)),
                ),
            )
        });
    screen.get_by_text("CN");
    screen.get_by_text("SM");
    let side = |id: &str| screen.get_by_test_id(id).rect().unwrap().width;
    assert_eq!(
        (side("sm"), side("default"), side("lg")),
        (24.0, 32.0, 40.0)
    );
    screen.get_by_role(Role::Image, "PIC");
    assert!(screen.query_by_role(Role::Image, "CN").is_none());
    // The fallback disc is muted gray.
    assert!(distance(pixel_in(&screen, "default", 4.0), 0xf5f5f5) <= 4);
    assert_clean(&screen);
}

#[test]
fn table_publishes_roles_and_shares_width() {
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page(
                    basic::table((
                        basic::table_header(basic::table_row((
                            basic::table_head("Name").width(120.0),
                            basic::table_head("Amount").align(basic::TableAlign::End),
                        ))),
                        basic::table_body((
                            basic::table_row((
                                basic::table_cell("Ada").width(120.0),
                                basic::table_cell("$10").align(basic::TableAlign::End),
                            )),
                            basic::table_row((
                                basic::table_cell("Grace").width(120.0),
                                basic::table_cell("$20").align(basic::TableAlign::End),
                            ))
                            .selected(true),
                        )),
                    ))
                    .style(style().width(400)),
                ),
            )
        });
    screen.get_by_role(Role::Table, "");
    assert_eq!(
        screen.all_by_role(Role::RowGroup, "").len(),
        2,
        "head and body"
    );
    assert_eq!(screen.all_by_role(Role::ColumnHeader, "").len(), 2);
    assert_eq!(screen.all_by_role(Role::Row, "").len(), 3);
    assert_eq!(screen.all_by_role(Role::Cell, "").len(), 4);
    let selected: Vec<bool> = screen
        .all_by_role(Role::Row, "")
        .iter()
        .map(|row| row.is_selected())
        .collect();
    assert_eq!(selected, [false, false, true]);
    let name = screen.get_by_text("Name").rect().unwrap();
    let ada = screen.get_by_text("Ada").rect().unwrap();
    assert_eq!(name.x, ada.x, "cells of a column align");
    let head = screen.get_by_text("Amount").rect().unwrap();
    let cell = screen.get_by_text("$10").rect().unwrap();
    assert!(
        (head.x + head.width - cell.x - cell.width).abs() <= 1.0,
        "end-aligned cells share their right edge"
    );
    assert!(head.x > 120.0 + name.x, "the second column takes the rest");
    assert_clean(&screen);
}

#[test]
fn breadcrumb_and_pagination_reverse_in_a_right_to_left_direction() {
    let mut trails = Vec::new();
    for dir in [
        kivact::view::ReadingDirection::Ltr,
        kivact::view::ReadingDirection::Rtl,
    ] {
        let (screen, ()) =
            kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), move || {
                (
                    (),
                    page(kivact::view::provide_direction(dir, || {
                        column((
                            basic::breadcrumb_content((
                                basic::breadcrumb_link("Home").on_press(|| {}),
                                basic::breadcrumb_separator(),
                                basic::breadcrumb_page("Page"),
                            ))
                            .style(style().width(300)),
                            basic::pagination_content((
                                basic::pagination_previous(),
                                basic::pagination_next(),
                            ))
                            .style(style().width(300)),
                        ))
                        .style(style().gap(12))
                    })),
                )
            });
        let x = |text: &str| screen.get_by_text(text).rect().unwrap().x;
        trails.push((x("Home") < x("Page"), x("Previous") < x("Next")));
        assert_clean(&screen);
    }
    assert_eq!(trails, [(true, true), (false, false)]);
}

#[test]
fn button_group_joins_borders_and_sizes_its_cells() {
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page(basic::button_group((
                    kivact::ui::button("One")
                        .variant(ButtonVariant::Outline)
                        .test_id("one"),
                    kivact::ui::button("Two")
                        .variant(ButtonVariant::Outline)
                        .test_id("two"),
                ))),
            )
        });
    let one = screen.get_by_test_id("one").rect().unwrap();
    let two = screen.get_by_test_id("two").rect().unwrap();
    // The second drops its left border (`border-l-0`): the two touch, and
    // the line between them is the first one's right border.
    assert_eq!(two.x, one.x + one.width, "neighbours share a border");
    assert_eq!(one.height, two.height);
    assert_clean(&screen);
}

#[test]
fn scroll_area_scrolls_its_content() {
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page(
                    basic::scroll_area(column(
                        (0..40)
                            .map(|n| text(format!("Row {n}")).style(style().height(20)))
                            .collect::<Vec<_>>(),
                    ))
                    .style(style().height(100).width(200))
                    .test_id("area"),
                ),
            )
        });
    screen.get_by_role(Role::ScrollView, "");
    let area = screen.get_by_test_id("area").rect().unwrap();
    assert_eq!(area.height, 100.0);
    assert!(screen.get_by_text("Row 0").rect().unwrap().y < area.y + area.height);
    assert_clean(&screen);
}

#[test]
fn a_small_button_draws_a_14_px_icon() {
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page(row((
                    kivact::ui::button("Small")
                        .size(kivact::ui::ControlSize::Small)
                        .icon(kivact::control::icon("plus").test_id("small")),
                    kivact::ui::button("Default")
                        .icon(kivact::control::icon("plus").test_id("default")),
                ))),
            )
        });
    let side = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert_eq!(side("small"), 14.0, "size-3.5");
    assert_eq!(side("default"), 16.0, "size-4");
    screen.assert_layout();
    assert_clean(&screen);
}

/// Every display builder with the caller's width, padding and fill (and, where
/// it has a role, the caller's name): the caller's `.style(..)` and `.name(..)`
/// come after the constructor's, so they must win over the kit's defaults.
#[test]
fn caller_styles_and_names_win_on_display_components() {
    let svg: &'static [u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#c00"/></svg>"##;
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), move || {
            let page_index = signal(2usize);
            let first: Vec<View> = vec![
                kivact::ui::progress(Some(0.4))
                    .name("Custom progress")
                    .style(caller_style())
                    .test_id("progress")
                    .into_view(),
                basic::card()
                    .content(move || text("Card body").test_id("card-body"))
                    .style(caller_style())
                    .test_id("card")
                    .into_view(),
                basic::badge("Plain")
                    .style(caller_style())
                    .test_id("badge")
                    .into_view(),
                basic::badge("Real badge")
                    .on_press(|| {})
                    .name("Custom badge")
                    .style(caller_style())
                    .test_id("badge-press")
                    .into_view(),
                basic::alert("Real alert")
                    .description("Details")
                    .name("Custom alert")
                    .style(caller_style())
                    .test_id("alert")
                    .into_view(),
                basic::avatar("AV")
                    .image(svg)
                    .name("Custom avatar")
                    .style(style().width(CALLER_WIDTH))
                    .test_id("avatar")
                    .into_view(),
                basic::attachment("Real attachment")
                    .on_press(|| {})
                    .name("Custom attachment")
                    .style(caller_style())
                    .test_id("attachment")
                    .into_view(),
                basic::breadcrumb_link("Real link")
                    .on_press(|| {})
                    .name("Custom link")
                    .style(caller_style())
                    .test_id("breadcrumb-link")
                    .into_view(),
                basic::button_group((text("A"), text("B")))
                    .name("Custom group")
                    .style(caller_style())
                    .test_id("button-group")
                    .into_view(),
                // The bubble's own `max-width: 80%` stays unless the caller lifts it.
                basic::bubble("Bubble")
                    .style(caller_style().max_width(500))
                    .test_id("bubble")
                    .into_view(),
                basic::message(text("Message"))
                    .style(caller_style())
                    .test_id("message")
                    .into_view(),
            ];
            let second: Vec<View> = vec![
                basic::empty_state("Nothing")
                    .style(caller_style())
                    .test_id("empty")
                    .into_view(),
                basic::item("Real item")
                    .description("Details")
                    .name("Custom item")
                    .style(caller_style())
                    .test_id("item")
                    .into_view(),
                basic::marker("Marker")
                    .style(caller_style())
                    .test_id("marker")
                    .into_view(),
                basic::message_scroller(text("Scrolled"))
                    .style(caller_style().height(60))
                    .test_id("scroller")
                    .into_view(),
                basic::scroll_area(text("Scrolled"))
                    .style(caller_style().height(60))
                    .test_id("scroll-area")
                    .into_view(),
                basic::table_row((
                    basic::table_cell("Cell")
                        .style(caller_style())
                        .test_id("cell"),
                    basic::table_cell("Other"),
                ))
                .style(caller_style())
                .test_id("table-row")
                .into_view(),
                basic::trace_line("Real trace")
                    .style(caller_style())
                    .test_id("trace-line")
                    .into_view(),
                basic::separator()
                    .style(style().width(CALLER_WIDTH).background(rgb(0xff00ff)))
                    .test_id("separator")
                    .into_view(),
                basic::pagination_link("7")
                    .name("Custom page")
                    .style(caller_style())
                    .test_id("page-link")
                    .into_view(),
                // Three pages: the bar fits inside the caller's width and
                // padding (five pages' links reach over the padding).
                basic::pagination(page_index, 3usize)
                    .name("Custom pages")
                    .style(caller_style())
                    .test_id("pages")
                    .into_view(),
                basic::pagination_content(basic::pagination_link("1"))
                    .name("Custom pagination")
                    .style(caller_style())
                    .test_id("pagination")
                    .into_view(),
                basic::breadcrumb_content(basic::breadcrumb_page("Here"))
                    .name("Custom breadcrumb")
                    .style(caller_style())
                    .test_id("breadcrumb")
                    .into_view(),
            ];
            ((), page((col(first), col(second))))
        });
    // Names win where the component has a role.
    screen.get_by_role(Role::ProgressBar, "Custom progress");
    screen.get_by_role(Role::Button, "Custom badge");
    screen.get_by_role(Role::Alert, "Custom alert");
    screen.get_by_role(Role::Image, "Custom avatar");
    screen.get_by_role(Role::Button, "Custom attachment");
    screen.get_by_role(Role::Link, "Custom link");
    screen.get_by_role(Role::Group, "Custom group");
    screen.get_by_role(Role::ListItem, "Custom item");
    screen.get_by_role(Role::Link, "Custom page");
    screen.get_by_role(Role::Navigation, "Custom pages");
    screen.get_by_role(Role::Navigation, "Custom pagination");
    screen.get_by_role(Role::Navigation, "Custom breadcrumb");
    // Width and fill win everywhere; the avatar is covered by its own disc.
    for id in [
        "progress",
        "card",
        "badge",
        "badge-press",
        "alert",
        "attachment",
        "breadcrumb-link",
        "button-group",
        "bubble",
        "message",
        "empty",
        "item",
        "marker",
        "scroller",
        "scroll-area",
        "table-row",
        "trace-line",
        "separator",
        "page-link",
        "pages",
        "pagination",
        "breadcrumb",
    ] {
        assert_caller_won(&screen, id, true);
    }
    assert_caller_won(&screen, "cell", false);
    let wide = |id: &str| screen.get_by_test_id(id).rect().expect("laid out").width;
    assert_eq!(wide("avatar"), CALLER_WIDTH);
    // Padding wins too: the text starts after the caller's 24 px.
    let inset = |id: &str, text: &str| {
        screen.get_by_text(text).rect().expect("laid out").x
            - screen.get_by_test_id(id).rect().expect("laid out").x
    };
    assert!(inset("card", "Card body") >= 24.0 + 16.0);
    assert!(inset("alert", "Real alert") >= 24.0);
    assert!(inset("item", "Real item") >= 24.0);
    assert_clean(&screen);
}

#[test]
fn bubble_and_breadcrumb_link_report_presses() {
    let (screen, (bubbled, linked)) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let bubbled = signal(0usize);
            let linked = signal(0usize);
            (
                (bubbled, linked),
                page((
                    basic::bubble("Tap this bubble")
                        .variant(basic::BubbleVariant::Outline)
                        .on_press(move || bubbled.update(|n| *n += 1)),
                    basic::bubble("Not pressable"),
                    basic::breadcrumb_content((
                        basic::breadcrumb_link("Home").on_press(move || linked.update(|n| *n += 1)),
                        basic::breadcrumb_separator(),
                        basic::breadcrumb_link("Docs"),
                        basic::breadcrumb_separator(),
                        basic::breadcrumb_page("Now"),
                    )),
                )),
            )
        });
    // A pressed bubble reports a click and Enter on the focused bubble.
    screen.get_by_text("Tap this bubble").click();
    assert_eq!(bubbled.get_untracked(), 1);
    assert!(screen.focused().is_some(), "the click focused the bubble");
    screen.press_key(Key::Enter);
    assert_eq!(bubbled.get_untracked(), 2);
    screen.press_key(Key::Space);
    assert_eq!(bubbled.get_untracked(), 3);
    // A link reports clicks and keys; one without a callback is no control.
    let home = screen.get_by_role(Role::Link, "Home");
    home.click();
    home.focus();
    home.press_key(Key::Enter);
    assert_eq!(linked.get_untracked(), 2);
    assert!(screen.query_by_role(Role::Link, "Docs").is_none());
    assert!(screen.get_by_role(Role::Link, "Now").is_disabled());
    assert_clean(&screen);
}

#[test]
fn separators_and_button_groups_follow_their_orientation() {
    let (screen, ()) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            (
                (),
                page((
                    row((
                        text("Blog"),
                        basic::separator()
                            .orientation(Orientation::Vertical)
                            .test_id("vertical"),
                        text("Docs"),
                    ))
                    .style(style().gap(8).height(20).align_items(Align::Center)),
                    basic::separator()
                        .test_id("horizontal")
                        .style(style().width(120)),
                    // In a horizontal group the separator is a vertical line, in a
                    // vertical group a horizontal one.
                    basic::button_group((
                        kivact::ui::button("One"),
                        basic::button_group_separator().test_id("across-row"),
                        kivact::ui::button("Two"),
                    )),
                    basic::button_group((
                        kivact::ui::button("Top"),
                        basic::button_group_separator().test_id("across-column"),
                        kivact::ui::button("Bottom"),
                    ))
                    .orientation(Orientation::Vertical),
                )),
            )
        });
    let size = |id: &str| {
        let rect = screen.get_by_test_id(id).rect().expect("laid out");
        (rect.width, rect.height)
    };
    assert_eq!(size("vertical").0, 1.0);
    assert!(size("vertical").1 >= 16.0, "it stretches across the row");
    assert_eq!(size("horizontal"), (120.0, 1.0));
    assert_eq!(
        screen.get_by_test_id("vertical").orientation(),
        Orientation::Vertical
    );
    assert_eq!(size("across-row").0, 1.0);
    assert!(size("across-row").1 >= 28.0, "{:?}", size("across-row"));
    assert_eq!(size("across-column").1, 1.0);
    assert!(
        size("across-column").0 >= 40.0,
        "{:?}",
        size("across-column")
    );
    assert_eq!(
        screen.get_by_test_id("across-column").orientation(),
        Orientation::Horizontal
    );
    screen.assert_layout();
    assert_clean(&screen);
}

#[test]
fn pagination_pages_formats_its_page_labels() {
    let (screen, index) =
        kivact::test::render_with_handle(options(kivact::style::ThemeMode::Light), || {
            let index = signal(0usize);
            (
                index,
                page(basic::pagination(index, 3usize).label_format(|number| format!("p{number}"))),
            )
        });
    for label in ["p1", "p2", "p3"] {
        screen.get_by_role(Role::Link, label);
    }
    assert!(screen.query_by_role(Role::Link, "1").is_none());
    screen.get_by_role(Role::Link, "p3").click();
    assert_eq!(index.get_untracked(), 2);
    assert_clean(&screen);
}
