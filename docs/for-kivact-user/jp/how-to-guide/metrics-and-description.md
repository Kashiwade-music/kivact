# 指標と説明リスト

`kivact::ui` の標準 builder は、インストールしたテーマで表示されます。値とラベルにはリアクティブな文字列を渡せます。数値と単位の書式はアプリ側で決めます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{self, ProgressVariant, Trend};

fn overview() -> impl IntoView {
    let completed = signal(Some(0.7));
    let revenue = signal("$12,400".to_owned());
    column((
        ui::heading("概要", 2),
        ui::progress(completed)
            .variant(ProgressVariant::Ring)
            .name("アップロード"),
        ui::stat("売上", revenue).delta("今月 +8%").trend(Trend::Up),
        ui::description_list((
            ui::description("担当者", text("Ada")),
            ui::description("プロフィール", button("開く")),
        )).name("プロジェクトの詳細"),
    )).style(style().gap(16).width(320))
}
```

progress は 0～1 の割合を受け取り、範囲外の値を丸めます。`None` は進捗が未確定であることを表します。`ProgressVariant::{Bar, Ring, Gauge}` は独自の外観と同じ `.variant(impl Variant)` を使い、同じ型を後から渡すと前の値を置き換えます。どの外観でも progress の role、範囲、支援技術向けの百分率を保ちます。既定の Ring は百分率の文字も表示し、Gauge は区切られた目盛りを描きます。`.size(..)` と `.moving(..)` も使えます。アニメーションはテーマと動きを減らす設定に従います。

`stat(label, value)` はラベルを名前とする group を公開し、アプリが書式化した値を支援技術向けの値にします。`.delta(impl IntoText)` は変化の説明、`.trend(impl IntoValue<Trend>)` は `Up`、`Down`、`Neutral` の方向をすべての recipe に渡します。増減だけでは成功や危険を意味しません。意味を表す tone は `.variant(..)` で渡し、テーマで外観を決めます。`.icon(impl IntoView)` は装飾用の view です。`.label_content(..)` と `.value_content(..)` は支援技術向けのラベルと値を保ちながら表示を置き換えます。`.chart(..)` には kit の sparkline など、アプリが用意したチャートを渡します。数値のアニメーション、独自書式、単位はリアクティブな文字列で渡せます。stat 自体は文字列を解析したり数値としてアニメーションしたりしません。

`heading(text, level)` は `Role::Heading` と 1～6 に丸めた階層を公開します。文字の外観はテーマの heading recipe が決めます。見出しの文字列もリアクティブです。

`description_list(children)` は list を公開します。`description(term, detail)` は項目名を名前とする list item で、詳細の group は表示された項目名を `labelled_by` で参照します。この公開された関連は項目名が変わると更新されます。詳細には任意の view や操作用の部品を置けます。`.term_content(impl IntoView)` は項目名の表示をアイコンと文字などに置き換え、支援技術向けの項目名は保ちます。列幅、grid、間隔は通常の style や description list の recipe で設定します。長いリストの件数制限や展開は、アプリ側の disclosure と children の構成で行えます。編集できる inspector の行は Creative kit に残ります。

メディアの縦横比には、通常の style のプロパティを使います。

```rust,no_run
use kivact::prelude::*;

fn preview() -> impl IntoView {
    node().style(style().width(320).aspect_ratio(16.0 / 9.0))
}
```

テーマでは `UiRecipes::heading`、`UiRecipes::stat`、`UiRecipes::description_list` を変更します。stat の part は `root`、`label`、`value`、`delta`、`icon`、`chart`、description list の part は `root`、`item`、`term`、`detail` です。progress の `track` と `fill` に加えて Ring の文字用の `label` recipe があります。各 part は部品に渡した variant を受け取ります。
