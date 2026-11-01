//! Role: Validate a sign-up form with memos and show messages with show_some.
#![warn(redundant_imports)]

use kivact::prelude::*;

fn app() -> impl IntoView {
    let email = signal(String::new());
    let password = signal(String::new());
    let signed_up = signal(None);

    let email_error = memo(move || {
        (!email.get().contains('@')).then(|| "Enter an email address with @".to_owned())
    });
    let password_error = memo(move || {
        (password.with(|p| p.chars().count()) < 8).then(|| "Use at least 8 characters".to_owned())
    });
    let valid = memo(move || email_error.get().is_none() && password_error.get().is_none());

    column((
        text_input("Email", email).style(style().width(280)),
        show_some(email_error, text),
        text_input("Password", password)
            .password(true)
            .style(style().width(280)),
        show_some(password_error, text),
        button("Sign up")
            .disabled(move || !valid.get())
            .on_press(move || signed_up.set(Some(email.get()))),
        show_some(signed_up, |email| {
            text(move || format!("Signed up as {}", email.get()))
        }),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact · Form").run(app)
}

#[cfg(test)]
mod tests {
    //! Role: Drive the sign-up form example in a test window.

    use super::*;

    #[test]
    fn messages_follow_the_input_and_submit_enables_when_valid() {
        let screen = kivact::test::render(app);
        let submit = || screen.get_by_role(Role::Button, "Sign up");
        screen.get_by_text("Enter an email address with @");
        screen.get_by_text("Use at least 8 characters");
        assert!(submit().is_disabled());

        screen
            .get_by_role(Role::TextBox, "Email")
            .type_text("ann@example.com");
        assert!(
            screen
                .query_by_text("Enter an email address with @")
                .is_none()
        );
        screen.get_by_text("Use at least 8 characters");
        assert!(submit().is_disabled());

        screen
            .get_by_role(Role::TextBox, "Password")
            .type_text("short");
        screen.get_by_text("Use at least 8 characters");
        screen
            .get_by_role(Role::TextBox, "Password")
            .type_text("-enough");
        assert!(screen.query_by_text("Use at least 8 characters").is_none());
        assert!(!submit().is_disabled());

        submit().click();
        screen.get_by_text("Signed up as ann@example.com");
    }
}
