//! Role: Catalog page of Card: login form, small, bordered header and image.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "card",
        name: "Card",
        description: "見出し、内容、フッターを持つ枠付きの表示領域です。header、image、content、footer の各スロットに渡します。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("card.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "card_header に見出しと説明、右上の操作を渡し、content にフォーム、footer にボタンを置いたログインの例です。",
                default,
            ),
            Example::new(
                "small",
                "Small",
                "size に Sm を渡すと、余白が 12 px になります。",
                small,
            ),
            Example::new(
                "bordered",
                "Bordered header",
                "bordered で見出しの下に線を引きます。",
                bordered,
            ),
            Example::new(
                "image",
                "Image",
                "image は上端いっぱいに表示され、上の余白がなくなります。",
                with_image,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let email = signal(String::new());
    let password = signal(String::new());
    basic::card()
        .header(
            basic::card_header("Login to your account")
                .description("Enter your email below to login to your account")
                .action(button("Sign Up").variant(ButtonVariant::Link)),
        )
        .content({
            basic::field_group((
                kivact::ui::field(
                    "Email",
                    text_input("Email", email).placeholder("m@example.com"),
                ),
                kivact::ui::field("Password", text_input("Password", password).password(true)),
            ))
        })
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
    basic::card()
        .size(basic::CardSize::Sm)
        .header(basic::card_header("Small card").description("Tighter spacing."))
        .content(text("Content"))
        .style(style().width(360))
}
// end example

// example: bordered
fn bordered() -> impl IntoView {
    basic::card()
        .header(basic::card_header("Notifications").bordered(true))
        .content(text("You have 3 unread messages."))
        .style(style().width(360))
}
// end example

// example: image
fn with_image() -> impl IntoView {
    let landscape: &'static [u8] = include_bytes!("../../assets/images/landscape.png");
    basic::card()
        .image(image(landscape).style(style().width(pct(100)).aspect_ratio(16.0 / 9.0)))
        .header(basic::card_header("Design systems meetup").description("A practical talk."))
        .footer(button("View Event").style(style().width(pct(100))))
        .style(style().width(360))
}
// end example
