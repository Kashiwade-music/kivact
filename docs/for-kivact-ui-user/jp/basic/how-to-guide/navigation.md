# ナビゲーションを作る: サイドバー、タブ、パンくず、メニューバー、ページネーション

[English](../../../en/basic/how-to-guide/navigation.md)

この手引きでは、利用者がアプリの中を移動できるようにする方法を説明します。サイドバーのあるアプリの枠、ページの中のタブ、パンくず、メニューバー、上部のナビゲーションメニュー、ページ番号です。[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

これらの部品はどれも、現在の位置を持ちません。位置はシグナル（`page`、`current` など）で持ち、部品に渡して、そのシグナルが示すものを表示します。何かを押すとシグナルが書き換わり、残りの画面がそれに追従します。

## サイドバーのあるアプリの枠を作る

`kivact::ui::sidebar_provider(open, sidebar, page)` は、ページの領域の横にサイドバーを置き、`open` のシグナルを両者で共有します。メニューのボタンは `.active(..)` で現在のページを示し、`.on_press(..)` で切り替えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn shell() -> impl Element {
    let open = signal(true);
    let page = signal("home");
    let entry = move |key: &'static str, label: &'static str, name: &'static str| {
        kivact::ui::sidebar_menu_button(label)
            .icon(icon(name))
            .active(move || page.get() == key)
            .on_press(move || page.set(key))
    };
    let sidebar = kivact::ui::sidebar()
        .rail(true)
        .header(basic::large("Acme Inc"))
        .content({
            kivact::ui::sidebar_group("Application").content({
                kivact::ui::sidebar_menu(vec![
                    entry("home", "Home", "house").into_view(),
                    entry("inbox", "Inbox", "inbox").into_view(),
                    entry("settings", "Settings", "settings").into_view(),
                ])
            })
        });
    let content = column((
        row((
            kivact::ui::sidebar_trigger(),
            basic::separator()
                .orientation(kivact::view::Orientation::Vertical)
                .style(style().height(16)),
            kivact::ui::breadcrumb(move || vec![
                kivact::interaction::ListItem::new("root", "Acme"),
                kivact::interaction::ListItem::new("current", page.get().to_string()),
            ])
            .separator(|| icon("chevron-right").style(style().size(14, 14))),
        ))
        .style(style().height(48).padding_x(12).gap(8).align_items(Align::Center)),
        kivact::ui::heading(move || page.get().to_string(), 4).style(style().padding(12)),
    ))
    .style(style().flex_grow(1));
    column(
        kivact::ui::sidebar_provider(open, sidebar, content)
            .collapsible(kivact::ui::SidebarCollapsible::Icon),
    )
    .style(
        style()
            .size(pct(100), pct(100))
            .background(basic::BACKGROUND)
            .color(basic::FOREGROUND),
    )
}
```

- プロバイダーは親を埋めます。枠の大きさが決まっているなら、親に幅と高さを与えてください。
- プロバイダーの `.collapsible(..)` は、閉じたときの動きを決めます。`Offcanvas`（既定）はサイドバーを画面の外へ滑らせ、`Icon` はラベルが吹き出しになったアイコンの 48 px の列に縮め、`None` は閉じられなくします。`.rail(true)` は、端に細い帯を足して開閉を切り替えます。
- `kivact::ui::sidebar_trigger()`、レール、Ctrl+B（macOS では Cmd+B）は、どれも `open` に書き込みます。ウィンドウのどこでもショートカットが効くようにするには、プロバイダーをアプリのルートのビューにしてください。
- `.side(kivact::ui::SidebarSide::Right)` はサイドバーを右に移し、`.frame(kivact::ui::SidebarFrame::Floating)` か `Inset` は枠の形を変えます。
- `.header(..)`、`.content(..)`、`.footer(..)` はビューを受け取ります。グループは `kivact::ui::sidebar_group(label)` で作り、`.content(..)` と任意の `.action(..)` を持たせます。グループの間は `kivact::ui::sidebar_separator()` で区切ります。
- サイドバーに検索欄を置くには `kivact::ui::sidebar_input(signal)` を使います。その `.name(..)` がラベルです。

## タブで表示を切り替える

タブは標準の `tabs(label, items, value)` です。`current` は選ばれたタブのキーを持ち、それは必ず選択肢のどれかです。`.content(|key| ..)` は、選ばれたキーのパネルを作ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn account() -> impl Element {
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

- 既定は箱のような見た目です。`.variant(basic::TabsVariant::Line)` は、下線の見た目にします。
- `.orientation(Orientation::Vertical)` は、タブをパネルの横に縦に並べます。
- 最初の引数はタブの一覧のアクセシブルな名前です。矢印キーでタブのあいだを移動できます。

## 利用者が今どこにいるかを示す

キー付きの `ListItem` を `kivact::ui::breadcrumb(items)` に渡します。最後の項目が現在のページです。`.on_activate(..)` は、前の項目のキーを受け取ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn trail() -> impl Element {
    let at = signal(String::from("Components"));
    kivact::ui::breadcrumb(move || vec![
        kivact::interaction::ListItem::new("home", "Home"),
        kivact::interaction::ListItem::new("components", "Components"),
        kivact::interaction::ListItem::new("current", at.get()),
    ])
    .on_activate(move |key| at.set(if key == "home" { "Home" } else { "Components" }.into()))
    .separator(|| icon("chevron-right").style(style().size(14, 14)))
}
```

隠れている階層を表す無効な項目には、`.item(..)` で省略記号を描きます。`.separator(..)` は装飾の山形を与えます。右から左へ書く環境では `chevron-left` を選んでください。

## メニューバーを追加する

`basic::menubar(menus).on_activate(choose)` は、`(タイトル, 項目)` の組の一覧を受け取り、タイトルを 1 行に表示します。それぞれが標準のメニューを開きます。`choose` は、選ばれた項目のキーを受け取ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn bar() -> impl Element {
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

項目は、どのメニューとも同じ `MenuEntry` の値です（項目、チェックと選択の項目、見出し、サブメニュー。[オーバーレイ](overlays.md#ボタンや右クリックからメニューを開く)を参照してください）。`Vec` の代わりにクロージャを渡すと、メニューがシグナルに従います。バーはシステムのメニューを奪いません。ウィンドウの中の普通の行です。

## ページの上部からセクションへリンクする

`kivact::ui::navigation_menu(label)` は、ナビゲーションの項目の行です。`navigation_menu(...).action(key, label, action)` は、押されると `action` を実行します。`navigation_menu(...).link(key, label, content)` は、項目の下にパネルを開きます。`content` はパネルのビューを、開くたびに作ります。パネルの中の行には `kivact::ui::navigation_menu_link(title)` を使います。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn top_nav() -> impl Element {
    let clicks = signal(0usize);
    let getting_started = move || {
        column((
            kivact::ui::navigation_menu_link("Introduction")
                .description("How the pieces fit together."),
            kivact::ui::navigation_menu_link("Installation")
                .description("Add the crates to your project.")
                .icon(icon("download")),
        ))
        .style(style().width(320))
    };
    column((
        kivact::ui::navigation_menu("Navigation")
            .link("getting-started", "Getting started", getting_started)
            .action("docs", "Docs", move || clicks.update(|n| *n += 1))
            .close_delay(std::time::Duration::ZERO),
        basic::muted(move || format!("Docs pressed: {}", clicks.get())),
    ))
    .style(style().gap(12))
}
```

## 長い一覧をページ送りする

`kivact::ui::pagination(count, page)` は、前へと次へのボタンとページ番号を表示します。多いときは省略記号を使います。`page` は 0 から始まるシグナル、`count` はページ数です。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn pager() -> impl Element {
    let page = signal(0usize);
    column((
        kivact::ui::pagination(12usize, page),
        basic::muted(move || format!("Page {} of 12", page.get() + 1)),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
```

- `.label_format(|number| format!("p.{number}"))` は、ページ番号の読み方を変えます。
- 表示するセルの中身を変えるには `.item(|cell| ..)` を使います。選択と移動は、ページのバインディングがこれまでどおり持ちます。
- 「Previous」「Next」の語と accessible label は `basic.pagination.*` のメッセージから来ます。差し替えられます。[テーマと変数](theme-and-tokens.md#フォントを選ぶ)を参照してください。
- 表の行をページ送りするなら、`data_table(..).page(page, size)` にページ送りが組み込まれています。[データを表示する](data-display.md)を参照してください。

## 関連

- [オーバーレイ](overlays.md): メニュー、ポップオーバー、ダイアログ。
- [データを表示する](data-display.md): 並べ替えとページ送りのあるテーブル。
- リファレンス: サイドバーの部品とナビゲーションの部品は `cargo doc -p kivact-ui --open`。

`basic::MenuVariant::Bar` は、Basic のメニューバーが使う標準のメニューの、公開されたコンパクトな見た目です。同じ見た目を直接組み立てるときに `.variant(..)` で渡します。
