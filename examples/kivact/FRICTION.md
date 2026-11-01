# 5つの基本プログラムの摩擦

counter・form・list・async・componentを、標準UIと現在の公開APIで書いたときに残る記述を整理します。対象は2026-10-25のリファクタ（公開 API の口の統一）後の各 `examples/kivact/<例>/src/main.rs` です。以下は同じ規則で取得したソースの静的集計です。コンパイルと試験の結果は、対応するcommitとPRの検証記録を参照してください。

| 例 | code行 | `.into_view()` | turbofish（`::<`） | 型注釈 | `.clone()` | `move` |
|---|---:|---:|---:|---:|---:|---:|
| counter | 14 | 0 | 0 | 0 | 0 | 4 |
| form | 31 | 0 | 0 | 0 | 0 | 6 |
| list | 54 | 0 | 1 | 0 | 0 | 6 |
| async | 34 | 0 | 0 | 0 | 0 | 5 |
| component | 48 | 0 | 0 | 0 | 0 | 6 |

対象は最初の独立行 `#[cfg(test)]` の直前までです。code行は空行と、先頭空白を除いて `//` で始まる行を除きます。記号はこのcode部分の文字列一致、`move` は単語一致です。型注釈はclosure引数の `|x: T|` と `let x: T` だけを数えます。関数の引数型・戻り型、構造体field、`V: IntoView`、deriveは含めません。これはRust構文解析器による集計ではありません。

## 短くなった記述

5例ともKivactのimportは `use kivact::prelude::*;` の1行です。標準UIと `binding` のための別importはありません。asyncだけは別に標準ライブラリの `Duration` をimportし、sleepは `kivact::task::sleep` と修飾します。

listは `todos.item(|t| t.id, id).field(..)` で一行の `done` を元の一覧へ結び、呼出側で `binding`・`memo`・`callback` を組み立てません。componentの `on_close` は `impl Fn() + Copy + 'static` で受け、`button("Close").on_press(on_close)` へ渡します。`.into()`、`.call(())`、親側の `move |_| ...` は不要になりました。

元のmain `9f5f4985` を同じ規則で集計すると、code行は15/32/66/39/50、componentの `move` は7でした。`53c9865f` では14/31/63/37/48、componentの `move` は6で、現在は14/31/54/34/48、listの `move` は6、asyncの `move` は5です（`Signal::item`・`Binding::field` と、signalをそのまま受ける `resource` のsourceによる）。turbofish・型注釈・明示cloneの件数は同じです。行数の減少だけで、学習負担や性能が同じ比率で改善したとは扱いません。

## 初学者がまだつまずく点

### list：行から元のデータへ書き戻す

`for_each` の行factoryは行ownerのsetupで実行されるので、行の中で setup 専用の関数を呼べます。factoryが受け取る `todo` は読取り専用の `Value<Todo>` ですが、元の一覧の `Signal<Vec<Todo>>` から `todos.item(|t| t.id, id)` で key が `id` の一行を読み書きする `Binding<Todo>` を作り、`.field(|t| &t.done, |t| &mut t.done)` で `done` へ投影します。以前の例が10行で書いていた「読取りclosureと、id検索して更新する書込みclosure」の組は、この1行になりました。

項目が消えたあとの読みは最後に見えた値を返し、書込みは何もしません（debug buildでは `WriteAfterDispose` の診断を残します）。`unwrap()` で消失時にpanicさせるかを利用側で決める必要はありません。

`Todo` の `Clone + PartialEq` は、`for_each` が行の値を複製・比較する条件と、`item`・`field` が変化を比べる条件に対応します。例は `Vec::<Todo>::new()` の型明示を一つ持ちます。空の一覧から始めるため、要素型を書く場所がほかにないからです。

checkboxが編集する `Todo.done` は業務データであり、行の選択状態とは別です。`ListSelection` へ置換して短くすると意味が変わります。項目選択・キー移動・表cellには、[一覧と表のガイド](../../docs/for-kivact-user/how-to-guide/lists-tables.md)のList/Tableを使い分けます。

### component：子の型と寿命、callbackの条件

`children: impl Fn() -> V` の戻り型を表すため、名前付きの `V: IntoView` が必要です。`show` は再表示時に子を作るfactoryを受けるため、完成済みviewを一つ渡す場合とも区別します。

親の `show(first, move || card(...))` が使う `first` は、後で呼ばれるcallbackにも渡ります。外側のfactoryにも `move` が必要です。現在の `on_close` は引数なしの `Fn() + Copy + 'static` で、状態handleだけを捕まえるclosureはこの条件を満たします。任意の非Copyな所有値を捕まえたclosureまで、そのまま渡せるという意味ではありません。

動的なbutton labelは `String` を返すため、`"Collapse"` / `"Expand"` という `&str` から `.to_owned()` する記述が残ります。直接の文字列propが行う正規化と、closureの戻り型の条件は同じではありません。

### form：空の値と検証結果を表す

検証memoは `Option<String>` を返し、`(条件).then(|| "...".to_owned())` でエラーの有無を表します。`Option` の `Some` / `None`、`then`、ownedな `String` を作る意味を知る必要があります。`show_some` の行が受け取る値も `Value<T>` です。

### async：source、future、分岐の型

`resource` のsourceは `IntoValue<S>` なので、`id` のsignalをそのまま渡します。複数の状態から作るときは `move || (id.get(), page.get())` のようなclosureを渡します。fetchは `|id| async move { ... }`、戻りは `Result<T, UiError>` に揃えます。

この例は `dynamic` の `match` でIdle/Pending/Failed/Readyを表示します。Rustの全armは同じ戻り型を返す必要があり、現在はどのarmも `Node` です。処理中判定だけなら `state().is_pending()` を使えますが、この例の実測sourceは全状態の `match` を保っています。

`Duration` は標準ライブラリからimportし、専門的なsleepは修飾pathで呼びます。二つのKivact importが必要、という説明にはしません。

### counter：動的な読み取りをclosureへ入れる

表示やmemo、handlerへ `move || ...` を渡します。状態更新で部品関数全体が再実行されるわけではないため、変化を追わせる値はclosure内で読みます。明示的なcloneはありませんが、`get()` が返すデータの複製までゼロだという意味ではありません。

