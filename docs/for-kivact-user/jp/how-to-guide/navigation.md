# ナビゲーション

[English](../../en/how-to-guide/navigation.md)

中核のナビゲーション部品にアプリケーションのデータを渡します。kit の theme を設定すると recipe と token が切り替わります。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{self, PaginationMode};

fn navigation() -> impl IntoView {
    let page = signal(0usize);
    let step = signal(1usize);
    column((
        ui::breadcrumb(vec![
            ListItem::new("home", "Home"),
            ListItem::new("orders", "Orders"),
        ]).on_activate(|key| println!("Open {key}")),
        ui::pagination(20usize, page).siblings(2usize),
        ui::pagination(20usize, page).mode(PaginationMode::Input),
        ui::stepper(vec![
            StepperItem::new("Account"),
            StepperItem::new("Address").description("Delivery address"),
            StepperItem::new("Confirm").status(StepStatus::Success),
        ], step),
        ui::navigation_menu("Main navigation")
            .link("guide", "Guide", || button("Read the guide"))
            .link("help", "Help", || text("Contact support")),
    ))
}
```

Pagination の binding は 0 始まり、表示する番号と入力欄は 1 始まりです。`.siblings` は現在のページの前後に見せる数、`.first_last(true)` は先頭と末尾への操作、`.step` は前後に動かすページ数です。`.item` は `PageItem` の内容を差し替えます。操作、現在のページの意味、focus は部品が持ちます。`.label_format` には 1 始まりの番号が渡ります。

コンストラクターの count がページ数の出所です。`.total_items(total)` を呼ぶと出所を総件数に切り替え、`.page_size(binding)` の値で割ります（省略時は 1）。`.page_size_options` は reactive な選択肢を受け取ります。件数の操作で page size を変えると先頭のページに戻ります。`.has_more(true)` は総数がまだ不明なときも次への操作を許します。`PaginationMode` はページ番号、点、件数、簡潔な操作、入力欄、ページ操作なしを選びます。入力は Enter または focus を外した時に確定し、範囲内に収めます。

Stepper の現在位置も 0 始まりです。説明、status、省略可能か、無効か、indicator、追加の内容は `StepperItem` に設定します。`.on_step_press` は操作を通知し、現在位置はアプリケーションが変えます。`.orientation` は並べる向きです。密度、間隔、indicator の見た目は kit の recipe、token、variant で変えます。

Breadcrumb は navigation landmark と list を公開します。末尾は `CurrentItem::Page` を持ち、操作の対象にはなりません。それ以前の項目は Enter、Space、pointer の操作で key を通知します。`.item` は項目の内容、`.separator` は装飾の区切りを差し替えます。

Navigation の trigger は hover で開きます。click、Enter、Space、ArrowDown で固定し、Escape で閉じます。固定した panel は外側の press または focus の移動でも閉じます。`.open_delay` と `.close_delay` は `Duration` を受け、`.placement` と `.align` で位置を決めます。独自のナビゲーションは `interaction::use_navigation_menu()` の上に作れます。`NavOpen` が開閉の唯一の状態で、`hover_after`、`leave_after`、`pin`、`toggle`、`close` がその owner へ遷移を送ります。

`breadcrumb(items).current(current_key)` は既定の末尾の現在ページを上書きします。リアクティブな `Option<K>` を受け取り、`Some(key)` はそのキー、明示的な `None` は現在ページなしを表します。メソッドを省略すると末尾が現在ページになります。

## 表示を変えても標準の操作を保つ

`.summary(|facts| view)` はページ数や件数の表示を追加します。受け取る `PaginationSummary` は現在の page、page count、総件数などの読み取り用の値です。件数を別のシグナルへコピーせず、この値からラベルを作れます。独自のボタンの内容には `.item(...)` を使い、ページを変える処理や focus の移動は Pagination に任せます。

Stepper の `.collapsed_steps(CollapsedSteps::Current)` は、コンパクトな幅で現在の項目だけを表示する既定の方針です。`All` は全項目を表示します。項目の `.indicator(...)` に `StepIndicator::Auto`、`Number`、`None` を渡すと、状態に応じた印、常に番号、印なしを選べます。文字や画像など任意の表示には `StepIndicator::custom(|| view)` を渡します。

テーマを書く場合、各工程の `StepperItemState` から index、count、実際の orientation を読めます。`layout` と `track` は配置、`leading`、`trailing`、`progress` は追加の装飾です。Breadcrumb の `BreadcrumbItemState` には current と実際の入力状態が渡るため、現在ページや hover に合った文字色・下線を設定できます。詳細は [テーマのガイド](theming.md) を参照してください。

## メニュー内から画面を切り替える

単純なリンクのコールバックには標準の `navigation_menu` の action を使います。独自のパネル内容では `navigation_menu_link(label)` を使うと、操作に必要な意味とメニューを閉じる動作を同じナビゲーションに接続できます。アプリはリンク先への遷移を行い、開閉やキーボード処理をコピーしません。

独自のトリガーが必要な場合は `use_menu_with_open(entries, open_binding)` で開閉をアプリの binding に接続します。`.trigger_action(callback)` は通常の押下動作を置き換える設定で、表示変更の監視用 callback ではありません。外部から表示する場合と、トリガーで表示する場合では最初の focus の扱いが異なるため、表示後に手動で focus を動かす補正を重ねないようにします。[独自コントロール](custom-controls.md)には Part の合成方法があります。
