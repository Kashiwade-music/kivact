# ドック・パネルグループ・アウトライナーを作る

[English](../../../en/creative/how-to-guide/panels-docks-and-outliners.md)

目的: ツールのパネルを、利用者が並べ替えられるドックに配置し、名前の変更・並べ替え・絞り込みができる階層を表示し、インスペクターの区画をまとめ、コマンドを検索できるようにします。

これらの部品はどれも、あなたが所有する状態を表示します。ドックのレイアウト、アウトライナーのアイテム、パネルグループのリストは、あなたのシグナルの中の普通の値です。このガイドは、小さなドックとアウトライナーを段階的に作る[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## パネルをドックに並べる

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn workspace() -> impl IntoView {
    let layout = signal(ui::DockLayout::new(ui::DockNode::split(
        ui::DockAxis::Horizontal,
        [
            (
                0.25,
                ui::DockNode::tabs(vec![ui::DockTab::new("assets", "アセット").icon("folder")]),
            ),
            (
                0.75,
                ui::DockNode::split(
                    ui::DockAxis::Vertical,
                    [
                        (
                            0.7,
                            ui::DockNode::tabs(vec![
                                ui::DockTab::new("viewport", "ビューポート").closable(false),
                                ui::DockTab::new("preview", "プレビュー"),
                            ]),
                        ),
                        (
                            0.3,
                            ui::DockNode::tabs(vec![ui::DockTab::new("console", "コンソール")]),
                        ),
                    ],
                ),
            ),
        ],
    )));

    ui::dock(layout, |panel: &'static str| match panel {
        "assets" => text("アセット").into_view(),
        "viewport" => text("ビューポート").into_view(),
        "preview" => text("プレビュー").into_view(),
        "console" => text("コンソール").into_view(),
        _ => node().into_view(),
    })
    .style(style().width(pct(100)).height(pct(100)))
}

fn main() -> kivact::Result<()> {
    App::new("ドック")
        .theme(ui::theme(ThemeMode::Dark))
        .run(workspace)
}
```

利用者は、タブの切り替え、タブを別のコンテナの中・隣（コンテナを分割します）・ドック全体の端へドラッグすること、境界線でのリサイズ、タイトルのダブルクリック（または `F2`）によるパネル名の変更、アイコンのダブルクリックによる最大化、各コンテナのメニュー（**Close Panel**、**Maximize Panel**、**Split Right** など）を使えます。

### ドックの仕組み

- **レイアウトはあなたが所有する値です。** `DockLayout<K>` は木構造です。`DockNode::Split` は、軸に沿って重み付きの子を持ち、`DockNode::Tabs` は `DockTab` を持つコンテナです。`K` はパネルのキーとしてあなたが選ぶ型で、`Clone + Eq + Hash` が必要です。`&'static str` が最も簡単で、大きなツールなら `enum` のほうが向いています。利用者の操作による変更は、すべて、正規化された完全なレイアウトとしてあなたのシグナルへ書き戻されます。そのため、保存、復元、元に戻すことができます。境界線のドラッグでは、ポインターが動くたびにレイアウトが書き込まれます。
- **パネルはキーごとに 1 回だけ作られます。** `dock` に渡したクロージャはキーごとに 1 回だけ呼ばれ、できたビューは、タブが隠れても、移動しても、木が組み替えられても、マウントされたままです。利用者がパネルを並べ替えても、パネルは内部の状態を保ちます。タブを閉じるとビューは破棄され、同じキーをもう一度開くと新しく作られます。
- **キーが指すパネルは 1 つだけです。** 重複したキーは、レイアウトが正規化されるときに後のものが捨てられます。
- **ドックには大きさを与えてください。** 上のように、`.style(style().height(600))` やパーセントで指定します。

### コードからパネルを追加・移動・閉じる・最大化する

`DockLayout` には、成功したかどうかを返す普通のメソッドがあります。`layout.update(..)` の中で呼びます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn commands(layout: Signal<ui::DockLayout<&'static str>>) {
    // 右端の新しいコンテナにパネルを開く。
    layout.update(|l| {
        l.insert(
            ui::DockTab::new("history", "履歴").icon("clock"),
            ui::DockPlacement::Edge(ui::DockSide::Right),
        )
    });
    // プレビューのタブとして、その隣に移す。
    layout.update(|l| {
        l.move_tab(
            &"history",
            ui::DockPlacement::Tabs { container: "preview", index: None },
        )
    });
    layout.update(|l| l.maximize(&"viewport"));
    layout.update(|l| l.restore());
    layout.update(|l| l.close(&"history").is_some());
}
```

`DockPlacement` は、移動先のコンテナを、その中にあるどれかのタブで指定します。そのため、木が変わっても配置先は有効なままです。`insert` は、すでにあるキーを拒否します。開いているかもしれないパネルを「表示」したいときは、先に `layout.get().tab(&key)` で確かめ、あれば `activate` を呼んでください。

### 閉じたときの処理とメニューの命令を追加する

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn dock_with_hooks(layout: Signal<ui::DockLayout<&'static str>>) -> impl IntoView {
    ui::dock(layout, |panel: &'static str| text(panel))
        // タブが閉じた後（レイアウトは更新済み）に呼ばれる: 資源を解放する。
        .on_close(move |key| {
            let _ = key;
        })
        // 自分の命令が先に並び、区切り線の後に組み込みの命令が続く。
        .menu(
            |_container| vec![MenuEntry::item("duplicate", "パネルを複製")],
            move |(key, command): (&'static str, &'static str)| {
                if command == "duplicate" {
                    layout.update(|l| {
                        l.insert(
                            ui::DockTab::new("copy", format!("{key} のコピー")),
                            ui::DockPlacement::Tabs { container: key, index: None },
                        )
                    });
                }
            },
        )
        // 境界線のドラッグは、数値フィールドと同じく Begin・Commit・Cancel を報告する。
        .on_edit(|_phase| {})
}
```

`menu` の第 1 引数のクロージャは、メニューが属するコンテナを受け取り、あなたの項目を返します。第 2 引数のクロージャは、そのコンテナで有効なタブのキーと、選ばれた項目のキーを受け取ります。

## アウトライナーで階層を表示する

`ui::outliner(items, state)` は、`TreeItem` のリストを表示します。各アイテムは親を指し、兄弟はアイテムの並び順で表示されます。アウトライナーは、キーボード操作（矢印キー、`Home`、`End`、名前変更の `F2`）、`Shift` と `Ctrl` による範囲選択と追加選択、絞り込み、名前の変更、ドラッグ＆ドロップによる親の変更、アイテムごとの表示・選択可否・レンダリングの切り替えに対応しています。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

/// `drop` の指示どおりにアイテムを動かす: `drop.target` の下の、`drop.after` の直後へ。
fn apply_drop(items: &mut Vec<ui::TreeItem>, drop: ui::TreeDrop) {
    let Some(from) = items.iter().position(|item| item.id == drop.id) else {
        return;
    };
    let mut moved = items.remove(from);
    moved.parent = drop.target;
    let at = match drop.after {
        Some(after) => items
            .iter()
            .position(|item| item.id == after)
            .map_or(items.len(), |index| index + 1),
        None => items
            .iter()
            .position(|item| item.parent == drop.target)
            .unwrap_or(items.len()),
    };
    items.insert(at, moved);
}

fn scene_outliner() -> impl IntoView {
    let items = signal(vec![
        ui::TreeItem::new(1, None, "シーン", "folder"),
        ui::TreeItem::new(2, Some(1), "カメラ", "camera"),
        ui::TreeItem::new(3, Some(1), "ライト", "folder"),
        ui::TreeItem::new(4, Some(3), "キーライト", "bulb"),
    ]);
    let state = ui::OutlinerState::new([1, 3]);
    let filter = signal(String::new());
    let snapshot = signal(None::<Vec<ui::TreeItem>>);

    column((
        kivact::ui::text_input("絞り込み", filter),
        ui::outliner(items, state)
            .filter(filter)
            .on_rename(move |(id, name): (u64, String)| {
                items.update(|all| {
                    if let Some(item) = all.iter_mut().find(|item| item.id == id) {
                        item.label = name;
                    }
                });
            })
            .on_item_drop(move |drop| items.update(|all| apply_drop(all, drop)))
            .on_restriction(move |event| match event {
                ui::EditorEvent::Begin => snapshot.set(Some(items.get_untracked())),
                ui::EditorEvent::Request(change) => items.update(|all| {
                    for item in all.iter_mut().filter(|item| change.ids.contains(&item.id)) {
                        match change.flag {
                            ui::RestrictionFlag::Visible => item.visible = change.value,
                            ui::RestrictionFlag::Selectable => item.selectable = change.value,
                            ui::RestrictionFlag::Rendered => item.rendered = change.value,
                        }
                    }
                }),
                ui::EditorEvent::Commit => snapshot.set(None),
                ui::EditorEvent::Cancel => {
                    if let Some(before) = snapshot.update(|saved| saved.take()) {
                        items.set(before);
                    }
                }
            }),
    ))
    .style(style().gap(6).padding(8).width(280))
}

fn main() -> kivact::Result<()> {
    App::new("アウトライナー")
        .theme(ui::theme(ThemeMode::Dark))
        .run(scene_outliner)
}
```

- **アイテムは読み取り専用で、要求が外へ出ます。** キットのほかのエディタと同じく、アウトライナーは `items` を変更しません。名前の変更は `on_rename` に `(id, 新しい名前)` として、ドラッグ＆ドロップは `on_item_drop` に `TreeDrop { id, target, after }` として、アイテムの表示・選択可否・レンダリングのフラグの切り替えは `on_restriction` に届きます。あなたが適用すると、アウトライナーがその結果を表示します。
- **`on_item_drop` を指定しないと、ドラッグは無効です。** 行の上でのポインターの縦位置が、置き場所を決めます。上の 3 分の 1 は兄弟としてその行の前、下の 3 分の 1 は兄弟としてその後、真ん中の 3 分の 1 は子としてその最後に入ります。自分自身や自分の部分木の中へは置けません。`Esc` でドラッグを取り消します。
- **制限のペイント。** 表示のトグルを押したまま行をまたいでドラッグすると、1 回のジェスチャーで多くの行に同じ値を設定します。流れは、`Begin`、それまでに触れたすべての ID と 1 つの `value` を持つ要求、そして `Commit` か `Cancel` です。選択した行が複数あるときは、押した行のフラグの反対の値にそろいます。トグルを隠すには `.restrictions(false)` を使います。
- **`OutlinerState`** は、`expanded`（開いている ID）、`selection`、`renaming` をまとめます。どれも自分のシグナルに差し替えて共有できます。チュートリアルで選択を共有したのと同じです。
- **行のそのほかのオプション。** `.active(id)` は、選択とは別に現在のアイテムを示し、`reveal_active` はその祖先を開きます。`.on_activate(f)` は `Enter` かダブルクリックで実行されます（その場合、ダブルクリックでは名前の変更をせず、`F2` で変更します）。`.cell(width, render)` は、すべての行に、自分の部品を入れる固定幅のセルを加えます。`TreeItem::tone(..)` と `placeholder(true)` は、ラベルの見た目を変えます。

## インスペクターの区画をまとめる

`ui::panel_group(panels, content)` は、折りたたみできる区画を表示し、利用者はつまみのドラッグで並べ替えられます。`InspectorPanel` のリストはあなたのシグナルで、`content` が各区画の本体を組み立てます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn sections() -> impl IntoView {
    let panels = signal(vec![
        ui::InspectorPanel::new(1, "トランスフォーム"),
        ui::InspectorPanel::new(2, "見た目").collapsed(),
    ]);
    let position = signal(0.0);
    let opacity = signal(1.0);

    ui::panel_group(panels, move |panel: Value<ui::InspectorPanel>| {
        match panel.get_untracked().id {
            1 => ui::property_row("X", ui::number("X", position).label_visible(false)).into_view(),
            _ => ui::property_row(
                "不透明度",
                ui::number("不透明度", opacity).range(kivact::interaction::ScalarRange::new(0.0, 1.0).expect("valid range")).label_visible(false),
            )
            .into_view(),
        }
    })
}
```

区画の本体は、折りたたまれても並べ替えられてもマウントされたままなので、フィールドはフォーカスと状態を保ちます。見出しを `Ctrl`（または `Cmd`）を押しながらクリックすると、その区画だけが開き、ほかは閉じます。区画の中では、`ui::property_row(label, control)` がラベルを列にそろえます。状態のボタンやアクションを置くには `.leading(..)` と `.trailing(..)` のスロットを使います。グループ化せずにタイトル付きの 1 つの箱が欲しいときは `ui::panel(title, content)` を使います。サイズを変えられる 2 つのペインには、標準の `kivact::ui::split(sizes, (first, second))` を使います。見た目はドックの境界線と同じです。

## コマンドやアイテムを検索できるようにする

`ui::search_picker(items)` は、ポップアップに表示される検索できるリストです。入力で絞り込み、`↑` と `↓` で移動し、`Enter` かクリックで選び、`Esc` で取り消します。フォーカスは検索欄に残ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn add_node_button() -> impl IntoView {
    let last = signal(String::from("まだありません"));
    let picker = ui::search_picker(vec![
        ui::PickerItem::new("blur", "ぼかし").section("フィルター").shortcut("Shift B"),
        ui::PickerItem::new("noise", "ノイズ")
            .section("生成")
            .description("フラクタルノイズ")
            .icon("wave"),
        ui::PickerItem::new("solid", "ベタ塗り").section("生成").enabled(false),
    ])
    .on_activate(move |key: &'static str| last.set(key.to_string()))
    .on_preview(|_key: Option<&'static str>| {
        // 行がアクティブな間は `Some(key)`、ピッカーが閉じると `None`。
    });
    row((
        picker.attach(button("ノードを追加…")),
        text(move || format!("選択: {}", last.get())),
    ))
    .style(style().gap(8))
}
```

`attach(button)` はボタンの下にピッカーを開きます。ショートカットで開くピッカーには、代わりに `.at(anchor, position)` でウィンドウ上の点を指定します。キーの型（ここでは `&'static str`）はあなたが決め、ピッカーは選ばれたアイテムのキーを返します。`.column(title, items)` を複数回呼ぶと、検索欄を共有する複数列のピッカーになります。

放射状のメニューも同じように使えます。`ui::pie_menu(items, state).on_activate(choose)` は、`PieEntry` を 1 点の周りに表示し、公開されている `PieState` を使うと、自分のショートカットで開けます（キーを押したときに開き、離したときに確定します）。

## 関連項目

- [チュートリアル ステップ 3: ドックに置く](../getting-started/03-dock-and-outliner.md)
- [Creative キットのテーマ・フォント・アイコンを設定する](theme-and-icons.md): タブや行のアイコン名、ドックのメニューの文言の翻訳。
- [タイムライン・ドープシート・グラフ・アレンジメントを編集する](timeline-and-arrangement.md): 同じ要求とスナップショットの手順。
- API の全体: `cargo doc -p kivact-ui-creative --open`
- カタログのレイアウト・アウトライナー・ピッカーの章: `cargo run -p kivact-ui-creative-catalog`
