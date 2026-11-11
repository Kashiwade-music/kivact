# 一覧とツリーでアイテムを見せる

[English](../../../en/game/how-to-guide/item-lists-and-selection.md)

インベントリ、ミッション一覧、ファイルツリー、ユニットの表など、プレイヤーが選ぶアイテムを見せるための手引きです。どれも同じ決まりに従います。**アイテムと選択はアプリが持つシグナル**です。コンポーネントはそれを描き、プレイヤーの選択を書き戻すだけで、コピーは持ちません。

## 一覧から 1 つ選ぶ

`list(label, items, selected)` は標準コンポーネントです。Game テーマは、直線的な角と選択の縁取りで描きます。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_game as game;

fn inventory() -> impl IntoView {
    let selected = signal(Some(2_u32));
    let items = vec![
        ListItem::new(1, "Rail Rifle").with_description("POWER 42").with_icon("bolt"),
        ListItem::new(2, "Pulse Blade").with_description("POWER 35"),
        ListItem::new(3, "Prototype Cannon").with_description("LOCKED").disabled(true),
    ];
    column((
        list("Inventory", items, selected),
        game::mono(move || format!("SELECTED {:?}", selected.get())),
    ))
    .style(style().gap(12).width(320).margin(24))
}

fn main() -> kivact::Result<()> {
    App::new("Inventory")
        .theme(game::theme(ThemeMode::Dark))
        .run(inventory)
}
```

- 第 1 引数は支援技術向けに一覧の名前を付けるもので、画面には表示されません。見出しは自分で描いてください。
- `ListItem` は**キー**（ここでは `u32`）とラベルを持ち、任意で説明、キットのアイコン名、`disabled(true)` を付けられます。キーには `Clone + Eq + Hash` を満たす任意の型（id、文字列、自分の `enum`）が使え、選択のシグナルに入るのがこのキーです。
- 選択は `Signal<Option<K>>` で、`None` は何も選んでいない状態です。コードからシグナルを書き換えると、選択も動きます。
- キーボード: 矢印キーで移動と選択、Home と End でジャンプ、名前を打つとその文字で始まる項目へジャンプし、無効な項目は飛ばされます。Tab で一覧に入ると、選択中の項目に着地します。

## 自分のデータからアイテムを作る

自分のアイテム型をシグナルに持ち、`memo` で `ListItem` を導き出します。データが変わるたびに一覧が作り直され、型には好きなものを持たせられます。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Weapon {
    id: u32,
    name: String,
    ammo: u32,
}

fn weapons(all: Signal<Vec<Weapon>>, selected: Signal<Option<u32>>) -> impl IntoView {
    let rows = memo(move || {
        all.with(|all| {
            all.iter()
                .map(|w| ListItem::new(w.id, w.name.clone()).with_description(format!("AMMO {}", w.ammo)))
                .collect::<Vec<_>>()
        })
    });
    list("Weapons", rows, selected)
}
# fn main() {}
```

アイテムを削除するときは、選択がそのアイテムを指していたなら選択を空にしてください。一覧は、プレイヤーの選択を書き込むとき以外、シグナルを書き換えません。

## 複数のアイテムを選ぶ

第 3 引数は単一選択なら `Signal<Option<K>>`、複数選択なら `Signal<Vec<K>>` を受け取ります。クリックで選択し、Ctrl（macOS は Cmd）で切り替え、Shift で範囲を選びます。

```rust,no_run
use kivact::prelude::*;

fn squad() -> impl IntoView {
    let picked = signal(vec![1_u32]);
    list(
        "Squad",
        vec![
            ListItem::new(1, "Aya"),
            ListItem::new(2, "Ben"),
            ListItem::new(3, "Cho"),
        ],
        picked,
    )
}
# fn main() {}
```

## 各行を自分で描く

`.item(|item, state| view)` は、行に表示する内容を置き換えます。キー、キーボードの動き、選択はそのままです。`item` は `Value<ListItem<K>>`、`state` はその行が選択中か、アクティブかを教えてくれます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn missions() -> impl IntoView {
    let selected = signal(Some("alpha"));
    list(
        "Missions",
        vec![ListItem::new("alpha", "Alpha"), ListItem::new("bravo", "Bravo")],
        selected,
    )
    .item(|item, state| {
        row((
            game::mono(move || if state.get().selected { ">" } else { " " }),
            text(move || item.with(|item| item.label())),
        ))
        .style(style().gap(8))
    })
}
# fn main() {}
```

シグナルは、クロージャの中（`state.get()`、`item.with(..)`）で読んでください。そうすれば、変化に行が追従します。

## 階層をツリーで見せる

`kivact::ui::tree(label, items, selected)` は Game テーマで平坦な `TreeNode` の一覧を描画します。展開、選択、変更マーカーはアプリケーションが所有します。選択 binding でフォルダを除外すれば葉だけを選択できます。`.on_activate(..)` で Enter とダブルクリックによるフォルダの開閉を実装します。`expanded` に含めないキーは閉じた状態で始まります。行の slot にはアイコン、ラベル、変更マーカーを組み合わせられます。ツリー全体の Tab ストップは 1 つで、矢印キーと展開マーカーを使えます。ノードのラベルはリアクティブな入力に追従します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::TreeNode;
use kivact_ui_game as game;

fn files() -> impl IntoView {
    let selected = signal(Some("save-1".to_owned()));
    let expanded = signal(vec!["saves".to_owned()]);
    let nodes = vec![
        TreeNode::new("saves".to_owned(), None, "saves"),
        TreeNode::new("save-1".to_owned(), Some("saves".to_owned()), "slot-1.sav"),
        TreeNode::new("save-2".to_owned(), Some("saves".to_owned()), "slot-2.sav"),
        TreeNode::new("mods".to_owned(), None, "mods"),
        TreeNode::new("readme".to_owned(), None, "README.txt"),
    ];
    column((
        kivact::ui::tree("Files", nodes, binding(selected, move |key: Option<String>| {
                if key.as_deref().is_none_or(|id| !["saves", "mods"].contains(&id)) {
                    selected.set(key);
                }
            }))
            .expanded(expanded)
            .on_activate(move |key| {
                if key == "saves" {
                    expanded.update(|keys| if keys.contains(&key) {
                        keys.retain(|id| id != &key);
                    } else { keys.push(key); });
                }
            })
            .item(|item, _| {
                let key = item.get_untracked().key;
                let label = match key.as_str() {
                    "save-1" => "slot-1.sav", "save-2" => "slot-2.sav",
                    "readme" => "README.txt", "saves" => "saves", _ => "mods",
                };
                row((text(label).style(style().grow(1)), game::mono(if key == "save-2" { "M" } else { "" })))
                    .style(style().grow(1))
            }),
        game::mono(move || format!("OPEN {}", selected.get().unwrap_or_default())),
    )).style(style().gap(12).width(300).margin(24))
}
# fn main() {}
```

## レコードを表で見せる

`data_table(rows, key, columns)` は標準のテーブルで、Game テーマがコンソールの表として描きます。`game::GameCellVariant` を `.cell(..)` から返すと、列のセルにコンソール風の見た目を付けられます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::Tone;
use kivact_ui_game as game;

#[derive(Clone, PartialEq)]
struct Unit {
    id: u32,
    name: &'static str,
    status: &'static str,
    hp: f64,
}

fn units(rows: Signal<Vec<Unit>>) -> impl IntoView {
    let selected = signal(Vec::<u32>::new());
    data_table(
        rows,
        |unit: &Unit| unit.id,
        vec![
            DataColumn::new("Unit", |unit: &Unit| unit.name.to_owned()).sortable(),
            DataColumn::new("Status", |unit: &Unit| unit.status.to_owned()).cell(|unit: &Unit| {
                game::GameCellVariant::Badge {
                    label: unit.status.to_owned(),
                    tone: if unit.status == "ALIVE" { Tone::Success } else { Tone::Danger },
                }
            }),
            DataColumn::new("HP", |unit: &Unit| format!("{:.0}", unit.hp))
                .compare(|a, b| a.hp.total_cmp(&b.hp))
                .cell(|unit: &Unit| game::GameCellVariant::Progress { value: unit.hp, tone: Tone::Info })
                .numeric(true),
        ],
    )
    .selection(selected)
}
# fn main() {}
```

テーブルが持つのは、渡された状態だけです。行、選択、そして（`.sort(..)`、`.filter(..)`、`.page(..)` を使えば）並べ替え、検索テキスト、ページです。`.toolbar(|context| view)` は表の上の領域を置き換え、選択した行への一括操作などに使えます。`.footer(|context| view)` は下の領域を置き換えます。セルの種類は `GameCellVariant::Identifier`、`Detail { title, description }`、`User { name, initials, color }`、`Badge { label, tone }`、`Progress { value, tone }`（0 から 100）です。

## 関連

- [メーターと状態を見せる](meters-and-status.md)
- [選択肢とクイック操作を用意する](choices-and-toolbars.md)
- チュートリアルの [タブとアイテム](../getting-started/03-tabs-and-items.md): 各タブの中に一覧を置いています。
- コンポーネントの API: `cargo doc -p kivact-ui-game --open`、および `cargo doc -p kivact --open` の `kivact::ui::List`、`DataTable`
