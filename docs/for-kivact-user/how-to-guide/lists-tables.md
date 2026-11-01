# 一覧と表を使う

選べる一覧は `kivact::ui::list`、並べ替え・絞り込み・選択・頁分けのある表は `kivact::ui::data_table` で書きます。どちらも標準の部品なので、kit の theme を入れると kit の見た目になります。選択・並べ替え・絞り込み・頁・列の幅はアプリの signal が正本で、部品は渡された状態だけを持ちます。行の値そのものも、アプリの model に置きます。見た目や操作の違う部品を作る場合だけ、振る舞いの層（`interaction::use_list`・`use_table`）を使います。それは [部品作者の手引き](../../for-component-author/interaction.md) にあります。

## 選べる一覧

`list(label, items, selected)` は `ListItem` の一覧を `Role::ListBox` として描き、選んだ項目の key を `selected` へ書きます。`label` は一覧の名前で、画面には描きません。矢印キーで選択が移り、Tab は選択中の項目に入り、文字を打つとその名前の項目へ移ります。使えない項目（`disabled(true)`）は飛ばします。

```rust,no_run
use kivact::prelude::*;

fn colors() -> impl IntoView {
    let selected = signal(Some(2_u32));
    list(
        "色",
        vec![
            ListItem::new(1_u32, "赤"),
            ListItem::new(2, "青").description("落ち着いた色"),
            ListItem::new(3, "緑").disabled(true),
        ],
        selected,
    )
}
```

`.multiple(keys)` にすると複数選べます（Ctrl で個別に、Shift で範囲を選びます）。選択は `keys` だけが正本になり、`list` に渡した単一の binding は使いません。`.on_edit(..)` は選択の変更を一回の編集として知らせます。項目の中身を替えるときは `.item(|item, state| view)` に項目の値（`Value<ListItem<K>>`）と状態（選択中か、など）を受ける view を返します。

```rust,no_run
use kivact::prelude::*;

fn files() -> impl IntoView {
    let picked = signal(Vec::<u32>::new());
    list(
        "ファイル",
        vec![ListItem::new(1_u32, "notes.txt"), ListItem::new(2, "plan.md")],
        signal(None),
    )
    .multiple(picked)
    .item(|item, state| {
        text(move || {
            let mark = if state.get().selected { "● " } else { "○ " };
            format!("{mark}{}", item.get().label())
        })
    })
}
```

見た目は theme の `ListRecipes`（一覧・項目・説明）が決めます。呼び側の `.style(..)` は recipe より後に重なり、常に勝ちます。

## 表

`data_table(rows, key, columns)` は、行（`impl IntoValue<Vec<R>>`）、行の key を返す関数、列（`Vec<DataColumn<R>>`）を受けます。列は `DataColumn::new(title, |row| text)` で作ります。`text` はセルの表示と支援技術の名前です。

| 列の指定 | 働き |
|---|---|
| `.compare(比較する関数)` / `.sortable()` | header が昇順、続けて降順に並べ替える。`sortable()` は text の順（数値は数として） |
| `.filter(照合する関数)` | 表の絞り込みの文字列に合う行を残す。どの列も合わなければ行は消える |
| `.numeric(true)` | header とセルを末尾へ揃える |
| `.width(px)` | 幅（logical pixel）。渡さない列は残りの幅を分け合う |
| `.cell(行から view を作る関数)` | セルの表示だけを view に替える。text は名前として残る |
| `.visible(bool か signal)` | 列を隠す |

並べ替えられるのは `compare` を持つ列だけです。何も渡さない列は並べ替えません。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Payment { id: u32, email: String, amount: f64 }

fn payments(data: Signal<Vec<Payment>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    let query = signal(String::new());
    let page = signal(0_usize);
    data_table(
        data,
        |p: &Payment| p.id,
        vec![
            DataColumn::new("メール", |p: &Payment| p.email.clone())
                .sortable()
                .filter(|p, query| p.email.contains(query)),
            DataColumn::new("金額", |p: &Payment| format!("{:.2}", p.amount))
                .compare(|a, b| a.amount.total_cmp(&b.amount))
                .numeric(true)
                .width(100.0),
        ],
    )
    .row_label(|p| p.email.clone())
    .selection(selected)
    .filter(query)
    .page(page, 10)
}
```

状態は、渡した機能だけが付きます。

| 状態 | 付くもの |
|---|---|
| `.selection(Binding<Vec<K>>)` | 行と header の check box と、「n of m row(s) selected.」。header の check box は頁の行だけを選び、別の頁や絞り込みで隠れた行の選択は残る |
| `.sort(Binding<Option<(usize, SortDirection)>>)` | 並べ替えている列（`columns` の位置）と向き。渡さなくても header は表の中の状態で並べ替える。値を書けば表も追う |
| `.filter(Binding<String>)` | 絞り込みの欄。文字を打つと最初の頁へ戻る |
| `.page(Binding<usize>, size)` | 頁と Previous / Next。頁が最後を越えると最後の頁を見せ、binding は書き換えない |
| `.resizable(Binding<Vec<f32>>)` | header の resize の取っ手。幅は列の位置ごとに書く（Escape で drag を戻す） |
| `.row_label(行の名前を返す関数)` | 行・check box・typeahead の名前（渡さなければ最初の列の text） |
| `.row_actions(幅, 行から view を作る関数)` | 最後の、題のない操作の列。Tab で届く通常の部品 |
| `.on_reorder((key, 行き先) を受ける関数)` | 行の drag。並べ替えを頼むだけで、適用か拒否かはアプリが決める。頼むと sort を消す |

キーボードは、既定では cell が roving focus を持ち、矢印・Home・End で動き、Space で行を選びます。`.focus(TableFocus::Rows)` にすると行がフォーカスの単位になり、check box を出さずに行の押下・Ctrl・Shift で選びます。`.policy(..)` で範囲選択と typeahead の policy を渡せます。

表の上と下は、既定では絞り込みの欄と、選択数・頁送りです。`.toolbar(|context| view)` と `.footer(|context| view)` は、その領域を丸ごと自分の view に替えます。`context` は表の今の状態です（`model` の件数・頁数・絞り込み後の key、`selection`・`sort`・`filter`・`page` の binding）。列の表示の切り替えや、選択した行への一括操作はここで作ります。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Task { id: u32, title: String }

fn tasks(data: Signal<Vec<Task>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    data_table(
        data,
        |t: &Task| t.id,
        vec![DataColumn::new("題名", |t: &Task| t.title.clone()).sortable()],
    )
    .selection(selected)
    .toolbar(move |context| {
        let selection = context.selection.expect("selection is on");
        button("選択を解除").on_press(move || selection.set(Vec::new()))
    })
    .footer(|context| {
        let count = context.model.row_count();
        text(move || format!("{} 件", count.get()))
    })
}
```

`.row_details(|row| view)` は各行の下に、`.cell_controls(|row, column| view)` は各セルに、行の値（`Value<R>`）から作る view を置きます。これらは行の key ごとに一度だけ作り、行の編集・並べ替え・絞り込み・頁送りで作り直しません。入力欄やスイッチのように、行を編集しても状態を保つ部品はここに置きます（列の `.cell(..)` は行が替わるたびに作り直します）。`.row_height(..)` と `.row_color(..)` は行の高さと左端の色を recipe へ渡します。

見た目は theme の `DataTableRecipes`（枠・上・下・header・行・cell・check box の箱・並べ替えの印・頁送り）が決めます。呼び側の `.style(..)` は recipe より後に重なり、常に勝ちます。文言は message で替えます（`data_table.label`・`data_table.filter`・`data_table.empty`・`data_table.actions`・`data_table.select_all`・`data_table.select_named`・`data_table.selected`・`data_table.previous`・`data_table.next`・`data_table.page`・`data_table.resize`）。

Todo の `done` のような業務 field は、行選択とは別物です。行から元の一覧へ書き戻すには、一覧の signal から key で一行を指す binding を作り、field へ投影します。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo { id: u32, title: String, done: bool }

fn todo_rows(todos: Signal<Vec<Todo>>) -> impl IntoView {
    column(for_each(todos, |todo| todo.id, move |id, todo| {
        let done = todos.item(|t| t.id, id).field(|t| &t.done, |t| &mut t.done);
        checkbox(move || todo.get().title, done)
    }))
}
```

`Signal<Vec<T>>::item(key, id)` は key が `id` の行を読み書きする `Binding<T>` で、`Binding::field` はその field への `Binding<U>` です。行が消えた後の読みは最後に見えた値を返し、書込みは何もしません。どちらも setup で作ります（行の factory は行の setup で実行されます）。
