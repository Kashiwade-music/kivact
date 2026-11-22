# ダイアログ、メニュー、ツールチップ、トーストを表示する

[English](../../en/how-to-guide/overlays.md)

オーバーレイは、画面の上に浮かぶ層です。Kivact の標準コンポーネントは、フォーカスの扱い、Escape、外側のクリックによる閉じる動作、アクセシビリティをそれぞれ標準で備えています。モーダルウィンドウとポップオーバーには `dialog`、操作メニューとコンテキストメニューには `menu`、ヒントには `tooltip`、通知には `toast_region` を使います。このガイドでは、それぞれの使い方を示します。

## ダイアログを表示する

ダイアログの開閉状態は、あなたが持つ `bool` です。`true` を書くと開き、ユーザーが閉じると、ダイアログが `false` を書きます。

```rust,test_harness
use kivact::interaction::DismissOn;
use kivact::prelude::*;
use kivact::ui::DialogKind;

fn app() -> impl IntoView {
    let open = signal(false);
    let deleted = signal(false);

    column((
        button("Delete note").on_press(move || open.set(true)),
        text(move || if deleted.get() { "Deleted" } else { "Kept" }),
        dialog("Delete this note?", open)
            .mode(DialogKind::AlertModal)
            .dismiss(DismissOn { escape: true, outside: false })
            .description("This cannot be undone.")
            .content(|| text("The note and its attachments will be removed."))
            .footer(move || {
                row((
                    button("Cancel")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("Delete")
                        .variant(ButtonVariant::Danger)
                        .on_press(move || {
                            deleted.set(true);
                            open.set(false);
                        }),
                ))
                .style(style().gap(8))
            }),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Notes").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_closes_the_dialog_and_returns_the_focus() {
        let screen = kivact::test::render(app);
        let opener = screen.get_by_role(Role::Button, "Delete note");
        opener.focus();
        opener.click();
        screen.get_by_role(Role::AlertDialog, "Delete this note?");

        screen.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::AlertDialog, "Delete this note?").is_none());
        assert!(opener.is_focused());

        opener.click();
        screen.get_by_role(Role::Button, "Delete").click();
        screen.get_by_text("Deleted");
    }
}
```

ダイアログが自動で行うことです。

- **フォーカス。** 開くとフォーカスがダイアログの中へ移ります(内容の最初のコントロール、または `.initial_focus(node_ref)` で指定したもの)。モーダルでは Tab が中に留まり、閉じるとフォーカスは開いたコントロールへ戻ります。
- **閉じる操作。** Escape と外側のクリックで閉じます。変えるには `.dismiss(Dismiss { escape, outside })` を使います。ヘッダーの閉じるボタンは既定で付き、`.closable(false)` で外せます。モーダルは、画面のほかの部分へのポインター入力とキーボード入力をふさぎます。
- **アクセシビリティ。** タイトルがダイアログのアクセシブルな名前と見出しになります。`.hide_title(true)` は名前を保ったまま見出しを描かず、`.description(..)` は説明を加えます。
- **幅。** `.width(480)` でテーマの幅を上書きします。

`DialogKind` で種類を選びます。

| 種類 | 見た目 | 外側のクリック |
| --- | --- | --- |
| `Modal`(既定) | 中央のダイアログ | 閉じる |
| `AlertModal` | 答えを求める中央のダイアログ(`Role::AlertDialog`) | `Dismiss` で許可しない限り閉じない |
| `Drawer(Edge::Right)` | 端に固定されたシート(`Left`、`Right`、`Top`、`Bottom`) | 閉じる |

### 内容は開くたびに作り直される

`.content(..)` と `.footer(..)` はファクトリーです。ダイアログが開くたびに、新しい owner の中で実行されます。ファクトリーの中で作った `signal` は、ダイアログが閉じると破棄されます。閉じても残したいもの(たとえばフォームの文字列)は、ファクトリーの**外**で作り、ハンドルを取り込みます。

```rust,no_run
use kivact::prelude::*;

#[component]
fn rename(name: Signal<String>) -> impl Element {
    let open = signal(false);
    let draft = signal(String::new()); // survives the dialog closing

    column((
        button("Rename").on_press(move || {
            draft.set(name.get());
            open.set(true);
        }),
        dialog("Rename", open)
            .content(move || text_input(draft).label("Name"))
            .footer(move || {
                button("Save").on_press(move || {
                    name.set(draft.get());
                    open.set(false);
                })
            }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Rename").run(|| rename(signal("Untitled".to_owned())))
}
```

### トリガーから開く、ポップオーバーにする

`.attach(trigger)` は、トリガーでダイアログを開くようにして、トリガーとダイアログをまとめて返します。トリガーの隣に出る非モーダルのパネルには、ポップアップの振る舞いを使います。`use_popup(open)` はトリガーに Part を与え、`surface(placement, content)` はパネルをトリガーの位置に置きます。もう一度トリガーを押すと閉じます。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::use_popup;
use kivact::view::Placement;

#[component]
fn help() -> impl Element {
    let open = signal(false);
    let popup = use_popup(open);
    row((
        button("Help").with_node(move |n| n.part(popup.trigger())),
        popup.surface(Placement::Below, || text("Press Tab to move between fields.")),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Help").run(help)
}
```

`Placement::Above` を `surface` に渡すと側を選べます(`Placement` は `kivact::view` にあります)。パネルが収まらないときは、Kivact が反対側へ反転します。

## ツールチップを付ける

`tooltip(text, trigger)` は、ポインターがトリガーの上で止まるか、トリガーにキーボードフォーカスがあるとき、少し遅れてヒントを表示します。Escape かポインターが離れると隠れます。一度に表示されるツールチップは 1 つだけです。トリガーがすでに説明を持っていなければ、ヒントは支援技術に対するトリガーの説明にもなります。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    row((
        tooltip("Save the document (Ctrl+S)", button("Save")),
        tooltip("Open another file", button("Open")),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tooltips").run(app)
}
```

ツールチップは短いヒントです。読ませるべき文章や、クリックさせるものには使わず、その場合はポップオーバーを使います。

## メニューを表示する

`menu(label, entries).on_activate(choose)` は、`label` という名前のボタンを描き、それが項目を開きます。`choose` は、ユーザーが選んだ項目の key を受け取ります。メニューは状態を持たないので、チェックマークはあなたのシグナルから作ります。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let show_grid = signal(true);
    let last = signal(String::from("nothing"));

    let entries = move || {
        vec![
            MenuEntry::item("new", "New").with_shortcut("Ctrl N"),
            MenuEntry::item("open", "Open"),
            MenuEntry::separator(),
            MenuEntry::checkbox("grid", "Show grid", show_grid.get()).keep_open(true),
            MenuEntry::submenu(
                "export",
                "Export",
                vec![MenuEntry::item("png", "PNG"), MenuEntry::item("svg", "SVG")],
            ),
            MenuEntry::item("delete", "Delete").destructive(true),
        ]
    };

    column((
        menu("File", entries).on_activate(move |key: &'static str| {
            if key == "grid" {
                show_grid.update(|on| *on = !*on);
            }
            last.set(key.to_owned());
        }),
        text(move || format!("last: {}", last.get())),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Menus").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_an_item_reports_its_key() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "File").click();
        screen.get_by_role(Role::MenuItem, "Open").click();
        screen.get_by_text("last: open");
    }
}
```

- 項目は `MenuEntry::item`、`checkbox(key, label, checked)`、`radio`、`submenu`、`separator`、`heading`、`group` で作ります。行の種類はコンストラクターが決めます。`.with_shortcut("Ctrl N")`(表示のみで、メニューはキーを割り当てません)、`.with_icon(..)`、`.with_description(..)`、`.disabled(bool)`、`.destructive(true)`、`.keep_open(true)` で修飾できます。項目の種類に当てはまらない修飾(区切り線への `shortcut`、サブメニューへの `keep_open` など)は使い方の誤りです。
- チェックマークやラベルがシグナルに依存するときは、上のように `entries` にクロージャを渡します。メニューが再び読み取ります。固定のメニューなら素の `Vec` で十分です。
- key はあなたのものです。`&'static str` か、`Clone + Eq + Hash` を満たす enum を使います。
- 矢印キー、Enter、Space、サブメニューのための右・左キー、Escape、先頭の文字を打つ操作がすべて使えます。項目を選ぶとメニューは閉じます。ただし、その項目に `.keep_open(true)` があれば開いたままです。

自分のコントロールからメニューを開くには `.attach(trigger)` を使います。右クリックまたはコンテキストメニューキーで開くには `.attach_context(target)` を使い、メニューはポインターの位置に現れます。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let entries = vec![
        MenuEntry::item("copy", "Copy"),
        MenuEntry::item("paste", "Paste"),
    ];
    menu("Edit", entries).on_activate(|_key: &'static str| {})
        .attach_context(node().child(text("Right-click me")).style(style().padding(48)))
}

fn main() -> kivact::Result<()> {
    App::new("Context menu").run(app)
}
```

メニューが開く位置は `.placement(..)`、`.align(..)`、`.offset(px)`、`.at_least_anchor_width()` で決めます。

## トーストを表示する

トーストは、自然に消える短い通知です。アプリのルート付近に `toast_region` を 1 つ置き、どこからでも `Toast` を push します。

```rust,test_harness
use std::time::Duration;

use kivact::interaction::{Tone, use_toaster};
use kivact::prelude::*;

fn app() -> impl IntoView {
    let toasts = use_toaster();
    let count = signal(0);

    column((
        button("Save").on_press(move || {
            toasts.push(
                Toast::new("Saved")
                    .body("Your changes were written to disk.")
                    .tone(Tone::Success),
            );
        }),
        button("Delete").on_press(move || {
            count.update(|n| *n += 1);
            toasts.push(
                Toast::new("Deleted")
                    .tone(Tone::Danger)
                    .duration(Some(Duration::from_secs(10)))
                    .action("Undo", move || count.update(|n| *n -= 1)),
            );
        }),
        text(move || format!("deleted: {}", count.get())),
        toast_region(toasts),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Toasts").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_toast_appears_and_its_action_runs() {
        let screen = kivact::test::render(app);
        screen.get_by_role(Role::Button, "Save").click();
        screen.find_by_text("Saved");

        screen.get_by_role(Role::Button, "Delete").click();
        screen.get_by_text("deleted: 1");
        screen.find_by_role(Role::Button, "Undo").click();
        screen.get_by_text("deleted: 0");
    }
}
```

- `use_toaster()` はアプリで共有される唯一のキューを返します。どのコンポーネントから呼んでも同じキューが得られます。ハンドラーへはクロージャで取り込んで渡します(`Copy` のハンドルです)。
- トーストは既定で 5 秒表示されます。`.duration(Some(d))` で変え、`.duration(None)` にするとユーザーが閉じるまで残ります。ポインターがトーストの上にあるか、フォーカスが中にある間は、時間が止まります。
- `Tone`(`Neutral`、`Info`、`Success`、`Warning`、`Danger`、`Accent`)はアイコンと読み上げ方を決めます。`Warning` と `Danger` はすぐに読み上げられ、ほかは控えめに読み上げられます。
- `.action(label, f)` は、`f` を実行してトーストを閉じるボタンを 1 つ加えます。
- 同時に表示されるのは最大 3 つです(`toasts.max_visible(n)` で変更)。残りは待機します。`toasts.dismiss(id)` で途中で消せます。`id` は `push` が返した値です。
- テーマが region をウィンドウの右下に置きます。region はアプリのルート付近に 1 つ置いてください。

## 関連項目

- [フォーム](forms.md): 未保存の編集をダイアログで確認する。
- [テーマ](theming.md): ダイアログ、メニュー、トーストの見た目を変える。
- [翻訳とロケール](i18n.md): 閉じるボタンなどの組み込みの文言を翻訳する。
- [再利用できるコンポーネントを書く](custom-components.md): `kivact::interaction` の部品と `kivact::view::overlay` で独自のオーバーレイを作る。
- リファレンス: `Dialog`、`Menu`、`Tooltip`、`ToastRegion` は `cargo doc -p kivact-ui --open`。
