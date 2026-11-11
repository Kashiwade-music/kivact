# キャンバスに生きたビューを描く（drawable）

[English](../../en/how-to-guide/drawables.md)

キャンバスは図形とテキストを描きます。drawable はその反対側で、ボタン、テキスト入力、シグナルを持つ、本物のビューの部分木です。キャンバスは、それを任意の点に任意の変換の下で描けます。HTML-in-canvas と同じ考え方です。描かれたビューは、普通のビューのままです。フォーカスを受け取り、キーボードと IME の入力を受け取り、スクリーンリーダーに読まれ、テストから見つけられます。3D のカードの上の設定パネル、ダイヤルの上で動くコントロール、別のレンダラーが表示するビューなどに使います。

このガイドでは、drawable をマウントする方法、変換の下で描く方法、自動的に得られるもの（入力とアクセシビリティ）、そして drawable のピクセルを 3D エンジンなどの自前のレンダラーへ書き出す方法を説明します。

## ビューをマウントして描く

`drawable()` は、セットアップでハンドルを作ります。内容は一度だけ、キャンバスの `.drawable(ハンドル, 内容)` でマウントされ、キャンバスは `draw.drawable(ハンドル, at)` で描きます。`at` は、内容の左上の角を置く位置で、キャンバスの座標で指定します。変換の下に置くには、呼び出しを `draw.layer` で囲みます。

```rust,no_run
use kivact::draw::Layer;
use kivact::prelude::*;

fn settings_card() -> impl IntoView {
    let card = drawable();
    let angle = signal(20.0_f64);

    column((
        button("回す").on_press(move || angle.update(|a| *a += 15.0)),
        canvas(move |draw, size| {
            let width = card.size().get().width;
            let at = Point::new((size.width - width) / 2.0, 40.0);
            draw.layer(
                Layer::new().transform(
                    Transform::new().perspective(800.0).rotate_y(angle.get()),
                ),
                |draw| draw.drawable(card, at),
            );
        })
        .drawable(card, || {
            column((text("設定"), button("適用").on_press(|| ())))
        })
        .style(style().size(320, 200)),
    ))
    .style(style().gap(8).padding(12))
}
# fn main() -> kivact::Result<()> { App::new("設定カード").run(settings_card) }
```

- `card.size()` は、内容のレイアウト後の大きさを値として返します。最初のレイアウトまでは 0 なので、キャンバスはそのレイアウトの後に描きます。
- 内容は、キャンバスの子として絶対位置に置かれ、その左上の角が基準になります。キャンバスの大きさは変わりません。キャンバスには自分の大きさを与えてください。
- キャンバスは、読んでいるシグナル（ここでは `angle` と内容の大きさ）が変わると描き直されます。内容は自分で更新されます。内容のシグナルとハンドラーは、どこでと同じように働きます。
- drawable は `Copy` なので、キャンバスのクロージャはシグナルと同じように捕捉できます。

## 入力とアクセシビリティは自動で付く

描かれたビューのヒットテストやアクセシビリティを書く必要はありません。エンジンは、各点をキャンバスが描いたときの変換で逆に写し、元の内容の位置を求めます。そのため次のとおりです。

- 回転した「適用」ボタンをクリックすると、ユーザーに見えているとおりに、そのボタンに当たります。
- キーボードのフォーカスは、文書の順で内容へ移ります。Tab と Enter は、どのビューでも同じように働きます。
- 内容は、キャンバスのアクセシビリティツリーに現れ、その境界は描かれた位置へ変換されます。スクリーンリーダーは、ほかのビューと同じように読み上げます。
- テストは、役割と名前で内容を見つけられます（例: 名前「適用」の `Role::Button`）。そしてクリックできます。[アプリをテストする](testing.md)を参照してください。

現在のフレームでキャンバスが描いていない drawable は、ポインター入力を受け取りません。

## 自前のレンダラーへ書き出す

3D エンジンや独自のレンダラーは、画面に描かずに絵として内容を必要とすることがよくあります。その場合は、内容を `Viewport` にマウントします。Viewport は内容をレイアウトし、書き出し用に描きますが、画面には描きません。`share()` は `DrawableShare` を返します。これは描画スレッドから使える `Send` と `Sync` のハンドルです。ピクセルを要求し、フレームを待ちます。

```rust,no_run
use std::time::Duration;
use kivact::draw::Pixels;
use kivact::geometry::Matrix4;
use kivact::prelude::*;
use kivact::view::{DrawableFrame, ShareError, ShareUpdate, viewport};

fn engine_panel() -> impl IntoView {
    let panel = drawable();
    let share = panel.share();
    share.request_pixels();
    // エンジンは、パネルを viewport の右へ 40 px、下へ 24 px ずらした位置に表示します。
    share.set_geometry(Some(Matrix4::translate(40.0, 24.0, 0.0)));

    std::thread::spawn(move || {
        let mut seen = 0;
        loop {
            match share.wait(seen, Duration::from_millis(100)) {
                Ok(ShareUpdate::Frame(frame)) => {
                    seen = frame.number();
                    if let DrawableFrame::Pixels { pixels, .. } = frame {
                        // `pixels`（ストレートアルファの RGBA8）を、エンジンのテクスチャへ送ります。
                        let _ = pixels;
                    }
                }
                Ok(_) => {}
                Err(ShareError::Timeout) => {}
                Err(_) => break, // drawable が破棄された、または要求が失敗した
            }
        }
    });

    viewport(Pixels::new(1, 1, vec![0, 0, 0, 0]))
        .drawable(panel, || column((text("設定"), button("適用").on_press(|| ()))))
        .style(style().size(320, 200))
}
# fn main() -> kivact::Result<()> { App::new("エンジンのパネル").run(engine_panel) }
```

- `wait(after, timeout)` は、番号が `after` より大きい次のフレームを返します。来ないまま時間が過ぎると `Timeout` を返します。最後に見た番号を渡してください。`0` を渡すと、どのフレームでも受け取ります。フレーム番号は、要求をまたいでも増え続けるので、新しい要求の後でも最後に見た番号を渡せば正しく動きます。
- `latest()` は、待たずに最新のフレームを返します。描画ループが、手元にある最新の絵を使いたいときに使います。
- `request_pixels()` は新しい要求を始めます。古い要求のフレームは捨てられます。`stop()` は書き出しを止め、ウィンドウのバッファを解放します。
- フレームは、ストレートアルファの RGBA8 で、デバイスピクセル単位です。既定ではウィンドウの scale に従い、share の `set_scale(..)` で変えられます。
- `is_closed()` は、drawable が破棄されたかどうかを返します。上のループは、`Closed` を受け取ったときにも終わります。
- GPU のテクスチャとしてデバイスと共有するには、`gpu-interop` の機能を使い、ウィンドウのアダプターで `request_gpu(adapter)` を呼びます。取り込みによってテクスチャがウィンドウへ返される仕組みは、[自前の GPU レンダラーのテクスチャを表示する](gpu-texture-sharing.md)を参照してください。

- `set_geometry(Some(matrix))` は、エンジンがパネルをどこに描いたかをビューへ伝えます。matrix は、drawable の箱（左上を原点とする論理ピクセル）を、それをホストする viewport の論理ピクセルへ写します。エンジンが使うのは matrix の x・y・w の部分だけで、z は無視されます。そのため 3D の変換は、パネルの平面へ先に適用し、その結果の 2D の写像を渡してください。具体的には、`m` の行 0・1・3 と列 0・1・3 の成分です。こうすると、3D ビュー上のパネルのクリックが、パネルのボタンに届きます。
- `set_geometry(None)` は、パネルをヒット判定から外します。フレームは届き続けますが、クリックは内容に届かなくなります。
- 位置は、現在の要求に属します。`request_pixels()` と `request_gpu(..)` は、位置のない新しい要求を始めるので、その後で `set_geometry` を呼び直してください。

## 注意点

- **マウントは一度だけ。** drawable は一か所にだけマウントできます。二度目のマウントは報告され、その場所は空のままになります。
- **一つの記録の中で描くのは一度だけ。** 同じ drawable を同じキャンバスで二度描くと、一度目だけが描かれ、二度目はエラーとして報告されます。
- **内容は普通のビュー。** 内容は自分の状態、ハンドラー、フォーカスを持ちます。ほかのビューと同じです。
- **描いた形とヒットは一致します。** 3D で回転、拡大、持ち上げられた drawable は、描かれている位置で当たります。[3D の変換と奥行きを使う](3d-and-depth.md)を参照してください。
- **書き出しには描画の費用がかかります。** 要求のたびに内容を描きます。読まなくなった書き出しは `stop()` で止めてください。

## 関連項目

- [キャンバスで描く](canvas.md): 図形、パス、テキスト、描いた領域のヒット領域。
- [3D の変換と奥行きを使う](3d-and-depth.md): drawable が描かれる変換。
- [自前の GPU レンダラーのテクスチャを表示する](gpu-texture-sharing.md): CPU への読み戻しなしの GPU のフレーム。
- [アプリをテストする](testing.md): 役割と名前で内容を見つけてクリックする。
- API リファレンス: `cargo doc -p kivact --open` を実行し、`kivact::view::drawable` と `kivact::view::DrawableShare` を見てください。
