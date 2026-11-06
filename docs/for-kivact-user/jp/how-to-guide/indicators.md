# インジケーターとキーボードショートカット

[English](../../en/how-to-guide/indicators.md)

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

`Avatar::src` と `fallback_src` は reactive な optional asset を受け取ります。読み込み中や失敗時は頭文字を表示します。画像 node が読み込み状態を所有し、source の変更で再試行します。`shape` は Circle、Rounded、Square です。`status` は在席の意味と支援技術向けの名前を指定し、`status_content` は独自の重ねる表示を指定します。ボタンには `on_press`、リンクの操作には `on_open` を使います。`name_tooltip(show)` は avatar の名前を tooltip に表示するかどうかを指定します。指定しなければ theme が決め（`AvatarTokens::name_tooltip`、既定は表示しない）、別の文字には `ui::tooltip(text)` を attach します。group の `max` は reactive で、矢印の移動は共通の roving focus が所有します。

skeleton は装飾です。reduced motion では時計を一時停止して静止します。spinner は `spinner.loading` の message で名前を付けた status で、Animated 一つで動きます。`label_content` が表示内容を追加し、`.name(...)` が支援技術向けの名前を変えます。`async_view` の既定の loading も同じ spinner を使います。

`kbd` は文字列を受け取り `mod` を解釈します。`kbd_combo` は reactive な論理 key の列を受け取り、`.modifiers(...)` で chord の修飾キーを追加します。`meta` は `Capabilities.command_key` に従い、host が Command と Ctrl を選びます。物理 Ctrl は常に Ctrl です。コンパイル対象 OS による分岐はありません。

`UiTokens::indicators` で時間と placeholder の寸法を設定します。`UiRecipes` には `badge`、`avatar`、`avatar_status`、`avatar_group`、`skeleton`、`spinner`、`kbd` があり、対応する `_recipe()` getter で既存 recipe を包めます。利用者の style が既定値を上書きします。

## キットの見た目を選ぶ

標準部品へ kit の `.variant(...)` を渡すと、操作や読み上げ名を保って外観を変えられます。badge の意味上の色は `.tone(...)`、装飾的な配色は kit の variant を使います。avatar の画像、形、status、独自の重ねる内容は標準の入力です。サイズ、影、枠などの kit 固有の選択肢は [各 kit のガイド](../../../for-kivact-ui-user/jp/README.md) を参照してください。

`UiTokens::indicators.reduced_period_ms` は、動きを減らす設定でも spinner を必要な表示として遅い周期で回すオプションです。`None` は装飾的な動きを止めます。周期を指定した場合は設定の変更に応じて glyph のアニメーション owner を置き換え、status のノードとラベルは保ちます。

`kbd("mod+shift+p")` と型付きの `kbd_combo` は、どちらもショートカットを個別の key cap に正規化します。`mod` は `Capabilities.command_key` に従います。読み上げる名前には翻訳済みの正式なキー名を使い、cap にはキーの glyph を表示します。command/cmd、control/ctrl、return/enter、esc/escape、矢印、プラス記号の cap 用の `plus` に対応します。テーマは枠のない行を `kbd`、各 `KbdKeyState` を `kbd_key` の scalar recipe で装飾します。メッセージカタログの `kbd.key.*` と `kbd.separator` で翻訳できます。

`UiTokens::indicators.skeleton_transition` は skeleton の基本 tween を上書きします。コンポーネントが初期遅延、index の時間差、交互の反復を加え、一つの `Animated` と動きを減らす場合の一時停止を保ちます。

`avatar_group(children).overlap(pixels)` は既定の重なりトークンをリアクティブに上書きし、読み方向に従います。`.total(total)` は一部だけ読み込んだグループの総数を表します。`.overflow_content(|count| view)` は実際の省略人数を受け取り、仮のメンバーなしに全一覧を開くラベル付きボタンなどを構成できます。Avatar のイニシャルは先頭と末尾の意味のある名前グリフを使用し、結合絵文字と結合記号も保持します。

`avatar_status_content` レシピは実際の独自状態コンテンツの枠を Avatar の下・末尾側に配置し、同じ `AvatarState` を受け取ります。枠は状態やラベルの複製を追加せず、内容自体の意味を保持します。既定の省略人数の画像ラベルは `avatar_group.overflow`（`{count} more`）で翻訳できます。
