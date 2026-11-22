# ステップ 6: スタイルとアクセシビリティ

[English](../../en/getting-started/06-style-and-accessibility.md)

このステップでは、アプリに見た目を与えます。タイトルを大きくし、補足の文字を灰色にし、一覧をカードで囲み、ボタンの種類を加えます。また、支援技術が読める名前をすべてのコントロールに与え、ダークモードのスイッチを加えます。

## ここまでのプログラム

```rust,no_run
use kivact::prelude::*;

const LOGO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><rect width="24" height="24" rx="6" fill="#336699"/><path d="M6 12.5l4 4 8-9" stroke="white" stroke-width="2.5" fill="none"/></svg>"##;

#[derive(Clone, PartialEq)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[component]
fn header(dark: Signal<bool>) -> impl Element {
    row((
        image(LOGO).name("Tasks logo").style(style().size(32, 32)),
        column((
            text("Tasks").role(Role::Heading).level(1).style(style().font_size(28).font_weight(700)),
            text("Things to do today").style(style().color(color::TEXT_MUTED)),
        ))
        .style(style().gap(2).flex_grow(1)),
        switch("Dark mode", dark),
    ))
    .style(style().gap(12).align_items(Align::Center))
}

#[component]
fn task_row(tasks: Signal<Vec<Task>>, task: Signal<Task>) -> impl Element {
    let id = task.get().id;
    row((
        checkbox(move || task.get().title, bind!(task.done)),
        button("Remove")
            .name(move || format!("Remove {}", task.get().title))
            .variant(ButtonVariant::Ghost)
            .on_press(move || tasks.update(|list| list.retain(|task| task.id != id))),
    ))
    .role(Role::ListItem)
    .style(style().gap(8).align_items(Align::Center).justify_content(Justify::SpaceBetween))
}

#[component]
fn task_list(tasks: Signal<Vec<Task>>) -> impl Element {
    column(for_each(
        tasks,
        |task| task.id,
        move |task| task_row(tasks, task),
    ))
    .role(Role::List)
    .name("Task list")
    .style(
        style()
            .gap(8)
            .padding(space::MD)
            .border_radius(radius::MD)
            .background(color::SURFACE_ALT)
            .border(1, color::BORDER),
    )
}

#[component]
fn tasks_screen(dark: Signal<bool>) -> impl Element {
    let tasks = signal(vec![
        Task { id: 1, title: "Buy milk".to_owned(), done: false },
        Task { id: 2, title: "Write the tutorial".to_owned(), done: false },
        Task { id: 3, title: "Go for a walk".to_owned(), done: false },
    ]);
    let next_id = var(4);
    let draft = signal(String::new());
    let left = memo(move || tasks.with(|list| list.iter().filter(|task| !task.done).count()));

    let add = callback(move |_| {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            tasks.update(|list| list.push(Task { id, title, done: false }));
            draft.set(String::new());
        }
    });

    column((
        header(dark),
        row((
            text_input(draft)
                .label("New task")
                .placeholder("What needs doing?")
                .on_submit(add)
                .style(style().width(240)),
            button("Add")
                .disabled(move || draft.with(|text| text.trim().is_empty()))
                .on_press(add),
        ))
        .style(style().gap(8)),
        show(move || tasks.with(|list| !list.is_empty()), move || task_list(tasks))
            .fallback(|| text("Nothing to do. Add a task above.")),
        text(move || format!("{} left", left.get())).style(style().color(color::TEXT_MUTED)),
    ))
    .style(
        style()
            .padding(24)
            .gap(16)
            .width(pct(100))
            .height(pct(100))
            .background(color::SURFACE)
            .color(color::TEXT),
    )
}

fn app() -> impl IntoView {
    let dark = signal(false);
    provide_theme(
        move || if dark.get() { Theme::dark() } else { Theme::light() },
        move || tasks_screen(dark),
    )
}

fn main() -> kivact::Result<()> {
    App::new("Tasks").run(app)
}
```

## 新しいこと

**スタイルは CSS の名前を使います。** `style()` には、文字、色、大きさ、枠線のためのメソッドがあります。名前は、対応する CSS のプロパティにならっています。

- `font_size(28)` と `font_weight(700)` は、view の中の文字と、その下にあるすべての文字の大きさと太さを設定します。`font_weight` は CSS 風の数値で、400 が通常、700 が太字です。
- `color(..)` は文字の色を設定します。`background(..)`、`border(1, ..)`、`border_radius(..)` は、箱そのものを装飾します。
- `flex_grow(1)` は、`row` の中の空きスペースを view が取れるようにします。これでスイッチが右端に押しやられます。`align_items(Align::Center)` と `justify_content(Justify::SpaceBetween)` は、子を揃えます。

**テーマ変数。** `color::SURFACE`、`color::TEXT`、`color::TEXT_MUTED`、`color::SURFACE_ALT`、`color::BORDER`、`color::ACCENT` などは、CSS の `var(--surface)` のような、*テーマ変数* です。どの値になるかは、有効なテーマが決めます。`space::MD` と `radius::MD` は、テーマの中くらいの間隔と角の丸みです。値を置きたい場所には、どこにでも変数を置けます。テーマが変わると、その変数を使うすべてのスタイルが、クロージャなしで追従します。ほとんどの色にはテーマ変数を使ってください。そうすると、同じコードがライトテーマとダークテーマの両方で動きます。色を 1 つに固定したいときは、`rgb(0x336699)` を使います。`padding(24)` のような数値は、論理ピクセルの固定の長さです。

**画像とフォントのバイト列。** `image(LOGO)` は、このプログラムに埋め込んだ SVG のバイト列を使います。`include_bytes!("logo.svg").as_slice()` は、ビルド時に埋め込まれたファイルを渡します。実行時に作られるバイト列には、`kivact::view::AssetBytes::new(bytes)` を使うと、所有する `Vec<u8>` を保持できます。生成された RGBA のピクセルには、`kivact::draw::Pixels::new(width, height, rgba)` を使います。`rgba` には、1 ピクセルあたり 4 バイトを含む、所有するベクターも渡せます。変わらないバイト列を再利用するときは、既存の `AssetBytes` を複製します。フォントはテーマに属します。アプリを起動するときに次のように書きます。`App::new("Tasks").theme(Theme::light().with_fonts([include_bytes!("Inter.ttf").as_slice()]))`。`with_fonts` はフォントの一覧を受け取ります。各フォントは、静的なバイト列、実行時に読み込む `Vec<u8>`、またはパスのいずれかです。

**ボタンの種類。** ボタンは、既定では主要な操作になります（`Primary`。アクセントの色で塗りつぶされます）。`.variant(ButtonVariant::Ghost)` は、Remove に控えめな見た目を与えます。ホバーするまで塗りつぶしがありません。ほかの種類は、`Secondary`、`Outline`、`Danger`、`Link` です。

**アクセシブルな名前。** 支援技術は、コントロールをロールと名前で読み上げます。キーボードやテストのツールも、同じ方法でコントロールを探します。そのため、すべてのコントロールに名前が必要です。

- 見えるラベルのあるコントロールは、すでに名前を持っています。ボタンのラベル、チェックボックスのラベル、スイッチのラベルがそうです。テキスト欄は、ステップ 4 の `.label(..)` から名前を得ました。
- `.name(..)` は、名前を設定または上書きします。Remove ボタンはすべて「Remove」と読まれてしまうので、それぞれをタスクの名前で呼びます。「Remove Buy milk」のようになります。名前はクロージャでもよいので、タイトルに従います。
- 画像のように文字のない要素は、代替テキストとして `.name(..)` が必要です。名前のない画像は装飾として扱われ、支援技術から隠されます。ここのロゴは「Tasks logo」という名前です。
- `.role(..)` は、view が何の種類のものかを表します。`Role::Heading` を付けた `text(..)` は、タイトルを見出しとして示します。`Role::List` と `Role::ListItem` を、一覧の名前と組み合わせると、スクリーンリーダーは「Task list」と読み上げ、項目の数を数えられます。

**ダークモード。** `provide_theme(theme, || ..)` は、その中のすべてにテーマを適用します。テーマはクロージャでもよく、ここでは `dark` シグナルを読みます。そのため、「Dark mode」のスイッチを切り替えると、テーマも変わります。Kivact は既存の view のスタイルを更新するだけで、作り直すことはありません。`provide_theme` 自体は何も描きません。そのため、一番上の view に `background(color::SURFACE)` と `color(color::TEXT)` を設定します。Web ページで `body` に色を設定するのと同じです。

スイッチが要らなければ、固定のテーマは 1 行で書けます。`App::new("Tasks").theme(Theme::dark())` です。

**文字の装飾と表示。** `text_decoration(TextDecorationLine::UNDERLINE)` と `text_decoration_style(TextDecorationStyle::Wavy)` を加えると、波線の下線が描かれます。既定の模様は `Solid` です。`Dashed` と `Dotted` も使えます。状態に応じて模様を変えたいときは、スタイル全体をクロージャにします。`.style(move || style().text_decoration_style(..))` です。`visibility(Visibility::Hidden)` は、部分木のレイアウトの空間を保ったまま、描画、支援技術、フォーカス、ポインターのヒットから除きます。`Visibility::Visible` は、それを元に戻します。これらの型は `kivact::style` にあります。文字の色の変化は、`TransitionProp::All` で補間されます。

## 実行する

```text
cargo run
```

ウィンドウには、ロゴ、灰色の補足文が付いた太字のタイトル、右側の「Dark mode」スイッチが表示されます。タスクは枠のあるカードに入り、それぞれにチェックボックスと控えめな Remove ボタンがあります。スイッチを入れると、ウィンドウ全体が一度にダークの色に変わります。Tab キーでコントロール間のフォーカスを移動し、Space キーでタスクにチェックを入れます。

次へ: [ステップ 7: 非同期処理](07-async.md)
