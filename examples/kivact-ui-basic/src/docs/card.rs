//! Catalog page of Card: login form, small, bordered header and image.

use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page::new("card", "Card", "Components", include_str!("card.rs"))
        .description("A bordered area with a header, content and a footer. Pass them to the header, image, content and footer slots.")
        .usage("use kivact::prelude::*;\nuse kivact_ui_basic as basic;")
        .example(Example::new(
            "default",
            "Default",
            "card_header takes the title, a description and an action at the top right; the content holds a form and the footer holds the buttons. This is a login example.",
            default,
        ))
        .example(Example::new(
            "small",
            "Small",
            "Pass Sm as size to make the padding 12 px.",
            small,
        ))
        .example(Example::new(
            "bordered",
            "Bordered header",
            "bordered draws a line under the header.",
            bordered,
        ))
        .example(Example::new(
            "image",
            "Image",
            "The image fills the top edge, so the top padding disappears.",
            with_image,
        ))
}

// example: default
fn default() -> impl IntoView {
    let email = signal(String::new());
    let password = signal(String::new());
    kivact::ui::card({
        basic::field_group((
            kivact::ui::field(
                "Email",
                text_input(email)
                    .label("Email")
                    .placeholder("m@example.com"),
            ),
            kivact::ui::field(
                "Password",
                text_input(password)
                    .label("Password")
                    .purpose(kivact::view::InputPurpose::Password),
            ),
        ))
    })
    .header(
        basic::card_header("Login to your account")
            .description("Enter your email below to login to your account")
            .action(button("Sign Up").variant(ButtonVariant::Link)),
    )
    .footer({
        column((
            button("Login").style(style().width(pct(100))),
            button("Login with Google")
                .variant(ButtonVariant::Outline)
                .style(style().width(pct(100))),
        ))
        .style(style().gap(8).width(pct(100)))
    })
    .style(style().width(360))
}
// end example

// example: small
fn small() -> impl IntoView {
    kivact::ui::card(text("Content"))
        .variant(basic::CardSize::Sm)
        .header(basic::card_header("Small card").description("Tighter spacing."))
        .style(style().width(360))
}
// end example

// example: bordered
fn bordered() -> impl IntoView {
    kivact::ui::card(text("You have 3 unread messages."))
        .variant(basic::CardHeaderBordered(true))
        .header(basic::card_header("Notifications"))
        .style(style().width(360))
}
// end example

// example: image
fn with_image() -> impl IntoView {
    let landscape: &'static [u8] = include_bytes!("../../assets/images/landscape.png");
    kivact::ui::card(())
        .media(image(landscape).style(style().width(pct(100)).aspect_ratio(16.0 / 9.0)))
        .header(basic::card_header("Design systems meetup").description("A practical talk."))
        .footer(button("View Event").style(style().width(pct(100))))
        .style(style().width(360))
}
// end example
