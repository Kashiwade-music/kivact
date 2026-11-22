# データを表示する: 表、グラフ、バッジ、アバター、コード

[English](../../../en/basic/how-to-guide/data-display.md)

この手引きでは、利用者が編集せず読むための情報の見せ方を説明します。小さな静的な表、並べ替えと絞り込みのできるデータテーブル、グラフ、状態のバッジ、アバター、コードのブロックです。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

キットはデータを持ちません。コレクションは普通の `Vec`（または、その変化に表示が追従するようシグナルに入れたもの）として渡します。利用者の操作（コピーの要求、選択）は、アプリが処理するシグナルかコールバックを通って戻ります。

## 小さな静的な表を出す

`kivact::ui::table` は、手で組んだ内容のための部品です。見出し、本体、必要なら末尾と見出しの文（caption）から組み立てます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn invoices() -> impl Element {
    kivact::ui::table((
        kivact::ui::table_header((kivact::ui::table_row((
            kivact::ui::table_head("Invoice").width(110.0),
            kivact::ui::table_head("Status"),
            kivact::ui::table_head("Amount").align(kivact::style::TextAlign::End),
        )),)),
        kivact::ui::table_body((
            kivact::ui::table_row((
                kivact::ui::table_cell("INV001").width(110.0),
                kivact::ui::table_cell("Paid"),
                kivact::ui::table_cell("$250.00").align(kivact::style::TextAlign::End),
            )),
            kivact::ui::table_row((
                kivact::ui::table_cell("INV002").width(110.0),
                kivact::ui::table_cell("Pending"),
                kivact::ui::table_cell("$150.00").align(kivact::style::TextAlign::End),
            )),
        )),
        kivact::ui::table_footer((kivact::ui::table_row((
            kivact::ui::table_cell("Total").width(110.0),
            kivact::ui::table_cell(""),
            kivact::ui::table_cell("$400.00").align(kivact::style::TextAlign::End),
        )),)),
        text("A list of recent invoices.").style(
            style()
                .margin_top(16)
                .color(basic::MUTED_FOREGROUND)
                .text_align(kivact::style::TextAlign::Center),
        ),
    ))
    .style(style().width(460))
}
```

固定の幅にしたい列は、見出しとセルに同じ `.width(..)` を与えます。ほかの列は残りを分け合います。行は選択できます。`kivact::ui::table_row(..).selected(move || chosen.get() == index).on_press(move || chosen.set(index))` のように書きます。

## 並べ替え・絞り込み・選択・ページ送りのある多数の行を出す

実際のデータには、標準の `kivact::ui::data_table(rows, key, columns)` を使います。テーマは Lyra の見た目で描きます。各 `DataColumn` はタイトルと、セルのテキストを作る関数を持ちます。任意の修飾子で、並べ替え（`.compare(..)`）、絞り込み（`.filter(..)`）、右寄せ（`.numeric(true)`）を足せます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    email: &'static str,
    amount: f64,
}

#[component]
fn payments() -> impl Element {
    let data = vec![
        Payment {
            id: 1,
            email: "ken99@example.com",
            amount: 316.0,
        },
        Payment {
            id: 2,
            email: "abe45@example.com",
            amount: 242.0,
        },
        Payment {
            id: 3,
            email: "silas22@example.com",
            amount: 874.0,
        },
    ];
    let query = signal(String::new());
    let selected = signal(Vec::<u32>::new());
    let sort = signal(None);
    let page = signal(0_usize);
    let columns = vec![
        kivact::ui::DataColumn::new("Email", |p: &Payment| p.email.to_string())
            .compare(|a, b| a.email.cmp(b.email))
            .filter(|p: &Payment, q: &str| p.email.contains(q)),
        kivact::ui::DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
            .compare(|a, b| a.amount.total_cmp(&b.amount))
            .numeric(true),
    ];
    column((
        text_input(query).label("Filter emails").placeholder("Filter emails..."),
        kivact::ui::data_table(data, |p: &Payment| p.id, columns)
            .row_label(|p: &Payment| p.email.to_string())
            .filter(query)
            .selection(selected)
            .sort(sort)
            .page(page, 5),
        basic::muted(move || format!("{} selected", selected.get().len())),
    ))
    .style(style().width(560).gap(8))
}
```

- 行の型には `Clone + PartialEq` が必要です。テーブルは、変わった部分だけを更新するために行を比べます。キーの関数（ここでは `p.id`）は、各行に選択とフォーカスのための安定した同一性を与えます。選択はキーを保持し、行の番号は保持しません。
- `selection`、`sort`、`filter`、`page` は、どれもアプリが持つ任意のシグナルです。利用者が見出し、チェックボックス、ページのボタンをクリックすると、テーブルがそれらに書き込みます。アプリからも書き込めます（たとえば選択を消すなど）。`sort` は `Option<(列の番号, 向き)>` を持ちます。
- `.row_label(..)` は各行のアクセシブルな名前を与えます。スクリーンリーダーは、そのチェックボックスについてこの名前を読み上げます。
- `.toolbar(|context| view)` はテーブルの上の領域を置き換えます。shadcn のデータテーブルのように、絞り込みの欄と「列」のメニューを並べられます。`.row_actions(width, |row| view)` は末尾にコントロールの列（行ごとの「...」メニュー）を足します。カタログのデータテーブルのページで、両方の例を見られます。
- テーブル自身の文言（`Filter...`、`No results found.`、`Page 1 of 3` など）は `data_table.*` のメッセージから来ており、置き換えられます。[テーマと変数](theme-and-tokens.md#フォントを選ぶ)を参照してください。

## グラフを描く

`basic::chart(config, rows)` は、棒、積み上げ棒、折れ線、面、円、ドーナツ、レーダー、放射のグラフを描きます。`ChartConfig` は系列の名前を持ち、各 `ChartRow` は見出しと、系列ごとに 1 つの値を持ちます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn visitors() -> impl Element {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
    ];
    kivact::ui::card(
        basic::chart(config, data)
            .kind(basic::ChartKind::Line)
            .height(220.0),
    )
    .header(
        column((
            text("Bar chart").style(style().font_size(14).line_height(20.0 / 14.0).font_weight(500)),
            text("January - March").style(
                style()
                    .line_height(1.625)
                    .color(basic::MUTED_FOREGROUND),
            ),
        ))
        .style(style().gap(4).flex_grow(1).min_width(0)),
    )
    .style(style().width(560))
}
```

- `.kind(..)` は `Bar`（既定）、`StackedBar`、`Line`、`Area`、`Pie`、`Donut`、`Radar`、`Radial` のどれかです。円、ドーナツ、放射のグラフでは、各系列は全行の値の合計で大きさが決まる扇形や輪になります。レーダーのグラフは、行ごとに 1 本の軸を持ちます。
- 系列は、テーマの 5 つのグラフの色を順に使うので、テーマを切り替えると色も変わります。1 つを固定するには `ChartSeries::new(..).color(rgb(0x2563eb))` を指定します。
- `.orientation(kivact::view::Orientation::Horizontal)` は棒を横に並べます。`.value_format(|v| format!("{v:.0}k"))` は、軸と吹き出しで値の読み方を変えます。既定では、値はアプリのロケールに従い、桁区切りと最大 3 桁の小数で表示されます（en-US なら `1,234.5`、de-DE なら `1.234,5`）。`.legend(false)`、`.grid(false)`、`.axis(false)`、`.tooltip(false)` は部品を消し、`.height(..)` は高さを決めます。
- 行のシグナル（`signal(Vec<ChartRow>)`）を `data` に渡すと、グラフがその場で更新されます。
- ホバーすると、値を示す吹き出しが出ます。グラフにはアクセシブルな要約もあり、その文言は `basic.chart.*` のメッセージから来ます。

## バッジで状態を、アバターで人を示す

バッジは小さなラベルです。アバターは、画像があればそれを、なければ代わりの文字を表示します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn people() -> impl Element {
    column((
        row((
            kivact::ui::badge("Active"),
            kivact::ui::badge("Draft").variant(basic::BadgeVariant::Secondary),
            kivact::ui::badge("Failed").variant(basic::BadgeVariant::Destructive),
            kivact::ui::badge("Verified")
                .variant(basic::BadgeVariant::Outline)
                .icon(icon("badge-check").style(style().size(12, 12))),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        kivact::ui::avatar_group((
            kivact::ui::avatar("AL").fallback_content(|| text("AL")),
            kivact::ui::avatar("GH").fallback_content(|| text("GH")),
            kivact::ui::avatar("AT")
                .fallback_content(|| text("AT"))
                .size(kivact::ui::ControlSize::Large),
        )).max(3usize).total(6usize),
    ))
    .style(style().gap(12))
}
```

- バッジの variant は `Default`、`Secondary`、`Destructive`、`Outline`、`Ghost`、`Link` です。`.on_press(..)` を渡すと、バッジはボタンになります。
- `kivact::ui::avatar(name).src(Some(asset))` は、リアクティブな画像を読みます。代わりの文字を正確に指定するには `.fallback_content(|| text("CN"))` を使います。指定がなければ、部品が名前からイニシャルを作ります。サイズは `ControlSize::Small`、`Medium`、`Large` です。`.status(Some(Tone::Success))` は意味を表す角の点を描き、`.status_content(view)` は独自の角のビューを受け取ります。`.name_tooltip(true)` は、名前を吹き出しで表示します。Basic のテーマは、頼まれない限りこれを出しません。
- `kivact::ui::avatar_group(children)` は、アバターを重ねて並べます。`.max(limit)` は表示する子の数を制限し、`.total(count)` は、まだ読み込まれていない参加者を含めます。はみ出しの円は、実際の残りの数を示します。

## コピーのボタン付きのコードを出す

`kivact::ui::code_block(source)` が、選択、スクロール、クリップボードのフィードバックを持ちます。Rust のトークンは `kivact::ui::rust_tokens(source)` で一度だけ作り、`basic::rust_lines` と組み合わせて `kivact::ui::highlighter(tokens, ..)` に渡します。ただのコードなら、ハイライターは要りません。`.on_copy(..)` は、中核がクリップボードに書いたあとのコピーを観察します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn snippet() -> impl Element {
    let source = "fn main() {\n    println!(\"Hello, Kivact!\");\n}";
    let tokens = kivact::ui::rust_tokens(source);
    kivact::ui::code_block(source.to_string())
        .highlight(kivact::ui::highlighter(tokens, |tokens, text, index: &usize| {
            basic::rust_lines(tokens, text, index)
        }))
        .title("main.rs")
        .line_numbers(true)
        .style(style().width(pct(100)))
}
```

- `.max_height(..)` は長いブロックをスクロールさせます。`.variant(basic::CodeBlockSize::Sm)` は小さいサイズ、`.footer(..)` はブロックの下に 1 行を足します。
- ほかの言語では、`basic::CodeLine` と `basic::CodeSpan` で行を手で組みます。span の `kind` は `kivact::ui::SyntaxKind` か、普通の文字なら `None` です。色はテーマの構文の変数（`kivact::ui::theme::syntax`）です。
- コードのフォントは `Geist Mono` です。[テーマと変数](theme-and-tokens.md#フォントを選ぶ)の説明に従って登録してください。登録しないときは、システムの等幅フォントが使われます。
- コピーに成功すると、ボタンは少しのあいだ「Copied」と表示します。その語は `code_block.*` のメッセージから来ます。

## 会話を出す

項目ごとに `kivact::ui::chat_message(author)` を 1 つずつ使い、必要に応じて `.content(..)`、`.avatar(..)`、`.footer(..)`、`.align(ChatAlign::End)` を付けます。`basic::ChatBubbleVariant` は Basic の色の扱いを選びます。単独の吹き出しは `.variant(basic::ChatLayout::Bubble)` と空の著者を使い、普通の行は既定の Message のレイアウトを使います。リアクションは標準の `.reactions(..)` と `.reactions_side(..)` のスロットを使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn conversation() -> impl Element {
    kivact::ui::chat_thread(column((
        kivact::ui::chat_message("Ada")
            .content("Lunch at noon?")
            .variant(basic::ChatBubbleVariant::Secondary),
        kivact::ui::chat_message("You")
            .content("Sounds good.")
            .align(kivact::ui::ChatAlign::End),
    )).style(style().gap(6)))
    .style(style().height(240).width(360))
}
```

`chat_thread` は、意味を持つログ、スクロール、最新へ移動する操作を提供します。高さに上限を与えてください。中核は、読者がスクロールして離れるまで新しい内容に追従し、読者が末尾に着くか移動の操作を使うと追従を再開します。

## 活動の詳細を出す

活動の行には普通の行を使います。展開できる詳細には、`kivact::ui::disclosure(label, open).variant(basic::DisclosureVariant::Trace).content(..).attach(trigger)` を使います。呼出し側は `trigger` にアイコン、タイトル、所要時間を渡します。同じ `open` のバインディングで、シェブロンを回転させられます。中核が操作、キーボード入力、展開の意味を持ち、Trace の variant は控えめな詳細と縦の線を描きます。

## 関連

- [フォームを作る](forms.md): 入力、ピッカー、検証。
- [テーマと変数](theme-and-tokens.md): グラフとコードの色はテーマの変数に従います。
- リファレンス: グラフ、表、コードブロックの型は `cargo doc -p kivact-ui-basic --open`、`data_table` は `cargo doc -p kivact-ui --open`。

`kivact::ui::item_list(children)` は、標準の `item_row(title)` の行をまとめます。`.variant(..)` で `basic::ItemVariant` と `basic::ItemMediaVariant` を設定します。`basic::ItemSize::Sm` は、40 px ではなく 32 px の画像を保ちます。メディア、操作、見出し、末尾は普通のビューのスロットのままです。

標準のカードは `kivact::ui::card(content)` を使い、`.header(..)`、`.media(..)`、`.footer(..)` に普通のビューを置きます。`basic::CardSize::Sm` は見た目の variant です。見出しの下に罫線を引くには `.variant(basic::CardHeaderBordered(true))` を使います。見出しのタイトル、説明、操作は、呼出し側が組み立てるテキストと行・列のビューです。

見出しには `kivact::ui::heading(text, level)` を使い、テーマは元の Basic の文字組みを保ちます。進捗は `.variant(..)` で `ProgressVariant::Bar`、`Ring`、`Gauge` に対応し、3 つとも Basic の primary と muted のパレットを使います。大きさを一定の比率にするには、呼出し側が持つノードのスタイルに `.aspect_ratio(width / height)` を置きます。統計と説明のリストは、標準の `stat` と `description_list` の部品を使います。
