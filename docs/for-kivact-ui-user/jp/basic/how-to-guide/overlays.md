# ダイアログ、シート、ポップオーバー、メニュー、トーストを出す

[English](../../../en/basic/how-to-guide/overlays.md)

この手引きでは、ページの上に何かを重ねる方法を説明します。ダイアログ、確認、横のシート、下のドロワー、ポップオーバー、ホバーカード、メニュー、トーストです。[チュートリアル](../getting-started/README.md)を終えていて、アプリが `basic::theme` を使っていることを前提にしています。

これらのほとんどは、キット固有の部品ではありません。ダイアログ、アラートのダイアログ、シートは、`.mode(..)` を変えた標準の `dialog(title, open)` です。ポップオーバーは `use_popup` の面で、メニューは標準の `menu(..)`、トーストは標準の `toast_region` と `use_toaster` です。テーマが Lyra の見た目で描きます。スワイプできるドロワーもこの標準のダイアログを使います。ホバーのプレビューは、Basic の面の recipe を使う標準の `kivact::ui::hover_card` です。

これらすべてに共通する 2 つの規則があります。

- **開いた状態はアプリが持ちます。** ダイアログ、シート、ドロワーは、`signal(bool)` が `true` のあいだ表示されます。ボタンがそれを `true` にし、中のボタンが `false` にします。Escape でも閉じ（`false` を書き込み）、フォーカスは、それを開いたコントロールに戻ります。
- **中身は開くたびに作ります。** `.content(..)` と `.footer(..)` は、重ねるものが開くときに実行される関数を受け取ります。そのため、閉じているものは何もコストがかからず、開き直すと新しく始まります。残すべきものは、クロージャの外のシグナルに持たせてください。

## ダイアログを出す

ダイアログはモーダルです。閉じるまでフォーカスは中に留まり、中身の最初にフォーカスできるコントロール（ここでは入力欄）から始まります。隅に閉じるボタンを描きます。`.closable(false)` で取り除けます。タイトルはダイアログのアクセシブルな名前でもあります。`.hide_title(true)` は名前を残したまま、見出しを表示しません。

```rust,no_run
use kivact::prelude::*;

#[component]
fn edit_profile() -> impl Element {
    let open = signal(false);
    let name = signal(String::from("Ada"));
    column((
        button("Edit profile")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Edit profile", open)
            .description("Make changes to your profile here.")
            .content(move || text_input(name).label("Name"))
            .footer(move || button("Save changes").on_press(move || open.set(false))),
    ))
}
```

## 確認を求める

アラートのダイアログは、種類 `AlertModal` のダイアログです。外をクリックしても閉じないので、利用者は末尾のボタンのどれかで答えなければなりません（Escape ではまだ取り消せます）。隅に閉じるボタンはないので、`.closable(false)` を渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact_ui_basic as basic;

#[component]
fn delete_chat() -> impl Element {
    let open = signal(false);
    let deleted = signal(false);
    column((
        button("Delete chat")
            .variant(kivact::ui::ButtonVariant::Danger)
            .on_press(move || open.set(true)),
        basic::muted(move || if deleted.get() { "Chat deleted." } else { "Chat kept." }),
        dialog("Delete chat?", open)
            .mode(DialogKind::AlertModal)
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

末尾はボタンのタプルを返します。フォーカスは最初のボタンから始まるので、安全な選択（Cancel）を先に置いてください。アラートのダイアログの variant は次のとおりです。

- `.variant(basic::AlertDialogSize::Sm)` は、幅の狭い、中央に置かれた 320 px のダイアログにします。幅を分け合うには、末尾の両方のボタンに `style().flex_grow(1).flex_basis(0)` を与えます。
- `.media(|| icon("trash-2").style(style().size(24, 24)))` は、タイトルの横にアイコンを表示します。

## シートを端から出す

シートは、種類 `Drawer(side)` のダイアログです。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::DialogKind;
use kivact::geometry::{Edge};
use kivact_ui_basic as basic;

#[component]
fn notifications() -> impl Element {
    let open = signal(false);
    column((
        button("Notifications")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        dialog("Notifications", open)
            .mode(DialogKind::Drawer(Edge::Right))
            .description("Choose what you want to hear about.")
            .content(|| basic::muted("No new notifications."))
            .footer(move || button("Close").on_press(move || open.set(false))),
    ))
}
```

`Edge` は `Left`、`Right`、`Top`、`Bottom` のどれかです。利用者がスワイプして閉じられる下のパネルには、代わりにキットの `kivact::ui::dialog(title, open).mode(DialogKind::Drawer(Edge::Bottom))` を使います。同じ `.description(..)`、`.content(..)`、`.footer(..)` を受け取り、`.mode(DialogKind::Drawer(Edge::..))` の既定は下です。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn move_goal() -> impl Element {
    let open = signal(false);
    let goal = signal(350);
    column((
        button("Open drawer")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(move || open.set(true)),
        kivact::ui::dialog("Move goal", open)
            .mode(kivact::ui::DialogKind::Drawer(kivact::geometry::Edge::Bottom))
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

`kivact::ui::dialog` と `kivact::ui::hover_card` は `Element` ではありません。置いた場所では何も描かないので、`.style(..)` と `.test_id(..)` は効きません。

## ポップオーバーをボタンに付ける

ポップオーバーは、ページのノードに固定された面です。`use_popup(open)` は `open` のシグナルで開閉し、`.anchor(node)` は、そこから下がるノードを指定し、`surface(placement, content)` が面を作ります。トリガーには `.node_ref(..)` でノードを結び付けます。

```rust,no_run
use kivact::interaction::use_popup;
use kivact::prelude::*;
use kivact::view::Placement;

#[component]
fn dimensions() -> impl Element {
    let open = signal(false);
    let width = signal(String::from("100%"));
    let anchor = node_ref();
    let popup = use_popup(open).anchor(anchor);
    column((
        button("Open popover")
            .variant(kivact::ui::ButtonVariant::Outline)
            .node_ref(anchor)
            .on_press(move || open.set(true)),
        popup.surface(Placement::Below, move || {
            column((
                kivact::ui::heading("Dimensions", 4),
                text_input(width).label("Width"),
            ))
            .style(style().gap(8).width(240))
        }),
    ))
}
```

- `surface` に渡す `Placement`（`Above`、`Below`、`Start`、`End`）は、アンカーのどの側に出すかを決めます。ウィンドウに収まらないときは、反対側に回ります。
- ポップオーバーはモーダルではありません。Escape、外のクリック、フォーカスがそれとトリガーの両方から離れたときに閉じます。

## ホバーで中身を見せる

`kivact::ui::hover_card` は、ポインターがトリガーの上に留まると開きます。600 ms で開き、ポインターが離れて 300 ms 後に閉じます。これらは `.open_delay(..)` と `.close_delay(..)` で変えられます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn mention() -> impl IntoView {
    kivact::ui::hover_card(|| {
        column((
            text("@kivact").style(style().font_weight(600)),
            basic::muted("A native GUI framework for Rust."),
        ))
        .style(style().gap(8))
    })
    .attach(button("@kivact").variant(kivact::ui::ButtonVariant::Link))
}
```

ポップオーバーと同じく、`.placement(..)` と `.align(..)` を受け取ります。これはプレビューなので、利用者が操作する必要があるものにはポップオーバーを使ってください。

## ボタンや右クリックからメニューを開く

メニューは標準の `menu(label, entries).on_activate(choose)` です。`entries` は `MenuEntry` の一覧で、`choose` は利用者が選んだ項目のキーを受け取ります。`.attach(trigger)` はボタンから開き、`.attach_context(target)` は `target` の上での右クリックで開きます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

#[component]
fn account_menu() -> impl Element {
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

#[component]
fn context_menu() -> impl IntoView {
    let area = node()
        .role(Role::Group)
        .name("Notes")
        .focusable(true)
        .style(style().width(240).height(96).border(1, basic::BORDER))
        .child(text("Right click here"));
    menu("Notes menu", vec![MenuEntry::item("copy", "Copy"), MenuEntry::item("paste", "Paste")]).on_activate(|_key: &'static str| {})
    .attach_context(area)
}
```

- 項目の種類: `item`、`checkbox(key, label, checked)`、`radio`、`heading`（ラベル）、`separator`、`submenu(key, label, entries)`。項目は `.icon(name)`、`.shortcut(text)`、`.disabled(true)`、`.destructive(true)` を持てます。項目の種類に合わない修飾子は、使い方の誤りです。
- メニューは状態を持ちません。チェックと選択の項目では、`entries` をシグナルを読むクロージャにし、選んでもメニューが閉じないように `.keep_open(true)` を足します。`choose` のクロージャがシグナルを更新すると、次の一覧が新しい印を表示します。

```rust,no_run
use kivact::prelude::*;

#[component]
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

- ウィンドウの上部のメニューバーは `basic::menubar(menus).on_activate(choose)` です。[ナビゲーション](navigation.md)を参照してください。

## トーストを出す

トーストのリージョンは、アプリのルートの近くに一度だけ配置します。そうすると、どのハンドラーからもトーストを push できます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{Toast, Tone, use_toaster};
use kivact::ui::{toast_region};

#[component]
fn root() -> impl Element {
    column((
        button("Create event")
            .variant(kivact::ui::ButtonVariant::Outline)
            .on_press(|| {
                use_toaster().push(
                    Toast::new("Event has been created")
                        .tone(Tone::Success)
                        .body("Sunday, December 03, 2023 at 9:00 AM"),
                );
            }),
        toast_region(use_toaster()).hold_all_on_hover(true),
    ))
}
```

- `.tone(..)` は `Neutral`（アイコンなし）、`Success`、`Info`、`Warning`、`Danger` です。テーマが色調のアイコンを描き、`Warning` と `Danger` は、スクリーンリーダーにより緊急に読み上げられます。
- トーストは 5 秒後に閉じます。`.duration(Some(Duration::from_secs(2)))` で変えられ、`.duration(None)` は消されるまで残します。`push` は id を返します。`use_toaster().dismiss(id)` で、そのトーストを消せます。
- `.action("Undo", move || ..)` はボタンを足します。押すとクロージャが実行され、トーストが消えます。クロージャはトーストのものなので、押したハンドラーがトーストを組み立てて push できます。
- 作業の進行を示すトーストには、`.duration(None)` と一緒に `.icon(|| kivact::ui::spinner())` を使い、作業が終わったら消します。
- 複数のトーストは、最新のものの後ろに、最大 3 つまで重なります。ポインターが載っているか、フォーカスが中にあると、一覧に開きます。`hold_all_on_hover(true)` を指定すると、開いているあいだは、すべてのトーストの時間が止まります。

## 関連

- [フォームを作る](forms.md): ダイアログの中身を欄に入れ、閉じる前に検証します。
- [ナビゲーション](navigation.md): メニューバー、ナビゲーションメニュー、サイドバー。
- [テーマと変数](theme-and-tokens.md): 重なる部品がテーマの切り替えに追従する仕組み。
- リファレンス: `drawer` と `hover_card` は `cargo doc -p kivact-ui-basic --open`、`dialog`、`menu`、`Toast` は `cargo doc -p kivact-ui --open`。
