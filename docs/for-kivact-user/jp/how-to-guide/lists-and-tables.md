# 一覧、選択できる一覧、表を表示する

[English](../../en/how-to-guide/lists-and-tables.md)

目的に合うビルダーを選びます。行が自分の状態を持つなら `for_each`、素の値なら `for_index`、数千行なら `virtual_list`、ユーザーが選ぶ一覧なら `list`、並べ替え・絞り込み・選択のある表なら `data_table` です。このガイドでは、それぞれの使い方と、行から一覧へ書き戻す方法を示します。

## ビルダーを選ぶ

| 手元にあるもの | 使うもの | 行の同一性 |
| --- | --- | --- |
| id を持つレコード。行が自分の状態や入力を持つ | `for_each(items, key, row)` | 返した key |
| id のない素の値(数値、文字列) | `for_index(items, row)` | 位置 |
| 高さがそろった数千行 | `virtual_list(items, key, row)` | key。見えている行だけが存在する |
| ユーザーが選ぶ一覧 | `list(name, items, selected)` | 項目の key |
| 並べ替え・絞り込み・頁分け・選択のある列 | `data_table(rows, key, columns)` | 行の key |

どれも項目として `Signal<Vec<T>>`、`Value`、memo、または `Vec<T>` を返すクロージャを受け取り、それが変わると再実行します。`T` は `Clone + PartialEq` が必要です。Kivact が古い項目と新しい項目を比較して、更新する行を判断するためです。

## 行に状態を持たせた key 付きの一覧

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

fn app() -> impl IntoView {
    let todos = signal(vec![
        Todo { id: 1, title: "牛乳を買う".into(), done: false },
        Todo { id: 2, title: "コードを書く".into(), done: false },
        Todo { id: 3, title: "散歩に行く".into(), done: false },
    ]);

    column(for_each(
        todos,
        |todo| todo.id,
        move |id, todo| {
            // チェックボックスを、一覧のこの行へ書き戻す。
            let done = todos.item(|t| t.id, id).field(|t| &t.done, |t| &mut t.done);
            // 行が持つ状態。key `id` が残っている間は生き続ける。
            let open = signal(false);
            column((
                row((
                    checkbox(move || todo.get().title, done),
                    button("詳細").on_press(move || open.update(|o| *o = !*o)),
                    button("削除")
                        .on_press(move || todos.update(|list| list.retain(|t| t.id != id))),
                ))
                .style(style().gap(8)),
                show(open, move || text(format!("Todo 番号 {id}"))),
            ))
        },
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Todos").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_state_follows_the_key_not_the_position() {
        let screen = kivact::test::render(app);
        screen.all_by_role(Role::Button, "詳細")[0].click();
        screen.get_by_text("Todo 番号 1");

        // 2 行目を削除しても、1 行目の状態は変わらない。
        screen.all_by_role(Role::Button, "削除")[1].click();
        screen.get_by_text("Todo 番号 1");
        assert!(screen.query_by_role(Role::CheckBox, "コードを書く").is_none());

        screen.get_by_role(Role::CheckBox, "牛乳を買う").click();
        assert_eq!(
            screen.get_by_role(Role::CheckBox, "牛乳を買う").checked(),
            Some(CheckedState::Checked)
        );
    }
}
```

仕組みです。

- `for_each(items, key, row)` は、各行の同一性を得るために `key(&item)` を呼びます。行は、その key が一覧にある限り、自分の owner と、行のクロージャの中で作った `signal` をすべて保ちます。並べ替えると既存の行が移動し、項目を削除するとその行だけが破棄されます。
- 行のクロージャは**行ごとに 1 回**だけ実行され、key と、行の現在の項目を常に保持する `Value<Todo>` を受け取ります。文字が編集に追従するよう、クロージャの中で読みます(`move || todo.get().title`)。一度読んだ結果を保存しないでください。
- key の型には `Clone + Eq + Hash` が必要です。位置ではなく、安定した id を使ってください。位置を使うと、順序が変わったとき行同士で状態が入れ替わってしまいます。
- 同じ key の項目が 2 つあると誤りとして報告されます。id は重複させないでください。

## 行から一覧へ書き戻す

行が持つのは項目のコピーなので、どの項目を変えるのかを示す必要があります。便利な順に 3 通りあります。

1. **フィールドへのバインディング。** `todos.item(|t| t.id, id)` は、key が `id` の項目への `Binding<Todo>` で、`.field(get, get_mut)` で 1 つのフィールドに絞れます。上のチェックボックスのように、そのバインディングを入力部品へ渡します。作るのはイベントハンドラーの中ではなく、行のクロージャのセットアップ中です。項目が消えると、バインディングは最後に見た値を保ち、書き込みは何もしません。
2. **ハンドラーの中で key による更新。** `todos.update(|list| list.retain(|t| t.id != id))` や `list.iter_mut().find(|t| t.id == id)` です。削除、移動、一覧そのものを変える操作に使います。
3. **行が持つ状態。** 行のクロージャの中で `signal(..)` を作ります。展開済みのパネルのように、一覧が知る必要のないものに使います。

ハンドラーに行の*インデックス*を取り込まないでください。削除の後では、別の項目を指してしまいます。

## 素の値: `for_index`

`for_index(items, row)` は、行を位置で識別します。位置 `i` に項目がある限り、行 `i` は自分の owner を保ち、その `Value<T>` はそこにある項目に追従します。項目に同一性がないときに使います。

```rust,no_run
use kivact::prelude::*;

fn scores() -> impl IntoView {
    let scores = signal(vec![30_u32, 10, 20]);
    column((
        column(for_index(scores, |i, score| {
            row((
                text(format!("{}位", i + 1)),
                text(move || score.get().to_string()),
            ))
            .style(style().gap(8))
        })),
        button("並べ替え").on_press(move || scores.update(|s| s.sort())),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Scores").run(scores)
}
```

「並べ替え」を押しても、3 つの行はその場に残り、数字だけが変わります。項目と一緒に動くべき状態(開いたままのエディター、フォーカスのある入力)を行が持つなら、id を使った `for_each` にします。

## 数千行: `virtual_list`

`virtual_list` はビューポート付近の行だけをマウントするので、1 万件でも 1 画面分のコストで済みます。高さに上限があることと、行の高さが分かっていることが必要です。

```rust,test_harness
use kivact::prelude::*;
use kivact::view::RowHeight;

#[derive(Clone, PartialEq)]
struct Entry {
    id: u32,
    message: String,
}

fn app() -> impl IntoView {
    let entries = signal(
        (0..10_000)
            .map(|id| Entry { id, message: format!("エントリー {id}") })
            .collect::<Vec<_>>(),
    );
    let target = signal(None::<u32>);

    column((
        button("5000 へ移動").on_press(move || target.set(Some(5_000))),
        virtual_list(entries, |e| e.id, |_, entry| text(move || entry.get().message))
            .row_height(RowHeight::Fixed(24.0))
            .scroll_to(target)
            .style(style().height(300).border(1, ColorToken::Border)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Log").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_rows_near_the_viewport_exist() {
        let screen = kivact::test::render(app);
        screen.get_by_text("エントリー 0");
        assert!(screen.query_by_text("エントリー 9999").is_none());

        screen.get_by_role(Role::Button, "5000 へ移動").click();
        screen.find_by_text("エントリー 5000");
        assert!(screen.query_by_text("エントリー 0").is_none());
    }
}
```

- `RowHeight::Fixed(px)` が高速な経路です。行の高さがまちまちなら `RowHeight::Measured { estimate }` を使うと、現れた行を計測し、まだ見ていない行には `estimate` を使います。
- `.overscan(px)` はビューポートの上下に余分にマウントするピクセル数です(既定は 200)。`.keep_focused(true)`(既定)は、フォーカスのある行がスクロールで見えなくなってもマウントしたままにします。
- `.scroll_to(signal)` は、値が `Some(key)` になるたびに、その key の行を表示範囲へスクロールします。行がまだマウントされていなくても動きます。
- 行はスクロールで見えるたびに作り直されます。行の状態は、項目の中か、一覧の外のシグナルに持たせてください。
- 高さに上限のない `virtual_list` はすべての行をマウントしてしまいます。それが 100 行以上になると Kivact は警告を報告します。高さを与えるか、高さが決まっている親に大きさを決めさせてください。

## ユーザーが選ぶ一覧

`list(name, items, selected)` は `ListItem` を listbox として表示し、選ばれた key を `Signal<Option<K>>` へ書きます。name は一覧のアクセシブルな名前で、画面には描かれません。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let selected = signal(Some(2_u32));
    let picked_many = signal(Vec::<u32>::new());

    column((
        list(
            "色",
            vec![
                ListItem::new(1_u32, "赤"),
                ListItem::new(2, "青").description("落ち着いた色"),
                ListItem::new(3, "緑").disabled(true),
            ],
            selected,
        ),
        text(move || format!("選択中: {:?}", selected.get())),
        list(
            "ファイル",
            vec![ListItem::new(1_u32, "notes.txt"), ListItem::new(2, "plan.md")],
            signal(None),
        )
        .multiple(picked_many)
        .item(|item, state| {
            text(move || {
                let mark = if state.get().selected { "[x] " } else { "[ ] " };
                format!("{mark}{}", item.get().label())
            })
        }),
        text(move || format!("{} 件のファイルを選択", picked_many.with(Vec::len))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Pick").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_selects_an_item_and_a_disabled_one_is_skipped() {
        let screen = kivact::test::render(app);
        screen.get_by_text("選択中: Some(2)");
        screen.get_by_role(Role::Option, "赤").click();
        screen.get_by_text("選択中: Some(1)");
        screen.get_by_text("0 件のファイルを選択");
    }
}
```

- 矢印キーで選択が移り、Tab は選択中の項目に入り、名前の先頭を打つとその項目へ移り、disabled の項目は飛ばされます。
- `.multiple(keys)` にすると `Signal<Vec<K>>` を使う複数選択になります。押すと 1 つを選び、Ctrl で個別に切り替え、Shift で範囲を選びます。`list` に渡した単一の `selected` バインディングは使われなくなります。
- `.item(|item, state| view)` は行の表示を置き換えます。`item` は `Value<ListItem<K>>`、`state` は `selected`、`active`、行の hover と focus の状態を持つ `Value<ListItemState>` です。振る舞いとキーボード操作は部品のままです。
- key の `K` は位置ではなく、あなたの id(`u32`、enum、`String`)です。そのため、並べ替えても選択は保たれます。
- 行の中のチェックボックス(`done` のような業務上のフラグ)は、行の選択とは別のものです。別々の場所に持たせてください。

## 並べ替え・絞り込み・頁分け・選択のある表

`data_table(rows, key, columns)` は、行、key を返す関数、`Vec<DataColumn<R>>` を受け取ります。表はデータのコピーを持ちません。選択、並べ替え、絞り込みの文字列、頁は、あなたが持つシグナルです。

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    email: String,
    amount: f64,
}

fn app() -> impl IntoView {
    let payments = signal(vec![
        Payment { id: 1, email: "ann@example.com".into(), amount: 120.0 },
        Payment { id: 2, email: "bob@example.com".into(), amount: 80.5 },
        Payment { id: 3, email: "cy@example.com".into(), amount: 310.0 },
    ]);
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    let page = signal(0_usize);

    data_table(
        payments,
        |p: &Payment| p.id,
        vec![
            DataColumn::new("メール", |p: &Payment| p.email.clone())
                .sortable()
                .filter(|p, query| p.email.contains(query)),
            DataColumn::new("金額", |p: &Payment| format!("{:.2}", p.amount))
                .compare(|a, b| a.amount.total_cmp(&b.amount))
                .numeric(true)
                .width(120.0),
        ],
    )
    .row_label(|p| p.email.clone())
    .selection(selected)
    .filter(query)
    .page(page, 10)
    .style(style().padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Payments").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_shows_its_rows_without_errors() {
        let screen = kivact::test::render(app);
        screen.get_by_text("ann@example.com");
        screen.get_by_text("310.00");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

機能は、バインディングを渡したときにだけ付きます。

| 呼び出し | 得られるもの |
| --- | --- |
| `.selection(Signal<Vec<K>>)` | 各行とヘッダーのチェックボックス、選択数の表示 |
| `.sort(Binding<Option<(usize, SortDirection)>>)` | 並べ替え中の列(位置)と向きの読み書き。渡さなくても、ヘッダーは自分で並べ替える |
| `.filter(Signal<String>)` | 絞り込みの欄。`DataColumn::filter` が一致する行だけを残す |
| `.page(Signal<usize>, size)` | `size` 行ごとの頁と、前へ・次へ |
| `.resizable(Signal<Vec<f32>>)` | ヘッダーの幅変更ハンドル。幅はあなたの値になる |
| `.row_actions(width, \|row\| view)` | ボタンやメニューを置く最後の列 |
| `.toolbar(\|ctx\| view)`、`.footer(\|ctx\| view)` | 上部または下部の領域を置き換える。`ctx` は件数のモデルとバインディングを公開する |
| `.row_details(\|row\| view)`、`.cell_controls(\|row, column\| view)` | 行の key ごとに 1 回だけ作る view。行が並べ替えや頁送りで動いても、入力は状態を保つ |

列は、比較(`.compare(..)`、またはセルの文字列を比較する `.sortable()`)を持つと並べ替えられ、`.filter(..)` を持つと絞り込みに加わります。`.cell(|row| view)` はセルの view を置き換えますが、文字列はセルのアクセシブルな名前として残ります。`.numeric(true)` は右寄せ、`.width(px)` は幅の固定で、ほかの列が残りを分け合い、`.visible(..)` は列を隠します。

既定では、キーボードのフォーカスはセル間を移動し、Space で行を選びます。`.focus(TableFocus::Rows)` にすると、行全体がフォーカスの単位になります。数万行の表では、絞り込み済みのデータに `.page(..)` を組み合わせてください。現在の頁の行はすべてマウントされるからです。

## 大きなデータ

- 項目の型は、比較が軽くなるようにします。`for_each` と `data_table` は変更のたびに項目を `PartialEq` で比較します。大きなデータは `Rc` や id の向こうに置いてください。
- `Vec` 全体を置き換えるのではなく、`update` か `item(..).field(..)` で 1 つの項目を更新します。変わった行だけが更新されます。
- 長いログやフィードには `virtual_list`、表形式のデータには `.page(..)` 付きの `data_table` を使います。
- 重い絞り込みや並べ替えは `memo` の中で行い、その memo を一覧へ渡します。

## 関連項目

- [レイアウト](layout.md): 一覧を置くスクロール領域の大きさを決める。
- [フォーム](forms.md): 行の中の入力と、下書きのパターン。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): ブロックせずに行を読み込む。
- [再利用できるコンポーネントを書く](custom-components.md): `kivact::interaction` の部品から、自分の一覧や表の見た目を作る。
- リファレンス: `List`、`DataTable`、`DataColumn` は `cargo doc -p kivact-ui --open`。
