# 改修後の API へアプリを移行する

[English](../../en/how-to-guide/migrating-to-rework.md)

Kivact の公開 API は、1 回の破壊的な変更で作り直しました。互換のための層はありません。下の旧い名前はすべて無くなっており、各行の右の列が書き換え先です。表は、import するモジュールごとに分けてあります。[チュートリアル](../getting-started/README.md)とほかのガイドは、すでに新しい API を使っています。

## 始める前に

**Kivact を使うリポジトリは、すべて同じ revision に固定してください。** アプリとコンポーネントのライブラリの git 依存で、同じ `rev` を使い、まとめて更新します。1 つのビルドに Kivact の revision が 2 つあると、別々の crate になり、型が互いに一致しません。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact", rev = "<commit>" }
```

**デバッグビルドの依存の最適化は 1 行で済みます。** `Cargo.toml` の `[profile.dev.package.<name>]` に Kivact の crate が並んでいたら、その節を消してください。Kivact の中の crate は、変更のたびに分割・統合されるためです。代わりに次のように書きます。

```toml
[profile.dev.package."*"]
opt-level = 3
```

自分の crate は最適化されず、デバッグしやすいままです。依存先の Kivact と描画部品を含め、それ以外はすべて最適化されます。

**このページは上から順に進めてください。** 最初にコンポーネント、次に状態、最後にスタイルを直します。コンポーネントを新しい形にすると、後の表のコンパイルエラーの多くが消えます。

## Components

| 変更前 | 変更後 |
|---|---|
| `fn header() -> impl Element { .. }`, `fn panel() -> impl IntoView { scope(move \|\| ..) }` | `#[component] fn header() -> impl Element { .. }` |
| `#[component] fn card(..) -> impl IntoView`（呼出し側は `.style(..)` を付けられなかった） | `#[component] fn card(..) -> impl Element`。呼出し側の `.style(..)`、`.name(..)`、`.test_id(..)`、`.disabled(..)` と handler は、部品の root に付く |
| `#[prop(optional)] on_close: Option<Callback<()>>` | `on_close: Option<Callback<()>>`。`Option<..>` の prop は、それだけで省略できる setter になる |
| `impl Element` と `impl IntoView` を持つ手書きの builder の struct | `#[component]` の関数 |
| 自分の prop の型の隣の `constant_value!(Tone);` | 型に `#[derive(Clone, PartialEq, Prop)]` を付ける |
| `scope(move \|\| view)`（prelude と `kivact::view` にある） | `#[component]`。macro を使わずに部品のライブラリを組む場合は `kivact::custom::scope` を使う |
| `impl IntoText` を取り `.into_text_value()` を呼ぶ補助関数 | `Value<String>` の prop |
| `impl IntoValue<T>` を取り `.into_value()` を呼ぶ補助関数 | `Value<T>` の prop |
| `impl Fn() -> V` の内容を取る補助関数 | `ViewFn` の prop |
| prop の型として使う `type Title = Value<String>;` | `Value<String>` と書く。macro は `Value`、`Signal`、`Callback`、`View`、`ViewFn`、`Vec<View>` という名前をそのまま認識し、型の別名は普通の prop になる |

`App::run` は view を返す任意の関数を受け取ります。名前が `app` の根の関数は、普通の関数のままにしてください。`#[component]` を付けると `App` という型が生成されるためです。

## State: `kivact::state`

| 変更前 | 変更後 |
|---|---|
| `Binding<T>` | `Signal<T>` |
| 引数 `value: impl IntoBinding<T>` | `value: Signal<T>` |
| `binding(get, set)` は `Binding<T>` を返していた | `binding(get, set)` は `Signal<T>` を返す |
| `task.project(\|t\| &t.done, \|t\| &mut t.done)` | `bind!(task.done)` |
| `Signal<Vec<T>>` の `for_each` は、各行に `Binding<T>` を渡していた | 各行は `Signal<T>` を受け取る。読み取り専用の元からは、引き続き `Value<T>` が渡される |
| 部品の本体での `task.get_untracked()` | `task.get()`。部品の本体は読み取りを追跡しない |
| `Notifier`、`notifier()` | `Trigger`、`trigger()`。`track()` と `notify()` を使う |
| `signal.notify()` | シグナルの隣の `Trigger`: `changed.notify()` |
| `use_context::<T>()` は `Value<T>` を返し、`session.get().user.get()` と書いた | `T` を返す: `session.user.get()` |
| `try_use_context::<T>()` は `Option<Value<T>>` を返していた | `Option<T>` を返す |
| `signal.update(f)` は `R` を返し、削除済みの部品のシグナルでは panic した | 何も返さない。削除済みの部品のシグナルへの書込みは報告され、何もしない。更新のあとで計算した値は `with` で読む |
| `Callback::bind(..)`、`Callback::to_fn()` | `Callback` をそのまま渡す。すべての `on_*` と callback の prop がそれを受け付ける |
| `let c: Callback<()> = (move \|\| ..).into();` | `let c = callback(move \|_\| ..);` |
| `kivact::custom::Reactive<T>` | `Value<T>` を保持する |
| `show`、`show_some`、`dynamic` の境界の `IntoReactive<bool>` | `impl IntoValue<bool>`。呼出し側は変わらない |
| `kivact::state::Owner` | `kivact::custom::Owner` |
| prelude にあった `IntoValue`、`IntoText`、`IntoBinding`、`PropValue`、`ItemKey`、`IntoSelection`、`Selection` | prelude には無い。`IntoValue`、`IntoText`、`IntoHandler` は `kivact::state` にあり、`IntoSelection` と `Selection` は `kivact::interaction` にある。`IntoBinding` は無くなった |

## Event handlers

| 変更前 | 変更後 |
|---|---|
| `on_press(impl Into<Callback<()>>)` | イベントの無い handler（`on_press` と、すべての `Callback<()>` の prop）は、クロージャ `move \|\| ..` か `Callback<()>` を受け取る |
| イベントのある handler（`on_key`、`on_activate`、`on_commit`、`on_edit`、`Callback<E>` の prop） | 変わらない。クロージャ `move \|event\| ..`。共有の `Callback<E>` を渡すには `move \|e\| cb.call(e)` と書く |
| `.on_draft_commit(f)` | `.on_commit(f)` |
| `stepper(items, current).on_step_press(f)` | `stepper(items, current)` に `current: Signal<usize>` を渡す。シグナルを読み、`.on_edit(\|phase\| ..)` で編集の各段階を受け取る |
| `tag_input(label, tags).on_add(f).on_remove(g)` | `tag_input(tags).label(label)`。`tags: Signal<Vec<String>>` |
| `avatar(..).on_open(f)` | 削除。avatar が開くものは `Signal<bool>` を受け取る。`avatar` の rustdoc を見てください |
| `hover_card(..).on_open_change(f)`、`navigation_menu(..).on_open_change(f)` | `.open(signal)` を使い、シグナルを読む |
| 二つの handler で共有するクロージャ（`let save = move \|\| ..;` を二度渡す）、または `Callback::from(..)` で作ったもの | `let save = callback(move \|_\| ..);` を作り、両方に `save` を渡す。共有する handler はすべて `Callback` で、イベントの有無を問わない |

## Views and elements: `kivact::view`

| 変更前 | 変更後 |
|---|---|
| 読込み中、エラー、完了のための `dynamic(move \|\| r.state(), \|state\| match state { .. .into_view() })` | `async_view(r, ready).loading(\|\| ..).fallback(\|error, retry\| ..)` |
| `.fallback(\|error, retry: Retry\| ..)` の `Retry` と `retry.run()` | `retry` は `Callback<()>`: `.on_press(retry)` または `retry.call(())` |
| `kivact::task::AsyncSnapshot` | `AsyncState` |
| `card(..).disabled(..)`、`card(..).on_press(..)`（一つの部品のメソッド） | 同じ呼出し。今はすべての部品が持つ `Element` のメソッド |
| 各部品が定義していた `.variant(..)` | 同じ呼出し。今は `Element::variant` |
| `use_direction()` は `Value<ReadingDirection>` を返していた | `ReadingDirection` を返す |
| 保存の前にフォーカスされた欄に下書きを確定させるための、不可視の node とタイマー | `kivact::view::commit_focused_edit().await` |
| `spacer()` は中身より小さくできなかった | `spacer()` の最小サイズは 0 |
| — | `PointerEvent::target_size()` と `DragUpdate::target_size()`: handler が付いている node の大きさ |

## Standard components: `kivact::ui`

| 変更前 | 変更後 |
|---|---|
| `text_input("Name", value)` | `text_input(value).label("Name")` |
| `field("Name", text_input("Name", value))` | `field("Name", text_input(value))`: `field` が操作部品に名前を付ける |
| `text_area("Bio", value)`、`select("Color", items, value)`、`combobox("Colors", items, value)`、`slider("Volume", value)`、`number_input("Count", value)`、`date_input("Date", day)`、`date_range_input(..)`、`time_input(..)`、`date_time_input(..)` | 名前は `.label("..")` に移るか、外側の `field("..", ..)` に移る |
| `button(label)`、`checkbox(label, value)`、`switch(label, value)`、`radio_group(name, items, value)` | 変わらない。ラベルは見える |
| 名前の無いフォーカスできる操作部品 | 部品が mount されたときに報告される（debug build と `kivact::test` ではエラー） |
| `tooltip(text).attach(trigger)` | `tooltip(text, trigger)` |
| `toaster()` | `use_toaster()` |
| `announcer()` | `use_announcer()` |
| `selection_field(label, value_label).content(f)` | `select(items, value).label(label).trigger(view)` |
| `input_group_with(label, \|\| children)` | `input_group(label, children)` |
| `dialog(title, open).mode(DialogMode::Popover)` | `kivact::interaction::use_popup(open)` と、その `surface(placement, content)` |
| `use_dialog(open, DialogMode::Popover).anchor(node)` | `kivact::interaction::use_popup(open).anchor(node)` と、その `surface(placement, content)`（モーダルでない surface は popup。`use_dialog` はモーダルだけ） |
| `MenuApi::while_open(f)`、`CommandPaletteApi::while_open(f)` | `api.surface(placement, content)`、または `show(api.is_open(), f)` |
| `use_menu_with_open(items, open)` | `use_menu(items, open)`。`open` のシグナルは、開く振る舞いのすべてで必須 |
| `api.control_state()`、`api.item_state(key)` など `*_state()` の getter | `api.control().state()`、`api.item(key).state()`: `Part::state()` |
| `kivact::ui::Snap` | `kivact::interaction::Snap` |
| `kivact::interaction::ComboboxMode` | `kivact::ui::ComboboxMode` |
| `kivact::interaction::ViewTransform` | `kivact::interaction::ZoomPan` |
| `UiRecipes`、`theme.ext::<UiRecipes>()`、`recipes.button(\|previous\| ..)` | `kivact::ui::theme::ButtonTheme` などの `<Name>Theme` の表: `theme.with(ButtonTheme::default().root(\|theme, look, base\| ..))` |
| 各部品の `UiTokens` と `*Tokens` | 各部品のテーマ変数。例: `kivact::ui::theme::button::HEIGHT_MD` |
| `kivact::ui` の `*Recipes`、`*State`、`Look`、`Variant` | `kivact::ui::theme` |
| — | `icon_button(icon, label)` |

## Async: `kivact::task`

| 変更前 | 変更後 |
|---|---|
| `action(ActionMode::Latest, f)` | `action(f)`: 新しい実行は、進行中のものを取り消す |
| `action(ActionMode::Queue, f)`、`action(ActionMode::QueueLatest, f)` | `queued_action(f)`: 実行は一つずつ順番に行われ、すべての実行が残る |
| `action.close()` | 削除。キューの action は、その持ち主と一緒に終わる |
| `.on_complete(\|outcome\| match outcome { ActionOutcome::Succeeded(v) => .., ActionOutcome::Failed(e) => .. })` | `.on_complete(\|result: Result<T, UiError>\| ..)` |
| スレッドから値を受け取るための channel と `spawn` | `let (value, sender) = kivact::task::remote(initial);`: `value` は `Signal<T>`、`sender` は `Send` で `set` を持つ。UI は turn ごとに最新の値を一度受け取る。部品が削除されたあとの `set` は捨てられる |
| UI をスレッドから起こすための容量 1 の channel | `let (changed, wake) = kivact::task::remote_trigger();`: `wake.notify()` は決してブロックしない |
| 時計のために部品ごとに置いたタイマー | `kivact::task::clock(every)`: 周期ごとに一つを共有する |
| 実行中の App の外での `blocking(..)` | 呼出し箇所で `WrongApp` として報告される |

## App and platform: `kivact::App`, `kivact::platform`

| 変更前 | 変更後 |
|---|---|
| `App::new(..).fonts(fonts)`、`App::new(..).font(bytes)` | `App::new(..).theme(Theme::light().with_fonts([regular, bold]))`。各項目は `Vec<u8>`、静的なバイト列（`include_bytes!(..).as_slice()`）、パス、共有されたバイト列のいずれか |
| `kivact::launch::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}` | `kivact::platform::{GpuPreference, GpuPower, GpuBackend, GpuStart, AdapterMatch}` |
| `prefer_gpu(false)`、`EmbedOptions.prefer_gpu` | `GpuPreference` の `GpuStart::Never` |
| `ARVANE_RENDERER=cpu` / `gpu` | `cpu`、`gpu`、`auto`。それぞれ `GpuStart::Never`、`AtLaunch`、`OnDemand` |
| `app.clone()` | `App` は `Clone` ではない。別の場所から使うには、`run` の前に `App::handle()` を取る |
| `kivact::AppOptions`、`kivact_core::launch::AppOptions` | `App` の setter |
| `platform::on_close_request(f)` は主 window だけを対象にしていた | 呼び出した部品の window を対象にする（主 window、`window(..)`、外部の window） |
| `kivact::test::TestFonts::Only(fonts)` | `RenderOptions::with_theme(theme.with_fonts([font]))` |

## Embedding: `kivact::embed`

| 変更前 | 変更後 |
|---|---|
| `kivact::embed::Visibility` | `kivact::embed::EmbedVisibility` |
| `EmbedNotice::Stopped`、そのあとの `embedded.status()` | `EmbedNotice::Stopped(EmbedStatus)`: `Closed`、`Failed(..)`、`Panicked(..)` のいずれか。一度だけ送られる |
| 埋め込みの無い platform で、埋め込みの code を囲んでいた `#[cfg(..)]` | `kivact::embed` はどの platform でもコンパイルできる。`App::embed` は、埋め込みが使えない場所では `ErrorKind::Unsupported` を返す |

## Styles and themes: `kivact::style`

| 変更前 | 変更後 |
|---|---|
| `ColorToken::Accent`、`Color::token(ColorToken::Accent)` | `color::ACCENT`（ほかに `SURFACE`、`SURFACE_ALT`、`TEXT`、`TEXT_MUTED`、`ACCENT_TEXT`、`BORDER`、`FOCUS`、`DANGER`、`SUCCESS`、`WARNING`、`OVERLAY`、`SHADOW`、`SELECTION`、`HOVER`、`PRESSED`、`RAISED`） |
| `ThemeColor`、`Color::of(..)` | `ThemeVar<Color>` と `Color::var(&VAR)` |
| `Palette { accent: .., ..base.colors().clone() }` と `Theme::with_colors(palette)` | `theme.set(&color::ACCENT_VAR, rgb(..))`。変数ごとに一回の呼出し |
| `Spacing`、`Theme::with_space`、`theme.space().md` | `space::MD`（`XS`、`SM`、`MD`、`LG`、`XL`）。`theme.set(&space::MD_VAR, 12)` |
| `Radii`、`Theme::with_radius`、`theme.radius().md` | `radius::MD`（`SM`、`MD`、`LG`）。`theme.set(&radius::MD_VAR, 6)` |
| `Theme::with_text`、`theme.text().size` | `font::SIZE`（`SIZE_SMALL`、`SIZE_LARGE`）。`theme.set(&font::SIZE_VAR, 15)` |
| `theme.colors().accent` | `theme.get(&color::ACCENT_VAR)` |
| `use_tokens()` | テーマ変数 |
| `.style(move \|\| { let t = use_theme().get(); style().padding(t.space().md) })` | `.style(style().padding(space::MD))`。クロージャは不要で、値はテーマに従う |
| 色や大きさを持つ `ThemeExtension` | 自分の `ThemeVar` の static。`ThemeExtension` は関数の表のために残る |
| `theme.with_control_height(h)`、`theme.control_height()` | `theme.set(&control::HEIGHT_VAR, h)`、`theme.get(&control::HEIGHT_VAR)`。style の中では `control::HEIGHT`（`kivact::style::control`） |
| `theme.with_disabled_opacity(o)`、`theme.disabled_opacity()` | `theme.set(&control::DISABLED_OPACITY_VAR, o)`、`theme.get(&control::DISABLED_OPACITY_VAR)` |
| `theme.with_overlay_margin(m)`、`theme.overlay_margin()` | `theme.set(&control::OVERLAY_MARGIN_VAR, m)`、`theme.get(&control::OVERLAY_MARGIN_VAR)` |
| `ThemeHint` | `ThemeMode` |
| `.weight(700)` | `.font_weight(700)` |
| `.italic(true)` | `.font_style(FontStyle::Italic)` |
| `.radius(8)`、`.radius_top_left(8)` などの `radius_*` | `.border_radius(8)`、`.border_radius_top_left(8)`、`border_radius_*` |
| `.ring(2, color)`、`.ring_offset(2)`、`.ring_none()` | `.outline(2, color)`、`.outline_offset(2)`、`.outline_none()` |
| `.basis(..)`、`.grow(1)`、`.shrink(0)`、`.wrap(..)`、`.direction(..)` | `.flex_basis(..)`、`.flex_grow(1)`、`.flex_shrink(0)`、`.flex_wrap(..)`、`.flex_direction(..)` |
| `Direction::Row` | `FlexDirection::Row` |
| `.pressed(style())` | `.active(style())` |
| `text_decoration_style(move \|\| ..)` | `text_decoration_style(TextDecorationStyle::Wavy)` は普通の値を受け取る。style 全体を反応させるには `.style(move \|\| ..)` を使う |
| `color.components()`、`a.mix(b, t)` | `color.components(&theme)`、`a.mix(b, t, &theme)`。新しく `relative_luminance()` と `contrast_ratio(other)` |
| `Dimension`、`TextLen` | `Length`、`TextLength` |
| `GridFlow` | `GridAutoFlow` |
| `Insets` | `Edges<f64>` |
| `SpanStyle`、`TextExtras`、`DecorationStyleValue` | `TextStyle`。段落、span、エディタの範囲で使う、テキストの性質の一つの集合 |
| 描画の性質だけを受け付けていた状態の style | `.hover(..)`、`.active(..)`、`.focus(..)`、`.focus_visible(..)`、`.disabled(..)`、`.selected(..)`、`.group_hover(..)`、`.group_focus(..)` はすべての性質を受け付ける |
| `kivact::draw` の `Shadow`、`Filter`、`Transform`、`Paint`、グラデーション、`Shape`、`CornerRadius`、`BlendMode`、`Mask*`、`BorderImage*`、`PaintShader`、`FilterShader`、`FilterGraph` | `kivact::style` から使う |

## Drawing: `kivact::draw`

| 変更前 | 変更後 |
|---|---|
| `Layer::new().blend(BlendMode::Multiply)` | `Layer::new().mix(Mix::Blend(BlendMode::Multiply))` |
| `Layer::new().composite(Composite::SourceIn)` | `Layer::new().mix(Mix::Composite(Composite::SourceIn))` |
| `kivact-core` の `Offscreen` | `kivact::draw::Offscreen`（今は facade の後ろにある独立した crate）: `render(width, height)`、`render_svg(width, height)`、`.scale(f64)` |
| フォントの機能、variation、ブレンド、テキストの装飾で、SVG の書き出しが失敗していた | それらは書き出される。SVG 1.1 や CSS に相当するものが無い機能だけが失敗する |

## Kits and plots

| 変更前 | 変更後 |
|---|---|
| `kivact_ui_basic::{plot, PlotModel, PlotSeries, PlotAxis, PlotRange, AxisScale, TickFormat, LineDash}` | `kivact_plot::{..}`（crate `kivact-plot`）。どの kit とも使える |
| kit の `ColorName` と `Tokens` の field（`theme.ext::<Tokens>().colors.primary`） | kit のテーマ変数。例: `kivact_ui_basic::PRIMARY` |
| kit の `*Appearance`、`*Look`、`*Hue`、`*Tone`、kit の接頭辞の付いた variant の型 | kit の `<Component>Variant` の型。kit のガイドを見てください |

## 関連項目

- [再利用できるコンポーネントを書く](custom-components.md)：部品を書く唯一の方法です。
- [React や SolidJS の知識を Kivact に対応づける](coming-from-react-or-solid.md)：新しい API を 1 ページにまとめています。
- [テーマとコンポーネントの見た目を変える](theming.md)：テーマ変数とコンポーネントのテーマです。
