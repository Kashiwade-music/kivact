# 3. ドックに置く

[English](../../../en/creative/getting-started/03-dock-and-outliner.md)

このステップでは、インスペクターをドックワークスペースのパネルの 1 つにします。もう 1 つのパネルとしてレイヤーを一覧するアウトライナーを置き、そこで選んだレイヤーの値をインスペクターに表示します。最後に、完成したアプリを利用者と同じように操作するテストを書きます。

完成したアプリは、すべてのレイヤーを 1 つのシグナルに持ちます。アウトライナーとインスペクターはそのシグナルの 2 つの見え方で、両者が共有する**選択**が 3 つ目の状態です。

`src/main.rs` を、次のプログラム全体に置き換えます。ステップ 2 のフィールドを再利用しますが、今回はバラバラのシグナルではなく、*レイヤー*に結び付けます。

```rust,test_harness
use kivact::Binding;
use kivact::prelude::*;
use kivact::style::{Color, ThemeMode, rgb};
use kivact_ui_creative as ui;

#[derive(Clone, PartialEq)]
struct Layer {
    id: u64,
    name: String,
    x: f64,
    opacity: f64,
    fill: Color,
    easing: ui::CubicBezier,
}

fn sample_layers() -> Vec<Layer> {
    [
        (1, "背景", 0x20242c),
        (2, "タイトル", 0x8b7cff),
        (3, "サブタイトル", 0xffd25a),
    ]
    .into_iter()
    .map(|(id, name, fill)| Layer {
        id,
        name: name.into(),
        x: 0.0,
        opacity: 1.0,
        fill: rgb(fill),
        easing: ui::CubicBezier::default(),
    })
    .collect()
}

/// `current` が指すレイヤーの 1 つのフィールドへのバインディング。
fn layer_field<T: Clone + PartialEq + 'static>(
    layers: Signal<Vec<Layer>>,
    current: Value<Option<u64>>,
    fallback: T,
    read: fn(&Layer) -> T,
    write: fn(&mut Layer, T),
) -> Binding<T> {
    binding(
        move || {
            layers.with(|all| {
                let id = current.get()?;
                all.iter().find(|layer| layer.id == id).map(read)
            })
            .unwrap_or_else(|| fallback.clone())
        },
        move |value: T| {
            layers.update(|all| {
                let id = current.get_untracked();
                if let Some(layer) = all.iter_mut().find(|layer| Some(layer.id) == id) {
                    write(layer, value);
                }
            });
        },
    )
}

fn layer_form(layers: Signal<Vec<Layer>>, current: Value<Option<u64>>) -> impl IntoView {
    let x = layer_field(layers, current, 0.0, |l| l.x, |l, v| l.x = v);
    let opacity = layer_field(layers, current, 1.0, |l| l.opacity, |l, v| l.opacity = v);
    let fill = layer_field(layers, current, rgb(0), |l| l.fill, |l, v| l.fill = v);
    let easing = layer_field(
        layers,
        current,
        ui::CubicBezier::default(),
        |l| l.easing,
        |l, v| l.easing = v,
    );
    column((
        ui::property_row(
            "X",
            ui::number("X", x)
                .range(-1000.0, 1000.0)
                .step(1.0)
                .precision(1)
                .unit(" px")
                .label_visible(false),
        ),
        ui::property_row(
            "不透明度",
            ui::number("不透明度", opacity)
                .range(0.0, 1.0)
                .step(0.01)
                .precision(2)
                .label_visible(false),
        ),
        ui::property_row("塗り", ui::color_popup("塗り", fill)),
        ui::bezier_curve(easing),
    ))
    .style(style().gap(6).padding(10))
}

fn editor(layers: Signal<Vec<Layer>>) -> impl IntoView {
    let outline = ui::OutlinerState::new([]);
    outline.selection.set(vec![2]);
    // インスペクターが編集するレイヤー: 選択されている最初のもの
    let current = memo(move || outline.selection.get().first().copied());

    let dock_layout = signal(ui::DockLayout::new(ui::DockNode::split(
        ui::DockAxis::Horizontal,
        [
            (
                0.4,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("layers", "レイヤー").icon("stack").closable(false),
                ]),
            ),
            (
                0.6,
                ui::DockNode::tabs(vec![
                    ui::DockTab::new("inspector", "インスペクター")
                        .icon("adjustments")
                        .closable(false),
                ]),
            ),
        ],
    )));

    ui::dock(dock_layout, move |panel: &'static str| match panel {
        "layers" => ui::outliner(
            move || {
                layers
                    .get()
                    .iter()
                    .map(|layer| ui::TreeItem::new(layer.id, None, layer.name.clone(), "box"))
                    .collect::<Vec<_>>()
            },
            outline,
        )
        .restrictions(false)
        .on_rename(move |(id, name): (u64, String)| {
            layers.update(|all| {
                if let Some(layer) = all.iter_mut().find(|layer| layer.id == id) {
                    layer.name = name;
                }
            });
        })
        .into_view(),
        "inspector" => show_some(current, move |_| layer_form(layers, current))
            .fallback(|| text("レイヤーを選んでください"))
            .into_view(),
        _ => node().into_view(),
    })
    .style(style().width(pct(100)).height(pct(100)))
}

fn main() -> kivact::Result<()> {
    App::new("レイヤーインスペクター")
        .theme(ui::theme(ThemeMode::Dark))
        .window_size(900, 520)
        .run(|| editor(signal(sample_layers())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kivact::test::{RenderOptions, render_with_handle};

    #[test]
    fn the_inspector_edits_the_selected_layer() {
        let options = RenderOptions::from_env().theme(ui::theme(ThemeMode::Dark));
        let (screen, layers) = render_with_handle(options, || {
            let layers = signal(sample_layers());
            (layers, editor(layers))
        });

        // 「タイトル」が最初から選ばれている: End キーで X が範囲の上限になる。
        screen.get_by_role(Role::SpinButton, "X").focus();
        screen.press_key(Key::End);
        assert_eq!(layers.get_untracked()[1].x, 1000.0);

        // アウトライナーで別のレイヤーを選ぶと、インスペクターの対象が変わる。
        screen.get_by_role(Role::TreeItem, "サブタイトル").click();
        screen.get_by_role(Role::SpinButton, "X").focus();
        screen.press_key(Key::Home);
        assert_eq!(layers.get_untracked()[2].x, -1000.0);
        assert_eq!(layers.get_untracked()[1].x, 1000.0, "タイトルは変わらない");
    }
}
```

`cargo run` で実行します。ウィンドウが 2 つに分かれ、左に **レイヤー** パネルが `背景`・`タイトル`・`サブタイトル` を一覧します（`タイトル` が選択状態です）。右の **インスペクター** パネルには、そのレイヤーの X・不透明度・塗り・イージング曲線が表示されます。

## 試してみる

1. `タイトル` の **X** を変更します。アウトライナーで `サブタイトル` をクリックすると、インスペクターは `サブタイトル` 自身の値を表示します。`タイトル` に戻ると、先ほどの X が残っています。
2. `サブタイトル` をダブルクリックして新しい名前を入力し、`Enter` を押します。アウトライナーの表示が新しい名前に変わります。`Esc` で名前の変更をやめられます。
3. パネルの間の境界線をドラッグするか、パネルのタブ欄の右端にあるメニューボタンから **Split Right** を選びます。**インスペクター** のタブをウィンドウの別の端へドラッグすると、その場所にドックできます。
4. `cargo test` を実行します。末尾のテストは、アプリを実際のウィンドウで開き、レイヤーを選んでキーボードで編集します。

## 書いたコードの説明

- **1 つのモデル、2 つの見え方。** レイヤーはすべて `layers` というシグナル 1 つに入っています。アウトライナーは `layers` を `TreeItem` の一覧として表示し、インスペクターは 1 つのレイヤーの 1 つのフィールドを編集します。どちらもデータを所有しないので、両者がずれることはありません。
- **`ui::OutlinerState`** は、アウトライナーの表示状態を、あなたが所有するバインディングとしてまとめた構造体です。`expanded`（開いている項目）、`selection`、`renaming` を持ちます。選択が自分で持つ `Binding<Vec<u64>>` なので、インスペクターから読めます。これが 2 つのパネルで選択を共有する仕組みです。キットのエディタにはどれも同じような状態の構造体があります（`TimeState` など）。
- **`layer_field`** は「選択中のレイヤーのこのフィールド」を `Binding<T>` にします。`binding(get, set)` は、読み出し用のクロージャ（読んでいる値が変わると再実行されます）と書き込み用のクロージャを組み合わせます。部品からは普通のバインディングに見え、背後にリストがあることを知りません。`memo` は `current` をキャッシュするので、選択中の ID が実際に変わったときだけ、読み出しが再実行されます。
- **`show_some(current, ..).fallback(..)`** は、レイヤーが選ばれている間はフォームを、そうでなければヒントを表示します。選択中のレイヤーが別のレイヤーに変わっても、同じフォームが使われ、バインディングの結果だけが変わります。
- **ドックのレイアウトはデータです。** `ui::DockLayout<K>` は、分割（`DockNode::split`）の木で、葉にタブ付きのコンテナ（`DockNode::tabs`）が付きます。`K` は `Clone + Eq + Hash` を満たす好きなキーで、ここでは `&'static str` で各パネルを区別しています。レイアウトはシグナルとして `ui::dock(layout, factory)` に渡します。利用者の操作（タブのドラッグ、分割、リサイズ）はすべてそのシグナルへ書き戻されるので、ほかの状態と同じように保存、復元、元に戻すことができます。
- **パネルのファクトリはキーごとに 1 回だけ呼ばれます。** タブを切り替えてもパネルは破棄されず非表示になるだけなので、パネルは自分の状態を保てます。`closable(false)` は、タブの閉じるボタンを取り除きます。
- **テスト**は `kivact::test::render_with_handle` を使います。テスト用のウィンドウの中にアプリを組み立て、セットアップのクロージャが返した値（ここでは `layers` シグナル）を取り出せるようにして、`Screen` を返します。要素は、支援技術から見えるのと同じアクセシビリティの役割と名前（`X` という名前の `Role::SpinButton`）で探し、`focus()`・`click()`・`press_key(..)` で操作します。`RenderOptions::from_env()` は `KIVACT_TEST_MIRROR`（`off`・`auto`・`required`、既定 `auto`）と `KIVACT_TEST_GPU=1` を読みます。ディスプレイなしで実行するときは `KIVACT_TEST_MIRROR=off` を設定します。

## ここまでのまとめ

数値フィールド（元に戻すためのフック付き）、色のポップアップ、イージング曲線、アウトライナー、ドックを、すべてアプリの状態に結び付けたプロパティインスペクターと、そのテストができました。どの段階でも、キットがデータを所有することはありませんでした。

## 次へ

- タイムラインを自分のクリップにつなぐ: [タイムラインとアレンジメントを編集する](../how-to-guide/timeline-and-arrangement.md)
- 同じやり方でほかの値を編集する: [色・イージング曲線・グラデーションを編集する](../how-to-guide/color-curves-and-gradients.md)
- パネルについてさらに: [ドック・パネルグループ・アウトライナーを作る](../how-to-guide/panels-docks-and-outliners.md)
- すべてのガイド: [Creative のハウツーガイド](../how-to-guide/README.md)
