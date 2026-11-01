# 公式 UI を使う

`kivact-ui-basic`、`kivact-ui-pop`、`kivact-ui-game`、`kivact-ui-creative` は、標準部品の theme と、キット固有の部品を提供します。Basic は shadcn/ui の Lyra preset を写した Neutral の汎用キット、Pop は落ち着いた配色の業務アプリ向けキット、Game は操作パネル向け、Creative は制作ツール向けです。button・checkbox・switch・text_input・text_area・select・dialog・menu・tooltip・tabs・radio_group・slider・splitter・toast_region は `kivact::ui` の共通実装を使い、kit の theme が token・recipe・icon の表で見た目を決めます。テーマは各キットの `theme(mode)` で作り、引数は `kivact::style::ThemeMode` だけです。密度などのキットの選択肢は、キットの `ThemeOptions` の method で足します（Game の `theme(mode).density(..)`）。結果は普通の `Theme` なので、`App::theme` にも `provide_theme` にもそのまま渡せます。キットの色や大きさは `theme.get::<Tokens>()` で読みます。button のキット固有の色調や形は `button(..).variant(..)` でキットの型を渡します（Pop の `ButtonTone`・`Slant`、Game の `ButtonTone`・`Slant`）。テーマの一部だけを変える方法は [テーマ](../for-kivact-user/how-to-guide/theming.md)を参照してください。core の [入門](../for-kivact-user/getting-started/README.md) を終えたら、用途に合うキットを選んでください。

| キット | 入門 | 作業別ガイド | 起動するカタログ |
|---|---|---|---|
| Basic | [設定画面を作る](getting-started/basic/README.md) | [テーマ・overlay・データ](how-to-guide/basic.md) | `cargo run -p kivact-ui-basic-catalog` |
| Pop | [フォームを作る](getting-started/pop/README.md) | [テーマと入力](how-to-guide/pop.md) | `cargo run -p kivact-ui-pop-catalog` |
| Game | [操作パネルを作る](getting-started/game/README.md) | [状態と tabs](how-to-guide/game.md) | `cargo run -p kivact-ui-game-catalog` |
| Creative | [編集パネルを作る](getting-started/creative/README.md) | [数値の編集](how-to-guide/creative.md) | `cargo run -p kivact-ui-creative-catalog` |

各入門は、起動、コード変更、操作結果の確認へ順に進むチュートリアルです。型とメソッドの全一覧は各 crate の rustdoc が reference です。部品の既定の文言は英語です。

[作業別ガイドの一覧](how-to-guide/README.md)から、標準 UI の使い方を確認できます。recipe の差替えを超えて独自部品を作る場合は、[部品作者向けガイド](../for-component-author/README.md)を参照してください。

初期開発版で、API は予告なく破壊的に変わります。キットが画面全体の業務規則、保存、通信を引き受けるわけではありません。renderer は実行アプリが選びます。編集や選択の状態はアプリの signal/Binding に持たせます。GUI 検証の許可規則は [CONTRIBUTING](../../CONTRIBUTING.md) にあります。
