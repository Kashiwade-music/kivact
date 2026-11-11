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

Stepper の項目の recipe は、同じ responsive layout の投影から `index`、`count`、実際の `orientation` を受け取ります。見た目の variant が向きや項目数をコピーする必要はありません。

Breadcrumb の項目の recipe は `BreadcrumbItemState`（`Current`、`Label`、`Link(input)` のどれか）を受け取ります。`is_current()` は現在ページかどうか、`input()` は実際のリンクの Part の値で、操作できないラベルや現在ページのラベルでは `None` です。hover 時の下線のような状態に応じた文字の装飾はこの値で設定します。描画だけの疑似状態の style では文字の装飾を変えられません。

## ページ操作の形を選ぶ

ページ番号を知っている利用者には `PaginationMode::Input`、小さな前後の操作には `Compact`、短い視覚的な並びには `Dots` を使います。Dots はすべてのページを含み、Left と Right で選択と focus を動かし、端で折り返し、読む方向に従います。件数の多い結果では、数百個の点より番号付きのページか入力欄が向いています。

`PaginationWindow::Adjacent` は近くのページ番号を表示します。`.window(PaginationWindow::Buffered)` は先頭と末尾の近くでより広い範囲を表示します。これは構築時の設定で、ページ数と現在のページは引き続き変えられます。

項目の範囲を表示するには `.summary(|facts| view)` を使います。callback は 0 始まりの現在のページ、ページ数、省略可能な総件数、page size を持つ `Value<PaginationSummary>` を受け取ります。ページや page size の変更に追従するよう、表示する範囲は text の closure の中でこの値から組み立てます。件数を別のシグナルへコピーする必要はありません。

## コンパクトな工程表示を使いやすく保つ

`.collapsed_steps(CollapsedSteps::Current)` はコンパクトな配置で現在の項目を表示し、`All` はすべての項目を表示します。変わるのは表示だけで、どの工程が現在かは変わりません。前後の操作は無効な工程を飛ばします。選択と status の出所は、現在位置の binding と各 `StepperItem` のままです。

工程の `.indicator(..)` は印を選びます。`StepIndicator::Auto` は工程の status に従い、`Number` は常に番号を表示し、`None` は印を表示しません。文字や画像など任意の内容には `StepIndicator::custom(|| view)` を渡します。

## メニュー内から画面を切り替える

パネルを持たずに直接画面を切り替える最上位の項目には `.action(key, label, press)` を使います。パネルの中の項目は `navigation_menu_link(title)` で作ります。リンクの意味を持ち、操作されると囲んでいるパネルを閉じます。遷移はアプリケーションが行い、開閉やキーボード処理をコピーしません。

```rust,no_run
use kivact::prelude::*;
use kivact::ui;

fn docs_menu() -> impl IntoView {
    ui::navigation_menu("Docs")
        .action("home", "Home", || println!("Go home"))
        .link("guides", "Guides", || {
            ui::navigation_menu_link("Getting started")
                .description("Build the first window")
                .on_press(|| println!("Open getting started"))
        })
}
```

独自のメニューのトリガーでは、`interaction::use_menu_with_open(entries, open_binding)` で開閉をアプリケーションの binding に接続します。`.trigger_action(callback)` は setup 時にトリガーの通常の押下動作を置き換える設定で、表示の変更を監視する callback ではありません。外部から開いた場合は focus をメニューへ移さず、トリガーからキーボードで開いた場合は標準の最初の focus を保つため、表示後に手動で focus を動かす補正を重ねないようにします。

## 見た目を変える

kit の theme を設定すると、そのナビゲーションの見た目になります。kit 固有の選択には `.variant(..)`、局所的な配置には `.style(..)` を使います。テーマを書く場合は、pagination の `controls`、`sizes`、`input` の recipe、stepper の `leading`、`trailing`、`progress`、`layout`、`track` の recipe を差し替えられます。標準の Part の見た目を変えても、キーボード操作とアクセシビリティの動作は保たれます。詳細は [テーマのガイド](theming.md) を参照してください。

独自のナビゲーションの部品は、trigger ごとに表示用のシグナルを持たせず、`use_navigation_menu` とその panel の key から始めます。controller が遅延して開く動作、固定した panel、閉じる動作、取り外しを扱います。下位の Part を合成する前に [独自コントロール](custom-controls.md) を参照してください。
