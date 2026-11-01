# 独自の見た目のコントロールを作る

[English](../../en/how-to-guide/custom-controls.md)

ピル型のスイッチ、セグメントコントロール、独自のドロップダウン、ドラッグで変える値、評価バーのように、標準コンポーネントでは足りない見た目が必要でも、コントロールとして正しく感じられる部分、つまりキーボード操作、フォーカス、選択、読み上げソフトのロール、タッチやポインターの扱いはそのまま欲しい。そんなときのガイドです。[再利用できるコンポーネントを書く](custom-components.md)を読んでいることを前提にしています。

考え方は分業です。`kivact::interaction` の**コントローラー**が振る舞いと状態を持ちます。何も描画しません。コントローラーが渡す **Part** が、入力、フォーカス、アクセシビリティのセマンティクスを運びます。Part を自分でスタイルしたノードに付け、Part が報告する状態からそのノードを描きます。アプリケーションは `kivact::ui` や kit を使います。コントローラーはコントロールや kit を作る人のためのもので、公式 kit も同じものから作られています。

| 必要なもの | コントローラー | 渡される Part |
|---|---|---|
| ボタン | `use_button` | `control()` |
| チェックボックス、スイッチ、トグルボタン | `use_checkbox`、`use_switch`、`use_toggle_button` | `control()` |
| リストから 1 つまたは複数の選択 | `use_list` | `list()`、`item(key)` |
| リストを開くドロップダウン | `use_select` | `trigger()`、`content()`、`item(key)` |
| メニュー | `use_menu` | `trigger()`、`context_trigger()`、`content()`、`item(key)` |
| メニューバー | `use_menubar` | `bar()`、`item(at)` |
| 候補つきの検索ボックス | `use_combobox` | `input()`、`content()`、`item(key)` |
| コマンドパレット | `use_command_palette` | `content()`、`input()`、`list()`、`item(key)` |
| 開閉する本体、ポップオーバー、ツールチップ | `use_disclosure`、`use_popup`、`use_tooltip` | `trigger()`、`content()` |
| ダイアログ | `use_dialog` | `trigger()`、`content()`、`title()`、`description()`、`close_trigger()` |
| タブ | `use_tabs` | `list()`、`item(key)`、`panel(key)` |
| ツリー | `use_tree` | `list()`、`item(key)` |
| テーブル | `use_table` | `grid()`、`header(column)`、`row(key)`、`cell(key, column)` |
| スライダー、数値入力 | `use_scalar`、`use_range_scalar` | `thumb()`、`track(axis)`、`input()` |
| サイズ変更できる仕切り | `use_splitter` | `handle()` |
| ドラッグ、ズームとパン | `use_drag`、`use_zoom_pan` | `source(..)`、`viewport()` |
| ネイティブのテキスト編集 | `use_text_edit` | `control()` |

ここに載せていないコントローラーや各コントローラーの詳細は、`cargo doc -p kivact-interaction --open` で見られます。

## 基本の型: コントローラー、Part、ノード

最短のカスタムコントロールはスイッチです。動作はすべてコントローラーが持ち、コードは見た目だけを決めます。

```rust,test_harness
use kivact::Binding;
use kivact::interaction::use_switch;
use kivact::prelude::*;
use kivact::style::Direction;

/// オンのとき右へつまみが動く、ピル型のスイッチ。
pub fn pill_switch(label: impl IntoText, on: impl Into<Binding<bool>>) -> Node {
    let control = use_switch(on).control();
    // Part がノードへ move される前に、状態を読んでおきます。
    let state = control.state();
    node()
        .part(control)
        .name(label)
        .style(move || {
            let state = state.get();
            let on = state.checked.is_checked();
            style()
                .direction(Direction::Row)
                .justify_content(if on { Justify::End } else { Justify::Start })
                .size(44, 24)
                .padding(3)
                .radius(12)
                .background(if on { ColorToken::Accent } else { ColorToken::Border })
                .opacity(if state.input.pressed { 0.8 } else { 1.0 })
        })
        .child(node().style(style().size(18, 18).radius(9).background(ColorToken::Surface)))
}

fn app() -> impl IntoView {
    let notifications = signal(false);
    column((
        pill_switch("Notifications", notifications),
        text(move || if notifications.get() { "On" } else { "Off" }.to_owned()),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Switch").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_part_brings_the_role_the_keys_and_the_state() {
        let screen = kivact::test::render(app);
        let switch = screen.get_by_role(Role::Switch, "Notifications");
        assert_eq!(switch.is_checked(), Some(false));

        switch.click();
        assert_eq!(switch.is_checked(), Some(true));
        screen.get_by_text("On");

        switch.press_key(Key::Space);
        assert_eq!(switch.is_checked(), Some(false));
    }

    #[test]
    fn a_disabled_switch_ignores_input() {
        let screen = kivact::test::render(|| pill_switch("Locked", signal(false)).disabled(true));
        let switch = screen.get_by_role(Role::Switch, "Locked");
        assert!(switch.is_disabled());
        switch.force_click();
        assert_eq!(switch.is_checked(), Some(false));
    }
}
```

書いていないのに、次の 4 つが成り立っています。

- **ロールと状態。** `use_switch(..).control()` が `Role::Switch`、チェック状態、フォーカスの停止位置を設定します。読み上げソフトは「Notifications、スイッチ、オフ」と読み上げます。
- **入力。** クリック、Space、読み上げソフトの操作は、すべて同じ `Binding` を反転させます。スイッチは Enter を無視します。コントロールは渡された binding に値を書き込むので、状態は呼び出し側に残ります。
- **無効化。** `pill_switch(..).disabled(true)` は動作します。無効なノードは入力を無視し、状態には `disabled` が反映されます。ルートを自分で持つビルダーは、標準の `checkbox` と同様に `root.accessible_disabled()` を読んでコントローラーの `.disabled(..)` に渡します。そうすれば 1 つの modifier で両方が無効になります。
- **描画のための状態。** `control.state()` は実際の対象に追従する `Value` で、押下、ホバー、フォーカス、フォーカス可視、無効、`checked` を持ちます。これから描いてください。状態を真似る別のシグナルを持たないでください。

どのコントローラーにも共通の規則が 3 つあります。

1. **コントローラーは setup で作り、コントロールごとに 1 回呼ぶ。** `Copy` のハンドルです。
2. **Part をアタッチする前に `state()` を読む。** Part は move 専用です。`.part(p)` が消費します。その前に取った状態と `node_ref()` は、ノードに追従し続けます。
3. **コントローラーがやることを繰り返さない。** 自前の `on_key` で Space を処理すると、コントロールが二重に反転します。自分の `on_press`、`on_key`、`on_pointer` は Part の既定動作より**先に**動きます。自分の処理だけ実行して既定動作を止めたいときは `Handled::PreventDefault`、すべてを止めたいときは `Handled::Yes` を返します。

## キーボード操作つきでリストから選ぶ

セグメントコントロールは、1 つが選ばれている選択肢の行です。`use_list` は選択、矢印キー、Home と End、ローミングタブストップ、読み上げソフト向けのリストのセマンティクスを提供します。キー、順序、見た目は呼び出し側が用意します。

```rust,test_harness
use std::hash::Hash;

use kivact::Binding;
use kivact::interaction::{ListItem, ListPolicy, ListSelection, use_list};
use kivact::prelude::*;

/// 選択肢の行。矢印キーで移動と選択を行い、Tab で行の外へ出ます。
pub fn segmented<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: Vec<ListItem<K>>,
    selected: impl Into<Binding<Option<K>>>,
) -> Node {
    let options = options.into_value();
    let list = use_list(options, ListSelection::Single(selected.into())).policy(ListPolicy {
        axis: Some(Orientation::Horizontal),
        ..ListPolicy::default()
    });
    let rows = for_each(
        options,
        |option| option.key().clone(),
        move |key, option| {
            let part = list.item(key);
            let state = part.state();
            node()
                .part(part)
                .style(move || {
                    let state = state.get();
                    style()
                        .padding_x(12)
                        .padding_y(6)
                        .background(if state.selected { ColorToken::Accent } else { ColorToken::SurfaceAlt })
                        .color(if state.selected { ColorToken::AccentText } else { ColorToken::Text })
                        .opacity(if state.input.disabled { 0.4 } else { 1.0 })
                })
                .child(text(move || option.with(|option| option.label())))
        },
    );
    row(rows).part(list.list()).name(label)
}

fn app() -> impl IntoView {
    let range = signal(Some("week"));
    column((
        segmented(
            "Range",
            vec![
                ListItem::new("day", "Day"),
                ListItem::new("week", "Week"),
                ListItem::new("month", "Month"),
                ListItem::new("year", "Year").disabled(true),
            ],
            range,
        ),
        text(move || format!("range: {}", range.get().unwrap_or("none"))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Segmented").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_and_arrow_keys_move_the_selection_and_skip_disabled_items() {
        let screen = kivact::test::render(app);
        assert!(screen.get_by_role(Role::Option, "Week").is_selected());

        screen.get_by_role(Role::Option, "Month").click();
        screen.get_by_text("range: month");

        // Year は無効なので、矢印キーは Month で止まります。
        screen.get_by_role(Role::Option, "Month").press_key(Key::ArrowRight);
        screen.get_by_text("range: month");
        screen.get_by_role(Role::Option, "Month").press_key(Key::ArrowLeft);
        screen.get_by_text("range: week");
    }
}
```

リストが提供するものです。

- **ロール。** コンテナーは `ListBox` です (`ListSelection::Multiple` では複数選択可)。各項目は、ラベルを名前とする `Option` で、選択に含まれているときは `selected` になります。
- **選択は呼び出し側のもの。** リストはコピーを持ちません。ユーザーの選択は、渡した binding に書き込みます (`Binding<Option<K>>` に対する `ListSelection::Single`、または Shift と Ctrl の範囲選択を伴う `Binding<Vec<K>>` に対する `ListSelection::Multiple`)。フィルターで隠れた選択済みキーは消えません。
- **ナビゲーション。** `ListPolicy` で、軸、端での折り返し、移動で選択するか、先頭文字による検索、確定を起動操作で行うかを決めます。プリセットの `ListPolicy::tabs()`、`radio()`、`menu()`、`listbox()` が一般的な部品をカバーします。ポリシーは最初の Part を作る前に設定してください。
- **タブストップは 1 つ。** 行全体が Tab の停止位置 1 つで、中は矢印キーで移動します。無効な項目 (`ListItem::disabled(true)`) は飛ばされます。

## リストを開閉する

ドロップダウンは、リストの上に開閉、フォーカスの復帰、閉じる操作を足したものです。`use_select` は 3 つの Part を渡します。**trigger** (コンボボックス)、**content** (浮かぶ面)、選択肢ごとの **item** です。`shown()` が true のあいだ、`show` と `overlay` で content を mount します。

```rust,test_harness
use std::hash::Hash;

use kivact::Binding;
use kivact::interaction::{ListItem, use_select};
use kivact::prelude::*;
use kivact::primitive::NodeExt;
use kivact::view::{Placement, overlay};

pub fn dropdown<K: Clone + Eq + Hash + 'static>(
    label: impl IntoText,
    options: Vec<ListItem<K>>,
    value: impl Into<Binding<Option<K>>>,
) -> impl IntoView {
    let label = label.into_text_value();
    let options = options.into_value();
    let select = use_select(options, value);
    let trigger = select.trigger();
    let anchor = trigger.node_ref();
    (
        node()
            .part(trigger)
            .name(label)
            .style(
                style()
                    .padding(8)
                    .border(1, ColorToken::Border)
                    .radius(6)
                    .min_width(120),
            )
            .child(text(select.label())),
        show(select.shown(), move || {
            let rows = for_each(
                options,
                |option| option.key().clone(),
                move |key, option| {
                    let part = select.item(key);
                    let state = part.state();
                    node()
                        .part(part)
                        .style(move || {
                            let state = state.get();
                            style()
                                .padding(8)
                                .background(if state.active { ColorToken::Selection } else { ColorToken::Raised })
                        })
                        .child(text(move || option.with(|option| option.label())))
                },
            );
            overlay(anchor, Placement::Below, column(rows).style(style().min_width(120)))
                .part(select.content())
                .name(label)
        }),
    )
}

fn app() -> impl IntoView {
    let size = signal(None::<&'static str>);
    column((
        dropdown(
            "Size",
            vec![ListItem::new("s", "Small"), ListItem::new("m", "Medium"), ListItem::new("l", "Large")],
            size,
        ),
        text(move || format!("size: {}", size.get().unwrap_or("none"))),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Dropdown").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_trigger_opens_the_list_and_a_choice_closes_it() {
        let screen = kivact::test::render(app);
        let trigger = screen.get_by_role(Role::ComboBox, "Size");
        assert_eq!(trigger.is_expanded(), Some(false));

        trigger.click();
        assert_eq!(trigger.is_expanded(), Some(true));
        screen.get_by_role(Role::Option, "Medium").click();

        screen.get_by_text("size: m");
        assert!(screen.query_by_role(Role::Option, "Medium").is_none());
        // フォーカスはトリガーへ戻っています。
        assert!(screen.get_by_role(Role::ComboBox, "Size").is_focused());
    }
}
```

コントローラーは次のことをすべて引き受けています。閉じたトリガーでの上下キーは開かずに値を変え、Alt+Down、F4、Enter、Space、クリックで開き、リストは選択中のオプションにフォーカスを取り、Escape は選ばずに閉じ、フォーカスはトリガーに戻ります。`trigger()`、`content()`、`show` の同じ組み合わせは `use_popup`、`use_disclosure`、`use_tooltip`、`use_dialog`、`use_menu` でも使え、違うのは Part だけです。モーダルな面では、ダイアログの content が Tab を内側に閉じ込め、閉じると開いた要素にフォーカスを戻します。

## ドラッグで値を変える

`use_drag` はポインターの動きを、開始、更新、終了のあるジェスチャーに変えます。`EditSession` はそのジェスチャーを 1 つの編集にします。`Begin`、書き込み、`Commit` の順に進み、ユーザーが Escape を押したときは `Cancel` になって値は開始時に戻ります。

```rust,test_harness
use std::cell::RefCell;
use std::rc::Rc;

use kivact::draw::Point;
use kivact::interaction::{DragCoordinates, DragUpdate, EditPhase, EditSession, use_drag};
use kivact::prelude::*;

/// 横にドラッグして変える数値。
pub fn scrub(label: impl IntoText, value: Signal<f64>, history: Rc<RefCell<Vec<EditPhase>>>) -> Node {
    let edit = EditSession::new(move |initial| value.set(initial))
        .on_edit(move |phase| history.borrow_mut().push(phase));
    let origin = signal(0.0);
    let source = use_drag::<()>(DragCoordinates::Window)
        .edit(edit, move || value.get_untracked())
        .on_start(move |_| origin.set(value.get_untracked()))
        .on_update(move |update: DragUpdate<()>| {
            value.set(origin.get_untracked() + f64::from(update.delta.x));
        })
        .source(());
    let state = source.state();
    node()
        .part(source)
        .name(label)
        .focusable(true)
        .style(move || {
            style()
                .width(120)
                .padding(8)
                .border(1, ColorToken::Border)
                .background(if state.get().active { ColorToken::SurfaceAlt } else { ColorToken::Surface })
        })
        .child(text(move || format!("{:.0}", value.get())))
}

fn app() -> impl IntoView {
    let value = signal(10.0);
    let history = Rc::new(RefCell::new(Vec::new()));
    scrub("Width", value, history).test_id("scrub")
}

fn main() -> kivact::Result<()> {
    App::new("Scrub").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_is_one_edit() {
        let history = Rc::new(RefCell::new(Vec::new()));
        let log = history.clone();
        let screen = kivact::test::render(move || {
            let value = signal(10.0);
            scrub("Width", value, log).test_id("scrub")
        });
        let field = screen.get_by_test_id("scrub");
        field.drag(Point::new(20.0, 10.0), Point::new(70.0, 10.0));
        screen.get_by_text("60");
        assert_eq!(*history.borrow(), vec![EditPhase::Begin, EditPhase::Commit]);
    }
}
```

押しただけでは編集とみなしたくない場合、たとえば押下で選択し動かして初めて編集になるキャンバスでは、`edit` の代わりに `edit_on_change` を使います。よくある値のコントロールなら、これらを使わずに済みます。`use_scalar` と `use_range_scalar` が、キー、ステップ、範囲、同じ編集フェーズを備えたスライダーと数値入力 (`thumb()`、`track(axis)`、`input()`) を提供し、`use_splitter` がサイズ変更できる仕切りを提供します。

## コントロールが何であるかを伝える

コントローラーはロールと状態を設定してくれます。素のノードから作るコントロールは、次の modifier で自分で伝えます。`role`、`checked`、`expanded`、`selected`、`value_text` は `Node` のメソッドで、ビルダーには `NodeExt` が提供します。

| modifier | 意味 |
|---|---|
| `.role(Role::Slider)` | コントロールの種類。支援技術の利用者が期待するロールを選びます。 |
| `.name("Rating")` | アクセシブルな名前。どのコントロールにも必要です。 |
| `.focusable(true)` | キーボードを受ける (Tab の停止位置になる)。 |
| `.value_text(..)`、`.value_min(..)`、`.value_max(..)`、`.value_now(..)` | スライダーやプログレスバーの値を、言葉と数値で。 |
| `.checked(..)`、`.selected(..)`、`.expanded(..)` | チェック、選択、開閉の状態。 |
| `.on_a11y(A11yAction::Increment, ..)` | 読み上げソフトから呼べる操作。コントロールにできることだけを提供します。 |

ロールもフォーカスもないノードの `.on_press(..)` はポインターでしか届かず、デバッグビルドで報告されます。まずコントローラー (`use_button`) を使ってください。

## 自前の形を測って描く

レイアウトの規則では表せないサイズのコントロールには、**プリミティブ**を使います。プリミティブは、サイズを関数で計算するリーフ要素です。そのうえで、描画レイヤー、入力ハンドラー、フォーカス、アクセシビリティ、テスト ID と、ノードの持つものはすべて備えています。この API は実験的です。

```rust,test_harness
use kivact::Binding;
use kivact::draw::{Point, Size};
use kivact::prelude::*;
use kivact::primitive::{Constraints, NodeExt, primitive};
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent, PointerPhase};

const CELL: f32 = 20.0;

/// `max` 個の点の列。点灯している点の数が評価です。
pub fn rating(max: u32, value: impl Into<Binding<u32>>) -> impl IntoView {
    let value = value.into();
    primitive()
        // サイズは `max` だけで決まります。変わるとレイアウトが測り直します。
        .measure(max, |max: &u32, _: Constraints| Size::new(*max as f32 * CELL, CELL))
        .draw(move |d, size| {
            for i in 0..max {
                let color = if i < value.get() { ColorToken::Accent } else { ColorToken::Border };
                d.circle(Point::new(i as f32 * CELL + CELL / 2.0, size.height / 2.0), 7.0, color);
            }
        })
        .role(Role::Slider)
        .name("Rating")
        .value_min(0.0)
        .value_max(f64::from(max))
        .value_now(move || f64::from(value.get()))
        .value_text(move || format!("{} of {max}", value.get()))
        .focusable(true)
        .on_pointer(move |event: PointerEvent| {
            if event.phase == PointerPhase::Down {
                value.set(((event.pos.x / CELL) as u32 + 1).min(max));
            }
        })
        .on_key(move |event: KeyEvent| match event.key {
            Key::ArrowRight => {
                value.set((value.get() + 1).min(max));
                Handled::Yes
            }
            Key::ArrowLeft => {
                value.set(value.get().saturating_sub(1));
                Handled::Yes
            }
            _ => Handled::No,
        })
        .on_a11y(A11yAction::Increment, move || value.set((value.get() + 1).min(max)))
        .on_a11y(A11yAction::Decrement, move || value.set(value.get().saturating_sub(1)))
        .test_id("rating")
}

fn app() -> impl IntoView {
    let stars = signal(2);
    column((rating(5, stars), text(move || format!("stars: {}", stars.get()))))
        .style(style().gap(8).padding(16).align_items(Align::Start))
}

fn main() -> kivact::Result<()> {
    App::new("Rating").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_sized_by_its_content_and_acts_like_a_slider() {
        let screen = kivact::test::render(app);
        let rating = screen.get_by_role(Role::Slider, "Rating");
        let size = rating.rect().expect("laid out");
        assert_eq!((size.width, size.height), (100.0, 20.0));
        assert_eq!(rating.value().as_deref(), Some("2 of 5"));

        rating.press_key(Key::ArrowRight);
        screen.get_by_text("stars: 3");
        rating.click();
        screen.get_by_text("stars: 3");
    }
}
```

測定について覚えておくことです。

- **測定関数は純粋にする。** 測定はレイアウトの中で実行されるため、シグナル、テーマ、ツリーは読めません。サイズに影響するものはすべて content (ここでは `max`) に入れてください。レイアウトは content が変わったときだけ測り直します。
- **`Constraints` は空きと文字エンジンを渡す。** `max_width()` は使ってよい幅で、`measure_text` と `measure_paragraph` は画面と同じ shaping で文字を測ります。`Size` の代わりに `Measurement` を返すと、`Align::Baseline` 用の最初のベースラインを足せます。
- **レイアウトがすでに決めたサイズが優先される。** 列の中では、子は既定で幅いっぱいに引き伸ばされ、`known_width` が測った値を置き換えます。コントロールに自分の幅を保たせたいときは、例のように親を `Align::Start` にしてください。
- **プリミティブはリーフ。** 子要素、箱の外にはみ出すヒット領域、ネイティブウィジェットはサポートされません。1 つの描画の中に複数のヒット領域がほしいときは、[キャンバスで描く](canvas.md#描いた部分をクリック可能にしアクセシブルにする)で説明している region を宣言します。

### 描画レイヤー

`.draw(..)` は子の背面に、`.draw_over(..)` (`NodeExt`) は子の前面に描きます。どちらも `Draw` とレイアウト済みの `Size` を受け取り、サイズか読んでいるシグナルが変わったときにだけ再実行されます。スタイルの `paint` はテーマ向けです。呼び出し側が置き換えられる既定の見た目を描くのに使い、`.draw(..)` はコントロール自身のものに取っておいてください。

## 自前の枠でテキストを編集する

`use_text_edit` はノードにネイティブのテキスト編集 (キャレット、選択、IME の変換、クリップボード、取り消し、標準のキー操作) を与えます。何も描画しないので、枠、余白、カウンターは自分で用意します。標準の `text_input` は、このコントローラーにテーマの枠を付けたものです。

```rust,test_harness
use kivact::Binding;
use kivact::interaction::use_text_edit;
use kivact::prelude::*;
use kivact::style::Align;

/// 文字数カウンターつきの入力欄。`max` はエディターが強制します。
pub fn limited_input(label: impl IntoText, value: impl Into<Binding<String>>, max: usize) -> Node {
    let value = value.into();
    let control = use_text_edit(value).max_length(max).placeholder("Type here").control();
    let state = control.state();
    let editor = node().part(control).name(label).style(move || {
        style()
            .grow(1)
            .border_bottom(if state.get().focused { 2 } else { 1 }, ColorToken::Accent)
            .padding_y(6)
    });
    row((editor, text(move || format!("{}/{max}", value.with(|text| text.chars().count())))))
        .style(style().align_items(Align::Center).gap(8))
}

fn app() -> impl IntoView {
    let title = signal(String::new());
    column((limited_input("Title", title, 8), text(move || format!("title: {}", title.get()))))
        .style(style().gap(8).padding(16).width(320))
}

fn main() -> kivact::Result<()> {
    App::new("Limited").run(app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_is_limited_and_counted() {
        let screen = kivact::test::render(app);
        let input = screen.get_by_role(Role::TextBox, "Title");
        input.type_text("Hello, Kivact");
        screen.get_by_text("title: Hello, K");
        screen.get_by_text("8/8");
    }
}
```

binding は確定した値を受け取ります。IME の変換中は、エディターが未確定文字列を自分で保持し、binding には最終的なテキストが渡ります。変更のたびにコードを動かしたいときは、シグナルの代わりに `binding(read, write)` を渡します。ほかにも設定があります。`control()` の前に `.multiline(true)`、`.password(..)`、`.read_only(..)`、1 行入力で Enter を受ける `.on_submit(..)`、矢印、Tab、Escape のキーを自前のハンドラー (たとえば候補リスト) に渡す `.forward_keys([...])` です。

コードエディターのように、キー入力のたびに全体をコピーできないほど大きな文書には、より低い層の `kivact::primitive::TextEditing` を使います。範囲編集を通知し、キーのポリシーを受け取り、行に色を付けられます。実験的な API です。rustdoc を見てください。

## 関連項目

- [再利用できるコンポーネントを書く](custom-components.md): props、ビルダー、テーマ、アクセシビリティの転送。
- [ダイアログ、メニュー、ツールチップ、トーストを表示する](overlays.md): 同じコントローラーから作られた標準のオーバーレイ。
- [キャンバスで描く](canvas.md): region と、描画レイヤーで使う描画の語彙。
- [アプリをテストする](testing.md): 上のテストのように、ロール、キー、フォーカスを確かめる方法。
- `cargo doc -p kivact-interaction --open` ですべてのコントローラーと Part、`cargo doc -p kivact --open` で `kivact::primitive`。
