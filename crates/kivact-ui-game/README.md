# kivact-ui-game

密度の高い運用コンソール向け公式 UI 部品。依存は `kivact` の公開 facade のみ。

```rust
use kivact::prelude::*;
use kivact::ui::{button, checkbox};
use kivact::style::ThemeMode;
use kivact_ui_game::{theme, Density, Slant, ThemeOptions as _};

fn screen() -> impl IntoView {
    let approved = signal(false);
    column((
        checkbox("承認済み", approved),
        button("承認する").variant(Slant).on_press(move || approved.set(true)),
    )).style(style().gap(12).padding(20))
}

fn main() -> kivact::Result<()> {
    App::new("Operations")
        .theme(theme(ThemeMode::Light).density(Density::Compact))
        .run(screen)
}
```

`kivact::ui::Button`、`kivact::ui::Checkbox`、`kivact::ui::DataTable` は `NodeBuilder` を実装し、その標準modifierで拡張できる。標準のボタン・チェック・スイッチ・入力・選択・メニュー・ダイアログ・ツールチップ・タブ・ラジオ・スライダー・スプリッター・トーストは `kivact::ui` から使い、Game テーマがその見た目を設定する（ラジオは菱形、スライダーは平行四辺形のつまみ、トーストは紺のカード）。タブのアイコンは `ListItem::icon`、件数などは `Tabs::item` の slot で描き、ボックス型のタブは `tabs(..).variant(Boxed)` で表す。単一選択の見た目も kit の部品ではなく標準部品の variant で、`radio_group(..).variant(Segmented)`（斜めの segment）、`radio_group(..).variant(ChoiceCard).item(card_label)`（プランの card）、`tabs(..).orientation(Orientation::Vertical).variant(Navigation).item(navigation_label)`（縦のナビゲーション）と書く。説明とアイコンは `ListItem`、価格や件数は slot で `(card_label(item, state), 価格の view)` のように並べ、`radio_group` の `.on_edit(..)` が選択ごとに一つの編集を報告する。segmented は tabs ではなくラジオグループである。トーストは右からすべり込み、消えるときは右へ戻る（theme の toast recipe の `enter` と `exit`）。`command_palette` は core の `kivact::interaction::CommandPalette` に Game の見た目（部位を使う描画）を付けたもので、command は core の `Command<K>`、開く shortcut はアプリが持つ。`tree(entries, selection)` は core の `kivact::interaction::Tree` に file tree の見た目を付けたもので、tree 全体が tab stop 一つ（最後に focus した行）、Up/Down/Home/End で表示中の行を移り、Right/Left でフォルダを開閉して子・親へ移る（ARIA の tree view）。フォルダは focus できるが `selection` に書くのは葉だけで、chevron・Enter・ダブルクリックでもフォルダを開閉する。テーマの追加値は公開 `Tokens`、状態は利用者の `Binding` / `Signal` をコンストラクタ引数で渡す。

`native_window(title, explorer_width, explorer, document)` は内容を持たない枠で、explorer と document の間は標準の `splitter`、`explorer_width` はその幅の binding（範囲の既定は 140..420 px、`.explorer_range(range)` で変える）。メニュー・ツールバー・ステータスは `.menu_bar(|| ..)`・`.toolbar(|| ..)`・`.status_bar(|| ..)` でアプリが渡す。指定しなければ描かない。inspector や編集・プレビューの切替もアプリが document の中に組み立てる。

`kpi(..)` は `.heading(|| view)` で見出しの view に差し替え、`.trend(samples)` で読取専用の傾向データ、`.format(|value| ..)` で数値の書式を受け取る。既定では sparkline を描かず、数値は `decimals` 桁で区切りなしに表示する。`dropzone(label).on_files(on_files)` の説明は `.hint(..)`、native picker の filter は `.filters(Vec<(String, Vec<String>)>)` で渡す。どちらも既定は空。通知は標準の `kivact::ui::toast_region(toaster())` に `Toast` を push する。region はアプリ全体の設定を変更しない。表示上限はアプリの setup で `kivact::ui::toaster().max_visible(..)` を設定する。

標準の `kivact::ui::progress(Some(0.0 から 1.0))` は確定値の bar、`None` は不定の bar で、色は `.variant(kivact::ui::Tone::Success)`、細さは `.size(ControlSize::Small)`、動く縞は `.variant(g::Striped).moving(true)` で表します。開閉する見出しは標準の `kivact::ui::disclosure` です。slot は view を作る closure で受けます。`banner(title, tone, action)` の message は `.content(|| ..)`（文字列か任意の view）、field は標準の `kivact::ui::field(label, control).hint(text).error(value)` を使い、豊かな説明と caption は呼出元で合成します。`alert(title).description(text).tone(tone)` の操作は `.actions(|| view)` です。specimen・swatch・shape_tile・feature_card・native_window は catalog の非公開 module にあります。`activity(..)` は `ActivityEntry::new(time, message, tone)` の一覧を受け取り、message は文字列でも任意の view でもかまいません。`property_list(entries)` の値も文字列か view です。

テーマは `theme(mode)` で作り、密度などの選択肢は `ThemeOptions` の method（`theme(mode).density(Density::Compact)`）で足す。結果は普通の `kivact::style::Theme` なので、`App::theme`・`provide_theme`・signal にそのまま渡せる。icon は `kivact::control::icon("name")` で描き、kit の画像は theme の `IconSet` に入っている。

比較用カタログは `cargo run -p kivact-ui-game-catalog --release`。

標準の `kivact::ui::tag_input(label, tags, on_add, on_remove)` は項目列を読取専用で受け、`on_add` で追加する文字列、`on_remove` で削除する index を報告する。アプリが要求を適用または拒否する。数値は標準の `kivact::ui::number_input(label, value, range)`、command palette は標準の `kivact::ui::command_palette(open, commands, run)` で、Game の theme が描く。

`tree(entries, selection)` の項目列は反応する `impl IntoValue<Vec<T>>` で受ける。表は標準の `kivact::ui::data_table(rows, key, columns)` で、Game の theme（`DataTableRecipes`）がコンソールの表に描く。列は `kivact::ui::DataColumn::new(title, |row| text)` で、`.width(..)` が最小の幅（既定は 160 px。列は表の余った幅を分け合う）、`.numeric(true)` が末尾揃え、`.cell(|row| view)` が表示だけを `TableCell` の見た目などに替える。選択した行への一括操作と選択数の bar は表の `.toolbar(|context| view)`、表示件数などは `.footer(|context| view)` で作る。

`TreeEntry.folder` は子の有無とは独立した指定である。空の folder も folder のアイコンと identity を保ち、葉として選択しない。現在の core Tree は子のある行だけに展開の意味情報と操作を付けるため、空 folder の支援技術による展開はできない。子を追加すれば開閉できる。

`alert(..)` の閉じる button は `.on_dismiss(..)` がある場合だけ表示する。callback は要求を報告し、表示状態はアプリが `show` と自身の状態で管理する。

`kpi(label, target)` の任意値は `.decimals(0)`・`.unit(text)`・`.change(text)`・`.tone(Tone::Neutral)` です。`avatar(initials)` の既定は 36 px・theme accent・online なし、`gauge(label, value)` の total は 100、tone は Neutral です。`avatar_stack` と file filter は反応する一覧を受けます。activity と property_list は view を一度だけ消費する構成です。Game の segmented は clip_path の平行四辺形を使い、既定の radio label と icon は直立のまま描きます。
