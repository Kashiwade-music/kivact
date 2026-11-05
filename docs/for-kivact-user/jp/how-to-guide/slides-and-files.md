# スライドとネイティブファイル

`ui::carousel(index, slides)` は前後のボタンと矢印キーで呼び出し側の index を更新します。横方向は読み方向に従い、縦方向は上下キーを使います。`.looping()` は端で折り返し、`.items_per_view(2.0)` は複数のスライドを表示します。`CarouselMode::Scroll` はネイティブスクロールを使います。`Snap::Mandatory` はスクロールが止まって 120 ms 後に最寄りのスライドへ揃えます。`Snap::None` は自由な位置を維持しながら最寄りの index を報告します。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{carousel, CarouselMode, dropzone};
fn content() -> impl IntoView {
    let index = signal(0usize);
    column((
        carousel(index, (text("One"), text("Two")))
            .mode(CarouselMode::Scroll)
            .style(style().width(240).height(160)),
        dropzone("Choose or drop text files")
            .multiple()
            .pick_on_press()
            .validate(|file| {
                (file.path.extension().and_then(|x| x.to_str()) != Some("txt"))
                    .then(|| "Choose a text file.".into())
            })
            .on_files(|files| { println!("{} files", files.len()); }),
    ))
}
```

`FileInfo` はネイティブの `path` を持ち、`name()` は表示名を導出します。検証は通知の前に各ファイルを受け取ります。サイズ制限にはアプリの validator 内で metadata を取得してください。部品はファイルシステムへアクセスしません。単一ファイルのモードは複数ファイルのドロップ全体を拒否します。picker のキャンセルは待機状態に戻り、検証・callback・picker の失敗は status に表示されます。

選択の保存、クリア、選択名の表示は呼び出し側が持ちます。`.on_files` で保存し、その名前を表示し、通常のボタンで保存値を消します。`.hint(..)` は案内文、`.filters(..)` はネイティブ picker のフィルターです。フィルターだけではドロップしたファイルを検証しません。

Carousel の recipe は root、viewport、track、slide、previous、next を公開します。Dropzone は root、label、hint、status です。`UiTokens` は `carousel` と `dropzone` にテーマ用の任意の上書き値をまとめます。標準の見た目の既定値は標準 recipe にあります。

ネイティブの意味付けは [APG carousel pattern](https://www.w3.org/WAI/ARIA/apg/patterns/carousel/) に基づきます。role description がないため group の名前に Carousel/Slide を含めます。フォーカス可能な carousel group は矢印での移動も提供します。自動回転はなく、通常のボタンは起動時にキーボードフォーカスを維持します。独自のビューは `interaction::use_carousel` と `interaction::use_dropzone` を利用できます。
