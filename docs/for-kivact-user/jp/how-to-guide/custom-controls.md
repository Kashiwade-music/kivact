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
| ホバーカード、遅れて出るヒント、一時的な確認表示 | `use_delayed` | `is_open()`、`dismiss()`、`pulse()` |
| ダイアログ | `use_dialog` | `trigger()`、`content()`、`title()`、`description()`、`close_trigger()` |
| タブ | `use_tabs` | `list()`、`item(key)`、`panel(key)` |
| ツリー | `use_tree` | `list()`、`item(key)` |
| テーブル | `use_data_table` | `grid()`、`header(column)`、`row(key)`、`cell(key, column)` |
| スライダー、数値入力 | `use_scalar`、`use_range_scalar` | `thumb()`、`track(axis)`、`input()` |
| サイズ変更できる仕切り | `use_splitter` | `handle()` |
| ドラッグ、ズームとパン | `use_drag`、`use_zoom_pan`、`use_space_pan` | `source(..)`、`viewport()`、`control()` |
| ネイティブのテキスト編集 | `use_text_edit`、`use_text_area` | `control()` |

ここに載せていないコントローラーや各コントローラーの詳細は、`cargo doc -p kivact-interaction --open` で見られます。

## 基本の型: コントローラー、Part、ノード

最短のカスタムコントロールはスイッチです。動作はすべてコントローラーが持ち、コードは見た目だけを決めます。

```rust,test_harness
use kivact::state::Binding;
use kivact::interaction::use_switch;
use kivact::prelude::*;
use kivact::style::Direction;

/// オンのとき右へつまみが動く、ピル型のスイッチ。
pub fn pill_switch(label: impl IntoText, on: impl IntoBinding<bool>) -> impl Component {
    let api = use_switch(on);
    let state = api.control_state();
    let control = api.control();
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
        assert_eq!(switch.checked(), Some(CheckedState::Unchecked));

        switch.click();
        assert_eq!(switch.checked(), Some(CheckedState::Checked));
        screen.get_by_text("On");

        switch.press_key(Key::Space);
        assert_eq!(switch.checked(), Some(CheckedState::Unchecked));
    }

    #[test]
    fn a_disabled_switch_ignores_input() {
        let screen = kivact::test::render(|| pill_switch("Locked", signal(false)).disabled(true));
        let switch = screen.get_by_role(Role::Switch, "Locked");
        assert!(switch.is_disabled());
        switch.force_click();
        assert_eq!(switch.checked(), Some(CheckedState::Unchecked));
    }
}
```

書いていないのに、次の 4 つが成り立っています。

- **ロールと状態。** `use_switch(..).control()` が `Role::Switch`、チェック状態、フォーカスの停止位置を設定します。読み上げソフトは「Notifications、スイッチ、オフ」と読み上げます。
- **入力。** クリック、Space、読み上げソフトの操作は、すべて同じ `Binding` を反転させます。スイッチは Enter を無視します。コントロールは渡された binding に値を書き込むので、状態は呼び出し側に残ります。
- **無効化。** `pill_switch(..).disabled(true)` は動作します。無効なノードは入力を無視し、状態には `disabled` が反映されます。Part は mount 時に対象の最終的な disabled を読むため、1 つの modifier で入力と見た目の両方を無効にします。
- **描画のための状態。** `api.control_state()` は実際の対象に追従する `Value` で、押下、ホバー、フォーカス、フォーカス可視、無効、`checked` を持ちます。これから描いてください。状態を真似る別のシグナルを持たないでください。

どのコントローラーにも共通の規則が 3 つあります。

1. **コントローラーは setup で作り、コントロールごとに 1 回呼ぶ。** `Copy` のハンドルです。
2. **描画の状態はコントローラーから読む。** `api.control_state()` は実際の対象に追従します。参照が必要なら `node_ref()` と `.node_ref(anchor)` で明示的に付けます。Part は振る舞いを付けます。
3. **コントローラーがやることを繰り返さない。** 自前の `on_key` で Space を処理すると、コントロールが二重に反転します。自分の `on_press`、`on_key`、`on_pointer` は Part の既定動作より**先に**動きます。自分の処理だけ実行して既定動作を止めたいときは `Handled::PreventDefault`、すべてを止めたいときは `Handled::Yes` を返します。

## キーボード操作つきでリストから選ぶ

セグメントコントロールは、1 つが選ばれている選択肢の行です。`use_list` は選択、矢印キー、Home と End、ローミングタブストップ、読み上げソフト向けのリストのセマンティクスを提供します。キー、順序、見た目は呼び出し側が用意します。

```rust,test_harness
use std::hash::Hash;

use kivact::state::Binding;
use kivact::interaction::{ListItem, ListOptions, Selection, use_list};
use kivact::prelude::*;

/// 選択肢の行。矢印キーで移動と選択を行い、Tab で行の外へ出ます。
pub fn segmented<K: ItemKey>(
    label: impl IntoText,
    options: Vec<ListItem<K>>,
    selected: impl IntoBinding<Option<K>>,
) -> impl Component {
    let options = options.into_value();
    let list = use_list(
        options,
        selected.into_binding(),
        ListOptions {
            axis: Some(Orientation::Horizontal),
            ..ListOptions::default()
        },
    );
    let rows = for_each(
        options,
        |option| option.key().clone(),
        move |option| {
            let key = option.with_untracked(|option| option.key().clone());
            let state = list.item_state(key.clone());
            let part = list.item(key);
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

- **ロール。** コンテナーは `ListBox` です (`Selection::Multiple` では複数選択可)。各項目は、ラベルを名前とする `Option` で、選択に含まれているときは `selected` になります。
- **選択は呼び出し側のもの。** リストはコピーを持ちません。ユーザーの選択は、渡した binding に書き込みます (`Binding<Option<K>>` に対する `Selection::Single`、または Shift と Ctrl の範囲選択を伴う `Binding<Vec<K>>` に対する `Selection::Multiple`)。フィルターで隠れた選択済みキーは消えません。
- **ナビゲーション。** `ListOptions` で、軸、端での折り返し、移動で選択するか、先頭文字による検索、確定を起動操作で行うかを決めます。プリセットの `ListOptions::tabs()`、`radio()`、`menu()`、`listbox()` が一般的な部品をカバーします。ポリシーは `use_list` の 3 番目の引数で渡します。
- **タブストップは 1 つ。** 行全体が Tab の停止位置 1 つで、中は矢印キーで移動します。無効な項目 (`ListItem::disabled(true)`) は飛ばされます。

## リストを開閉する

ドロップダウンは、リストの上に開閉、フォーカスの復帰、閉じる操作を足したものです。`use_select` は 3 つの Part を渡します。**trigger** (コンボボックス)、**content** (浮かぶ面)、選択肢ごとの **item** です。`is_open()` が true のあいだ、`show` と `overlay` で content を mount します。

```rust,test_harness
use std::hash::Hash;

use kivact::state::Binding;
use kivact::interaction::{ListItem, use_select};
use kivact::prelude::*;
use kivact::view::{Placement, overlay};

pub fn dropdown<K: ItemKey>(
    label: impl IntoText,
    options: Vec<ListItem<K>>,
    value: impl IntoBinding<Option<K>>,
) -> impl IntoView {
    let label = label.into_text_value();
    let options = options.into_value();
    let select = use_select(options, value);
    let trigger = select.trigger();
    let anchor = node_ref();
    (
        node()
            .part(trigger)
            .node_ref(anchor)
            .name(label)
            .style(
                style()
                    .padding(8)
                    .border(1, ColorToken::Border)
                    .radius(6)
                    .min_width(120),
            )
            .child(text(select.label())),
        show(select.is_open(), move || {
            let rows = for_each(
                options,
                |option| option.key().clone(),
                move |option| {
            let key = option.with_untracked(|option| option.key().clone());
                    let state = select.item_state(key.clone());
                    let part = select.item(key);
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
                .with_node(|host| host.part(select.content()))
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

ドラッグは既定で押した瞬間に始まります。`.slop(10)` を足すと移動が 10 logical pixel を越えるまで待ち、`.axis_lock()` を足すと閾値を越えた最初の移動の軸に delta を固定します。`.track_velocity()` を足すと `DragUpdate.velocity` から直近 100 ms の速度を logical pixel/ms で読めます。指定しない場合、速度は 0 です。閾値を越える前に離した場合、開始・更新・終了・編集の callback は呼びません。既存の drag handle の setup 時に指定します。

押した時点のアプリの値を payload に記録する場合は `.source_with(move || value.get_untracked())` を使います。受け付けた press ごとに factory を一度だけ呼びます。slop の判定待ちに入る場合も、その press の時点で記録し、同じ gesture の全 update で保持します。次に受け付けた press では取り直します。`.source(payload)` は setup 時に与えた固定の値を使います。

```rust,test_harness
use std::cell::RefCell;
use std::rc::Rc;

use kivact::geometry::Point;
use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};
use kivact::view::{EditPhase};
use kivact::prelude::*;

/// 横にドラッグして変える数値。
pub fn scrub(label: impl IntoText, value: Signal<f64>, history: Rc<RefCell<Vec<EditPhase>>>) -> impl Component {
    let edit = EditSession::new(move |initial| value.set(initial))
        .on_edit(move |phase| history.borrow_mut().push(phase));
    let origin = signal(0.0);
    let drag = use_drag::<()>(DragCoordinates::Window)
        .edit(edit, move || value.get_untracked())
        .on_start(move |_| origin.set(value.get_untracked()))
        .on_update(move |update: DragUpdate<()>| {
            value.set(origin.get_untracked() + update.delta.x);
        })
;
    let state = drag.source_state();
    let source = drag.source(());
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

コントローラーはロールと状態を設定してくれます。素のノードから作るコントロールは、次の modifier で自分で伝えます。`role`、`checked`、`expanded`、`selected`、`value_text` は `Node` のメソッドで、ビルダーには `Node` が提供します。

| modifier | 意味 |
|---|---|
| `.role(Role::Slider)` | コントロールの種類。支援技術の利用者が期待するロールを選びます。 |
| `.name("Rating")` | アクセシブルな名前。どのコントロールにも必要です。 |
| `.focusable(true)` | キーボードを受ける (Tab の停止位置になる)。 |
| `.value_text(..)`、`.value_min(..)`、`.value_max(..)`、`.value_now(..)` | スライダーやプログレスバーの値を、言葉と数値で。 |
| `.checked(..)`、`.selected(..)`、`.expanded(..)` | チェック、選択、開閉の状態。 |
| `.a11y_action(A11yAction::Increment, ..)` | 読み上げソフトから呼べる操作。コントロールにできることだけを提供します。 |

ロールもフォーカスもないノードの `.on_press(..)` はポインターでしか届かず、デバッグビルドで報告されます。まずコントローラー (`use_button`) を使ってください。

## 自前の形を測って描く

レイアウトの規則では表せないサイズのコントロールには、**プリミティブ**を使います。プリミティブは、サイズを関数で計算するリーフ要素です。そのうえで、描画レイヤー、入力ハンドラー、フォーカス、アクセシビリティ、テスト ID と、ノードの持つものはすべて備えています。この API は実験的です。

```rust,test_harness
use kivact::state::Binding;
use kivact::geometry::{Point, Size};
use kivact::prelude::*;
use kivact::custom::{Constraints, primitive};
use kivact::view::{A11yAction, Handled, KeyEvent, PointerEvent, PointerPhase};

const CELL: f64 = 20.0;

/// `max` 個の点の列。点灯している点の数が評価です。
pub fn rating(max: u32, value: impl IntoBinding<u32>) -> impl Component {
    let value = value.into_binding();
    primitive()
        // サイズは `max` だけで決まります。変わるとレイアウトが測り直します。
        .measure(max, |max: &u32, _: Constraints| Size::new(*max as f64 * CELL, CELL))
        .draw(move |d, size| {
            for i in 0..max {
                let color = if i < value.get() { ColorToken::Accent } else { ColorToken::Border };
                d.circle(Point::new(i as f64 * CELL + CELL / 2.0, size.height / 2.0), 7.0, color);
            }
        })
        .with_node(move |host| host.role(Role::Slider)
        .name("Rating")
        .value_min(0.0)
        .value_max(f64::from(max))
        .value_now(move || f64::from(value.get()))
        .value_text(move || format!("{} of {max}", value.get()))
        .focusable(true)
        .on_pointer(move |event: PointerEvent| {
            if matches!(event.phase(), PointerPhase::Down(_)) {
                value.set(((event.pos().x / CELL) as u32 + 1).min(max));
            }
        })
        .on_key(move |event: KeyEvent| match event.key() {
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
        .a11y_action(A11yAction::Increment, move |_| value.set((value.get() + 1).min(max)))
        .a11y_action(A11yAction::Decrement, move |_| value.set(value.get().saturating_sub(1)))
        .test_id("rating"))
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

`.draw(..)` は子の背面に、`.draw_over(..)` (`Node`) は子の前面に描きます。どちらも `Draw` とレイアウト済みの `Size` を受け取り、サイズか読んでいるシグナルが変わったときにだけ再実行されます。スタイルの `paint` はテーマ向けです。呼び出し側が置き換えられる既定の見た目を描くのに使い、`.draw(..)` はコントロール自身のものに取っておいてください。

## 自前の枠でテキストを編集する

`use_text_edit` はノードにネイティブのテキスト編集 (キャレット、選択、IME の変換、クリップボード、取り消し、標準のキー操作) を与えます。何も描画しないので、枠、余白、カウンターは自分で用意します。標準の `text_input` は、このコントローラーにテーマの枠を付けたものです。

```rust,test_harness
use kivact::state::Binding;
use kivact::interaction::use_text_edit;
use kivact::prelude::*;
use kivact::style::Align;

/// 文字数カウンターつきの入力欄。`max` はエディターが強制します。
pub fn limited_input(label: impl IntoText, value: impl IntoBinding<String>, max: usize) -> impl Component {
    let value = value.into_binding();
    let api = use_text_edit(value).max_length(max).placeholder("Type here");
    let state = api.control_state();
    let control = api.control();
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

binding は確定した値を受け取ります。IME の変換中は、エディターが未確定文字列を自分で保持し、binding には最終的なテキストが渡ります。変更のたびにコードを動かしたいときは、シグナルの代わりに `binding(read, write)` を渡します。ほかにも設定があります。複数行は `use_text_area(value)` で作ります。`.password(..)`、`.read_only(..)`、1 行入力で Enter を受ける `.on_submit(..)`、矢印、Tab、Escape のキーを自前のハンドラー (たとえば候補リスト) に渡す `.forward_keys([...])` です。

コードエディターのように、キー入力のたびに全体をコピーできないほど大きな文書には、より低い層の `kivact::custom::TextEditing` を使います。範囲編集を通知し、キーのポリシーを受け取り、行に色を付けられます。実験的な API です。rustdoc を見てください。

## 状態機械で振る舞いを書く

`Machine` で状態とイベントを明示します。純粋な `transition` は、次の状態、実行する effect、
遅延イベントを `Step` にまとめて返します。reactive な入力は `machine` の factory で読み、
副作用は effect runner で実行します。ハンドルの `get()` で状態を読み、`send()` でイベントを送ります。
タイマーは owner に属し、破棄時にキャンセルされます。

```rust,no_run
use kivact::prelude::*;
use kivact::state::{Machine, Step, machine};
use std::time::Duration;

#[derive(Clone, PartialEq)]
enum CopyState { Idle, Copied }
#[derive(Clone, PartialEq)]
enum CopyEvent { Pressed, Expired }
enum CopyEffect { Write(String) }

struct CopyButton { text: String }               // Self が遷移の読む入力
impl Machine for CopyButton {
    type State = CopyState;
    type Event = CopyEvent;
    type Effect = CopyEffect;
    fn transition(&self, state: &CopyState, event: CopyEvent) -> Step<CopyState, CopyEvent, CopyEffect> {
        match (state, event) {
            (_, CopyEvent::Pressed) => Step::to(CopyState::Copied)
                .with(CopyEffect::Write(self.text.clone()))
                .after(Duration::from_secs(2), CopyEvent::Expired),
            (CopyState::Copied, CopyEvent::Expired) => Step::to(CopyState::Idle),
            (CopyState::Idle, CopyEvent::Expired) => Step::stay(),
        }
    }
}

fn copy_button(text: impl IntoValue<String>) -> impl Component {
    let text = text.into_value();
    let m = machine(CopyState::Idle, move || CopyButton { text: text.get() }, |_, effect| match effect {
        CopyEffect::Write(s) => { spawn(kivact::platform::copy_text(s)); }
    });
    button(move || if m.get() == CopyState::Copied { "Copied" } else { "Copy" })
        .on_press(move || m.send(CopyEvent::Pressed))
}
```

この例は書き方を示すものです。既存のコピー用コントローラーが合う場合はそちらを使ってください。

## 関連項目

- [再利用できるコンポーネントを書く](custom-components.md): props、ビルダー、テーマ、アクセシビリティの転送。
- [ダイアログ、メニュー、ツールチップ、トーストを表示する](overlays.md): 同じコントローラーから作られた標準のオーバーレイ。
- [キャンバスで描く](canvas.md): region と、描画レイヤーで使う描画の語彙。
- [アプリをテストする](testing.md): 上のテストのように、ロール、キー、フォーカスを確かめる方法。
- `cargo doc -p kivact-interaction --open` ですべてのコントローラーと Part、`cargo doc -p kivact --open` で `kivact::custom`。


### native の編集する文字に描き足す

`TextEditing::line_styles(content, f)` は文字の形や編集の動作を変えずに、行の見た目を設定します。
関数は content、改行を除いた行、前の行が残した状態を受け取り、`(Vec<StyledRange>, 次の状態)`
を返します。`kivact::style::StyledRange` の範囲は行内の UTF-8 byte です。
`color`、`background`、`decoration` で文字色、背景、下線などを設定します。
`TextDecoration` は theme の色、文字の長さ、`TextDecorationStyle` を受け取ります。

文書全体の byte 範囲には `TextEditing::decorations(content, f)` を使い、
`kivact::custom::Decoration` を返します。種類は `Background`、`Outline`、
`Underline(TextDecorationStyle)` です。content と theme の変更に追従します。
password の値と placeholder にはこれらを描きません。

編集する node には子（`custom::inline_view` を含む）、`draw`、`draw_over` を付けられます。
描画 layer は native の文字、選択、caret の上に重なります。
`Node::draw_with_layout` は canvas 座標の `custom::TextLayout` も受け取ります。
行の箱、caret、選択の矩形、`hit_test` を使い、文字、選択、配置の変更に追従できます。
`NodeRefExt::text_layout` の単独の問い合わせは window 座標で、指定した行だけを整形します。
caret と `hit_test` の offset は Unicode scalar 数、見た目の範囲は UTF-8 byte 数です。
