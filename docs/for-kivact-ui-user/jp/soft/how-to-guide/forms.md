# フォーム

[English](../../../en/soft/how-to-guide/forms.md)

文字列には Kivact 標準のテキスト入力を、日付には Soft の型付き日付入力を使います。各フィールドを個別のシグナルにバインドします。保存時は両方の値を読み取り、一つのレコードとして検証できます。

```rust,no_run
use kivact::i18n::CalendarDate;
use kivact::prelude::*;
use kivact_ui_soft as soft;

fn event_form(save: impl Fn(String, CalendarDate) + 'static) -> impl IntoView {
    let name = signal(String::new());
    let date = signal(None::<CalendarDate>);

    column((
        text_input("Event name", name),
        soft::date_input("Event date", date),
        button("Save event")
            .variant(ButtonVariant::Primary)
            .disabled(move || name.get().trim().is_empty() || date.get().is_none())
            .on_press(move || {
                if let Some(date) = date.get() {
                    save(name.get().trim().to_owned(), date);
                }
            }),
    ))
    .style(|| style().gap(soft::tokens().spacing.s3))
}
```

入力エラーは該当フィールドの近くに表示し、修正方法を伝えます。永続化の境界でも送信値を検証します。
