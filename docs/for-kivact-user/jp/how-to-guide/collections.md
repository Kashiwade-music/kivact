# 静的なコレクションを表示する

静的な表には `ui::table`、セルのグリッド移動・並べ替え・フィルターが必要な表には `ui::data_table` を使います。静的な表は table・row-group・row・column-header・cell の役割を公開します。キーボード移動や選択の動作は追加しません。

```no_run
use kivact::prelude::*;
use kivact::style::TextAlign;

fn invoices() -> impl IntoView {
    table((
        table_header((table_row((table_head("Invoice"), table_head("Amount"))),)),
        table_body((table_row((
            table_cell("INV001"),
            table_cell("$250.00").align(TextAlign::End),
        )),)),
        table_footer((table_row((table_cell("One invoice").col_span(2usize),)),)),
    )).name("Invoices")
}
```

セルは行の利用可能な幅を分け合います。固定幅の列には、すべての行で同じ `.width(..)` を設定します。`.col_span(..)` は定数または reactive な列数を受け取り、セルの列の結合数を公開し、可変幅のセルにその列数分の幅を割り当てます。1 未満は 1 として扱います。表の読み上げ名は `.name(..)` で付けます。キャプションは表に隣接する普通のテキストで表示できます。

`ui::item_list` と `ui::item_row` は list・list-item の役割を公開します。先頭のメディア、タイトル、説明、末尾の操作を含むコンテンツに使います。選択可能な listbox には `ui::list` を使います。

```no_run
use kivact::prelude::*;

fn documents() -> impl IntoView {
    item_list((
        item_row("Quarterly report")
            .description("Updated today")
            .media(text("PDF"))
            .actions(button("Download")),
        item_row("Meeting notes").on_press(|| println!("Open notes")),
    )).name("Documents")
}
```

`.on_press(..)` を付けると行がフォーカスを受け取り、クリック・Enter・Space・支援技術による実行に対応します。list-item の役割は保ちます。`.header(..)` と `.footer(..)` は主な行の上下にコンテンツを追加します。`.title_content(..)` は読み上げ名を保ったまま表示するタイトルを置き換え、`.description_content(..)` は説明の view を置き換えます。説明は `described_by` で関連付けます。先頭のマーカーは、ほかのメディアと一緒に `.media(..)` に置きます。ほかのコントロールを実行するには、継承された `NodeBuilder` の操作から対象の `NodeRef::activate()` を呼べます。

階層には `ui::tree(label, nodes).expanded(expanded).selection(selected)` を使います。共有のツリー移動を使い、展開状態は枝にだけ公開します。`.item(..)` は行の中身、`.header(..)` は行の前のコンテンツを指定します。`.reveal(gap, transition)` は presence による枝の表示と退場を使います。選択されたキーとフォーカスを持つ行は別の状態です。展開と選択の binding はアプリケーションが所有します。

`UiTokens` の `table`・`item_list`・`tree` で寸法を調整できます。`UiRecipes::table`・`item_list`・`tree` で各 part のスタイルを変更でき、各 builder は `.variant(..)` で kit の variant を受け取ります。密度・区切り線・縞・メディアの枠・文字の見た目は recipe と style で表します。呼び出し側のスタイルは既定の recipe より優先されます。
