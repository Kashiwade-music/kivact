//! Role: The Overlays page.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact::view::Node;
use kivact::view::Placement;
use kivact_ui_pop as ui;
/// The page of the Overlays components.
pub fn page() -> Page {
    Page::new(
        "overlays",
        "Overlays",
        "Components",
        include_str!("overlays.rs"),
    )
    .description("Dialogs, drawers, popovers, menus and palettes that open above the page.")
    .example(Example::new(
        "overlays",
        "Overlays",
        "Each overlay opens from its trigger and closes on its own action.",
        overlays_demo,
    ))
}

// xtask-allow(state): independent: confirm is the open state of the confirm dialog, written only by its button and that dialog; form is the open state of the form dialog, written only by its button and that dialog; sheet is the open state of the drawer, written only by its button and the drawer; command is the open state of the command palette, written only by its button and the palette; popover_open is the open state of the popover, written only by its trigger and its own Got it button
fn overlays_demo() -> impl IntoView {
    // example: overlays
    let confirm = signal(false);
    let form = signal(false);
    let sheet = signal(false);
    let command = signal(false);
    let popover_open = signal(false);
    section(
        "08",
        "Overlays",
        "Modals, drawers, menus and tooltips",
        cards_grid((
            third(card(
                "Dialogs",
                column((
                    kivact::ui::button("Confirm dialog").icon(kivact::view::icon("sword")).on_press(move || confirm.set(true)),
                    kivact::ui::button("Form dialog").icon(kivact::view::icon("pencil"))
                        .variant(kivact::ui::ButtonVariant::Secondary).on_press(move || form.set(true)),
                    kivact::ui::button("Open drawer").icon(kivact::view::icon("layout-sidebar-left-collapse"))
                        .variant(kivact_ui_pop::ButtonTone::Soft)
                        .on_press(move || sheet.set(true)),
                    kivact::ui::button("Command palette · Ctrl K").icon(kivact::view::icon("command"))
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .on_press(move || command.set(true)),
                ))
                .style(style().gap(16)),
            )),
            third(card(
                "Dropdown and popover",
                wrap((
                    kivact::ui::menu("Actions", vec![
                            MenuEntry::item("rename", "Rename").with_shortcut("F2"),
                            MenuEntry::item("copy", "Duplicate").with_shortcut("Ctrl D"),
                            MenuEntry::item("archive", "Archive"),
                            MenuEntry::item("delete", "Delete"),
                        ]).on_activate(|_: &'static str| {}),
                    kivact::ui::dialog("What is a halo?", popover_open).mode(kivact::interaction::DialogMode::Popover).content(move || {
                            column((
                            super::feedback::halo_loader(),
                            muted("The ring above a character's head. This UI uses it as a decorative motif for avatars and loaders."),
                            row((
                                kivact::ui::button("Got it").size(kivact::ui::ControlSize::Small).on_press(move || popover_open.set(false)),
                                kivact::ui::button("Details").variant(kivact::ui::ButtonVariant::Ghost).size(kivact::ui::ControlSize::Small),
                            )).style(style().gap(8)),
                        ))
                        .style(style().gap(6))
                        }).attach(kivact::ui::button("Popover")
                        .variant(kivact_ui_pop::ButtonTone::Soft)),
                ))
                .style(style().gap(8).align_items(Align::Start)),
            )),
            third(card(
                "Tooltips",
                wrap((
                    kivact::ui::tooltip("Shown above", kivact::ui::button("Above").variant(kivact::ui::ButtonVariant::Secondary)).align(kivact::view::OverlayAlign::Center).offset(6.0).placement(Placement::Above),
                    kivact::ui::tooltip("Shown below", kivact::ui::button("Below").variant(kivact::ui::ButtonVariant::Secondary)).align(kivact::view::OverlayAlign::Center).offset(6.0).placement(Placement::Below),
                    kivact::ui::tooltip("Shown on the right", kivact::ui::button("Right").variant(kivact::ui::ButtonVariant::Secondary)).align(kivact::view::OverlayAlign::Center).offset(6.0).placement(Placement::End),
                    muted("Save · Ctrl S").style(style().font_size(12)),
                )),
            )),
        )),
    )
    // end example
}

fn request_dialog(
    form_dialog: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> kivact::ui::Dialog {
    let request_title = signal(String::new());
    let request_priority = signal(Some("normal"));
    let request_detail = signal(String::new());
    let request_date = signal(CalendarDate::new(2026, 10, 10));
    kivact::ui::dialog("Create a new request", form_dialog)
        .content(move || {
            request_fields(
                request_title,
                request_priority,
                request_date,
                request_detail,
                form_dialog,
                notify,
            )
        })
        .footer(move || {
            row((
                kivact::ui::button("Cancel")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || form_dialog.set(false)),
                kivact::ui::button("Create")
                    .disabled(move || request_title.get().trim().is_empty())
                    .test_id("pop-create-request")
                    .on_press(move || {
                        form_dialog.set(false);
                        notify.call((
                            ui::Hue::Mint,
                            "Request created".into(),
                            "Students will be notified.".into(),
                        ));
                    }),
            ))
            .style(style().gap(8).justify_content(Justify::End))
        })
}

fn operation_confirmation(
    confirm: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> kivact::ui::Dialog {
    let remember_confirmation = signal(false);
    kivact::ui::dialog("Start the mission?", confirm)
        .content(move || {
            column((
                text("Spend 20 activity points to start 'Outer district patrol'."),
                row((
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("MN"),
                        AvatarLook::default().halo(true),
                    ),
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("HK"),
                        AvatarLook::default().hue(330.0).halo(true),
                    ),
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("YZ"),
                        AvatarLook::default().hue(45.0).halo(true),
                    ),
                    decorated_avatar(
                        kivact::state::IntoText::into_text_value("AR"),
                        AvatarLook::default().hue(160.0).halo(true),
                    ),
                ))
                .style(
                    style()
                        .gap(14)
                        .padding(20)
                        .justify_content(Justify::Center)
                        .background(color::SURFACE_ALT)
                        .border_radius(10),
                ),
                kivact::ui::checkbox("Do not show this confirmation again", remember_confirmation),
            ))
            .style(style().gap(16))
        })
        .footer(move || {
            row((
                kivact::ui::button("Cancel")
                    .variant(kivact::ui::Slant)
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .on_press(move || confirm.set(false)),
                kivact::ui::button("Deploy")
                    .variant(kivact::ui::Slant)
                    .variant(kivact_ui_pop::ButtonTone::Accent)
                    .test_id("pop-confirm-start")
                    .on_press(move || {
                        confirm.set(false);
                        notify.call((
                            ui::Hue::Blue,
                            "Mission started".into(),
                            "The squad has deployed to the outer district.".into(),
                        ));
                    }),
            ))
            .style(style().gap(8).justify_content(Justify::End))
        })
}

fn request_fields(
    request_title: Signal<String>,
    request_priority: Signal<Option<&'static str>>,
    request_date: Signal<CalendarDate>,
    request_detail: Signal<String>,
    form_dialog: Signal<bool>,
    notify: kivact::state::Callback<(ui::Hue, String, String)>,
) -> Node {
    column((
        kivact::ui::field(
            "Request title *",
            kivact::ui::text_input(request_title)
                .label("Request title")
                .on_submit(move || {
                    if !request_title.get_untracked().trim().is_empty() {
                        form_dialog.set(false);
                        notify.call((
                            ui::Hue::Mint,
                            "Request created".into(),
                            "Students will be notified.".into(),
                        ));
                    }
                })
                .placeholder("e.g. library book sorting")
                .test_id("pop-request-title"),
        ),
        row((
            kivact::ui::field(
                "Priority",
                kivact::ui::select(
                    vec![
                        ListItem::new("normal", "Normal"),
                        ListItem::new("high", "High"),
                        ListItem::new("urgent", "Urgent"),
                    ],
                    request_priority,
                )
                .label("Priority"),
            )
            .style(style().flex_grow(1)),
            kivact::ui::field(
                "Due date",
                {
                    let value: kivact::state::Signal<kivact::i18n::CalendarDate> = request_date;
                    let value = kivact::state::binding(
                        move || Some(value.get()),
                        move |day| {
                            if let Some(day) = day {
                                value.set(day);
                            }
                        },
                    );
                    kivact::ui::date_input(value)
                        .label("Request due date")
                        .presentation(kivact::ui::DatePresentation::Desktop)
                        .calendar(move |calendar| {
                            calendar
                                .today(CalendarDate::new(2026, 10, 3))
                                .month_title(|date| format!("{}.{:02}", date.year(), date.month()))
                        })
                }
                .style(style().width(180)),
            ),
        ))
        .style(style().gap(16)),
        kivact::ui::field(
            "Details",
            kivact::ui::text_area(request_detail)
                .label("Request details")
                .placeholder("Describe the request…"),
        ),
    ))
    .style(style().gap(16))
}
