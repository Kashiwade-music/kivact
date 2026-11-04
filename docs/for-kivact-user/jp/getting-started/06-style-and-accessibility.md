# ステップ 6: スタイルとアクセシビリティ

[English](../../en/getting-started/06-style-and-accessibility.md)

このステップでは、アプリに見た目を付けます。大きなタイトル、控えめな色の補足テキスト、一覧を囲むカード、ボタンの種類を加えます。さらに、すべてのコントロールに支援技術が読める名前を付け、ダークモードのスイッチを加えます。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;
use kivact::style::provide_theme;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

fn header(dark: Signal<bool>) -> impl IntoView {
    row((
        image(LOGO).name("タスクのロゴ").style(style().size(32, 32)),
        column((
            text("タスク").role(Role::Heading).style(style().font_size(28).weight(700)),
            text("今日やること").style(style().color(ColorToken::TextMuted)),
        ))
        .style(style().gap(2).grow(1)),
        switch("ダークモード", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

fn task_row(tasks: Signal<Vec<Task>>, task: Binding<Task>) -> impl IntoView {
    let id = task.get_untracked().id;
    let done = bind!(task.done);
    row((
        checkbox(move || task.get().title, done),
        button("削除")
            .name(move || format!("{} を削除", task.get().title))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

fn task_list(tasks: Signal<Vec<Task>>) -> impl IntoView {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("タスク一覧")
    .style(
        style()
            .gap(8)
            .padding(12)
            .radius(8)
            .background(ColorToken::SurfaceAlt)
            .border(1, ColorToken::Border),
    )
}

fn tasks_screen(dark: Signal<bool>) -> impl IntoView {
    let tasks = signal(vec![
        Task { id: 1, title: "牛乳を買う".to_owned(), done: false },
        Task { id: 2, title: "チュートリアルを書く".to_owned(), done: false },
        Task { id: 3, title: "散歩に行く".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    };

    column((
        header(dark),
        row((
            text_input("新しいタスク", draft)
                .placeholder("やることを入力")
                .on_submit(add)
                .style(style().width(240)),
            button("追加")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("やることはありません。上の欄からタスクを追加してください。")),
        text(move || format!("残り {} 件", left.get())).style(style().color(ColorToken::TextMuted)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(ColorToken::Surface)
            .color(ColorToken::Text),
    )
}

fn app() -> impl IntoView {
    let dark = signal(false);
    let theme = memo(move || if dark.get() { Theme::dark() } else { Theme::light() });
    provide_theme(theme, move || tasks_screen(dark))
}

fn main() -> kivact::Result<()> {
    App::new("タスク").run(app)
}
```

## 新しく出てきたもの

**スタイル。** `style()` には、文字、色、大きさ、枠線のためのメソッドがあります。

- `font_size(28)` と `weight(700)` は、その view とその下にあるすべての文字の大きさと太さを決めます。`weight` は CSS と同じ数値で、400 が標準、700 が太字です。
- `color(..)` は文字の色を決めます。`background(..)`、`border(1, ..)`、`radius(8)` は、箱そのもののスタイルです。
- `grow(1)` は、`row` の余った幅をその view に割り当てます。ここでは、スイッチが右端に寄ります。`align_items(Align::Center)` と `justify_content(Justify::SpaceBetween)` は、子の位置をそろえます。

**テーマの色。** `ColorToken::Surface`、`Text`、`TextMuted`、`SurfaceAlt`、`Border`、`Accent` などは色の*名前*で、実際にどんな色になるかは、そのときのテーマが決めます。ほとんどの色にはトークンを使うと、同じコードがライトテーマでもダークテーマでも通用します。固定の色が必要なときは、`rgb(0x336699)` を使います。

**ボタンの種類。** ボタンは、既定では主要な操作です (`Primary`、アクセントカラーで塗りつぶし)。`.variant(kivact::ui::ButtonVariant::Ghost)` を付けると、ホバーするまで塗りのない控えめな見た目になるので、削除ボタンに使っています。ほかの種類は、`Secondary`、`Outline`、`Danger`、`Link` です。

**アクセシブルな名前。** スクリーンリーダーはコントロールを役割と名前で読み上げ、キーボード操作やテストのツールも、同じ方法でコントロールを見つけます。そのため、すべてのコントロールに名前が必要です。

- 文字から作るコントロールは、すでに名前を持っています。ボタンのラベル、チェックボックスの名前、テキスト欄の名前です。
- `.name(..)` は、名前を設定または上書きします。削除ボタンはどれも「削除」と読まれてしまうので、それぞれ「牛乳を買う を削除」のように、タスクの名前を含めています。名前にはクロージャを渡せるので、タイトルの変更に追従します。
- 画像のように文字を持たない要素は、`.name(..)` が代替テキストになるので、必ず付けます。名前のない画像は装飾とみなされ、支援技術には伝わりません。ここのロゴには「タスクのロゴ」という名前を付けています。
- `.role(..)` は、view がどんな種類のものかを表します。`text(..)` に `Role::Heading` を付けると、タイトルが見出しになります。`Role::List` と `Role::ListItem` を使い、一覧に名前を付けると、スクリーンリーダーは「タスク一覧」と読み上げ、項目数も伝えられます。

**ダークモード。** アプリのテーマは値で、`provide_theme(theme, || ..)` で、その中のすべてに適用できます。シグナル、`memo`、クロージャを渡せば、アプリの実行中にテーマを切り替えられます。Kivact は既存の view のスタイルを更新するだけで、作り直しはしません。ここでは、「ダークモード」スイッチが `dark` シグナルを編集し、`memo` がそれを `Theme::dark()` または `Theme::light()` に変換します。`provide_theme` 自身は何も描かないため、いちばん外側の view に `background(ColorToken::Surface)` と `color(ColorToken::Text)` を付けています。Web ページが `body` に指定するのと同じです。

スイッチが不要なら、テーマを固定するのは 1 行で済みます: `App::new("タスク").theme(Theme::dark())`。

**文字装飾と visibility。** `text_decoration(TextDecorationLine::UNDERLINE)` と `text_decoration_style(TextDecorationStyle::Wavy)` で波線の下線を描きます。既定は `Solid` で、`Dashed` と `Dotted` も選べます。種類には定数、signal、closure を渡せます。`visibility(Visibility::Hidden)` は subtree の layout 領域を保ち、描画、支援技術、focus、pointer の当たり判定から外します。`Visibility::Visible` で戻せます。これらの enum は `kivact::style` にあります。文字色の変化は `TransitionProp::Paint` で補間します。`TextDecoration` を直接構築する場合は `style` field を指定するか、`..TextDecoration::default()` を使います。

## 実行する

```text
cargo run
```

ロゴ、太字のタイトルと灰色の補足、右端の「ダークモード」スイッチが表示されます。タスクは枠線つきのカードに入り、それぞれにチェックボックスと控えめな削除ボタンが付きます。スイッチをオンにすると、ウィンドウ全体が一度にダークの配色に変わります。Tab キーでコントロール間のフォーカスを移動し、Space キーでタスクにチェックを入れてみてください。

次へ: [ステップ 7: 非同期処理](07-async.md)
