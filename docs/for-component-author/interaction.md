# Interaction の部位から部品を作る

アプリの作者は `kivact::ui` の標準部品と公式 kit を使います。`kivact::interaction` の `use_*` は、部品の作者のための見た目を持たない振る舞いです（react-aria の `useButton`・`useMenu` などと同じ層）。部品を新しく作る、または kit の部品を作り直すときにだけ使います。

controller は `use_*` で setup のうちに作り、`trigger`・`content`・`item` などの Part を自分の node へ付けます。

> **Part を `.with()` で消費する前に `state()` を読みます。** `state()` は実際に付いた対象の hover、focus、disabled や選択を返します。`.with(part)` の後では part は手元に残りません。状態を似せた別の signal は作りません。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::use_checkbox;

fn check(label: impl IntoText, value: Signal<bool>) -> Node {
    let control = use_checkbox(value).control();
    let state = control.state();
    node().name(label).part(control).style(move || {
        style().size(24, 24).border(1, ColorToken::Border)
            .background(if state.get().checked.is_checked() {
                ColorToken::Accent
            } else {
                ColorToken::Surface
            })
    })
}
```

click・Space・AT の toggle は同じ binding を使います。操作のたびに caller の writer に提案し、writer が受理した値を描きます。独自の on_key や a11y handler で同じ反転を重ねないでください。利用者の handler は既定の部位 handler より先に動き、処理済みや error を返すと後続が止まります。`Handled::PreventDefault` を返すと、後続と祖先へは `Handled::No` と同じく伝わり、Enter/Space の activate、Escape の Dismiss、wheel の scroll といった既定動作だけが止まります。

## button と二値の部品

押すだけの button は `use_button()` で作ります。`control()` が `Role::Button` と focus を付け、caller の `on_press` が click・Enter・Space・Activate で一度だけ動きます。`node().role(Role::Button).focusable(true)` は意味だけを付ける低い層で、起動を別の口（canvas の region、native control）が持つ要素に使います。

| react-aria | Kivact | role |
|---|---|---|
| `useButton` | `use_button()` | `Button` role（handle は `ButtonApi`） |
| `useToggleButton` | `use_toggle_button(pressed)` | `Button`（checked を持つ） |
| `useCheckbox` | `use_checkbox(checked)`（`.indeterminate(..)` あり） | `CheckBox` |
| `useSwitch` | `use_switch(checked)` | `Switch` |

二値の三つは同じ `Toggle` を返します。Space・click・Activate・Toggle で反転し、Enter は toggle button だけが反転します。

## 名前の規則（Radix / Ark UI との対応）

部位の名前は Ark UI と Radix の anatomy に合わせます。

- 浮く部分と、開閉する中身は `content()`。
- 一覧の項目は `item(key)`（list・menu・tree・select・combobox・palette・tabs）。項目の容器は `list()`（tabs の tablist を含む）。
- 表は `grid()`・`header(column)`・`row(key)`・`cell(key, column)`。表の容器は `root()` でなく `grid()` です。この部位そのものが ARIA の `grid` role を持ち、表の外側に別の容器の部位がないからです。
- zoom_pan の面は `viewport()`。slider は `thumb()` と `track(axis)`、number field の入力は `input()`。
- 一つの要素だけの部品（button、checkbox、text edit）は `control()`。
- 開いているかの問合せはすべて `shown() -> Value<bool>`。命令は `open()`・`close()`・`toggle()`。
- caller が持てる状態は、その名前の builder に `Binding` を渡します（`Table::sort(binding)`・`widths(binding)`、`List::anchor(binding)`）。同じ状態を読む問合せは `<名前>_value()` です（`sort_value()`・`widths_value()`・`anchor_value()`）。
- callback は `impl Fn(..) -> R + 'static`（`R: HandlerResult`）で受けます。
- handle はすべて Copy で、receiver は `self` です。

| Radix / Ark UI の部位 | Kivact の Part |
|---|---|
| Dialog: Trigger / Content / Title / Description / Close（CloseTrigger） | `use_dialog`: 開閉は caller の binding / `content()` / `title()` / `description()` / `close_trigger()` |
| Popover: Trigger / Content | `use_popup`: `trigger()` / `content()` |
| Tooltip: Trigger / Content | `use_tooltip`: `trigger()` / `content()` |
| Collapsible: Trigger / Content | `use_disclosure`: `trigger()` / `content()` |
| DropdownMenu・ContextMenu: Trigger / Content / Item / CheckboxItem / RadioGroup・RadioItem / Label / Group / Separator / Sub | `use_menu`: `trigger()`・`context_trigger()` / `content()` / `item(key)` / `MenuEntry::item`・`checked`・`radio`・`label`・`group`・`separator`・`submenu`・`inset` |
| Select: Trigger / Content / Item | `use_select`: `trigger()` / `content()` / `item(key)` |
| Combobox: Input / Content / Item | `use_combobox`: `input()` / `content()` / `item(key)`（`popup(open)` で開閉、`multiple(values)` で複数選択） |
| Menubar: Root / Trigger | `use_menubar`: `bar()` / `item(at)` |
| Tabs: List / Trigger / Content | `use_tabs`: `list()` / `item(key)` / `panel(key)` |
| Calendar（react-aria・Ark）: Grid / Row / Cell / PrevTrigger / NextTrigger | `use_calendar_grid`: `grid()` / `row()` / `cell(month, day)` / `previous_month()`・`next_month()`・`previous_year()`・`next_year()` |
| Listbox（Ark）: Content / Item | `use_list`: `list()` / `item(key)` |
| TreeView（Ark）: Tree / Item | `use_tree`: `list()` / `item(key)` |
| Slider: Track / Range / Thumb | `use_scalar`・`use_range_scalar`: `track(axis)` / caller の描画 / `thumb()`・`thumb(RangeThumb::Low)` |
| NumberInput（Ark）: Input | `use_scalar`: `input()` |
| Splitter（Ark）: ResizeTrigger | `use_splitter`: `handle()` |
| Toast（Ark）: Toaster / Root | `toaster()`: `region(live)` / `item(id)`（`ToastState::depth` は同じ region の中の深さ） |

## 編集の確定と取消し

値を書き換える部位（`use_scalar`・`use_range_scalar`・`use_splitter`・`use_zoom_pan`）は `.on_edit(f)` を持ち、drag・キー1回・AT の操作1回の始まり・確定・取消しを `EditPhase` で通知します。Escape・PointerCancel・drag 中の focus の喪失では、編集前の値へ戻してから `Cancel` を通知します。undo の履歴は利用者が持ちます。

slider は `use_scalar(value, range)` の `thumb()` を focus を受ける要素に、`track(axis)` を pointer が動く要素に付け、thumb を track の中に置きます。track は press の位置を値へ写し、thumb へ focus を移し、release まで pointer に従います。横は左端から（右から左の部分木では右端から、`kivact::view::current_direction()`）、縦は下端から値が増え、thumb の長さだけ両端を空けます。右から左の横の slider では ArrowLeft が増やします。2つの thumb は `use_range_scalar(low, high, range)` で、互いを越えません。

部位を自分で組むときは、連続した編集の開始値と取消しを `EditSession<T>` に任せます。setup で一度作り、最初の書込みより前に `begin(initial)`、release で `commit()`、Escape と PointerCancel で `cancel()` を呼びます。`cancel()` は開始値を binding へ戻してから `EditPhase::Cancel` を通知します。owner が破棄されたときも、進行中の編集は同じ経路で取り消します。`use_drag` には `.edit(session, || 今の値)` で渡せます。drag の開始で `begin`、release で `commit`、Escape と engine の Cancel で `cancel` が自動で呼ばれます。

```rust,no_run
use kivact::prelude::*;
use kivact::ui::EditPhase;
use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};

fn width_editor(value: Signal<f64>) -> Node {
    let edit = EditSession::new(move |initial| value.set(initial))
        .on_edit(|phase: EditPhase| println!("{phase:?}"));
    let origin = signal(0.0);
    let source = use_drag::<()>(DragCoordinates::Window)
        .edit(edit, move || value.get_untracked())
        .on_start(move |_| origin.set(value.get()))
        .on_update(move |update: DragUpdate<()>| {
            value.set(origin.get() + f64::from(update.delta.x));
        })
        .source(());
    node().name("Width").focusable(true).part(source)
}
```

`begin` は進行中なら何もせず、`commit` と `cancel` は編集中でなければ何もしません。部品の独自 snapshot や phase の通知を別に重ねる必要はありません。

押しただけでは値が変わらない drag（押下が選択や起動で、動かして初めて編集になる canvas の editor）は、`.edit(..)` の代わりに `.edit_on_change(session, || 今の値)` を使います。編集は押下ではなく最初の `drag.change(|| 書込み)` の直前に始まり、変更なしに終わった drag は何も通知しません。値の書込みはすべて `change` を通します。release の `on_end` の中の `change`（最後の位置）もその drag の編集に入ります。

canvas の複数選択も公開の `ListGesture::from_modifiers(modifiers, &policy)` で List と同じ command/Shift の規則を使えます。矩形は `Rect::from_points(a, b)` で端点の順序を吸収し、`Rect::intersects` は辺が触れる場合も選択に含めます。Creative の private な domain 分割と共通の押下状態の構成例は [Creative editor の設計](../for-kivact-developer/architecture/creative-editors.md) にあります。pointer の捕捉や編集の phase を独自に重ねず、公開の `Drag`・`EditSession` に任せます。

Escape は、edit の有無によらず、drag 中ならその drag のどの source に届いても drag を止めます。`on_end` は最後の位置の `DragPhase::Cancel` を受けるので、drop 先の線のような drag の後始末は `on_end` の Cancel だけに書き、Escape を手で扱いません。drag 中でない Escape は通るので、部品は選択を空にするなどの自分の Escape を持てます。ただし node 自身の handler（`on_key`・`on_key_default`）は part より先に走るので、drag の後に Escape を受ける key handler は source の後に付ける part に置きます。canvas の region が focus を持つなら、その part に `.region_events()` を付けます。中ボタンの pan のように主ボタン以外でも始まる drag は `.buttons([MouseButton::Middle])` で足し、`DragUpdate::button` で区別します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{DragCoordinates, DragUpdate, EditSession, use_drag};
use kivact::primitive::part;
use kivact::view::Handled;

fn stepper(value: Signal<f64>, selected: Signal<bool>) -> Node {
    let edit = EditSession::new(move |initial| value.set(initial));
    let origin = signal(0.0);
    let drag = use_drag::<()>(DragCoordinates::Window)
        .edit_on_change(edit, move || value.get_untracked())
        .on_start(move |_| origin.set(value.get()));
    // 10 px ごとに 1 進む。動かさない押下は編集にならない。
    let drag = drag.on_update(move |update: DragUpdate<()>| {
        let next = origin.get() + f64::from((update.delta.x / 10.0).round());
        if next != value.get() {
            drag.change(|| value.set(next));
        }
    });
    let clear = part(|input| input).on_key(move |event| {
        if event.key == Key::Escape {
            selected.set(false);
            Handled::Yes
        } else {
            Handled::No
        }
    });
    node().name("Step").focusable(true).part(drag.source(())).part(clear)
}
```

## Native 編集を付ける

`use_text_edit` は普通の node に付く headless な編集です。標準 UI の text input を使わず、外観を自分で作れます。

```rust,no_run
use kivact::prelude::*;
use kivact::{Binding};
use kivact::interaction::use_text_edit;

fn input(label: impl IntoText, value: impl Into<Binding<String>>) -> Node {
    let editor = use_text_edit(value).placeholder("入力してください").max_length(80_usize);
    node().name(label).part(editor.control())
        .style(style().width(240).height(36).padding(8)
            .border(1, ColorToken::Border).background(ColorToken::Surface))
}
```

`TextEdit` の設定は最初の mount より前、multiline は `control()` を作る前に選びます。一つの handle に同時に付く control は一つです。閉じた branch が control を破棄した後は、新しい Part を作って再表示できます。preedit は binding へ書かず（変化を受ける口は binding だけで、別の処理は `binding(read, write)` の write に書きます）、composition の確定・取消を native の経路で処理します。caret、選択、clipboard、IME、scroll と a11y を個別に再実装する必要はありません。入力 node は native leaf で、子や独自 draw を持てません。binding は `String` なので、確定のたびに全体の値を複製して比べます。長い文書や独自のキー操作が要る編集は、[`TextEditing`](primitive.md#native-編集の下位契約) の範囲の変更と behavior で作り、標準のキー操作は `text_edit_policy` に委ねます。

[独自入力の fixture](../../dev/tests/fixtures/external-design-system/src/text_input.rs)と、その E2E は role、IME、caret/selection、AT の SetValue を公開 API で検証します。[code editor の fixture](../../dev/tests/fixtures/external-design-system/src/code_editor.rs) は、字下げ・undo・キーワードの色・行数を同じ公開 API で作ります。これは手動の OS IME やスクリーンリーダーを実行した結果ではありません。

## 一覧・検索・表

`use_list` は順序と選択の binding に対して `list()`/`item(key)` の部位を返します。`ListItem` と `MenuEntry` は任意の `icon`（`kivact::style::Icon`。theme が名前で解決する `Icon::named("check")` か、明示の `Asset`）と `description` を持てます。`use_table` は同じ選択を使い `grid`/`header`/`row`/`cell`/`resize` を加えます。

並べ替え・絞り込み・頁分けは `use_table_model(rows, key, columns)` に任せます（TanStack Table の row model）。列ごとの `ColumnDef` に比較（`compare`）、絞り込みとの照合（`filter`）、表示（`visible`）を持たせ、sort と page を `Binding` で、filter の文字列を値で渡します。`rows()`・`keys()`・`columns()`・`row_count()`・`page_count()` を表示と `use_table` へ渡し（全頁の絞り込み後の key は `filtered_keys()`。「n of m row(s) selected.」のような集計に使います）、sort の binding は `use_table(..).sort(binding)` にも渡します。行と key の型は自由です。[一覧と表の例](../for-kivact-user/how-to-guide/lists-tables.md)を参照してください。

### 一覧と表の部位を自分で描く

標準の `kivact::ui::list` と `kivact::ui::data_table`（アプリ向けの手順は [一覧と表](../for-kivact-user/how-to-guide/lists-tables.md)）で足りない見た目や操作の部品だけ、`use_list` と `use_table` の部位を自分で描きます。

| 必要なこと | 使う口 |
|---|---|
| 安定した key ごとに行を作り、並べ替えても owner を保つ | `for_each` |
| 見える範囲だけ行を作る | `virtual_list`。残す状態は app の keyed model に置く |
| 項目の選択・矢印・typeahead・roving focus を付ける | `interaction::use_list` の list/item の Part |
| header・行・cell と二次元のキー移動を持つ表を作る | `interaction::use_table` の grid/header/row/cell の Part |
| 表の行を並べ替え・絞り込み・頁分けする | `interaction::use_table_model` |

`ListItem` は key・label・disabled と、任意の icon・description を持つ項目データで、`ListSelection` は caller の選択 binding を受け取ります。`for_each` で作る行に List の Part を重ねられます。`Table` も同じ選択を使い、表示する列やソート後の rows は caller が決めます。次の例が最小構成です。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{ListSelection, use_list};

fn choices() -> impl IntoView {
    let items = vec![ListItem::new(1_u32, "赤"), ListItem::new(2, "青")];
    let selected = signal(None);
    let choices = use_list(items.clone(), ListSelection::Single(selected.into()));
    column(for_each(items, |item| *item.key(), move |key, item| {
        let part = choices.item(key);
        let state = part.state();
        node().part(part)
            .child(text(move || item.get().label()))
            .style(move || style().padding(8)
                .opacity(if state.get().selected { 1.0 } else { 0.7 })
                .focus(style().ring(2, ColorToken::Focus)))
    })).part(choices.list()).name("色")
}
```

List の policy、page size、scroll callback は最初の Part を作る前に設定します。候補の順序と選択は binding が正本です。filter は選択を勝手に消しません。focus の候補は mount 済みで有効な部位に限られます。virtual list で画面外の行が破棄されても残す draft と選択は、安定した key の model に置きます。

`Table` は同じ選択の上に grid/header/row/cell/resize の部位を加えます。列は `ColumnDef` ひとつで、header が並べ替えるかどうかは `compare` を持つかどうかで決まります。次の例は二列の最小構成です。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{ColumnDef, ListSelection, use_table};

#[derive(Clone, PartialEq)]
struct Person { id: u32, name: String, age: u32 }

fn people() -> impl IntoView {
    let rows = signal(vec![Person { id: 1, name: "Ada".into(), age: 36 }]);
    let selected = signal(Vec::new());
    let columns = vec![
        ColumnDef::new("name", "名前").compare(|a: &Person, b: &Person| a.name.cmp(&b.name)),
        ColumnDef::new("age", "年齢").compare(|a: &Person, b: &Person| a.age.cmp(&b.age)),
    ];
    let grid = use_table(rows, |p| p.id, columns.clone(),
        ListSelection::Multiple(selected.into()));
    let headers = row(columns.iter().map(|c| {
        node().part(grid.header(*c.key())).child(text(c.label_value()))
            .style(style().width(120))
    }).collect::<Vec<_>>());
    let body = for_each(rows, |p| p.id, move |key, person| {
        row((
            node().part(grid.cell(key, "name")).child(text(move || person.get().name))
                .style(style().width(120).focus(style().ring(2, ColorToken::Focus))),
            node().part(grid.cell(key, "age")).child(text(move || person.get().age.to_string()))
                .style(style().width(120).focus(style().ring(2, ColorToken::Focus))),
        )).part(grid.row(key))
    });
    column((headers, body)).part(grid.grid()).name("利用者")
}
```

並べ替え・絞り込み・頁分けは `use_table_model` が行います。列ごとの `ColumnDef` に比較と絞り込みの関数を持たせ、sort と page の binding と filter の文字列を渡すと、表示する行（`rows()`）と列定義（`columns()`）、件数と頁数を返します。sort の binding は `use_table(..).sort(binding)` にも渡し、header の操作で model が並べ替わるようにします。`TableModel::columns()` は表示する列だけを返すので、`use_table` へそのまま渡します。

```rust,no_run
use kivact::prelude::*;
use kivact::interaction::{ColumnDef, ListSelection, use_table, use_table_model};
use kivact::view::SortDirection;

#[derive(Clone, PartialEq)]
struct Person { id: u32, name: String, age: u32 }

fn people(rows: Signal<Vec<Person>>, query: Signal<String>) -> impl IntoView {
    let sort = signal(None::<(&'static str, SortDirection)>);
    let page = signal(0_usize);
    let model = use_table_model(rows, |p: &Person| p.id, vec![
        ColumnDef::new("name", "名前")
            .compare(|a: &Person, b: &Person| a.name.cmp(&b.name))
            .filter(|p: &Person, q: &str| p.name.contains(q)),
        ColumnDef::new("age", "年齢").compare(|a: &Person, b: &Person| a.age.cmp(&b.age)),
    ])
    .sort(sort)
    .filter(query)
    .page(page, 20_usize);
    let selected = signal(Vec::new());
    let grid = use_table(model.rows(), |p: &Person| p.id, model.columns(),
        ListSelection::Multiple(selected.into()))
        .sort(sort);
    let body = for_each(model.rows(), |p| p.id, move |key, person| {
        row((
            node().part(grid.cell(key, "name")).child(text(move || person.get().name)),
            node().part(grid.cell(key, "age")).child(text(move || person.get().age.to_string())),
        )).part(grid.row(key))
    });
    column((
        column(body).part(grid.grid()).name("利用者"),
        text(move || format!("{} 件、{} / {} 頁", model.row_count().get(), page.get() + 1,
            model.page_count().get())),
    ))
}
```

column width の保存も binding で指定できます（`widths(binding)`）。`TableFocus::Cells` は cell を、`Rows` は行をキー操作の対象にします。policy と binding は Part を作る前に選びます。role だけを手で付けてキー操作を複製せず、同じ部位を使います。

`use_combobox(options, selection)` は query と active な候補を持ちます。`use_text_edit(combobox.query()).control()` と `combobox.input()` を同じ node に付け、`visible()` の key を描く行へ `item(key)`、行を含む箱へ `content()` を付けます。focus は入力に残り、active_descendant が候補を示します。候補を popup にするときは `.popup(open)` を付けます。矢印で開き、Enter は開いている間だけ選び、Escape と選択と focus を失うことで閉じるので、kit はこれらのキーを書きません。複数選択は `.multiple(values)` で、選ぶと `values` の中で切り替え、query を空にして開いたままにし、空の入力の Backspace が最後の一つを外します。`use_command_palette` も入力・候補・dialog の部位を caller が描きます（標準の `kivact::ui::command_palette` がその描画です）。どれも Enter や矢印の規則を再実装しません。

`use_menubar(titles, menus)` は menu の列の振る舞いです。`bar()` を行に、`item(at)` を各 menu の trigger の node に付けます（menu の `trigger()` を先に付け、`item(at)` を後に付けます）。左右の矢印は閉じた trigger の間で focus を、menu が開いているときは開いた menu を動かし（端で折り返し、右から左の向きでは反転）、開いている間に別の trigger にポインターを置くとその menu に移ります。

`use_dialog`・`use_popup` は既存の overlay/focus authority を使います。タイトルと説明の関係、dismiss、復帰を部位に任せ、caller は枠と配置を描きます。[external-design-system の実装](../../dev/tests/fixtures/external-design-system/src/lib.rs)は、その分離を示しています。

閉じたときの focus の戻し方は popup が一つの `FocusReturn { target, on }` で持ちます。`target` は anchor（`FocusReturnTarget::Anchor(node)`）か、surface を mount する前に focus を持っていた node（`Opener`）、`on` は戻す閉じ方の集合（`ReasonSet`）です。既定は anchor に Escape・選択・支援技術の dismiss で戻し、外側の press と Tab は押した先に残します。`use_popup(..).focus_return(FocusReturn::preserve())` は常に戻さず、`FocusReturn::opener(ReasonSet::DISMISS)` は右クリックの menu のように開く前の focus に戻します。`ReasonSet::PROGRAMMATIC` と `UNMOUNT` を足すと、アプリが閉じた場合と surface が取り除かれた場合も、focus が surface の中にあったときだけ戻します。戻せるか（対象が生きていて、focusable で、残る modal の中にあるか）は core が送る時点で決めるので、部品は検査しません。modal の復帰は overlay の cleanup が行うので、popup は重ねて戻しません。

## 絵を表示する面の拡大と pan

`use_zoom_pan(transform_binding).content_size(size).viewport()` を描画面の node に付けます。`ViewTransform` の `offset` と `scale` を描画へ渡し、pointer の座標は `to_content` で絵の座標へ戻します。`to_view` は逆の変換、`fit(content, view)` は全体が中央に収まる変換です。

wheel は pointer 中心、`+`・`-` は面の中央を中心に拡大縮小し、`0` は全体を表示します。中ボタンか Space+左ボタンの drag で pan し、通常の左ボタンは選択や gizmo に使えます。pan していない Space は `Handled::PreventDefault` で返すので、アプリの shortcut へは届き、面の中で focus 中の部位は起動しません。二本の指を面に置くと pinch になり、指の間の距離の変化で拡大縮小し、指の中点で pan します（指の下の絵の点が中点に付いてくる）。一本目の指だけでは何もしないので、選択や gizmo に使えます。pinch はどちらかの指を離すと確定し、指の cancel・Escape・focus の喪失・disabled・破棄で開始時へ戻ります。pan の Escape・pointer cancel・disabled・破棄は開始時へ戻し、`on_edit` は共有 `EditPhase` を通知します。専用の touchpad pinch イベントは現公開入力にはないため未対応です（touchpad の拡大は wheel として届きます）。

## touch の scroll と press

scroll container 内の primary touch は、最初の位置から 8 論理 px を超えるまで通常の press として届きます。主な移動方向に scroll できる祖先があると、元の capture に `PointerPhase::Cancel` が一度届き、以後は native scroll になります。部品は Cancel で drag/edit の途中の状態を戻し、Up を待って確定しないでください。境界では外側の scroll container へ移り、modal の外へは伝わりません。

scroll できる祖先がない方向の drag は press を保ち、閾値内の tap は一度だけ活性化します。二本目の touch が始まると単指 scroll の調停を終え、既存の pinch handler に pointer を渡します。慣性・overscroll・二本指 scroll はありません。接触の取消し、focus の喪失、suspend、元の対象の削除・非表示や modal の切替では capture を取り消し、後から Up が来ても確定・活性化しません。

native scroll に入った接触は Cancel 済みなので、後から二本目を足しても同じ接触の Down を合成しません。独自の pinch を始めるときは、接触を終えて新しく触れてください。scroll の閾値を超える前に二本目を置けば、元の二つの Down をそのまま pinch handler が受け取れます。
