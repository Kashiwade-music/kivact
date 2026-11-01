# Game でタブ付きパネルを作る

Game の theme は、`kivact::ui` の標準部品（tabs、radio_group、slider、splitter、toast_region など）に操作コンソールの見た目を与えます。タブは標準の `tabs` を使い、アイコンは `ListItem::icon`、件数のような追加の表示は `.item(..)` の slot で描きます。選択は安定した key の binding に置きます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn inventory() -> impl IntoView {
    let selected = signal("items");
    let count = signal(3);
    tabs(
        "インベントリ",
        vec![
            ListItem::new("items", "所持品").icon("box"),
            ListItem::new("quests", "依頼").icon("target"),
        ],
        selected,
    )
    .item(move |item, _state| {
        row((
            text(move || item.get().label()),
            show(
                move || *item.get().key() == "items",
                move || game::badge(move || count.get().to_string(), kivact::ui::Tone::Info),
            ),
        ))
        .style(style().gap(7))
    })
    .content(|key| text(format!("表示中: {key}")))
}

fn main() -> kivact::Result<()> {
    App::new("操作パネル")
        .theme(game::theme(kivact::style::ThemeMode::Light))
        .run(inventory)
}
```

テーマは `game::theme(mode)` で作ります。詰めた寸法にするときは `game::ThemeOptions` を import して `game::theme(mode).density(game::Density::Compact)` と書きます。結果は普通の `Theme` なので、`App::theme` にも `provide_theme` にもそのまま渡せます。

`.content(..)` は選択中の panel だけを mount します。panel を再作成しても残すデータは外側の model に置きます。ボックス型のタブは `tabs(..).variant(game::Boxed)` で表します。

選択の見た目は、標準の `radio_group` と `tabs` に Game の型を `.variant(..)` で渡して選びます。型を渡さない radio の組は、Game の theme で菱形の印になります。

- 単なる排他的な値の選択は `radio_group(..).variant(game::Segmented)` で、斜めの segment を横に並べます（`.orientation(Orientation::Horizontal)` も付けます）。tabs ではなく radio の組なので、矢印キーで移動と選択を行います。
- プランの選択は `radio_group(..).variant(game::ChoiceCard)` の card で、`.item(game::card_label)` が icon・label・説明を描きます。価格や件数は slot で `(game::card_label(item, state), 価格の view)` のように並べます。
- 縦の画面切替は `tabs(..).orientation(Orientation::Vertical).variant(game::Navigation)` で、slot の `(game::navigation_label(item, state), 件数の view)` で件数や badge を後ろに置きます。上下キーで移動し、端では反対側へ回ります。

説明とアイコンは `ListItem` の `description` と `icon` で渡し、選択ごとの編集の報告は `radio_group` の `.on_edit(..)` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn period() -> impl IntoView {
    let period = signal(Some("7d"));
    radio_group(
        "期間",
        vec![ListItem::new("24h", "24H"), ListItem::new("7d", "7D")],
        period,
    )
    .orientation(Orientation::Horizontal)
    .variant(game::Segmented)
}
```

`cargo run -p kivact-ui-game-catalog` でタブ、tree、command palette を確認できます。command の実行と開く shortcut はアプリが持ち、palette 内の検索とキー操作は共通の振る舞いに任せます。

## 項目列と表示状態を分ける

標準の `kivact::ui::tag_input(label, tags, on_add, on_remove)`、`tree(entries, selection)`、標準の `data_table(rows, key, columns)` は項目列を `impl IntoValue<Vec<T>>` で読取専用に受けます。タグの `on_add` は追加する文字列、`on_remove` は削除する index を報告します。アプリが要求を適用または拒否し、反映した一覧が表示されます。[Pop のタグの例](pop.md#項目列の変更をアプリで反映する)は Game の theme でもそのまま使えます。数値の入力は標準の `kivact::ui::number_input(label, value, range)` で、command palette は標準の `kivact::ui::command_palette(open, commands, run)` です（Game の theme で描かれます）。

`kivact::ui::data_table` は game の theme でコンソールの表（mono の文字、34 px の header、160 px を最小にして余った幅を分け合う列、accent の縁の選択行）に描かれます。行はアプリの型 `R`、key は `K` のままで、列は `kivact::ui::DataColumn::new(title, |row| text)` です。`.cell(|row| view)` で `TableCell` の見た目や任意の view に替えます。使い方は [一覧と表](../../for-kivact-user/how-to-guide/lists-tables.md) にあります。選択した行への一括操作と選択数の bar は `.toolbar(|context| view)`（`context.selection`）、表示件数の選択は `.footer(|context| view)` で作ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

#[derive(Clone, PartialEq)]
struct Order { id: u32, title: String, progress: f64 }

fn orders(rows: Signal<Vec<Order>>, query: Signal<String>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    data_table(rows, |order: &Order| order.id, vec![
        DataColumn::new("件名", |order: &Order| order.title.clone())
            .width(220.)
            .compare(|a: &Order, b: &Order| a.title.cmp(&b.title))
            .filter(|order: &Order, query: &str| order.title.contains(query)),
        DataColumn::new("進捗", |order: &Order| format!("{:.0}%", order.progress))
            .width(144.)
            .numeric(true)
            .cell(|order: &Order| game::TableCell::Progress {
                value: order.progress,
                tone: kivact::ui::Tone::Info,
            })
            .compare(|a: &Order, b: &Order| a.progress.total_cmp(&b.progress)),
    ])
    .selection(selected)
    .filter(query)
    .row_label(|order: &Order| order.title.clone())
}
```

`TreeEntry.folder` は子の有無と独立した folder の指定です。空でも folder のアイコンと identity を保ち、葉として選択しません。現在の core Tree は子を持つ行だけに展開の意味情報と操作を付けるため、空 folder に展開の支援技術操作はありません。項目列を更新して子を追加すると開閉できます。

`alert(..)` は `.on_dismiss(..)` がある場合だけ閉じる button を描きます。callback は要求の通知です。アプリが表示状態を変え、`show` で表示を管理します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn notice() -> impl IntoView {
    let visible = signal(true);
    show(visible, move || {
        game::alert("確認").description("処理が完了しました").tone(kivact::ui::Tone::Success)
            .on_dismiss(move || visible.set(false))
    })
}
```

## 枠とアプリの内容を組み合わせる

`native_window(title, explorer_width, explorer, document)` は窓の枠で、explorer と document の間は標準の `splitter` です。`explorer_width` は explorer の幅（px）の binding で、範囲の既定は 140..420 px、`.explorer_range(range)` で変えます。`.menu_bar(|| view)`・`.toolbar(|| view)`・`.status_bar(|| view)` は指定した内容だけを表示します。メニューの操作、保存、状態の文言、編集とプレビューの切替はアプリで組みます。

`kpi(..)` は `.heading(|| view)` で見出しの view に差し替えられます。`.trend(samples)` へ `IntoValue<Vec<f64>>` を渡します。空なら sparkline は描きません。`.format(|value| ..)` で数値の書式を指定し、省略時は `decimals` 桁の区切りなし表記になります。

標準の `kivact::ui::progress(Some(0.0 から 1.0))` は確定値の bar、`None` は不定の bar で、色は `.variant(kivact::ui::Tone::Success)`、細さは `.size(ControlSize::Small)`、動く縞は `.variant(g::Striped).moving(true)` で表します。開閉する見出しは標準の `kivact::ui::disclosure` です。部品の slot（`.content(..)` など、部品の中の決まった場所に置く内容）は、view を作る closure で渡します。`banner(title, tone, action)` の message は `.content(|| ..)` で文字列か任意の view を渡します。field は標準の `kivact::ui::field(label, control).hint(text).error(value)` を使います。caption や豊かな説明は呼出元の `row`・`column` で合成し、複数の control は `node().child((..))` で包みます。`alert(title).description(text).tone(tone)` の操作は `.actions(|| view)` です。specimen・swatch・shape_tile・feature_card・native_window は catalog の非公開 module にあります。`activity(..)` は `ActivityEntry::new(time, message, tone)` の一覧を受け取り、message は文字列でも任意の view でもかまいません。`property_list(entries)` の値は文字列でも view でもかまいません（混ぜるときは `.into_view()`）。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn dashboard() -> impl IntoView {
    let width = signal(240.0);
    let samples = signal(vec![12.0, 15.0, 14.0]);
    column((
        text("集計"),
        kivact::ui::splitter("項目一覧の幅", width,
            kivact::ui::ScalarRange::new(160.0, 480.0).unwrap())
            .start(text("項目一覧"))
            .end(game::kpi("件数", 14.0).decimals(0).unit("件")
                .change("前回比 -1").tone(kivact::ui::Tone::Info)
                .trend(samples).format(|value| format!("{value:.0}"))),
        text("更新済み"),
    ))
}
```

`dropzone(label).on_files(on_files)` の `.hint(..)` と `.filters(impl IntoValue<Vec<(String, Vec<String>)>>)` は既定で空です。受け取ったファイルの種類と大きさの検証はアプリが行います。通知は標準の `kivact::ui::toast_region(toaster())` を一度 mount し、`Toast::new(..).body(..).tone(..)` を push します。region は `Toaster` の設定を変えないので、表示上限はアプリの setup で設定します。

`kpi(label, target)` の任意値は `.decimals(0)`・`.unit(text)`・`.change(text)`・`.tone(Tone::Neutral)` です。`avatar(initials)` は既定で 36 px・theme accent・online なし、`gauge(label, value)` は total が 100、tone が Neutral です。`skeleton().rows(value)` と `breadcrumb(items)` は反応する入力を読みます。activity と property_list の view は一度だけ消費します。segmented の斜めの外形は clip_path で描き、既定の label と icon は直立します。
