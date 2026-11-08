# 画像を表示し、カスタムフォントを使う

[English](../../en/how-to-guide/images-and-fonts.md)

PNG、JPEG、SVG は `image(source)` で表示し、支援技術が説明できるよう名前を付け、スタイルで大きさを決めます。独自のフォントは `App::font` で登録します。このガイドでは、実行時に読み込む画像、動くアニメーション画像、アイコンとして使う SVG ファイル、自分で文字や画像を含む SVG、可変フォントの軸も扱います。

## 最短で動く画像

```rust,test_harness
use kivact::prelude::*;

// 例を自己完結させるために、SVG のバイト列をインラインで書いている。アプリでは
// ファイルを埋め込む: `static LOGO: &[u8] = include_bytes!("../assets/logo.svg");`
static LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="24" viewBox="0 0 48 24"><rect width="48" height="24" rx="6" fill="#2563eb"/><circle cx="12" cy="12" r="6" fill="white"/></svg>"##;

fn app() -> impl IntoView {
    column((
        image(LOGO).name("Kivact のロゴ").test_id("logo"),
        image(LOGO).style(style().width(96)).test_id("wide"),
    ))
    .style(style().gap(12).padding(16).align_items(Align::Start))
}

fn main() -> kivact::Result<()> {
    App::new("Images").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_image_takes_its_natural_size_and_keeps_its_aspect_ratio() {
        let screen = kivact::test::render(app);
        let size = |id: &str| {
            let r = screen.get_by_test_id(id).rect().expect("laid out");
            (r.width, r.height)
        };
        assert_eq!(size("logo"), (48.0, 24.0));
        assert_eq!(size("wide"), (96.0, 48.0));
        screen.get_by_role(Role::Image, "Kivact のロゴ");
        assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    }
}
```

`image(source)` は内容から形式を判断します。PNG（アニメーション PNG を含む）、JPEG、GIF、WebP、SVG と SVGZ に対応します。アニメーションする GIF、WebP、PNG は再生されます（[アニメーション画像を再生する](#アニメーション画像を再生する)を参照）。スタイルに大きさがなければ本来の大きさになり、片方の辺だけ指定すると、もう一方は縦横比に従って決まります。この例の column が `align_items(Align::Start)` を指定しているのは、既定の `Stretch` だと、どの画像も column と同じ幅に伸びてしまうからです。

## 画像の取得元

`image` は、`Asset` に変換できるものなら何でも受け取ります。

| 取得元 | 使いどころ |
| --- | --- |
| `include_bytes!("logo.png")` や `static` のバイト列 | 実行ファイルに同梱するファイル(ディスクからは読まない) |
| `Vec<u8>`、`AssetBytes` | ダウンロードなど、実行時に得たバイト列 |
| `"photos/cover.jpg"`、`String`、`PathBuf`、`&Path` | 最初に表示するときに読むファイル。カレントディレクトリからの相対パス |
| `Asset::from(..)` | 上のいずれかを、保持できる値にしたもの |
| クロージャ、`Signal<Asset>`、`Value<Asset>` | 変化する画像 |

```rust,no_run
use kivact::view::Asset;
use kivact::prelude::*;

fn gallery(paths: Vec<&'static str>) -> impl IntoView {
    let current = signal(0_usize);
    column((
        // `current` が変わるとクロージャが再実行され、別の Asset を返す。
        image(move || Asset::from(paths[current.get()])).style(style().height(160)),
        button("次へ").on_press(move || current.update(|i| *i = (*i + 1) % 3)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Gallery").run(|| gallery(vec!["a.jpg", "b.jpg", "c.jpg"]))
}
```

### 実行時に得たバイト列

`Asset` は内容ではなく**同一性**でキャッシュされます。同じファイルパス、同じ埋め込みスライス、同じ `AssetBytes` の不変バッファが、同じ Asset です。`Asset::from(bytes.clone())` は式が実行されるたびに新しいアロケーションを作り、したがって新しい Asset になって、そのたびにデコードし直されます。自分で作ったバイト列は、セットアップで `Asset` を 1 回だけ作り、その `Asset` を clone してください。`AssetBytes` は内部の共有方法を隠した不変バイト列です。通常は `Asset::from(bytes)` に `Vec<u8>` を渡せばよく、自分で共有ポインターを用意する必要はありません。複数の画像やフレームで同じバッファを渡す場合は、作成した `AssetBytes` や `Asset` のハンドルを clone します。

```rust,no_run
use kivact::view::Asset;
use kivact::prelude::*;

fn avatar(bytes: Vec<u8>) -> impl IntoView {
    let asset = Asset::from(bytes); // 1 回だけ作る
    column((
        image(asset.clone()).style(style().size(64, 64)).name("プロフィール画像"),
        image(asset).style(style().size(24, 24)), // デコード済みの同じ画像
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Avatar").run(|| avatar(Vec::new()))
}
```

埋め込みのバイト列は `const` ではなく `static` で名前を付けてください。`const` は使うたびにコピーされるので、同じファイルが複数回デコードされることがあります。

ネットワークやディスクの画像をウィンドウを止めずに読み込むには、`resource` でバイト列を取得し、届いたら画像を表示します([コンポーネントから非同期処理を実行する](async-and-tasks.md)を参照)。

## サイズとフィット

```rust,no_run
use kivact::view::ImageFit;
use kivact::prelude::*;

static PHOTO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#888"/></svg>"##;

fn thumbnails() -> impl IntoView {
    row((
        // 縦横比を保って箱の中に収める(既定)。
        image(PHOTO).style(style().size(80, 80)),
        // 箱を埋め、はみ出した部分を切り取る。
        image(PHOTO).fit(ImageFit::Cover).style(style().size(80, 80)),
        // 箱に合わせて引き伸ばす。
        image(PHOTO).fit(ImageFit::Fill).style(style().size(80, 80)),
    ))
    .style(style().gap(12).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Fit").run(thumbnails)
}
```

- 幅と高さの両方を指定したとき、画像が箱をどう埋めるかは `fit` が決めます。`Contain`(既定)、`Cover`、`Fill`、`None`(本来の大きさ。箱で切り取られる)、`ScaleDown`(本来の大きさ。ただし箱が小さければ縮小)があります。
- 縦横比を保つには、片方の辺だけ指定します。`style().height(160)` です。
- `style().radius(8)` は角を丸め、`style().clip(true)` は子を切り取り、`style().opacity(..)` は画像を薄くします。

## アニメーション画像を再生する

アニメーションする GIF、WebP、PNG は、自分で再生されます。`.playing(..)` で制御します。`bool` かシグナルを渡し、`false` にするとアニメーションは今のフレームで止まります。動きを減らす設定がオンのときは、アニメーション画像は最初のフレームを表示し、`playing` の値にかかわらず再生されません。

```rust,no_run
use kivact::prelude::*;

fn loading(busy: Signal<bool>) -> impl IntoView {
    column((
        image("assets/spinner.gif")
            .name("読み込み中")
            .playing(busy)
            .style(style().size(32, 32)),
        text(move || String::from(if busy.get() { "読み込み中" } else { "完了" })),
    ))
    .style(style().gap(8).padding(16).align_items(Align::Start))
}

fn main() -> kivact::Result<()> {
    App::new("Spinner").run(|| loading(signal(true)))
}
```

動きが何かを伝える画像（「読み込み中」と言うスピナーなど）には名前を付けてください。動きが飾りだけの画像には、名前を付けないでください。

## 画像を説明する

画像は、名前を付けたときだけ支援技術に公開されます。

- **意味のある画像**(ロゴ、グラフ、内容を伝える写真): `.name("Kivact のロゴ")` を付けます。見えない人が知るべきことを、`alt` テキストのつもりで書きます。
- **装飾**(飾り、背景、テキストのラベルの隣にあるアイコン): `.name(..)` を付けません。`alt=""` と同じで、スクリーンリーダーは読み飛ばします。
- test id だけでは、画像は公開されません。

## SVG をアイコンとして使う

画像はテキストの色だけで塗れるので、1 つの SVG をライトとダークのテーマで使え、hover、pressed、disabled の状態にも追従します。

```rust,no_run
use kivact::prelude::*;

static STAR: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2l3 7h7l-6 4 2 7-6-4-6 4 2-7-6-4h7z"/></svg>"#;

fn favorite() -> impl IntoView {
    row((
        image(STAR)
            .monochrome(true)
            .style(style().size(20, 20).color(ColorToken::Warning)),
        text("お気に入り"),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Icon").run(favorite)
}
```

`.monochrome(true)` は画像のアルファを保ち、node のテキスト色で塗ります。同じアイコンを名前でどこからでも使い(テーマで置き換えられるように)たいときは、テーマに登録して `icon("star")` で描きます。[テーマ](theming.md)の「アイコン」の節を参照してください。

## SVG の中に文字と画像を入れる

SVG は自分の `<text>` と埋め込みの画像を持てます。どちらも描かれます。

- `<text>` の要素は、アプリのフォントで描かれます。フォントの family 名は、登録したときの名前で書きます（[独自のフォントを登録する](#独自のフォントを登録する)を参照）。文字は、そのフォントの字形と寸法で描かれます。フォントは、最初の画面より前に登録してください。
- `href` が PNG、JPEG、GIF、WebP の `data:` URL である `<image>` は描かれます。ファイルへのリンク（`href="logo.png"`）は読まれません。その画像は描かれないまま SVG が描かれるので、バイト列を埋め込むか、画像は `image(..)` で隣に表示してください。
- テキストと埋め込みの画像のために多くのメモリが必要な SVG は、受け付けられず、ほかの読めない素材と同じく asset のエラーとして報告されます。

```rust,no_run
use kivact::prelude::*;

static BADGE: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="120" height="32" viewBox="0 0 120 32"><rect width="120" height="32" rx="6" fill="#1e3a8a"/><text x="12" y="21" font-family="Inter" font-size="14" fill="#ffffff">Online</text></svg>"##;

fn status() -> impl IntoView {
    image(BADGE).name("状態: オンライン")
}

fn main() -> kivact::Result<()> {
    let regular = std::fs::read("assets/Inter-Regular.ttf").expect("font file");
    App::new("Status").font(regular).run(status)
}
```

## 可変フォントの軸を調整する

可変フォントには、太さや幅などの軸があり、フォントの範囲の中の任意の値に設定できます。`font_variation_settings` で書きます。四文字のタグと値の組の一覧です。ほかの設定も、フォントが持つ軸に沿って字形を調整します。

- `font_variation_settings([("wght", 650.0)])` は、タグで軸を設定します。太さは `wght`、幅は `wdth`、傾きは `slnt`、光学サイズは `opsz` など。
- `font_width(75.0)`（CSS の `font-stretch`）は、標準の幅に対する百分率で幅を設定します。50 から 200 です。
- `font_optical_sizing(false)` は、光学サイズの軸を止めます。既定では文字の大きさに従うので、見出しと注釈が同じ字形になります。

```rust,no_run
use kivact::prelude::*;

fn headlines() -> impl IntoView {
    column((
        text("可変の太さ").style(
            style()
                .font_family("Inter")
                .font_variation_settings([("wght", 650.0)]),
        ),
        text("詰まった幅").style(style().font_family("Inter").font_width(75.0)),
        text("どの大きさでも同じ字形").style(
            style()
                .font_family("Inter")
                .font_size(48)
                .font_optical_sizing(false),
        ),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    let inter = std::fs::read("assets/Inter-Variable.ttf").expect("font file");
    App::new("Headlines").font(inter).run(headlines)
}
```

フォントが持たない軸への設定は何もしません。`font_feature_settings([("tnum", 1)])` は OpenType の機能を同じように入り切りし、`tabular_nums()` は `tnum` の機能の短い書き方です。

## 独自のフォントを登録する

`App::font` は、最初の画面の前に TTF、OTF、TTC ファイルを読み込みます。その後は、フォントの**ファミリー名**(フォントに書かれた名前で、ファイル名とは限りません)を `style().font_family(..)` に使えます。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    column((
        text("The quick brown fox").style(style().font_family("Inter")),
        text("グリフごとにフォールバック")
            .style(style().font_family("Inter, 'Noto Sans JP', sans-serif")),
        text("let x = 1;").style(style().font_family("monospace")),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    // アプリではファイルを埋め込む: `.font(include_bytes!("../assets/Inter-Regular.ttf"))`。
    let regular = std::fs::read("assets/Inter-Regular.ttf").expect("font file");
    let bold = std::fs::read("assets/Inter-Bold.ttf").expect("font file");

    App::new("Fonts").font(regular).font(bold).run(app)
}
```

- `.font(..)` はファイルごとに 1 回呼びます。標準と太字は別ファイルであることが多いので、両方を登録し、`style().weight(700)` で字体を選びます。`italic(true)` は、ファミリーに斜体があればそれを選びます。
- `font_family` は node の子孫に継承されるので、コンテナに指定するとパネル全体が変わります。総称名の `system-ui`、`sans-serif`、`serif`、`monospace` は常に使えます。
- カンマ区切りのリストは、**グリフごと**にフォントを選びます。その文字を持つ最初のファミリーが使われます。欧文フォントを日本語フォントの前に置くときに使います。
- インストールされていないファミリーは、システムの UI フォントにフォールバックし、`Asset` エラーとして 1 回だけ報告されます。使えないフォントのファイル(WOFF や壊れたファイル)も同様に報告されますが、アプリは起動します。
- アプリ全体の既定フォントを 1 つにするには、テーマのテキストのファミリーを設定します。`theme.with_text(Typography { family: Some("Inter".into()), ..theme.text().clone() })` です。`Typography` は `kivact::style::theme` にあります。
- `kivact::platform::font_families()` は、登録したものを含めて、使えるファミリーを一覧にします。セットアップかハンドラーの中で呼びます。

## 画像が表示されないとき

読み取れない、またはデコードできない取得元は、何も描かず、`Asset` エラーとして 1 回だけ報告されます。報告には呼び出し位置があり、ファイルの場合は試した絶対パスも含まれます。アプリのほかの部分は動き続けます。テストでは `screen.errors()` でこうした問題を確認できます([アプリをテストする](testing.md)を参照)。実行時の読み方は[エラーや更新の原因を調べる](diagnostics.md)で説明しています。

## 関連項目

- [テーマ](theming.md): テーマのアイコン、色、既定のフォント。
- [レイアウト](layout.md): 画像の大きさと配置。
- [コンポーネントから非同期処理を実行する](async-and-tasks.md): ウィンドウを止めずに画像を読み込む。
- [翻訳とロケール](i18n.md): 他の文字体系のフォントと文字の向き。
- リファレンス: `cargo doc -p kivact --open` で `Asset` と `ImageFit` を検索。

NodeRef の `scroll_state()` は `NotScrollable`・`AtStart`・`Middle`・`AtEnd`、`image_status()` は `Loading`・`Ready`・`Failed` のリアクティブな値を返します。reader は setup で作ります。画像の状態は再試行を含めて現在の source に追従します。
