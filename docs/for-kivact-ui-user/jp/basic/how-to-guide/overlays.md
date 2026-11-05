# ダイアログ、シート、ポップオーバー、メニュー、トーストを出す

[English](../../../en/basic/how-to-guide/overlays.md)

このガイドでは、ページの上に何かを重ねる方法を説明します。ダイアログ、確認、横からのシート、下からのドロワー、ポップオーバー、ホバーカード、メニュー、トーストです。[チュートリアル](../getting-started/README.md)を終えていて、アプリが `basic::theme` を使っていることを前提にしています。

これらの多くは、キット独自のコンポーネントではありません。ダイアログ、アラートダイアログ、シート、ポップオーバーは、標準の `dialog(title, open)` の `.kind(..)` を変えたもの、メニューは標準の `menu(..)`、トーストは標準の `toast_region` と `toaster` です。テーマがそれらを Lyra の見た目で描きます。スワイプドロワーも標準の `kivact::ui::dialog` を使います。ホバープレビューは標準の `kivact::ui::hover_card` を Basic の surface recipe で描きます。

すべてに共通する決まりが 2 つあります。

- **開閉の状態はアプリが持ちます。** ダイアログ、シート、ドロワーは、`signal(bool)` が `true` の間だけ表示されます。ボタンが `true` にし、中のボタンが `false` にします。Escape キーでも閉じ（このとき `false` が書かれます）、フォーカスは開いたコントロールへ戻ります。
- **中身は開くたびに作られます。** `.content(..)` と `.footer(..)` は、オーバーレイが開くときに実行される関数を取ります。閉じている間はコストがかからず、開き直すとまっさらな状態から始まります。残したい値は、クロージャの外の signal に持たせてください。

## ダイアログを表示する

```rust,no_run
use kivact::prelude::*;

fn edit_profile() -> impl IntoView {
    let open = signal(false);
    let name = signal(String::from("Ada"));
    column((
        button("Edit profile")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Edit profile", open)
            .description("Make changes to your profile here.")
            .content(move || text_input("Name", name))
            .footer(move || button("Save changes").on_press(move || open.set(false))),
    ))
}
```

ダイアログはモーダルです。閉じるまでフォーカスは中に留まり、最初は中身の最初のフォーカス可能なコントロール（ここでは入力欄）に当たります。角に閉じるボタンが出ます。取り除くには `.closable(false)` を使います。タイトルはダイアログの accessible name にもなります。`.hide_title(true)` は、名前を残したまま見出しを表示しません。

## 確認を求める

アラートダイアログは、種類が `AlertModal` のダイアログです。外側のクリックでは閉じないので、ユーザーはフッターのボタンのどれかで答える必要があります（Escape では取り消せます）。角の閉じるボタンは持たないので、`.closable(false)` を渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::DialogMode;
use kivact_ui_basic as basic;

fn delete_chat() -> impl IntoView {
    let open = signal(false);
    let deleted = signal(false);
    column((
        button("Delete chat")
            .variant(kivact::ui::ButtonVariant::Danger)
            .on_press(move || open.set(true)),
        basic::muted(move || if deleted.get() { "Chat deleted." } else { "Chat kept." }),
        dialog("Delete chat?", open)
            .mode(DialogMode::AlertModal)
            .closable(false)
            .description("This will permanently delete this conversation.")
            .footer(move || {
                (
                    button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Delete")
                        .variant(kivact::ui::ButtonVariant::Danger)
                        .on_press(move || {
                            deleted.set(true);
                            open.set(false);
                        }),
                )
            }),
    ))
    .style(style().gap(12))
}
```

フッターはボタンのタプルを返します。フォーカスは最初のボタンに当たるので、安全な選択肢（キャンセル）を先に置いてください。アラートダイアログのバリエーションは次のとおりです。

- `.variant(basic::AlertDialogSize::Sm)` は、幅 320 px の中央寄せの狭いダイアログにします。幅を分け合うには、フッターの 2 つのボタンに `style().grow(1).basis(0)` を付けます。
- `.media(|| icon("trash-2").style(style().size(24, 24)))` は、タイトルの脇にアイコンを表示します。

## 端からシートを開く

シートは、種類が `Drawer(side)` のダイアログです。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{DialogMode};
use kivact::geometry::{Edge};
use kivact_ui_basic as basic;

fn notifications() -> impl IntoView {
    let open = signal(false);
    column((
        button("Notifications")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Notifications", open)
            .mode(DialogMode::Drawer(Edge::Right))
            .description("Choose what you want to hear about.")
            .content(|| basic::muted("No new notifications."))
            .footer(move || button("Close").on_press(move || open.set(false))),
    ))
}
```

`Side` は `Left`、`Right`、`Top`、`Bottom` です。スワイプで閉じられる下からのパネルが欲しいときは、キットの `kivact::ui::dialog(title, open).mode(DialogMode::Drawer(Edge::Bottom))` を使います。`.description(..)`、`.content(..)`、`.footer(..)` と、`.mode(DialogMode::Drawer(Edge::..))`（既定は下）を同じように取ります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn move_goal() -> impl IntoView {
    let open = signal(false);
    let goal = signal(350);
    column((
        button("Open drawer")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        kivact::ui::dialog("Move goal", open)
            .mode(kivact::interaction::DialogMode::Drawer(kivact::geometry::Edge::Bottom))
            .variant(basic::DrawerVariant::Drawer)
            .closable(false)
            .description("Set your daily activity goal.")
            .content(move || text(move || format!("{} calories/day", goal.get())))
            .footer(move || {
                (
                    button("Submit").on_press(move || open.set(false)),
                    button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                )
            }),
    ))
}
```

`kivact::ui::dialog` と `kivact::ui::hover_card` は `NodeBuilder` ではありません。置いた場所には何も描かないので、`.style(..)` や `.test_id(..)` は使えません。

## ボタンにポップオーバーを付ける

ポップオーバーは、種類が `Popover` のダイアログです。`.attach(trigger)` は、ポップオーバーをトリガーの位置に置き、トリガーに開閉の役目を与えます。戻り値は、トリガーとポップオーバーをまとめたもので、ボタンを置きたい場所にそのまま置きます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::DialogMode;
use kivact::view::{OverlayAlign, Placement};

fn dimensions() -> impl IntoView {
    let open = signal(false);
    let width = signal(String::from("100%"));
    dialog("Dimensions", open)
        .mode(DialogMode::Popover)
        .description("Set the dimensions for the layer.")
        .placement(Placement::Below)
        .align(OverlayAlign::Start)
        .content(move || text_input("Width", width))
        .attach(button("Open popover").variant(kivact::ui::ButtonVariant::Outline))
}
```

- `.placement(..)` はトリガーのどちら側か（`Above`、`Below`、`Start`、`End`）を、`.align(..)` は揃え方（`Start`、`Center`、`End`）を決めます。ウィンドウに収まらないときは、反対側に反転します。
- ポップオーバーはモーダルではありません。Escape キー、外側のクリック、フォーカスがポップオーバーとトリガーの両方から外れたときに閉じます。
- `.hide_title(true)` は、ちょっとした文章だけのポップオーバーに向きます。タイトルは accessible name として残ります。
- トリガーへの修飾（`.style(..)`、`.test_id(..)`）は、`attach` の戻り値ではなく、渡すボタンに付けます。

## ホバーしたときにプレビューを出す

`kivact::ui::hover_card` は、ポインターがトリガーの上で止まると開きます。600 ms 後に開き、ポインターが離れて 300 ms 後に閉じます。`.open_delay(..)` と `.close_delay(..)` で変えられます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn mention() -> impl IntoView {
    kivact::ui::hover_card(|| {
        column((
            text("@kivact").style(style().weight(600)),
            basic::muted("A native GUI framework for Rust."),
        ))
        .style(style().gap(8))
    })
    .attach(button("@kivact").variant(kivact::ui::ButtonVariant::Link))
}
```

ポップオーバーと同じく `.placement(..)` と `.align(..)` を取ります。これはプレビュー用なので、ユーザーが操作する必要のある内容にはポップオーバーを使ってください。

## ボタンや右クリックからメニューを開く

メニューは標準の `menu(label, entries).on_activate(choose)` です。`entries` は `MenuEntry` の一覧で、`choose` はユーザーが選んだ項目の key を受け取ります。`.attach(trigger)` はボタンから、`.attach_context(target)` は `target` の上の右クリックで開きます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn account_menu() -> impl IntoView {
    let picked = signal(String::from("Nothing chosen yet."));
    let entries = vec![
        MenuEntry::heading("My account"),
        MenuEntry::item("profile", "Profile").with_icon("user").with_shortcut("Ctrl+P"),
        MenuEntry::item("settings", "Settings").with_icon("settings").disabled(true),
        MenuEntry::separator(),
        MenuEntry::item("logout", "Log out").with_icon("log-out").destructive(true),
    ];
    column((
        menu("Account", entries).on_activate(move |key: &'static str| {
            picked.set(format!("Chosen: {key}"))
        })
        .at_least_anchor_width()
        .attach(button("Account").variant(kivact::ui::ButtonVariant::Outline)),
        basic::muted(picked),
    ))
    .style(style().gap(12))
}

fn context_menu() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Notes")
        .focusable(true)
        .style(style().width(240).height(96).border(1, basic::ColorName::Border))
        .child(text("Right click here"));
    menu("Notes menu", vec![MenuEntry::item("copy", "Copy"), MenuEntry::item("paste", "Paste")]).on_activate(|_key: &'static str| {})
    .attach_context(area)
}
```

- 項目の種類は `item`、`checkbox(key, label, checked)`、`radio`、`heading`（見出し）、`separator`、`submenu(key, label, entries)` です。項目には `.icon(name)`、`.shortcut(text)`、`.disabled(true)`、`.destructive(true)` を付けられます。項目の種類に当てはまらない修飾は使い方の誤りです。
- メニューは状態を持ちません。チェック項目やラジオ項目では、`entries` に signal を読むクロージャを渡し、`.keep_open(true)` を付けて、選んでもメニューが閉じないようにします。`choose` のクロージャが signal を更新し、次の一覧に新しい印が反映されます。

```rust,no_run
use kivact::prelude::*;

fn view_menu() -> impl IntoView {
    let status = signal(true);
    let position = signal("bottom");
    let entries = move || {
        vec![
            MenuEntry::heading("Appearance"),
            MenuEntry::checkbox("status", "Status bar", status.get()).keep_open(true),
            MenuEntry::separator(),
            MenuEntry::radio("top", "Top", position.get() == "top").keep_open(true),
            MenuEntry::radio("bottom", "Bottom", position.get() == "bottom").keep_open(true),
        ]
    };
    menu("View", entries).on_activate(move |key: &'static str| match key {
        "status" => status.update(|on| *on = !*on),
        other => position.set(other),
    })
    .attach(button("View").variant(kivact::ui::ButtonVariant::Outline))
}
```

- ウィンドウの上端のメニューバーは `basic::menubar(menus).on_activate(choose)` です。[ナビゲーション](navigation.md)を参照してください。

## トーストを出す

トーストの領域は、アプリのルート付近に一度だけマウントします。あとは、どのハンドラからでもトーストを積めます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{Toast, Tone, toaster};
use kivact::ui::{toast_region};

fn root() -> impl IntoView {
    column((
        button("Create event")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(|| {
                toaster().push(
                    Toast::new("Event has been created")
                        .tone(Tone::Success)
                        .body("Sunday, December 03, 2023 at 9:00 AM"),
                );
            }),
        toast_region(toaster()).hold_all_on_hover(true),
    ))
}
```

- `.tone(..)` は `Neutral`（アイコンなし）、`Success`、`Info`、`Warning`、`Danger` です。テーマがトーンのアイコンを描きます。`Warning` と `Danger` は、スクリーンリーダーにより緊急のものとして通知されます。
- トーストは 5 秒で閉じます。`.duration(Some(Duration::from_secs(2)))` で変え、`.duration(None)` にすると、閉じられるまで残ります。`push` は id を返し、`toaster().dismiss(id)` でそのトーストを消せます。
- `.action("Undo", move || ..)` はボタンを足します。押すとクロージャが実行され、トーストが閉じます。クロージャはトーストが所有するので、押下のハンドラの中で作って積めます。
- 処理中を示すトーストには、`.icon(|| kivact::ui::spinner())` と `.duration(None)` を使い、処理が終わったら閉じます。
- 複数のトーストは、最新のものの後ろに最大 3 つまで重なります。ポインターが乗るか、フォーカスが入ると一覧に開きます。`hold_all_on_hover(true)` を付けると、開いている間はすべてのトーストの時間が止まります。

## 関連項目

- [フォーム](forms.md): ダイアログの中身を field にし、閉じる前に検証する。
- [ナビゲーション](navigation.md): メニューバー、ナビゲーションメニュー、サイドバー。
- [テーマとトークン](theme-and-tokens.md): オーバーレイがテーマの切り替えに追従する仕組み。
- rustdoc: `drawer` と `hover_card` は `cargo doc -p kivact-ui-basic --open`、`dialog`、`menu`、`Toast` は `cargo doc -p kivact-ui --open`。
