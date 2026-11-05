# ナビゲーション

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

Stepper の項目レシピには、同じレスポンシブ配置の投影から `index`、`count`、実際の `orientation` が渡されます。外観バリアントに軸や項目数を複製する必要はありません。

Breadcrumb の項目レシピは `BreadcrumbItemState { current, input }` を受け取ります。`input` は実際のリンクのパーツから渡され、非操作・現在ページのラベルにはありません。ホバー時の下線などの文字表現はこの投影を使用します。描画のみの疑似状態スタイルでは文字装飾を変更できません。

Stepper は実際の装飾パーツ `leading`、`trailing`、`progress`（既定は寸法ゼロ）と、`layout`・`track` コンテナのレシピを公開します。これらには実際の軸・項目数を含む同じ `StepperItemState` が渡されます。`.collapsed_steps(CollapsedSteps::Current)` はコンパクト時に現在の項目だけを表示します（既定）。`All` は選択やレスポンシブ閾値を変更せず全項目を表示します。

各工程のレシピには実際の `StepIndicatorKind` と独立した `interactive` 設定も渡され、メタデータを外観バリアントに複製する必要はありません。コンパクト表示の前後移動は同じ項目ベクトルと現在位置から最も近い有効な項目を導出します。

`pagination(count, page).summary(|facts| view)` は実際の `Value<PaginationSummary>` から要約を構成します。`page` はゼロ始まり、`count` はページ数、`total_items` は省略可能な項目総数、`page_size` はページごとの項目数です。表示する先頭・末尾の項目はコールバック内で導出し、範囲の端点を複製して保存しません。`has_more` は未確定のリモート件数を表します。`PaginationItemState.mode` は実際の構造モードを投影し、点と数字のレシピでバリアントに複製する必要はありません。

Dots は実際の全ページを表示します。左右キーは読み方向に従い、端で循環しながら選択と実際の対象フォーカスを移動します。点の Tab 順序には選択中の点だけが入ります。Dots は実際の対象に移動し、他のモードは既存のルートのロービングフォーカスを使用します。件数が変わると点のマウント範囲だけを作り直し、フォーカスのレジストリは持ちません。
