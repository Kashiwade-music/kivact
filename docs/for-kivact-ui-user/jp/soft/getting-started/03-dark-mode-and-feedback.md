# ダークモードとフィードバック

[English](../../../en/soft/getting-started/03-dark-mode-and-feedback.md)

表示モードと保存結果は別々のシグナルに保持します。設定画面を `provide_theme` で包むと、表示モードを切り替えたときに既存のコントロールにも新しい色が反映されます。保存の成功後にバナーを表示します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact::style::{provide_theme};
use kivact::view::{show};
use kivact_ui_soft as soft;

fn settings_feedback() -> impl IntoView {
    let dark = signal(false);
    let saved = signal(false);
    provide_theme(
        move || soft::theme(if dark.get() { ThemeMode::Dark } else { ThemeMode::Light }),
        move || {
            column((
                button("Toggle dark mode").on_press(move || dark.update(|on| *on = !*on)),
                button("Save").on_press(move || saved.set(true)),
                show(saved, move || {
                    kivact::ui::alert("Settings saved")
                        .tone(kivact::interaction::Tone::Success)
                        .icon(soft::banner_icon(kivact::interaction::Tone::Success))
                        .description("Your changes are ready to use.")
                        .dismissible(true)
                        .on_dismiss(move || saved.set(false))
                }),
            ))
            .style(|| style().gap(kivact::style::use_tokens::<soft::Tokens>().spacing.s3))
        },
    )
}
```

この例では保存が成功したものとして扱います。アプリでは書き込みの完了を受けて `saved` を更新します。失敗時は、該当する入力欄の近くかエラーバナーに理由を表示します。
