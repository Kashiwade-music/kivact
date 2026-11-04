# Kivact UI キット

[English](../en/README.md)

**キット**は、Kivact アプリのための既製の見た目です。`kivact::ui` の標準コンポーネント（ボタン、入力欄、タブ、テーブル、ダイアログなど）を描き替えるテーマと、日付ピッカーやメーターといった分野ごとのコンポーネントがセットになっています。どのキットも公開された `kivact` API だけを使い、アプリの状態はこれまでどおりシグナルに持ちます。キットは渡された値を描き、ユーザーの操作を伝えるだけです。キットを切り替えても、変わるのはアプリの見た目で、書き方ではありません。

Kivact は開発初期の段階にあり、API は予告なく変わります。キットはまだ crates.io にありません。Git 依存として追加します（方法は各チュートリアルに載っています）。

## キットを選ぶ

| キット | クレート | 見た目 | 向いている用途 |
| --- | --- | --- | --- |
| Basic | `kivact-ui-basic` | shadcn/ui の Lyra プリセットのような、無彩色で角の四角い見た目 | 特に決まった雰囲気がない、汎用的なアプリ |
| Pop | `kivact-ui-pop` | 落ち着いたスレート色の面、淡い青のグラデーション、黄色のアクセント | 業務アプリ: フォーム、ダッシュボード、表、スケジュール |
| Game | `kivact-ui-game` | 直線的な角、斜めのボタン、区切りのあるメーター | ゲームの操作パネル、HUD、運用コンソール |
| Creative | `kivact-ui-creative` | ダークな編集ツールのような、密度の高いコンパクトな見た目（ライトモードもあります） | 制作ツール: プロパティパネル、数値欄、カラーピッカー、ドック |
| Soft | `kivact-ui-soft` | ライトとダークに対応した Astryx Neutral | 丸みのある操作部品とニュートラルな配色を使うアプリ |

使うキットのチュートリアルから始め、具体的な作業があるときにそのキットの作業別ガイドを使います。

| キット | チュートリアル（アプリを作る） | 作業別ガイド（1 つの作業ごと） | 動かして見る |
| --- | --- | --- | --- |
| Basic | [チュートリアル](basic/getting-started/README.md) | [ガイド](basic/how-to-guide/README.md) | `cargo run -p kivact-ui-basic-catalog` |
| Pop | [チュートリアル](pop/getting-started/README.md) | [ガイド](pop/how-to-guide/README.md) | `cargo run -p kivact-ui-pop-catalog` |
| Game | [チュートリアル](game/getting-started/README.md) | [ガイド](game/how-to-guide/README.md) | `cargo run -p kivact-ui-game-catalog` |
| Creative | [チュートリアル](creative/getting-started/README.md) | [ガイド](creative/how-to-guide/README.md) | `cargo run -p kivact-ui-creative-catalog` |
| Soft | [チュートリアル](soft/getting-started/README.md) | [ガイド](soft/how-to-guide/README.md) | `cargo run -p kivact-ui-soft-catalog` |

右端の列は、キットのすべてのコンポーネントを見せるカタログアプリです。リポジトリのチェックアウトから起動してください。キットを見比べるのに最も手早い方法です。

## はじめる前に

チュートリアルは、Kivact の基本（シグナル、コンポーネント、レイアウト）を知っている前提です。まだなら、先に [コアのチュートリアル](../../for-kivact-user/jp/getting-started/README.md) を終えてください。

## キットの使い方

どのキットも同じように使うので、1 つで覚えたことはそのまま通用します。

```rust,no_run
use kivact::prelude::*;
use kivact::style::ThemeMode;

fn app() -> impl IntoView {
    column((
        text_input("Name", signal(String::new())),
        button("Save").variant(kivact::ui::ButtonVariant::Primary),
    ))
    .style(style().gap(12).padding(24))
}

fn main() -> kivact::Result<()> {
    App::new("Hello")
        .theme(kivact_ui_pop::theme(ThemeMode::Light))
        .run(app)
}
```

`kivact_ui_pop` を `kivact_ui_basic`、`kivact_ui_game`、`kivact_ui_creative`、`kivact_ui_soft` のいずれかに置き換えると、そのキットの外観を適用できます。キット固有のコンポーネントは `kivact_ui_pop::名前` のように使います。標準のものと同じビルダーなので、`.style(..)`、`.test_id(..)` などの修飾子を受け取ります。

## リファレンス

キットのコンポーネント、オプション、トークンの一覧は API ドキュメントにあります。手元で生成します。

```text
cargo doc -p kivact-ui-pop --open
```

ほかのキットには `kivact-ui-basic`、`kivact-ui-game`、`kivact-ui-creative`、`kivact-ui-soft` を指定します。標準コンポーネントそのものは `kivact::ui` に記載されています（`cargo doc -p kivact --open`）。
