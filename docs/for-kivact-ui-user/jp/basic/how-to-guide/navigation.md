# ナビゲーションを作る: サイドバー、タブ、パンくず、メニューバー、ページネーション

[English](../../../en/basic/how-to-guide/navigation.md)

このガイドでは、ユーザーがアプリの中を移動する手段を作ります。サイドバーのあるアプリのシェル、ページ内のタブ、パンくず、メニューバー、上部のナビゲーションメニュー、ページ番号です。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

これらのコンポーネントは、どれも現在の位置を持ちません。位置は signal（`page`、`current` など）に持ち、コンポーネントに渡して、signal が示すものを表示します。何かを押すと signal が書かれ、画面の残りがそれに従います。

## サイドバーでアプリのシェルを作る

`basic::sidebar_provider(open, sidebar, page)` は、ページ領域の隣にサイドバーを置き、`open` signal を両方で共有します。メニューボタンは `.active(..)` で現在のページを示し、`.on_press(..)` でページを変えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn shell() -> impl IntoView {
    let open = signal(true);
    let page = signal("home");
    let entry = move |key: &'static str, label: &'static str, name: &'static str| {
        basic::sidebar_menu_button(label)
            .icon(icon(name))
            .active(move || page.get() == key)
            .on_press(move || page.set(key))
    };
    let sidebar = basic::sidebar()
        .collapsible(basic::SidebarCollapsible::Icon)
        .rail(true)
        .header(basic::large("Acme Inc"))
        .content({
            basic::sidebar_group("Application").content({
                basic::sidebar_menu((
                    entry("home", "Home", "house"),
                    entry("inbox", "Inbox", "inbox"),
                    entry("settings", "Settings", "settings"),
                ))
            })
        });
    let content = column((
        row((
            basic::sidebar_trigger(),
            basic::separator()
                .orientation(Orientation::Vertical)
                .style(style().height(16)),
            basic::breadcrumb_content((
                basic::breadcrumb_link("Acme"),
                basic::breadcrumb_separator(),
                basic::breadcrumb_page(move || page.get().to_string()),
            )),
        ))
        .style(style().height(48).padding_x(12).gap(8).align_items(Align::Center)),
        kivact::ui::heading(move || page.get().to_string(), 4).style(style().padding(12)),
    ))
    .style(style().grow(1));
    column(basic::sidebar_provider(open, sidebar, content)).style(
        style()
            .size(pct(100), pct(100))
            .background(basic::ColorName::Background)
            .color(basic::ColorName::Foreground),
    )
}
```

- provider は親いっぱいに広がります。枠の中に置くときは、親に幅と高さを与えてください。
- `.collapsible(..)` は、閉じたときの動作を決めます。`Offcanvas`（既定）は画面外へスライドし、`Icon` は幅 48 px のアイコンの列に縮んでラベルがツールチップになり、`None` は閉じられません。`.rail(true)` は、端に細い帯を足し、それで開閉できるようにします。
- `basic::sidebar_trigger()`、rail、Ctrl+B（macOS では Cmd+B）は、どれも `open` を書きます。ショートカットをウィンドウのどこでも効かせたいときは、provider をアプリのルートの view にします。
- `.side(basic::SidebarEdge::Right)` はサイドバーを右に置き、`.variant(basic::SidebarVariant::Floating)` や `Inset` は枠の付け方を変えます。
- `.header(..)`、`.content(..)`、`.footer(..)` は関数を取ります。グループは `sidebar_group(label)` に `.content(..)` と、必要なら `.action(..)` を付けたもので、グループの間は `basic::sidebar_separator()` で区切ります。
- ボタンには `.badge("12")` で件数を、`.sub(|| (basic::sidebar_menu_sub_button("Overview").active(true), basic::sidebar_menu_sub_button("Roadmap")))` で入れ子の一覧を付けられます。
- サイドバーに検索欄を置くには `basic::sidebar_input(label, signal)` を使います。

## タブでビューを切り替える

タブは標準の `tabs(label, options, current)` です。`current` は選択中のタブの key を持ち、常に選択肢のどれかです。`.content(|key| ..)` は、選ばれた key のパネルを作ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn account() -> impl IntoView {
    let current = signal("account");
    let options = vec![
        ListItem::new("account", "Account"),
        ListItem::new("password", "Password"),
        ListItem::new("billing", "Billing").disabled(true),
    ];
    tabs("Settings", options, current)
        .variant(basic::TabsVariant::Line)
        .content(|key| {
            let (title, body) = match key {
                "account" => ("Account", "Make changes to your account here."),
                _ => ("Password", "Change your password here."),
            };
            column((kivact::ui::heading(title, 4), basic::muted(body))).style(style().gap(8).padding_top(16))
        })
}
```

- 既定は枠で囲んだ見た目です。`.variant(basic::TabsVariant::Line)` で下線の見た目になります。
- `.orientation(Orientation::Vertical)` は、タブをパネルの脇に縦に積みます。
- 第 1 引数は、タブ一覧の accessible name です。矢印キーでタブの間を移動します。

## 今どこにいるかを示す

パンくずは、リンク、区切り、現在のページを並べた `basic::breadcrumb_content((..))` です。リンクは `.on_press(..)` を取り、現在のページはリンクではありません。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn trail() -> impl IntoView {
    let at = signal(String::from("Components"));
    basic::breadcrumb_content((
        basic::breadcrumb_link("Home").on_press(move || at.set("Home".into())),
        basic::breadcrumb_separator(),
        basic::breadcrumb_link("Components").on_press(move || at.set("Components".into())),
        basic::breadcrumb_separator(),
        basic::breadcrumb_page(at),
    ))
}
```

隠した階層の代わりに、区切りの間へ `basic::breadcrumb_ellipsis()` を置きます。名前の一覧だけで足りるなら、`basic::breadcrumb(vec!["Home".to_string(), "Docs".to_string()])` が文字だけからパンくず全体を作ります。

## メニューバーを足す

`basic::menubar(menus).on_activate(choose)` は `(タイトル, 項目)` の組の一覧を取り、タイトルを横に並べます。それぞれが標準のメニューを開きます。`choose` は、選ばれた項目の key を受け取ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn bar() -> impl IntoView {
    let last = signal(String::from("Nothing chosen yet."));
    column((
        basic::menubar(vec![
                (
                    "File".to_string(),
                    vec![
                        MenuEntry::item("new", "New tab").with_shortcut("Ctrl+T"),
                        MenuEntry::separator(),
                        MenuEntry::item("print", "Print...").with_shortcut("Ctrl+P"),
                    ],
                ),
                (
                    "Edit".to_string(),
                    vec![
                        MenuEntry::item("undo", "Undo").with_shortcut("Ctrl+Z"),
                        MenuEntry::item("redo", "Redo").with_shortcut("Ctrl+Y"),
                    ],
                ),
            ]).on_activate(move |key: &'static str| last.set(format!("Chosen: {key}")))
        .style(style().width(240)),
        basic::muted(last),
    ))
    .style(style().gap(12))
}
```

項目は、ほかのメニューと同じ `MenuEntry` です（項目、チェック項目、ラジオ項目、見出し、サブメニュー。[オーバーレイ](overlays.md)を参照）。`Vec` の代わりにクロージャを渡すと、メニューが signal に追従します。バーはシステムのメニューを置き換えません。ウィンドウの中のふつうの行です。

## ページの上部からセクションへ移動する

`basic::navigation_menu(items)` は `NavItem` を並べた行です。`NavItem::link(label, action)` は押されると `action` を実行します。`NavItem::menu(label, content)` は、項目の下にパネルを開き、`content` がそのパネルの view を作る関数です。パネルの中の行には `basic::navigation_menu_link(title)` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn top_nav() -> impl IntoView {
    let clicks = signal(0usize);
    let getting_started = move || {
        column((
            basic::navigation_menu_link("Introduction")
                .description("How the pieces fit together."),
            basic::navigation_menu_link("Installation")
                .description("Add the crates to your project.")
                .icon(icon("download")),
        ))
        .style(style().width(320))
    };
    column((
        basic::navigation_menu(vec![
            basic::NavItem::menu("Getting started", getting_started),
            basic::NavItem::link("Docs", move || clicks.update(|n| *n += 1)),
        ]),
        basic::muted(move || format!("Docs pressed: {}", clicks.get())),
    ))
    .style(style().gap(12))
}
```

## 長い一覧をページに分ける

`basic::pagination(page, count)` は、前へ・次へのボタンとページ番号を表示し、ページが多いときは省略記号を使います。`page` は 0 から数える signal、`count` はページ数です。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn pager() -> impl IntoView {
    let page = signal(0usize);
    column((
        basic::pagination(page, 12usize),
        basic::muted(move || format!("Page {} of 12", page.get() + 1)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
```

- `.label_format(|number| format!("p.{number}"))` は、ページ番号の表記を変えます。
- 細かく決めたいときは、`basic::pagination_content((basic::pagination_previous(), basic::pagination_link("1"), basic::pagination_link("2").active(true), basic::pagination_ellipsis(), basic::pagination_next()))` を自分で組み立てます。
- 「Previous」「Next」の語と accessible label は `basic.pagination.*` のメッセージから来ます。置き換え方は[テーマとトークン](theme-and-tokens.md)を参照してください。
- 表の行をページに分けるなら、`data_table(..).page(page, size)` にページ分けが組み込まれています。[データの表示](data-display.md)を参照してください。

## 関連項目

- [オーバーレイ](overlays.md): メニュー、ポップオーバー、ダイアログ。
- [データの表示](data-display.md): 並べ替えとページ分けのある表。
- rustdoc: サイドバーの部品とナビゲーションのコンポーネントは `cargo doc -p kivact-ui-basic --open`。
