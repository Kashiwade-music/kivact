# テーブルで行を表示する

[English](../../../en/pop/how-to-guide/tables.md)

レコードの一覧を、検索・並べ替え・選択・ページ送りの付いたテーブルとして Pop の見た目で表示するための手引きです。テーブルは標準の `kivact::ui::data_table` です。Pop テーマは、角丸の面、高さ 40 px の見出し、高さ 52 px の行、幅 160 px の列として描き、収まらないときは横にスクロールします。キットが独自のテーブル型を足すことはありません。

## 行を表示する

行はアプリのシグナルに置いたままにします。テーブルに渡すのは、行、各行の一意なキーを返す関数、列の 3 つです。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: &'static str,
    status: &'static str,
    total: f64,
}

fn orders() -> impl IntoView {
    let rows = signal(vec![
        Order { id: 1, customer: "Haruka Sato", status: "Paid", total: 120.0 },
        Order { id: 2, customer: "Ren Mori", status: "Pending", total: 48.5 },
        Order { id: 3, customer: "Aoi Ito", status: "Paid", total: 310.0 },
    ]);
    data_table(
        rows,
        |order: &Order| order.id,
        vec![
            DataColumn::new("Customer", |order: &Order| order.customer.to_owned()).sortable(),
            DataColumn::new("Status", |order: &Order| order.status.to_owned()).cell(
                |order: &Order| {
                    let hue = if order.status == "Paid" {
                        pop::Hue::Mint
                    } else {
                        pop::Hue::Yellow
                    };
                    pop::badge(order.status).hue(hue)
                },
            ),
            DataColumn::new("Total", |order: &Order| format!("${:.2}", order.total))
                .compare(|a, b| a.total.total_cmp(&b.total))
                .numeric(true),
        ],
    )
    .style(style().margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Orders")
        .theme(pop::theme(ThemeMode::Light))
        .run(orders)
}
```

実行して見出しをクリックすると、行が昇順、次に降順で並び替わります。見出しが並べ替えられるのは、列に `.sortable()`（テキストで並べ替え、両方が数値なら数値として比較）か `.compare(..)`（自分で順序を決める）があるときです。どちらもない列は、シグナルの順序のままです。

**Rust メモ。** 行の型には `Clone + PartialEq`、キーの型には `Clone + Eq + Hash` が必要です。`u32` の id や `String` のスラッグが使えます。`|order: &Order|` と行の型を書いているのは、クロージャの型を推論させるためです。

`.cell(|row| view)` はセルの中身を置き換えます。ここではバッジです。列のテキストは、スクリーンリーダーやフィルタのために、セルの名前として使われ続けます。

## 検索・選択・ページ送りを足す

追加機能は、保持させたい状態をテーブルに渡すと有効になります。その状態はアプリのものなので、どこからでも読み書きできます。

```rust,no_run
use kivact::prelude::*;
use kivact::view::SortDirection;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: String,
    total: f64,
}

fn orders(rows: Signal<Vec<Order>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    let page = signal(0_usize);
    let sort = signal(Some((1_usize, SortDirection::Descending)));
    column((
        data_table(
            rows,
            |order: &Order| order.id,
            vec![
                DataColumn::new("Customer", |order: &Order| order.customer.clone())
                    .sortable()
                    .filter(|order, query| {
                        order.customer.to_lowercase().contains(&query.to_lowercase())
                    }),
                DataColumn::new("Total", |order: &Order| format!("${:.2}", order.total))
                    .compare(|a, b| a.total.total_cmp(&b.total))
                    .numeric(true),
            ],
        )
        .selection(selected)
        .filter(query)
        .page(page, 10)
        .sort(sort)
        .row_label(|order| format!("Order {}", order.id)),
        text(move || format!("{} selected", selected.with(Vec::len))),
    ))
    .style(style().gap(12).margin(24))
}
# fn main() {}
```

- `.selection(signal)` は、各行にチェックボックスを、見出しに「このページを選択」を追加し、チェックされた行のキーを保持します。ページ単位でチェックしても、ほかのページで選択済みの行には影響しません。テーブルの下に "2 of 3 row(s) selected." のような要約が出ます。
- `.filter(signal)` は、テーブルの上に絞り込みの欄を出します。いずれかの列の `.filter(|row, query| bool)` が一致すれば、その行が残ります。`.filter` のない列は検索の対象にならず、キーが検索されることもありません。
- `.page(signal, size)` は、1 ページに `size` 行を表示し、Previous と Next を出します。ページ番号は 0 から始まります。並べ替えや絞り込みのテキストが変わると先頭ページに戻り、最後を超えたページ番号は、シグナルを書き換えずに最後のページを表示します。
- `.sort(signal)` は任意です。最初から並べ替えておきたいときや、並べ替えの状態を読みたいときに使います。`Some((列の番号, 向き))` を持ち、番号は 0 から数えます。
- `.row_label(..)` は、スクリーンリーダー向けに各行とそのチェックボックスに名前を付けます。指定しなければ、最初の列のテキストが使われます。

## 検索の横にボタンを置く

`.toolbar(|context| view)` はテーブルの上の領域を置き換えます。通常は絞り込み欄だけが入っています。`context` にはテーブルの現在の状態が入っています。選択と絞り込みのバインディング、件数を持つ `context.model` です。絞り込み欄を残したい場合は、自分で描く必要があります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_pop as pop;

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: String,
}

fn orders(rows: Signal<Vec<Order>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    data_table(
        rows,
        |order: &Order| order.id,
        vec![DataColumn::new("Customer", |order: &Order| order.customer.clone()).sortable()],
    )
    .selection(selected)
    .filter(query)
    .toolbar(move |_context| {
        row((
            text_input("Filter orders", query)
                .placeholder("Search customers")
                .style(style().grow(1)),
            button("Delete selected")
                .variant(pop::ButtonTone::Soft)
                .disabled(move || selected.with(Vec::is_empty))
                .on_press(move || {
                    let doomed = selected.get();
                    rows.update(|rows| rows.retain(|order| !doomed.contains(&order.id)));
                    selected.set(Vec::new());
                }),
        ))
        .style(style().gap(8))
    })
}
# fn main() {}
```

行の削除はアプリのコードです。テーブルが行を書き換えることはありません。`.footer(|context| view)` は同じ方法で、テーブルの下の領域（選択の要約とページャ）を置き換えます。

## 行ごとのコントロール

- `.row_actions(width, |row| view)` は、メニューボタンのようなコントロールを並べた最後の列を足します。見出しにタイトルはなく、"Actions" という名前が付きます。
- `.resizable(signal_of_widths)` は、見出しにリサイズのハンドルを付け、幅（列ごとの `Vec<f32>`）をシグナルに保持します。
- `.row_details(|row| view)` は各行の下にビューを置き、`.cell_controls(..)` はセルの中にコントロールを置きます。どちらも、並べ替えやページ送りのあいだ配置されたままなので、入力途中の状態が保たれます。
- `.row_color(|row| Option<Color>)` は行に色を付けます。期限切れの行などに使えます。

## 文言

テーブル自身の文言（絞り込み欄の "Filter…"、"No results found."、"Previous"、"Next"、"Page {page} of {pages}"）は英語です。`App::messages` で差し替えます。キーは `data_table.filter`、`data_table.empty`、`data_table.previous`、`data_table.next`、`data_table.page` などで、`kivact::ui::data_table` の rustdoc に一覧があります。

## 関連

- [業務画面で数値・状態・レコードを見せる](status-and-figures.md): 行数の少ない一覧向け。
- [Pop のテーマとトークンを設定する](theme-and-tokens.md)
- テーブル、列、コンテキストの API: `cargo doc -p kivact --open` の `kivact::ui::DataTable`、`DataColumn`、`DataTableContext`
