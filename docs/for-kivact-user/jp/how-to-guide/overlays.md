# ダイアログ、メニュー、ツールチップ、トーストを表示する

[English](../../en/how-to-guide/overlays.md)

オーバーレイは、画面の上に浮かぶ層です。Kivact の標準コンポーネントは、フォーカスの扱い、Escape、外側のクリックによる閉じる動作、アクセシビリティをそれぞれ標準で備えています。モーダルウィンドウとポップオーバーには `dialog`、操作メニューとコンテキストメニューには `menu`、ヒントには `tooltip`、通知には `toast_region` を使います。このガイドでは、それぞれの使い方を示します。

## ダイアログを表示する

ダイアログの開閉状態は、あなたが持つ `bool` です。`true` を書くと開き、ユーザーが閉じると、ダイアログが `false` を書きます。

```rust,test_harness
use kivact::prelude::*;
use kivact::ui::{DialogKind, Dismiss};

fn app() -> impl IntoView {
    let open = signal(false);
    let deleted = signal(false);

    column((
        button("メモを削除").on_press(move || open.set(true)),
        text(move || if deleted.get() { "削除しました" } else { "残っています" }),
        dialog("このメモを削除しますか?", open)
            .kind(DialogKind::AlertModal)
            .dismiss(Dismiss { escape: true, outside: false })
            .description("この操作は元に戻せません。")
            .content(|| text("メモと添付ファイルが削除されます。"))
            .footer(move || {
                row((
                    button("キャンセル")
                        .variant(ButtonVariant::Outline)
                        .on_press(move || open.set(false)),
                    button("削除する")
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
        let opener = screen.get_by_role(Role::Button, "メモを削除");
        opener.focus();
        opener.click();
        screen.get_by_role(Role::AlertDialog, "このメモを削除しますか?");

        screen.press_key(Key::Escape);
        assert!(screen.query_by_role(Role::AlertDialog, "このメモを削除しますか?").is_none());
        assert!(opener.is_focused());

        opener.click();
        screen.get_by_role(Role::Button, "削除する").click();
        screen.get_by_text("削除しました");
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
| `Drawer(Side::Right)` | 端に固定されたシート(`Left`、`Right`、`Top`、`Bottom`) | 閉じる |
| `Popover` | トリガーの隣に出るモーダルでないパネル | 閉じる |

### 内容は開くたびに作り直される

`.content(..)` と `.footer(..)` はファクトリーです。ダイアログが開くたびに、新しい owner の中で実行されます。ファクトリーの中で作った `signal` は、ダイアログが閉じると破棄されます。閉じても残したいもの(たとえばフォームの文字列)は、ファクトリーの**外**で作り、ハンドルを取り込みます。

```rust,no_run
use kivact::prelude::*;

fn rename(name: Signal<String>) -> impl IntoView {
    let open = signal(false);
    let draft = signal(String::new()); // ダイアログが閉じても残る

    column((
        button("名前を変更").on_press(move || {
            draft.set(name.get());
            open.set(true);
        }),
        dialog("名前の変更", open)
            .content(move || text_input("名前", draft))
            .footer(move || {
                button("保存").on_press(move || {
                    name.set(draft.get());
                    open.set(false);
                })
            }),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Rename").run(|| rename(signal("無題".to_owned())))
}
```

### トリガーから開く、ポップオーバーにする

`.attach(trigger)` は、トリガーでダイアログを開くようにして、トリガーとダイアログをまとめて返します。`Popover` には必須です。パネルはトリガーの位置に出て、もう一度トリガーを押すと閉じます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::DialogKind;

fn help() -> impl IntoView {
    let open = signal(false);
    dialog("ヘルプ", open)
        .kind(DialogKind::Popover)
        .content(|| text("Tab で入力欄を移動できます。"))
        .attach(button("ヘルプ"))
}

fn main() -> kivact::Result<()> {
    App::new("Help").run(help)
}
```

側と寄せ方は `.placement(Placement::Above)` と `.align(OverlayAlign::End)`(どちらも `kivact::view`)で選びます。パネルが収まらないときは、Kivact が反対側へ反転します。

## ツールチップを付ける

`tooltip(text).attach(trigger)` は、ポインターがトリガーの上で止まるか、トリガーにキーボードフォーカスがあるとき、少し遅れてヒントを表示します。Escape かポインターが離れると隠れます。一度に表示されるツールチップは 1 つだけです。トリガーがすでに説明を持っていなければ、ヒントは支援技術に対するトリガーの説明にもなります。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    row((
        tooltip("文書を保存します (Ctrl+S)").attach(button("保存")),
        tooltip("別のファイルを開きます").attach(button("開く")),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Tooltips").run(app)
}
```

ツールチップは短いヒントです。読ませるべき文章や、クリックさせるものには使わず、その場合はポップオーバーを使います。`.placement(..)`、`.align(..)`、`.offset(px)` で位置を決め、`.content(|| view)` で素のテキストを自分の view に置き換えられます。

## メニューを表示する

`menu(label, entries, choose)` は、`label` という名前のボタンを描き、それが項目を開きます。`choose` は、ユーザーが選んだ項目の key を受け取ります。メニューは状態を持たないので、チェックマークはあなたのシグナルから作ります。

```rust,test_harness
use kivact::prelude::*;

fn app() -> impl IntoView {
    let show_grid = signal(true);
    let last = signal(String::from("なし"));

    let entries = move || {
        vec![
            MenuEntry::item("new", "新規").shortcut("Ctrl N"),
            MenuEntry::item("open", "開く"),
            MenuEntry::separator(),
            MenuEntry::checkbox("grid", "グリッドを表示", show_grid.get()).keep_open(true),
            MenuEntry::submenu(
                "export",
                "書き出し",
                vec![MenuEntry::item("png", "PNG"), MenuEntry::item("svg", "SVG")],
            ),
            MenuEntry::item("delete", "削除").destructive(true),
        ]
    };

    column((
        menu("ファイル", entries, move |key: &'static str| {
            if key == "grid" {
                show_grid.update(|on| *on = !*on);
            }
            last.set(key.to_owned());
        }),
        text(move || format!("最後に選んだ項目: {}", last.get())),
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
        screen.get_by_role(Role::Button, "ファイル").click();
        screen.get_by_role(Role::MenuItem, "開く").click();
        screen.get_by_text("最後に選んだ項目: open");
    }
}
```

- 項目は `MenuEntry::item`、`checkbox(key, label, checked)`、`radio`、`submenu`、`separator`、`heading`、`group` で作ります。行の種類はコンストラクターが決めます。`.shortcut("Ctrl N")`(表示のみで、メニューはキーを割り当てません)、`.icon(..)`、`.description(..)`、`.disabled(bool)`、`.destructive(true)`、`.keep_open(true)` で修飾できます。項目の種類に当てはまらない修飾(区切り線への `shortcut`、サブメニューへの `keep_open` など)は使い方の誤りです。
- チェックマークやラベルがシグナルに依存するときは、上のように `entries` にクロージャを渡します。メニューが再び読み取ります。固定のメニューなら素の `Vec` で十分です。
- key はあなたのものです。`&'static str` か、`Clone + Eq + Hash` を満たす enum を使います。
- 矢印キー、Enter、Space、サブメニューのための右・左キー、Escape、先頭の文字を打つ操作がすべて使えます。項目を選ぶとメニューは閉じます。ただし、その項目に `.keep_open(true)` があれば開いたままです。

自分のコントロールからメニューを開くには `.attach(trigger)` を使います。右クリックまたはコンテキストメニューキーで開くには `.attach_context(target)` を使い、メニューはポインターの位置に現れます。

```rust,no_run
use kivact::prelude::*;

fn app() -> impl IntoView {
    let entries = vec![
        MenuEntry::item("copy", "コピー"),
        MenuEntry::item("paste", "貼り付け"),
    ];
    menu("編集", entries, |_key: &'static str| {})
        .attach_context(node().child(text("右クリックしてください")).style(style().padding(48)))
}

fn main() -> kivact::Result<()> {
    App::new("Context menu").run(app)
}
```

メニューが開く位置は `.placement(..)`、`.align(..)`、`.offset(px)`、`.at_least_trigger_width(true)` で決めます。

## トーストを表示する

トーストは、自然に消える短い通知です。アプリのルート付近に `toast_region` を 1 つ置き、どこからでも `Toast` を push します。

```rust,test_harness
use std::time::Duration;

use kivact::prelude::*;
use kivact::ui::{Toast, Tone, toast_region, toaster};

fn app() -> impl IntoView {
    let toaster = toaster();
    let count = signal(0);

    column((
        button("保存").on_press(move || {
            toaster.push(
                Toast::new("保存しました")
                    .body("変更をディスクへ書き込みました。")
                    .tone(Tone::Success),
            );
        }),
        button("削除").on_press(move || {
            count.update(|n| *n += 1);
            toaster.push(
                Toast::new("削除しました")
                    .tone(Tone::Danger)
                    .duration(Some(Duration::from_secs(10)))
                    .action("元に戻す", move || count.update(|n| *n -= 1)),
            );
        }),
        text(move || format!("削除した数: {}", count.get())),
        toast_region(toaster),
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
        screen.get_by_role(Role::Button, "保存").click();
        screen.find_by_text("保存しました");

        screen.get_by_role(Role::Button, "削除").click();
        screen.get_by_text("削除した数: 1");
        screen.find_by_role(Role::Button, "元に戻す").click();
        screen.get_by_text("削除した数: 0");
    }
}
```

- `toaster()` はアプリで共有される唯一のキューを返します。どのコンポーネントから呼んでも同じキューが得られます。ハンドラーへはクロージャで取り込んで渡します(`Copy` のハンドルです)。
- トーストは既定で 5 秒表示されます。`.duration(Some(d))` で変え、`.duration(None)` にするとユーザーが閉じるまで残ります。ポインターがトーストの上にあるか、フォーカスが中にある間は、時間が止まります。
- `Tone`(`Neutral`、`Info`、`Success`、`Warning`、`Danger`、`Accent`)はアイコンと読み上げ方を決めます。`Warning` と `Danger` はすぐに読み上げられ、ほかは控えめに読み上げられます。
- `.action(label, f)` は、`f` を実行してトーストを閉じるボタンを 1 つ加えます。
- 同時に表示されるのは最大 3 つです(`toaster.max_visible(n)` で変更)。残りは待機します。`toaster.dismiss(id)` で途中で消せます。`id` は `push` が返した値です。
- テーマが region をウィンドウの右下に置きます。region はアプリのルート付近に 1 つ置いてください。

## 関連項目

- [フォーム](forms.md): 未保存の編集をダイアログで確認する。
- [テーマ](theming.md): ダイアログ、メニュー、トーストの見た目を変える。
- [翻訳とロケール](i18n.md): 閉じるボタンなどの組み込みの文言を翻訳する。
- [再利用できるコンポーネントを書く](custom-components.md): `kivact::interaction` の部品と `kivact::view::overlay` で独自のオーバーレイを作る。
- リファレンス: `Dialog`、`Menu`、`Tooltip`、`ToastRegion` は `cargo doc -p kivact-ui --open`。
