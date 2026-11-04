# データを表示する: 表、グラフ、バッジ、アバター、コード

[English](../../../en/basic/how-to-guide/data-display.md)

このガイドでは、ユーザーが編集せずに読む情報の見せ方を説明します。小さな静的な表、並べ替えと絞り込みのできるデータテーブル、グラフ、状態を表すバッジ、アバター、コードのブロックです。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

キットがあなたのデータを所有することはありません。コレクションはふつうの `Vec`（またはその signal。変更に view が追従します）で渡し、ユーザーの操作（コピーの要求や選択）は、signal かコールバックを通じてアプリに返ってきます。

## 小さな静的な表を見せる

`basic::table` は、手で配置する内容のための表です。ヘッダー、ボディ、必要ならフッターとキャプションから組み立てます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn invoices() -> impl IntoView {
    basic::table((
        basic::table_header(basic::table_row((
            basic::table_head("Invoice").width(110.0),
            basic::table_head("Status"),
            basic::table_head("Amount").align(basic::TableAlign::End),
        ))),
        basic::table_body((
            basic::table_row((
                basic::table_cell("INV001").width(110.0),
                basic::table_cell("Paid"),
                basic::table_cell("$250.00").align(basic::TableAlign::End),
            )),
            basic::table_row((
                basic::table_cell("INV002").width(110.0),
                basic::table_cell("Pending"),
                basic::table_cell("$150.00").align(basic::TableAlign::End),
            )),
        )),
        basic::table_footer(basic::table_row((
            basic::table_cell("Total").width(110.0),
            basic::table_cell(""),
            basic::table_cell("$400.00").align(basic::TableAlign::End),
        ))),
        basic::table_caption("A list of recent invoices."),
    ))
    .style(style().width(460))
}
```

幅を固定したい列は、ヘッダーのセルと本体のセルに同じ `.width(..)` を付けます。ほかの列が残りを分け合います。行は選択可能にもできます。`basic::table_row(..).selected(move || chosen.get() == index).on_press(move || chosen.set(index))` です。

## 多くの行を、並べ替え、絞り込み、選択、ページ分けつきで見せる

実データには、標準の `data_table(rows, key, columns)` を使います。テーマが Lyra の見た目で描きます。各 `DataColumn` は、タイトルとセルの文字を作る関数を持ち、修飾子で並べ替え（`.compare(..)`）、絞り込み（`.filter(..)`）、右寄せ（`.numeric(true)`）を足せます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    email: &'static str,
    amount: f64,
}

fn payments() -> impl IntoView {
    let data = vec![
        Payment { id: 1, email: "ken99@example.com", amount: 316.0 },
        Payment { id: 2, email: "abe45@example.com", amount: 242.0 },
        Payment { id: 3, email: "silas22@example.com", amount: 874.0 },
    ];
    let query = signal(String::new());
    let selected = signal(Vec::<u32>::new());
    let sort = signal(None);
    let page = signal(0_usize);
    let columns = vec![
        DataColumn::new("Email", |p: &Payment| p.email.to_string())
            .compare(|a, b| a.email.cmp(b.email))
            .filter(|p: &Payment, q: &str| p.email.contains(q)),
        DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
            .compare(|a, b| a.amount.total_cmp(&b.amount))
            .numeric(true),
    ];
    column((
        text_input("Filter emails", query).placeholder("Filter emails..."),
        data_table(data, |p: &Payment| p.id, columns)
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

- 行の型は `Clone + PartialEq` が必要です。表は行を比較して、変わったところだけを更新するためです。key 関数（ここでは `p.id`）は、選択やフォーカスのために各行へ安定した同一性を与えます。選択は行番号ではなく key を持ちます。
- `selection`、`sort`、`filter`、`page` は、どれもアプリが持つ任意の signal です。ユーザーがヘッダー、チェックボックス、ページのボタンを操作すると表がそれらを書き、アプリからも書けます（たとえば選択をクリアするとき）。`sort` は `Option<(列の番号, 方向)>` を持ちます。
- `.row_label(..)` は、各行の accessible name を与えます。スクリーンリーダーが行のチェックボックスについて読み上げる名前です。
- `.toolbar(|context| view)` は、表の上の領域を置き換えます。shadcn のデータテーブルのように、絞り込みの欄と「Columns」メニューを並べるときに使います。`.row_actions(width, |row| view)` は、行ごとの操作（「...」メニューなど）の列を右端に足します。カタログの Data Table のページに、どちらの例もあります。
- 表自身の文言（`Filter...`、`No results found.`、`Page 1 of 3` など）は `data_table.*` のメッセージから来ており、置き換えられます。[テーマとトークン](theme-and-tokens.md)を参照してください。

## グラフを描く

`basic::chart(config, rows)` は、棒、積み上げ棒、折れ線、面、円、ドーナツ、レーダー、ラジアルのグラフを描きます。`ChartConfig` は系列に名前を付け、各 `ChartRow` はラベルと、系列ごとの値を持ちます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn visitors() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
    ];
    basic::card()
        .header(basic::card_header("Bar chart").description("January - March"))
        .content(basic::chart(config, data).kind(basic::ChartKind::Line).height(220.0))
        .style(style().width(560))
}
```

- `.kind(..)` は `Bar`（既定）、`StackedBar`、`Line`、`Area`、`Pie`、`Donut`、`Radar`、`Radial` のどれかです。円、ドーナツ、ラジアルでは、各系列がすべての行にわたる値の合計に応じた扇形やリングになります。レーダーは、行ごとに 1 本の軸を持ちます。
- 系列にはテーマの 5 色のグラフ色が順に使われるので、テーマを切り替えると色も変わります。1 つを固定するには `ChartSeries::new(..).color(rgb(0x2563eb))` を使います。
- `.orientation(Orientation::Horizontal)` は棒を横向きにします。`.value_format(|v| format!("{v:.0}k"))` は、軸とツールチップでの値の書き方を変えます。`.legend(false)`、`.grid(false)`、`.axis(false)`、`.tooltip(false)` は部品を消し、`.height(..)` は高さを決めます。
- `data` に行の signal（`signal(Vec<ChartRow>)`）を渡すと、グラフがその場で更新されます。
- ホバーすると、値を載せたツールチップが出ます。グラフにはアクセシビリティ用の要約もあり、文言は `basic.chart.*` のメッセージから来ます。

## バッジで状態を、アバターで人を示す

バッジは小さなラベルです。アバターは画像を表示し、画像がないときは代わりの文字を表示します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn people() -> impl IntoView {
    column((
        row((
            basic::badge("Active"),
            basic::badge("Draft").variant(basic::BadgeVariant::Secondary),
            basic::badge("Failed").variant(basic::BadgeVariant::Destructive),
            basic::badge("Verified")
                .variant(basic::BadgeVariant::Outline)
                .icon(icon("badge-check")),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        basic::avatar_group((
            basic::avatar("AL"),
            basic::avatar("GH"),
            basic::avatar("AT").size(basic::AvatarSize::Lg),
            basic::avatar_group_count("+3"),
        )),
    ))
    .style(style().gap(12))
}
```

- バッジのバリアントは `Default`、`Secondary`、`Destructive`、`Outline`、`Ghost`、`Link` です。`.on_press(..)` を付けると、バッジはボタンになります。
- `basic::avatar(fallback).image(bytes)` は、`include_bytes!` の `&'static [u8]` のような画像のバイト列を受け取ります。代わりの文字は、画像が使えるようになるまで、また画像がない間表示されます。サイズは `Sm`、既定、`Lg` です。`.badge(|| icon("plus"))` は、角に小さな印を足します。
- `avatar_group` はアバターを重ねて並べます。`avatar_group_count("+3")` は、末尾の「ほかにも」を示す丸です。

## コピーボタンつきでコードを見せる

`basic::code_block(lines)` は、構文を色分けした行を等幅フォントで描きます。`basic::rust_lines(source)` は Rust のソースを色分けし、`basic::code_lines(text)` は色なしの文字を表示します。コピーボタンはクリップボードを直接は触りません。`.on_copy(..)` を呼ぶので、何をするかはアプリが決めます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn snippet() -> impl IntoView {
    let source = "fn main() {\n    println!(\"Hello, Kivact!\");\n}";
    basic::code_block(basic::rust_lines(source))
        .title("main.rs")
        .line_numbers(true)
        .on_copy(move || {
            spawn(kivact::platform::copy_text(source));
        })
        .style(style().width(pct(100)))
}
```

- `.max_height(..)` は長いブロックをスクロールにし、`.size(basic::CodeBlockSize::Sm)` はコンパクトなサイズ、`.footer(|| ..)` はブロックの下に 1 行足します。
- ほかの言語は、行を自分で組み立てます。`CodeLine` は `CodeSpan::new(text, CodeTone::Keyword)` などの並びです（トーンは `Plain`、`Keyword`、`String`、`Number`、`Comment`、`Function`、`Type`、`Macro`、`Lifetime`、`Punctuation`、`Attribute`）。色はテーマのトークンの `code` フィールドです。
- コードのフォントは `Geist Mono` です。[テーマとトークン](theme-and-tokens.md)の説明に従って登録してください。登録しないときは、システムの等幅フォントが使われます。
- コピーに成功すると、ボタンが少しの間「Copied」になります。この文言は `basic.code_block.*` のメッセージから来ます。

## 関連項目

- [フォーム](forms.md): 入力欄、ピッカー、検証。
- [テーマとトークン](theme-and-tokens.md): グラフとコードの色はテーマのトークンに従います。
- rustdoc: グラフ、表、コードブロックの型は `cargo doc -p kivact-ui-basic --open`、`data_table` は `cargo doc -p kivact-ui --open`。
