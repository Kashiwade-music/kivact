# 5. 条件分岐と一覧

文字だけが変わるなら `text(move || ...)` で十分です。部品の有無が変わるときは `show`、項目が増減するなら `for_each` を使います。

いくつかの状態で表示を切り替えるなら `dynamic(source, |value| match value { .. })` の中で Rust の `match` を書きます。arm がすべて `text`・`node`・`column` などの要素関数なら型は `Node` で揃います。`button` などの部品が混ざる arm では、各 arm の末尾に `.into_view()` を付けて `View` に揃えます。理由は [React・SolidJSからKivactへ](react-solid.md#分岐の-arm-の型) にあります。`.into_view()` が要るのはこの `dynamic` の arm だけです。

## 条件付きで部品を置く

```rust,test_harness
use kivact::prelude::*;

fn details() -> impl IntoView {
    let open = signal(false);
    column((
        button("切り替え").on_press(move || open.update(|v| *v = !*v)),
        show(open, || text("詳細を表示中"))
            .fallback(|| text("詳細は閉じています")),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("条件分岐の例").run(details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_switches_the_branch() {
        let screen = kivact::test::render(details);
        screen.get_by_text("詳細は閉じています");
        screen.get_by_role(Role::Button, "切り替え").click();
        screen.get_by_text("詳細を表示中");
    }
}
```

`show` の第 1 引数が `true` なら第 2 引数の関数で部品を作ります。`false` なら `fallback` 側です。分岐が消えると、その中で作った状態も片付けられます。再び表示すると、新しい部品として組み立てられます。

`show(open, move || editor())` のように、毎回作り直せる factory を渡します。先に `let child = editor();` を作り `move || child` で渡すと、消費する `View` を一度しか返せないため再表示できません。`View` は `Clone` な部品ではありません。

## 一覧には安定したキーを付ける

```rust,test_harness
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
    title: String,
}

fn item_list() -> impl IntoView {
    let items = signal(vec![Item { id: 1, title: "最初".into() }]);

    column((
        button("追加").on_press(move || {
            items.update(|all| {
                let id = all.len() as u32 + 1;
                all.push(Item { id, title: format!("項目 {id}") });
            });
        }),
        column(for_each(
            items,
            |item| item.id,
            |id, item| text(move || format!("{id}: {}", item.get().title)),
        )),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("一覧の例").run(item_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adding_appends_a_row() {
        let screen = kivact::test::render(item_list);
        screen.get_by_text("1: 最初");
        screen.get_by_role(Role::Button, "追加").click();
        screen.get_by_text("2: 項目 2");
    }
}
```

`for_each` の第 2 引数は項目からキーを取り出します。ここでは `id` を使います。並び順が変わっても同じキーの行は同じ行として扱われるため、行の中にある状態を保てます。`for_each` は項目に `Clone` と `PartialEq` を求めます。`PartialEq` は、同じキーの行の値が変わったかを判定し、変わった行にだけ新しい値を届けるために使われます。

行を項目ではなく位置（0 番目、1 番目…）に対応付ける `for_index` もあります。こちらは途中に挿入したり並べ替えたりすると、行の中の状態が別の項目へ移ります。多数の行のうち見える範囲だけを作る用途には `virtual_list` があります。まずは `for_each` と安定したキーを使い、必要になってから選んでください。

keyed row へ届く `Value<Item>` を setup で `get()` して文字列にすると、その時点の snapshot です。上の `move || item.get().title` や `memo(move || item.with(...))` で変化を追います。virtual list は画面外の row owner を破棄するので、スクロール外でも残す選択や draft は app の keyed model に保存します。

## 行から一覧へ書き戻す

行の checkbox で Todo の `done` を切り替えるときは、一覧の signal から key で一行を指す binding を作り、field へ投影します: `todos.item(|t| t.id, id).field(|t| &t.done, |t| &mut t.done)`。項目の選択やキー移動を付ける `interaction::List`・`Table` は [一覧と表の選択](../how-to-guide/lists-tables.md)にあります。

次は [見た目とアクセシビリティ](styling-accessibility.md) です。


## 自分で動かして確認する

`cargo run -p kivact-example-list` で一覧を操作します。上の例を入門 package に置き、行の状態を変えてから並べ替えてください。表示位置が変わっても同じキーの行に状態が残ることを確認します。
