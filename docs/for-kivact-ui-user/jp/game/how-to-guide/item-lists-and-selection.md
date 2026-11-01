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
        ListItem::new(1, "Rail Rifle").description("POWER 42").icon("bolt"),
        ListItem::new(2, "Pulse Blade").description("POWER 35"),
        ListItem::new(3, "Prototype Cannon").description("LOCKED").disabled(true),
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
                .map(|w| ListItem::new(w.id, w.name.clone()).description(format!("AMMO {}", w.ammo)))
                .collect::<Vec<_>>()
        })
    });
    list("Weapons", rows, selected)
}
# fn main() {}
```

アイテムを削除するときは、選択がそのアイテムを指していたなら選択を空にしてください。一覧は、プレイヤーの選択を書き込むとき以外、シグナルを書き換えません。

## 複数のアイテムを選ぶ

`.multiple(signal)` は一覧を複数選択にします。一覧はそのシグナル（`Signal<Vec<K>>`）だけを読み書きし、単一の `selected` 引数は使われないので、適当なシグナルを渡します。クリックで 1 つを選択、Ctrl（macOS では Cmd）で 1 つずつ切り替え、Shift で直前の項目から範囲を広げます。

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
        signal(None::<u32>),
    )
    .multiple(picked)
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

`game::tree(entries, selection)` はファイルツリーを描きます。`entries` は `game::TreeEntry` の一覧で、`TreeEntry::file(id, label)` と `TreeEntry::folder(id, label, children)` で作ります。`selection` は、選択された**葉**の id を持つ `Signal<String>` です。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_game as game;

fn files() -> impl IntoView {
    let selected = signal(String::from("save-1"));
    let entries = vec![
        game::TreeEntry::folder(
            "saves",
            "saves",
            vec![
                game::TreeEntry::file("save-1", "slot-1.sav"),
                game::TreeEntry::file("save-2", "slot-2.sav").marker("M"),
            ],
        ),
        game::TreeEntry::folder("mods", "mods", vec![]).collapsed(),
        game::TreeEntry::file("readme", "README.txt"),
    ];
    column((
        game::tree(entries, selected),
        game::mono(move || format!("OPEN {}", selected.get())),
    ))
    .style(style().gap(12).width(300).margin(24))
}
# fn main() {}
```

ツリー全体で Tab ストップは 1 つです。上下キーで見えている行を移動し、右キーと左キーでフォルダを開閉したり、フォルダと子のあいだを移動したりします。シグナルに書き込まれるのは葉だけで、フォルダはシェブロン、Enter、ダブルクリックで開閉します。`.collapsed()` はフォルダを閉じた状態で始め、`.marker("M")` は行の末尾に小さな印を付けます。エントリはツリーが作られるときに読まれ、シグナルにも追従するので、名前を変えたエントリは新しいラベルで表示されます。

## レコードを表で見せる

`data_table(rows, key, columns)` は標準のテーブルで、Game テーマがコンソールの表として描きます。`game::TableCell` を `.cell(..)` から返すと、列のセルにコンソール風の見た目を付けられます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::Tone;
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
                game::TableCell::Badge {
                    label: unit.status.to_owned(),
                    tone: if unit.status == "ALIVE" { Tone::Success } else { Tone::Danger },
                }
            }),
            DataColumn::new("HP", |unit: &Unit| format!("{:.0}", unit.hp))
                .compare(|a, b| a.hp.total_cmp(&b.hp))
                .cell(|unit: &Unit| game::TableCell::Progress { value: unit.hp, tone: Tone::Info })
                .numeric(true),
        ],
    )
    .selection(selected)
}
# fn main() {}
```

テーブルが持つのは、渡された状態だけです。行、選択、そして（`.sort(..)`、`.filter(..)`、`.page(..)` を使えば）並べ替え、検索テキスト、ページです。`.toolbar(|context| view)` は表の上の領域を置き換え、選択した行への一括操作などに使えます。`.footer(|context| view)` は下の領域を置き換えます。セルの種類は `TableCell::Identifier`、`Detail { title, description }`、`User { name, initials, color }`、`Badge { label, tone }`、`Progress { value, tone }`（0 から 100）です。

## 関連

- [メーターと状態を見せる](meters-and-status.md)
- [選択肢とクイック操作を用意する](choices-and-toolbars.md)
- チュートリアルの [タブとアイテム](../getting-started/03-tabs-and-items.md): 各タブの中に一覧を置いています。
- コンポーネントの API: `cargo doc -p kivact-ui-game --open`、および `cargo doc -p kivact --open` の `kivact::ui::List`、`DataTable`
