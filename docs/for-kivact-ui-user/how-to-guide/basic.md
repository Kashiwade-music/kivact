# Basic の作業別手順

`kivact-ui-basic` は shadcn/ui の Lyra preset（Neutral、Inter、Lucide、角丸 0）を写した公式キットです。ここでは作業ごとに手順を示します。初めて使うときは先に[入門](../getting-started/basic/README.md)を読みます。各部品の例は `cargo run -p kivact-ui-basic-catalog`、型とメソッドの一覧は `cargo doc -p kivact-ui-basic --no-deps` の rustdoc にあります。

## テーマを適用して実行中に切り替える

`basic::theme(mode)` を `App::theme` に渡します。切り替えが要るアプリは、signal から作る closure を `kivact::view::provide_theme` に渡します。部品の style は現在のテーマを読むので、切り替えても部品は作り直されず、入力中の値も残ります。

```rust,no_run
use kivact::prelude::*;
use kivact::style::theme::Typography;
use kivact_ui_basic as basic;

fn root() -> impl IntoView {
    let dark = signal(false);
    let theme = move || {
        let theme = basic::theme(if dark.get() {
            kivact::style::ThemeMode::Dark
        } else {
            kivact::style::ThemeMode::Light
        });
        let text = Typography {
            family: Some("Inter, sans-serif".into()),
            ..theme.text().clone()
        };
        theme.with_text(text)
    };
    kivact::view::provide_theme(theme, move || switch("Dark mode", dark))
}

fn main() -> kivact::Result<()> {
    App::new("Theme").run(root)
}
```

フォントは持ち込みます。font の bytes を `App::font` で登録し、`Theme::with_text` で `theme.text()` の `family` に Inter の family 名を書きます（入門の「フォントを登録する」を参照）。コードの表示は `"Geist Mono", monospace` を求めます。登録しない family は system UI のフォントで描かれます。部品が自分で出す文言（Loading、Copy など）は英語で、`App::messages` で差し替えます。

## token で色を調べる、変える

`basic::Tokens` は shadcn の CSS 変数に当たる色の表です（background、foreground、card、primary、muted、accent、destructive、border、input、ring、chart、sidebar の各色）。`basic::Color` はその各色の名前で、色を受け取る style の method にそのまま渡せます。描くときのテーマから色を読むので、テーマを切り替えると追従します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn banner() -> impl IntoView {
    text("お知らせ").style(
        style()
            .padding(12)
            .background(basic::Color::Muted)
            .color(basic::Color::MutedForeground)
            .border(1, basic::Color::Border),
    )
}
```

色から別の色を作るとき（`with_alpha` や `mix_in`）だけ、style の closure の中で `current_theme().get::<basic::Tokens>()` と値を読みます。

色を変えるときは `Tokens::new(mode)` を作って値を書き換え、`Theme::with` で載せます。部品の style は載せた token を読みます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn blue_theme() -> kivact::style::Theme {
    let mut tokens = basic::Tokens::new(kivact::style::ThemeMode::Light);
    tokens.primary = rgb(0x2563eb);
    basic::theme(kivact::style::ThemeMode::Light).with(tokens)
}

fn main() -> kivact::Result<()> {
    App::new("Blue").theme(blue_theme()).run(|| kivact::ui::button("Save"))
}
```

## 標準部品とキットの部品

shadcn の次の部品は、キットが新しく作らず、core の `kivact::ui` の標準部品を使います。キットは theme に Lyra の recipe を載せて見た目を決めます。操作、キー、focus、a11y は core の共通実装なので、キットを替えても変わりません。

| shadcn の名前 | 使うもの |
|---|---|
| Button | `kivact::ui::button`（icon を付けて `.hide_label(true)` にすると正方形の button） |
| Toggle | `kivact::ui::toggle`（枠は `.variant(basic::ToggleVariant::Outline)`） |
| Progress | `kivact::ui::progress(Some(0.0 から 1.0))`（`None` は不定） |
| Accordion / Collapsible | `kivact::ui::disclosure` を縦に並べ、開いている key の `signal(Vec<K>)` を各 `open` の binding に結ぶ（見た目のない開閉は `basic::collapsible`） |
| Checkbox | `kivact::ui::checkbox` |
| Switch | `kivact::ui::switch` |
| Input | `kivact::ui::text_input` |
| Textarea | `kivact::ui::text_area` |
| Select | `kivact::ui::select` |
| Dropdown Menu | `kivact::ui::menu`（メニューバーは `basic::menubar`） |
| Context Menu | `kivact::ui::menu(..).attach_context(target)` |
| Dialog | `kivact::ui::dialog` |
| Alert Dialog | `kivact::ui::dialog(..).kind(DialogKind::AlertModal)`（幅は `.variant(basic::AlertDialogSize::Sm)`） |
| Sheet | `kivact::ui::dialog(..).kind(DialogKind::Drawer(side))` |
| Popover | `kivact::ui::dialog(..).kind(DialogKind::Popover).attach(trigger)` |
| Tooltip | `kivact::ui::tooltip` |
| Tabs | `kivact::ui::tabs`（line の variant は `.variant(basic::TabsVariant::Line)`） |
| Radio Group | `kivact::ui::radio_group`（項目の説明は `ListItem::description`） |
| Slider | `kivact::ui::slider`、二つのつまみは `kivact::ui::range_slider` |
| Sonner | `kivact::ui::toast_region`（積む通知は `Toast::new(title).tone(..)`） |
| Resizable | 二つの面で px の大きさなら `kivact::ui::splitter`、割合で分けるなら `kivact::ui::split`（つまみは `.grip(true)`） |

これらのほか、card、badge、alert、table、drawer、hover card などは `basic::` の部品です。button の variant は shadcn と次の対応です。`ButtonVariant::Primary` が `default`、`Secondary`、`Outline`、`Ghost`、`Link` は同じ名前、`Danger` が `destructive` です。大きさは `ControlSize::ExtraSmall`、`Small`、`Medium`、`Large` が `xs`、`sm`、`default`、`lg` です。popover や menu を開く button は押しても動かず、開いている間は muted の背景になります。icon を添えた側の余白は shadcn と同じく狭くなります。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;

fn buttons() -> impl IntoView {
    row((
        button("Save"),
        button("Cancel").variant(ButtonVariant::Outline),
        button("Delete").variant(ButtonVariant::Danger),
        button("Small").size(ControlSize::Small),
        button("Add")
            .variant(ButtonVariant::Outline)
            .icon(icon("plus"))
            .hide_label(true),
    ))
    .style(style().gap(8))
}
```

キットの builder は `NodeBuilder` の共通の modifier（`.style(..)`、`.test_id(..)`、`.disabled(..)`、`.name(..)`）も受けます。

## 状態をアプリが持つ

キットの部品は状態を持ちません。編集できる値は `impl Into<Binding<T>>`（signal をそのまま渡せます）で受け、一覧は読取専用の `impl IntoValue<Vec<T>>`、操作の要求は callback で報告します。clipboard への書込み、保存、業務の規則はアプリが持ちます。

選択肢は `ListItem`（prelude にあります。key と表示名）の `Vec` で渡し、選択を `signal(Some(key))` に持たせます。`select` と `combobox` は同じ形です。`tabs` は常にどれかの tab を選ぶので、選択を `signal(key)` に持たせます。項目を足す、消す操作はアプリが一覧を更新します。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn fruit() -> impl IntoView {
    let fruit = signal(None);
    let options = vec![
        ListItem::new("apple", "Apple"),
        ListItem::new("banana", "Banana"),
        ListItem::new("grapes", "Grapes").disabled(true),
    ];
    column((
        select("Fruit", options, fruit).placeholder("Select a fruit"),
        basic::muted(move || format!("Selected: {}", fruit.get().unwrap_or("none"))),
    ))
    .style(style().gap(8))
}
```

## overlay：dialog、sheet、popover、toast

dialog・alert dialog・sheet・popover はどれも標準の `dialog(title, open)` で、`.kind(..)` で種類を選びます。開閉は `signal(bool)` で持ちます。button が `open.set(true)` を呼び、中身の button が `open.set(false)` を呼びます。Escape と close button でも閉じ、閉じると focus は元の button へ戻ります。overlay の中身は開くたびに作り直されるので、`.content(..)` と `.footer(..)` は関数で渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{DialogKind, Side};
use kivact_ui_basic as basic;

fn dialogs() -> impl IntoView {
    let open_dialog = signal(false);
    let open_sheet = signal(false);
    let name = signal(String::from("Ada"));
    column((
        row((
            button("Edit profile")
                .variant(ButtonVariant::Outline)
                .on_press(move || open_dialog.set(true)),
            button("Open sheet")
                .variant(ButtonVariant::Outline)
                .on_press(move || open_sheet.set(true)),
        ))
        .style(style().gap(8)),
        dialog("Edit profile", open_dialog)
            .description("Make changes to your profile here.")
            .content(move || text_input("Name", name))
            .footer(move || button("Save").on_press(move || open_dialog.set(false))),
        dialog("Notifications", open_sheet)
            .kind(DialogKind::Drawer(Side::Right))
            .description("Choose what you want to hear about.")
            .content(|| basic::muted("No new notifications."))
            .footer(move || button("Close").on_press(move || open_sheet.set(false))),
    ))
}
```

alert dialog は `.kind(DialogKind::AlertModal)` で、外側の押下では閉じません。取消しと確定の button は `.footer(..)` に並べ、どちらも `open.set(false)` を呼びます。alert-dialog.tsx には閉じる button がないので `.closable(false)` を付け、題名の脇の icon は `.media(..)`、320 px の中央寄せは `.variant(basic::AlertDialogSize::Sm)` です。popover は `.kind(DialogKind::Popover)` で `.attach(trigger)` に trigger を渡し、開く側は `.placement(Placement::..)`、揃え方は `.align(OverlayAlign::..)`、見出しを見せない panel は `.hide_title(true)` です。popover と hover card、dropdown menu は `.attach(trigger)`、context menu は `.attach_context(target)` で、node ではなく overlay の builder なので、style や name などの modifier は trigger に付けます。

toast は標準の `toast_region(toaster()).hold_all_on_hover(true)` をアプリの root に一度だけ置き、button などの handler から `toaster().push(Toast::new(title).tone(..))` で積みます。種類は `.tone(Tone::Success)` などで、tone の icon は Lyra の theme が描きます（`Tone::Neutral` は icon なし）。二行目は `.body(..)`、表示の長さは `.duration(..)` で足します。読み込み中の toast は `.icon(|| basic::spinner())` と `.duration(None)` です。`.action("Undo", move || ..)` の closure は toast が持つので、handler の中で作れます。複数の toast は新しいものの後ろに小さく重なり（最大 3 つ）、ポインターを重ねるか focus が入ると間隔を空けた一覧に開きます。開いている間は、すべての toast の時間が止まります。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::{Toast, Tone, toast_region, toaster};

fn root() -> impl IntoView {
    column((
        button("Create event")
            .variant(ButtonVariant::Outline)
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

## メニューとサイドバー

dropdown menu は標準の `menu(label, entries, choose)` で、Lyra の見た目（dropdown-menu.tsx）で描かれます。trigger は `.attach(trigger)` に渡し、shadcn と同じく trigger 以上の幅にするときは `.at_least_trigger_width(true)` を付けます。右クリックで開くメニューは同じ `menu(label, entries, choose)` を `.attach_context(target)` で対象に付け、context-menu.tsx の幅で描かれます。`basic::menubar(menus, choose)` は見出しと標準の menu の組を並べます。どれも同じ `MenuEntry` の一覧（`item`、`checked`、`radio`、`label`、`group`、`separator`、`submenu`）を受け、選ばれた項目の key を `choose` で報告します。check と radio は自分では変わりません。状態はアプリが持ち、次の一覧の `checked` に反映します。選んでもメニューを閉じないときは `.keep_open(true)` を付けます。一覧を返す関数を渡すと、状態の変化に追従します。`destructive(bool)` と `disabled(bool)` は項目に、`inset(true)` は項目と見出しに付けられ、check mark の幅だけ字下げして check の項目がある一覧と揃えます。

```rust,no_run
use kivact::prelude::*;

fn view_menu() -> impl IntoView {
    let status = signal(true);
    let position = signal("bottom");
    let rows = move || {
        vec![
            MenuEntry::heading("Appearance"),
            MenuEntry::item("status", "Status Bar").checked(status.get()).keep_open(true),
            MenuEntry::separator(),
            MenuEntry::radio("top", "Top", position.get() == "top").keep_open(true),
            MenuEntry::radio("bottom", "Bottom", position.get() == "bottom").keep_open(true),
            MenuEntry::separator(),
            MenuEntry::item("reset", "Reset")
                .icon("rotate-ccw")
                .destructive(true),
        ]
    };
    menu("View", rows, move |key| match key {
        "status" => status.update(|on| *on = !*on),
        "top" | "bottom" => position.set(key),
        _ => {}
    })
    .at_least_trigger_width(true)
    .attach(button("View").variant(ButtonVariant::Outline))
}
```

`basic::sidebar_provider(open, sidebar, page)` は、`open`（`signal(bool)`）で開閉を持つサイドバーと隣のページを組みます。`basic::sidebar_trigger()`、端の rail、Ctrl+B（Mac は Cmd+B）が `open` を書きます。`SidebarCollapsible::Icon` は閉じるとアイコンの列になり、ラベルがツールチップに変わります。provider は親の大きさいっぱいに広がるので、枠の中に置くときは親に幅と高さを与えます。Ctrl+B を window のどこでも効かせたいときは、provider を App のルートの view にします（window 全体のショートカットは、App の view への key handler として書くためです）。`kivact::ui::split(panes, sizes)` の `sizes` は合計 1 の割合の `signal(Vec<f32>)` で、境界の操作だけがそれを書きます。向きは既定が横並びで、`.orientation(Orientation::Vertical)`で縦に積みます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn shell() -> impl IntoView {
    let open = signal(true);
    let sizes = signal(vec![0.3f32, 0.7]);
    basic::sidebar_provider(
        open,
        basic::sidebar().collapsible(basic::SidebarCollapsible::Icon).rail(true).content(
            move || basic::sidebar_group("Application").content(move || basic::sidebar_menu((
                basic::sidebar_menu_button("Home").icon(icon("house")).active(true),
                basic::sidebar_menu_button("Inbox").icon(icon("inbox")).badge("12"),
            ))),
        ),
        column((
            basic::sidebar_trigger(),
            kivact::ui::split((text("List"), text("Detail")), sizes).grip(true),
        )),
    )
}
```

## データを表とグラフで見せる

表は標準の `kivact::ui::data_table(rows, key, columns)` で、basic の theme が shadcn の data-table の見た目（枠・40 px の header・薄い行の hover と選択）で描きます。使い方は [一覧と表](../../for-kivact-user/how-to-guide/lists-tables.md) にあります。絞り込みの欄の hint や「Columns」メニューのような shadcn の部品は、表の上の領域を替える `.toolbar(|context| view)` で作ります（列の表示は `DataColumn::visible` に同じ signal を渡します）。例は catalog の Data Table のページにあります。

```rust,no_run
use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Payment {
    id: u32,
    email: &'static str,
    amount: f64,
}

fn payments() -> impl IntoView {
    let data = vec![
        Payment { id: 1, email: "ken99@example.com", amount: 316.0 },
        Payment { id: 2, email: "abe45@example.com", amount: 242.0 },
        Payment { id: 3, email: "silas22@example.com", amount: 874.0 },
    ];
    let query = signal(String::new());
    let selected = signal(Vec::new());
    let sort = signal(None);
    let page = signal(0_usize);
    let columns = vec![
        DataColumn::new("Email", |p: &Payment| p.email.to_string())
            .compare(|a, b| a.email.cmp(b.email))
            .filter(|p: &Payment, q: &str| p.email.contains(q)),
        DataColumn::new("Amount", |p: &Payment| format!("${:.2}", p.amount))
            .compare(|a, b| a.amount.total_cmp(&b.amount))
            .numeric(true),
    ];
    data_table(data, |p: &Payment| p.id, columns)
        .row_label(|p: &Payment| p.email.to_string())
        .filter(query)
        .selection(selected)
        .sort(sort)
        .page(page, 5)
        .style(style().width(560))
}
```

グラフは `basic::chart(config, rows)` です。系列を `ChartConfig::new([ChartSeries::new(key, label)])` で名付け、行に `ChartRow::new(label, [値, ..])` を並べます。種類は `.kind(basic::ChartKind::Line)` などで選び、色の既定は token の `chart` です。棒グラフを横向きにするには `.orientation(Orientation::Horizontal)`、軸やツールチップの値の書き方は `.value_format(|v| format!("{v:.0}k"))` で変えます。card に入れて見出しをつけるのが定番です。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn visitors() -> impl IntoView {
    let config = basic::ChartConfig::new([
        basic::ChartSeries::new("desktop", "Desktop"),
        basic::ChartSeries::new("mobile", "Mobile"),
    ]);
    let data = vec![
        basic::ChartRow::new("Jan", [186.0, 80.0]),
        basic::ChartRow::new("Feb", [305.0, 200.0]),
        basic::ChartRow::new("Mar", [237.0, 120.0]),
    ];
    basic::card()
        .header(basic::card_header("Bar Chart").description("January - March"))
        .content(move || basic::chart(config, data))
        .style(style().width(560))
}
```

## フォームを組む

`kivact::ui::field(label, control)` が label、説明、エラーを入力部品とまとめます。検証はアプリの `memo` で行い、`.error(memo)` へ `Option<String>` を渡すと、メッセージを alert として読み上げさせます。label は control の支援技術向けの名前になり、hint と error は説明として関連付きます。error が Some の間は control の invalid も設定されます。横並びや豊かな説明は `row`・`column` で合成し、複数の部品を一つの control とする場合は `node().child((..))` で包みます。複数の field は `basic::field_set(basic::field_group((..)))` と `.legend(..)` でまとめます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn email() -> impl IntoView {
    let email = signal(String::from("ada@"));
    let error = memo(move || {
        (!email.get().contains("@example.com")).then(|| "Use an @example.com address.".to_string())
    });
    kivact::ui::field("Email", text_input("Email", email).invalid(move || error.get().is_some()))
        .hint("We never share it.")
        .error(error)
        .style(style().width(320))
}
```

入力欄に icon や button を添えるときは `basic::input_group(basic::input_group_input(name, value))` の `.start(..)`、`.end(..)` を使います。addon を押すと入力欄が focus を受けます。選択肢が多いときは `basic::combobox(items, selected)`（`signal(Vec::new())` を渡すと複数を選べる）、日付は `basic::date_picker(label, signal(None))` と `basic::date_range_picker` です。date picker のボタンは文字の幅になるので、幅が要るときは style で与えます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn pickers() -> impl IntoView {
    let query = signal(String::new());
    let framework = signal(None);
    let date = signal(None);
    let items = vec![
        ListItem::new("next", "Next.js"),
        ListItem::new("remix", "Remix"),
        ListItem::new("astro", "Astro"),
    ];
    column((
        basic::input_group(basic::input_group_input("Search", query).placeholder("Search..."))
            .start(icon("search"))
            .style(style().width(320)),
        basic::combobox(items, framework)
            .placeholder("Select a framework")
            .name("Framework")
            .style(style().width(320)),
        basic::date_picker("Date of birth", date).style(style().width(280)),
    ))
    .style(style().gap(12))
}
```

## 子を一つずつ受ける部品

`basic::button_group`、`basic::avatar_group`、`basic::carousel`、`kivact::ui::split` は、子を一つずつ整えたり包んだりします。そのため `column` のように一つの `impl IntoView` ではなく、子の tuple か `Vec` を受けます。種類の違う子を並べるときは tuple（16 個まで）、同じ種類を数えて並べるときは `Vec` です。どちらも `.into_view()` や `.into_node()` で型を揃える必要はありません。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn groups() -> impl IntoView {
    let index = signal(0_usize);
    let slides = (1..=3)
        .map(|n| text(format!("Slide {n}")))
        .collect::<Vec<_>>();
    column((
        basic::button_group((
            button("Archive").variant(ButtonVariant::Outline),
            basic::button_group_separator(),
            button("Report").variant(ButtonVariant::Outline),
        )),
        basic::avatar_group((
            basic::avatar("CN"),
            basic::avatar("LR"),
            basic::avatar_group_count("+3"),
        )),
        basic::carousel(slides, index).style(style().width(240).margin_x(48)),
    ))
    .style(style().gap(16))
}
```

## icon を使う

icon は中核の `kivact::control::icon("name")` で描きます。basic の theme は Lucide の icon を `basic::ICON_NAMES` の名前で theme の icon の表に入れるので、16 px、現在の文字色で描かれ、それより小さい箱（小さい button の icon の箱など）では箱に合わせて縮みます。表にない名前は何も描かず、debug build では呼んだ場所つきの診断（`ErrorKind::Asset`）として報告されます。button には `.icon(icon("check"))` で添えます。icon は装飾なので、icon だけの button は `button(label).icon(icon(name)).hide_label(true)` で作り、label が accessible name になります。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn icons() -> impl IntoView {
    assert!(basic::ICON_NAMES.contains(&"settings"));
    row((
        button("Download").icon(icon("download")),
        button("Settings").icon(icon("settings")).hide_label(true),
    ))
    .style(style().gap(8))
}
```

## 読む向き

`kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, || ..)` は部分木に読む向きを渡します。キットの向きに従う部品は、これに合わせて並びや矢印を反転させます。`kivact::view::current_direction()` で読めます。core の標準部品（`kivact::ui`）は反転しないので、右から左の言語では、並びの順序をアプリが決めます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_basic as basic;

fn rtl() -> impl IntoView {
    kivact::view::provide_direction(kivact::view::ReadingDirection::Rtl, || {
        text(format!("{:?}", kivact::view::current_direction()))
    })
}
```

標準の field は label を押すと control をクリックと同じに起動します。文字入力は focus し、checkbox と switch は切り替わります。disabled・非表示・利用できない control の起動は、共通の NodeRef activation が拒みます。
