# ノードグラフでノードをつなぐ

[English](../../../en/creative/how-to-guide/node-graph.md)

目的: 型付きソケットを持つノードとリンクを表示し、ノードの移動、接続の引き出しと付け替え、パラメータの変更、削除を利用者ができるようにします。グラフはアプリが保持し、どの編集が有効かもアプリが決めます。

キットのほかのエディタと同じく、`ui::node_editor(nodes, links, state)` は、あなたが所有するデータの見え方です。要求を報告するので、それを適用するか拒否するかはあなたが決めます。このガイドは、[チュートリアル](../getting-started/README.md)を終えていて、要求の仕組みを[タイムラインの編集](timeline-and-arrangement.md#契約の仕組み)で読んでいることを前提にしています。

## 動くグラフエディタ

```rust,no_run
use kivact::draw::{Point, Size};
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact_ui_creative as ui;

const IMAGE: &str = "Image";
const NUMBER: &str = "Number";

fn sockets_of_type(nodes: &[ui::NodeData], link: &ui::NodeLink) -> Option<(String, String)> {
    let from = nodes.iter().find(|n| n.id == link.from)?.outputs.get(link.output)?;
    let to = nodes.iter().find(|n| n.id == link.to)?.inputs.get(link.input)?;
    Some((from.type_name.clone(), to.type_name.clone()))
}

/// アプリの規則: 同じ型のソケットだけがつながる。
fn allowed(nodes: &[ui::NodeData], link: &ui::NodeLink) -> bool {
    sockets_of_type(nodes, link).is_some_and(|(from, to)| from == to)
}

fn initial_nodes() -> Vec<ui::NodeData> {
    let image = |label: &str| ui::NodeSocket::new(label, IMAGE, rgb(0xffd25a));
    let number = |label: &str| ui::NodeSocket::new(label, NUMBER, rgb(0x9b9ea8));
    vec![
        ui::NodeData::new(1, "入力", vec![], vec![image("画像")])
            .position(Point::new(30.0, 40.0))
            .size(Size::new(160.0, 90.0)),
        ui::NodeData::new(2, "ぼかし", vec![image("画像"), number("サイズ")], vec![image("画像")])
            .position(Point::new(280.0, 40.0))
            .size(Size::new(180.0, 140.0))
            .controls(vec![ui::NodeControl::number("半径", 100.0, 8.0, 0.0, 64.0, 1.0)]),
        ui::NodeData::new(3, "出力", vec![image("画像")], vec![])
            .position(Point::new(560.0, 40.0))
            .size(Size::new(160.0, 90.0)),
    ]
}

fn apply(
    nodes: &mut Vec<ui::NodeData>,
    links: &mut Vec<ui::NodeLink>,
    request: ui::NodeRequest,
) {
    match request {
        ui::NodeRequest::Move(positions) => {
            for (id, position) in positions {
                if let Some(node) = nodes.iter_mut().find(|n| n.id == id) {
                    node.position = position;
                }
            }
        }
        ui::NodeRequest::Connect(link) => {
            // キットは何も検査しない。アプリの規則を適用する。
            if allowed(nodes, &link) && !links.contains(&link) {
                // 入力は 1 本のリンクだけを受ける: 新しいリンクが古いものを置き換える。
                links.retain(|l| !(l.to == link.to && l.input == link.input));
                links.push(link);
            }
        }
        ui::NodeRequest::Disconnect(link) => links.retain(|l| *l != link),
        ui::NodeRequest::Delete { nodes: ids, links: gone } => {
            nodes.retain(|n| !ids.contains(&n.id));
            links.retain(|l| !gone.contains(l) && !ids.contains(&l.from) && !ids.contains(&l.to));
        }
        ui::NodeRequest::ToggleMute(ids) => {
            for node in nodes.iter_mut().filter(|n| ids.contains(&n.id)) {
                node.muted = !node.muted;
            }
        }
        ui::NodeRequest::Control { node, index, value } => {
            if let Some(control) = nodes
                .iter_mut()
                .find(|n| n.id == node)
                .and_then(|n| n.controls.get_mut(index))
            {
                control.value = value;
            }
        }
    }
}

fn graph_panel() -> impl IntoView {
    let nodes = signal(initial_nodes());
    let links = signal(vec![ui::NodeLink::new(1, 0, 2, 0), ui::NodeLink::new(2, 0, 3, 0)]);
    let state = ui::NodeState::new();

    // ノードとリンクは一緒に変わるので、スナップショットも 1 つで持つ。
    let snapshot = signal(None::<(Vec<ui::NodeData>, Vec<ui::NodeLink>)>);
    let on_edit = move |event: ui::EditorEvent<ui::NodeRequest>| match event {
        ui::EditorEvent::Begin => {
            snapshot.set(Some((nodes.get_untracked(), links.get_untracked())));
        }
        ui::EditorEvent::Request(request) => {
            nodes.update(|n| links.update(|l| apply(n, l, request)));
        }
        ui::EditorEvent::Commit => snapshot.set(None),
        ui::EditorEvent::Cancel => {
            if let Some((old_nodes, old_links)) = snapshot.update(|saved| saved.take()) {
                nodes.set(old_nodes);
                links.set(old_links);
            }
        }
    };

    ui::node_editor(nodes, links, state)
        .can_connect(move |link| nodes.with_untracked(|all| allowed(all, link)))
        .snap(true)
        .on_edit(on_edit)
        .name("ノードグラフ")
        .style(style().height(420))
}

fn main() -> kivact::Result<()> {
    App::new("ノード")
        .theme(ui::theme(ThemeMode::Dark))
        .run(graph_panel)
}
```

実行して、ノードをドラッグしてみてください。出力ソケット（右端）から、つなげられる入力ソケット（左端）へドラッグすると接続できます。つながっている入力ソケットから引き離すと、リンクを外したり付け替えたりできます。ノードまたはリンクを選んで `Delete` を押すと削除されます。マウスホイールで拡大縮小し、中ボタンのドラッグ、または `Space` を押しながらのドラッグでパンします。

## エディタが決めること、あなたが決めること

| エディタ | あなた |
|----------|--------|
| ノード・ソケット・リンクと、引き出し中のリンクを描く | ノードとリンクを保存し、各ノードに一意の `id` を付ける |
| `can_connect` で接続先を強調し、`true` を返したときだけ `Connect` を報告する | `apply` でも規則をもう一度検査する。`Connect` は重複や入力の置き換えの処理なしで届く |
| `Move`・`Connect`・`Disconnect`・`Delete`・`ToggleMute`・`Control` の要求を報告する | 要求を適用、制約、または無視する |
| 選択（`NodeState::selection`、`NodeState::link`）とパン/ズーム（`NodeState::transform`）をバインディングで持つ | それらのバインディングを自分のシグナルに差し替えて、読み取りや保存をする |

実際に役立つ細かい点です。

- **ソケットはインデックスで識別します。** `NodeLink::new(from, output, to, input)` は、両端のノード ID とソケットのインデックスを指します。同じ 2 つのノードの間にある複数のリンクは別々のリンクで、1 本を付け替えても他に影響しません。グラフを保存するなら、ソケットの順序を変えないでください。
- **型と色はあなたのものです。** `NodeSocket::new(label, type_name, color)` は、型を単なる文字列として、また、ソケットとそのリンクの色を持ちます。エディタは型を解釈しません。`can_connect` とあなたの `apply` が解釈します。
- **付け替えは 1 つのジェスチャーの中の 2 つの要求です。** リンクの端をある入力から別の入力へドラッグすると、`Disconnect` と `Connect` が 1 組の `Begin` と `Commit` の間に届くので、元に戻す手順は 1 つで済みます。
- **自分自身への接続は報告されません。**
- **`Move` の位置は絶対値で、** タイムラインのエディタと同じく、`Begin` 時点の位置から計算されます。
- **`snap(true)`** は、ドラッグでノードを動かす距離を、縦横とも 10 単位の倍数に丸めます。シグナルも渡せます。

## ノードにパラメータを追加する

`NodeData::controls` に `NodeControl` のリストを渡すと、指定した縦のオフセットの位置に、エディタがノードの中へ描きます。

```rust,no_run
use kivact_ui_creative as ui;

let controls = vec![
    ui::NodeControl::number("半径", 100.0, 8.0, 0.0, 64.0, 1.0),
    ui::NodeControl::choice("フィルター", 130.0, ["Gaussian", "Box"], 0),
    ui::NodeControl::toggle("乗算済みアルファ", 160.0, true),
];
# let _ = controls;
```

変更は `NodeRequest::Control { node, index, value }` として届き、`value` は絶対値の `NodeControlValue`（`Number { value, min, max, step }`、`Choice { options, selected }`、`Toggle(bool)`）です。上の `apply` と同じく、`node.controls[index].value` へ書き戻します。コントロールの値はノードのデータの中にあるので、データと一緒に保存・読み込みできます。数値のコントロールは、値を直接入力するポップオーバーも開きます。

ラベルだけのノードには、`NodeData::parameters(["ViewLayer"])` で表示用の文字列を並べられます。

## ノードにプレビュー画像を表示する

`NodeData::preview(pixels)` は、あなたが描いた画像をノードのプレビュー領域に表示します。描画はあなたの仕事なので、`Pixels` をキャッシュして、ノードの入力が変わったときだけ更新してください。`Pixels` の作り方と更新の仕方は、[アプリが描いたフレームを表示する](show-app-rendered-images.md)を参照してください。

## 表示状態を保持する

`NodeState` は、3 つのバインディングを持つ `Copy` の構造体です。`selection`（ノードの ID）、`link`（選択中のリンク）、`transform`（パンのオフセットと拡大率を持つ `ViewTransform`）です。ドキュメントを開いたときに表示を復元するには、自分のシグナルを作り、それを使って構造体を組み立てます。

```rust,no_run
use kivact::interaction::ViewTransform;
use kivact::prelude::*;
use kivact_ui_creative as ui;

let transform = signal(ViewTransform::default());
let selection = signal(Vec::<u64>::new());
let state = ui::NodeState {
    transform: transform.into(),
    selection: selection.into(),
    ..ui::NodeState::new()
};
# let _ = state;
```

## 関連項目

- [タイムライン・ドープシート・グラフ・アレンジメントを編集する](timeline-and-arrangement.md): 同じ `EditorEvent` の契約とスナップショットの手順。
- [アプリが描いたフレームを表示する](show-app-rendered-images.md)
- API の全体: `cargo doc -p kivact-ui-creative --open`
- カタログのノードの章: `cargo run -p kivact-ui-creative-catalog`
