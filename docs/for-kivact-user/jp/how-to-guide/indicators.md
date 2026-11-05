# インジケーターとキーボードショートカット

標準のインジケーターはすべての kit theme で使えます。recipe は共通の意味を受け取り、kit の variant が外観を指定します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact::ui::{self, ControlSize, SkeletonShape};
use kivact::view::Modifiers;

fn indicators() -> impl IntoView {
    column((
        ui::badge("Ready").tone(Tone::Success),
        ui::avatar("Ada Lovelace").size(ControlSize::Small)
            .status(Some(Tone::Success)),
        ui::avatar_group((ui::avatar("Ada"), ui::avatar("Grace"))).max(1),
        ui::skeleton().lines(3).shape(SkeletonShape::Text),
        ui::spinner().size(ControlSize::Small),
        ui::kbd("mod+K"),
        ui::kbd_combo(vec![Key::Char('K')])
            .modifiers(move || Modifiers { meta: true, ..Modifiers::default() }),
    ))
}
```

`Avatar::src` と `fallback_src` は reactive な optional asset を受け取ります。読み込み中や失敗時は頭文字を表示します。画像 node が読み込み状態を所有し、source の変更で再試行します。`shape` は Circle、Rounded、Square です。`status` は在席の意味と支援技術向けの名前を指定し、`status_content` は独自の重ねる表示を指定します。ボタンには `on_press`、リンクの操作には `on_open` を使います。tooltip は `ui::tooltip(name)` を attach します。group の `max` は reactive で、矢印の移動は共通の roving focus が所有します。

skeleton は装飾です。reduced motion では時計を一時停止して静止します。spinner は `spinner.loading` の message で名前を付けた status で、Animated 一つで動きます。`label_content` が表示内容を追加し、`.name(...)` が支援技術向けの名前を変えます。`async_view` の既定の loading も同じ spinner を使います。

`kbd` は文字列を受け取り `mod` を解釈します。`kbd_combo` は reactive な論理 key の列を受け取り、`.modifiers(...)` で chord の修飾キーを追加します。`meta` は `Capabilities.command_key` に従い、host が Command と Ctrl を選びます。物理 Ctrl は常に Ctrl です。コンパイル対象 OS による分岐はありません。

`UiTokens::indicators` で時間と placeholder の寸法を設定します。`UiRecipes` には `badge`、`avatar`、`avatar_status`、`avatar_group`、`skeleton`、`spinner`、`kbd` があり、対応する `_recipe()` getter で既存 recipe を包めます。利用者の style が既定値を上書きします。

## kit の option の移行

| 既存の option | 標準の組み立て |
| --- | --- |
| Basic badge の variant/icon/trailing_icon/on_press | `badge.variant/icon/trailing_icon/on_press` |
| Pop badge hue、Dot、Count、Slant、Game Solid/Outline/Dot、Soft の意味・装飾色 | kit の `Variant` と badge recipe。意味の色は `tone` |
| Basic avatar image、badge、group count | `src`、`status_content`、`avatar_group.max` または独自の子 |
| Pop avatar hue/halo/online、Game color/online | kit variant と recipe、在席は `status` |
| 各 kit の avatar size、任意の px | `ControlSize`、利用者の size style、theme recipe |
| Soft avatar fallback photo、shape、alt、tooltip、press/open、独自 status | `fallback_src`、`shape`、`.name`、tooltip の attach、`on_press/on_open`、`status_content` |
| Soft group 共通 size/shape/label と overflow label | 子の設定、group の `.name`、`max` または独自 overflow の子 |
| Basic/Pop/Game skeleton rows | `lines` |
| Soft skeleton radius/index | `shape`、recipe variant、`index` による開始のずれ |
| Pop dots loader、Soft spinner shade/ink | kit spinner variant と recipe |
| Soft spinner name、text label、任意 label view | `.name`、`label_content(text(...))`、`label_content` |
| Basic kbd group、Soft の解析する shortcut | `kbd` の row または型付き `kbd_combo` |

Creative の `KeyGlyph` は editor の keyframe marker を描く domain 型で、キーボードショートカットとは別のものとして残ります。

`UiTokens::indicators.reduced_period_ms` は、動きを減らす設定でも spinner を必要な表示として遅い周期で回すオプションです。`None` は装飾的な動きを止めます。周期を指定した場合は設定の変更に応じて glyph のアニメーション owner を置き換え、status のノードとラベルは保ちます。

`kbd("mod+shift+p")` と型付きの `kbd_combo` は、どちらもショートカットを個別の key cap に正規化します。`mod` は `Capabilities.command_key` に従います。読み上げる名前には翻訳済みの正式なキー名を使い、cap にはキーの glyph を表示します。command/cmd、control/ctrl、return/enter、esc/escape、矢印、プラス記号の cap 用の `plus` に対応します。テーマは枠のない行を `kbd`、各 `KbdKeyState` を `kbd_key` の scalar recipe で装飾します。メッセージカタログの `kbd.key.*` と `kbd.separator` で翻訳できます。
