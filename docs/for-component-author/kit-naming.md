# kit の部品の命名規則

公式 kit と、自社のデザインシステムを kit として作るときに、部品と builder の名前を揃えるための規則です。名前は、見た目の流儀ではなく、部品の意味（role と状態の持ち方）で決めます。同じ意味の部品は、どの kit でも同じ名前で呼べるようにします。

## 共通の部品名

kit がその意味の部品を持つときは、次の名前を使います。別の意味の部品に、この名前を流用しません。

| 名前 | 意味 |
|---|---|
| `button` | 押すと操作を実行する。`Role::Button` |
| `toggle` | 押した状態（aria-pressed）を持つ toggle button だけ。標準の `kivact::ui::toggle` を recipe（`UiRecipes::toggle`）で描き、kit に作らない。checkbox や switch の行には使わない |
| `progress` | 進み具合の bar。標準の `kivact::ui::progress(Option<f64>)`（0 から 1、`None` は不定）を `ProgressRecipes` で描く。tone や縞は `.variant(..)`、細さは `.size(..)` |
| `disclosure` | 開閉する見出しと内容。標準の `kivact::ui::disclosure` を `DisclosureRecipes` で描く。accordion は disclosure の列と開いている key の `Binding<Vec<K>>` で作り、kit に `accordion` を作らない |
| `split` | 割合で分ける 2 つ以上の面。標準の `kivact::ui::split(panes, sizes)` で、境界は `SplitterRecipes::handle`、つまみは `SplitRecipes::grip`。kit に `resizable_group` や `split_view` を作らない |
| `icon_button` | 作らない。icon を付けて label を隠した標準の button（`button(label).icon(..).hide_label(true)`）で、recipe は `ButtonState::icon_only` で正方形にする |
| `checkbox` | checked の状態を持つ箱の行。`Role::CheckBox` |
| `switch` | オンとオフを切り替える行。`Role::Switch` |
| `radio_group` | 排他的な選択を radio の組で表す。横に並べた segmented control もこの部品で、kit の型（`Segmented`）を `.variant(..)` で渡し、項目の内容は `.item(..)` で受ける。kit に `segmented` という別の部品を作らない。tabs ではない |
| `toggle_group` | toggle の組。受ける Binding の型で、押せる数が決まる（下記） |
| `slider` | 連続値または段階値を drag とキーで選ぶ |
| `tabs` | 表示を切り替える TabList と TabPanel の関係を持つ |
| `table` | 静的な表。並べ替え・選択・頁分けを持つ表は kit に作らず、標準の `kivact::ui::data_table` を kit の theme（`DataTableRecipes`）と slot で描く |
| `list` | 選択できる一覧。kit に作らず、標準の `kivact::ui::list` を `ListRecipes` で描く |
| `command_palette` | 検索して実行する面。kit に作らず、標準の `kivact::ui::command_palette(open, commands, run)` を `CommandPaletteRecipes` で描く。`Command::icon(..)` で行の icon を渡す |
| `tag_input`・`number_input` | 標準の `kivact::ui::tag_input(label, tags, on_add, on_remove)` と `number_input(label, value, range)` を `TagInputRecipes`・`NumberInputRecipes` で描く |
| `menu` | 操作の一覧。標準の `kivact::ui::menu` を、button から開くものは `.attach(trigger)`、右クリックで開くものは `.attach_context(target)` で付ける。kit に `context_menu` や `dropdown_menu` を作らず、見た目の違いは recipe（`context_surface`）か `.variant(..)` で表す |
| `dialog` | 応答を求める面。標準の `kivact::ui::dialog` の kind（Modal・AlertModal・Drawer・Popover）で modal・確認・端の sheet・popover を表し、kit に `sheet`・`alert_dialog`・`popover` を作らない。見た目は recipe と `.variant(..)`、確認の button は footer の slot |

kit の独自の部品には、分野の言葉をそのまま名前にします（`calendar`、`timeline`、`dock` など）。共通の名前に接頭辞や接尾辞を付けて別の意味に使うことは避けます。

`toggle_group` は、受ける値の型で単一か複数かを決めます。`Binding<Vec<K>>`（または `Signal<Vec<K>>`）は何個でも押せる組、`Binding<Option<K>>`（または `Signal<Option<K>>`）は高々一つを押せる組です。`toggle_group_single` や `toggle_group_multiple` のような組を作りません。選択肢から選ぶほかの部品（Basic の `combobox` など）も同じく、受ける値の型で一つか複数かを決めます。同じ意味の入口を二つ作らず、経路を一つにします。

## 共通の構成関数の署名

同名の部品は次の入力を使います。存在しない kit に部品を追加する規則ではありません。返す builder の型と見た目は kit ごとに異なります。

| 名前 | kit | 入力の署名 |
|---|---|---|
| `dropzone` | game, pop | `(label: impl IntoText)` |
| `skeleton` | basic, game, pop | `()` |
| `pagination` | basic, game | `(page: impl Into<Binding<usize>>, count: impl IntoValue<usize>)` |
| `stepper` | game, pop | `(labels: impl IntoValue<Vec<String>>, current: impl IntoValue<usize>)` |
| `empty_state` | basic, game, pop | `(title: impl IntoText)` |
| `breadcrumb` | basic, game, pop | `(items: impl IntoValue<Vec<String>>)` |
| `alert` | basic, game, pop | `(title: impl IntoText)` |
| `field` | 標準のみ (`kivact::ui`) | `(label: impl IntoText, control: impl NodeBuilder + IntoView)` |

この表は `cargo test -p kivact-workspace-policy-tests --test kit_signatures` で検査します。crate root の明示的な公開 export から構成関数の定義をたどり、七つの共通名について、引数名と空白を除いた入力の型・型引数の制約・where 句を、表と他の kit の両方に照合します。builder の戻り値は比較しません。`field` の行は標準部品に公開口があり、kit に wrapper がないことを確認する所属の規則です。

pagination の Binding は 0 始まり、表示は 1 始まりです。count が 0 の間は選べるページがありません。範囲外の Binding は表示と操作の間だけ末尾へ収め、アプリの値を勝手に書き換えません。Basic の任意の子を並べる API は `pagination_content` と `breadcrumb_content` です。

skeleton の `.rows(impl IntoValue<usize>)` は既定が 1、寸法は NodeBuilder の style で指定します。empty_state の `.description(impl IntoText)` は既定が空、`.content(impl FnOnce() -> V + 'static)` は mount 時に一度だけ実行します。alert の `.description(impl IntoText)` は既定が空、`.tone(ui::Tone)` は既定が Neutral です。

field は標準部品だけが持ち、`.hint(text)` と `.error(value)` で説明と検証を関連付けます。豊かな説明、caption、横並びは呼出元の `row`・`column` で合成します。複数の control は `node().child((..))` で包みます。Basic の `field_set`・`field_group`・`field_content`・`field_separator` は残ります。

## `*_with_*` を作らない

内容を差し替える別名の関数を作らず、基本の builder に任意の method を足します。kit が新たに持つ分野固有の部品では、必須の入力は三つ以下とし、任意の文言・icon・操作・外観は method で受けます。標準 UI へ委譲するときは標準の署名をそのまま使います。たとえば標準の `tag_input(label, tags, on_add, on_remove)` は四つの必須入力を保ち、この規則のために別の builder へ変えません。固定のデータ model を作る constructor や座標の変換は、この部品の規則の対象外です。

値の有無で形が変わる部品は、一つの関数に `Option` で渡します。標準の `progress(value)` は `Some(値)` なら確定値、`None` なら不定の bar です。

## slot は closure で受ける

部品の中の決まった場所に置く内容（`content`・`footer`・`header`・`title`・`heading`・`actions` など）を受ける method は、view ではなく view を作る closure を受けます。`kivact::ui::dialog(..).content(|| ..)` と同じ書き方です。

- 開くたびに作り直すもの（dialog や popover の中身など）は `impl Fn() -> V + 'static` を受けます。
- 部品が mount されるときに一度だけ作るもの（部品の見出し、表の上の操作など）は `impl FnOnce() -> V + 'static` を受け、部品が mount されるときに呼びます。builder の中で先に呼びません。
- 容器の子（`column((..))`、`panel(content)`、`field(label, control)` の control など）は view を直接受けます。容器は子を並べるだけで、置き場所を決めないからです。
- 行ごとの内容（表の `.cell(|row| ..)`、tabs の `.item(|item, state| ..)`）は、行の値を受ける closure です。

```text
banner("保守", Tone::Warning, retry).content(|| rich_text(spans))
```

## 必須の値はコンストラクタの引数で受ける

部品が動くのに必要な値（`Binding` など）は、`kivact::ui` と同じく、コンストラクタの引数で受けます。

```text
let on = signal(false);
toggle("太字", on)
```

あとから `.bind(..)` で渡す形や、渡されなければ部品の中で隠れた signal を作る fallback は作りません。本当に値を持たない部品（例：初期値だけを決める入力）に限って、`.default_value(..)` で初期値を渡します。

## 通知

通知は core の `kivact::ui::Toast` を `toaster().push(..)` で積み、`kivact::ui::toast_region(toaster)` が kit の theme で描きます。kit は通知を作る関数を持たず、どの kit でも `Toast::new(title).tone(Tone::Success)` で作ります。二行目は `Toast::body(..)`、表示時間は `.duration(..)`、操作は `.action(label, f)` のように core の method で足します。意味は core の `kivact::ui::Tone`（Neutral・Info・Success・Warning・Danger・Accent）で表し、kit に意味の enum を作りません。色相の名前（pop の `Hue`）のように見た目だけの型は `Variant` にします。toast の icon は kit の theme の `IconSet` に `toast-info`・`toast-success`・`toast-warning`・`toast-danger`・`toast-accent` の名前で入れます。

## 画面構成は kit に置かない

kit が持つのは、theme と分野固有の部品です。特定のアプリの画面構成（sidebar と topbar を固定の寸法で並べた枠、アプリの語彙を前提にした見出しなど）は、kit ではなくアプリか catalog に置きます。複数のアプリで使える構成（shadcn の blocks に当たるもの）で、寸法と内容が引数で決まるものは kit に置いてかまいません。

## theme(..) と Tokens

kit のテーマは一つの関数 `theme(mode)` で作ります。引数は `kivact::style::ThemeMode`（`Light`・`Dark`）だけで、kit 独自の mode の型は作りません。暗い配色しかない kit も mode を受け、明るい配色を用意します。

`theme(mode)` は core の `kivact::style::Theme` を返します。`App::theme`、`provide_theme`、signal、テストの設定にそのまま渡せるようにするためです。密度や accent のような kit の選択肢は、kit が `Theme` に足す trait（`ThemeOptions`）の method で受けます（`theme(ThemeMode::Dark).density(Density::Compact)`）。method は既定の値を替えるだけで、mode の配色と他の token はそのまま残します。method はどの kit でも `self` を受けて `Theme` を返し、`#[must_use]` を付けます。選択肢のない kit は `ThemeOptions` を持ちません（空の trait を足しません）。アプリは `use kit::ThemeOptions as _;` で method を使います。

```text
pub fn theme(mode: ThemeMode) -> Theme {
    // mode に合う core のテーマから始め、palette・余白・角丸・文字・focus_ring・
    // disabled_opacity・control_height を kit の値にする（`with_mode`・`with_colors`・
    // `with_space`・`with_radius`・`with_text`・`with_focus_ring`・...）
    Theme::light()
        .with_mode(mode)
        .with_icons(icons::set())            // kit の icon を足した IconSet
        .with(Tokens::new(mode, Density::Comfortable))
        .with(recipes::recipes())
}

pub trait ThemeOptions {
    #[must_use]
    fn density(self, density: Density) -> Theme;   // 寸法の token と control_height だけを替える
}
```

- **Tokens。** core の palette と寸法にない色・大きさ・書体は、kit に一つの `Tokens` 型としてまとめ、`kivact::style::ThemeExtension` を実装します。`fallback(theme)` は `theme.mode` の値を返し、kit の部品が他のテーマの下でも描けるようにします。読むのは `theme.get::<Tokens>()` だけです（style の closure、recipe、`Draw` の中で）。`Tokens::of` や `from_theme`、`palette(theme)` のような読み方を別に作りません。読みは共有され、毎回の複製はありません。

- **Color。** kit は `Tokens` の色の名前を、field のない `pub enum Color` にして公開し（`basic::Color::Border`）、`kivact::style::ThemeColor` を実装します。`resolve(theme)` は `theme.get::<Tokens>()` の色を返します。style と `Draw` の色を受ける method はこの値をそのまま受け、描くときのテーマから読むので、色を一つ読むだけの style の closure は書きません。色から別の色を作るとき（`with_alpha`、`mix_in`）だけ、`theme.get::<Tokens>()` の値を読みます。`ThemeColor` の値は process の間、値ごとに一つ保持されるので、数値を持つ値にはしません。
- **core の token で表せるものは core の token にします。** focus ring の幅と色は `focus_ring` と `ColorToken::Focus`、disabled の見た目は `disabled_opacity` です。focus できる部位は、style に focus の見た目がなければ focus ring を、disabled の間は `disabled_opacity` を自動で受けるので、同じ値を部品ごとに手で書きません。kit の focus の見た目が ring と違う（枠の色も変える）ときだけ、style の `focus_visible(..)` に書きます。
- **recipe。** 標準部品の見た目は `UiRecipes::default()` から method で部位ごとに差し替え、`Theme::with` で載せます。recipe は style だけを返し、項目の内容は標準部品の slot と IconSet に任せます。
- **標準部品の kit 固有の見た目。** `ButtonVariant` は意味の variant（Primary・Secondary・Outline・Ghost・Danger・Link）だけで、button にも `.variant(ButtonVariant::Primary)` で渡します。kit の色調や形は kit の型（`ButtonTone`、`Slant`、radio group の segmented など）で表し、`Variant` を実装して（`impl Variant for Slant {}`）、どの標準部品にも `.variant(ButtonTone::Pink)` で渡し、その部品の recipe で `look.variant::<ButtonTone>()` を読みます（recipe の引数は `&Look<S>`）。他のテーマはその型を知らず、普通の見た目で描きます。同じテーマの中の見た目の変種を、別の部品や部分木のテーマで作りません。
- **状態と動きの見た目。** hover（CSS の `:hover` と同じく子の上も含む）、祖先の hover や focus で変わる子（祖先に `group()`、子に `group_hover(..)`・`group_focus(..)`）、登場と退場（`enter(..)`・`exit(..)` と paint の transition）は、kit の部品でも recipe でも Style で書きます。pointer の Enter/Leave を読む signal や、入場だけのための `Animated` を作りません。
- **icon。** icon を描く経路は `kivact::control::icon("name")` だけです。kit は自分の画像を theme の `IconSet` に名前で入れ、kit の部品も同じ関数で描きます。kit に `icon(name)` のような関数を作りません。他のテーマはその名前に自分の画像を当てられます。

## 反応する一覧と一度だけ使う view

clone できる部品の一覧とサンプルは `IntoValue<Vec<T>>` で受け、変化に追従します。icon の入力は `IntoValue<Icon>` です。所有するデータ model の icon は `Icon` の値を持ちます。

`View` と `FnOnce` は消費するので、一覧を Value に入れて使い直しません。Basic の carousel・avatar_group、Game の activity と property_list など、view を持つ一覧は IntoView・IntoChildren による一度の構成として扱い、反応する値は各 view の内側へ渡します。

kit が所有する翻訳 key には `basic.`・`game.`・`pop.`・`creative.` を付けます。標準部品の key はそのままです。

標準の field は label を押すと control をクリックと同じに起動します。文字入力は focus し、checkbox と switch は切り替わります。disabled・非表示・利用できない control の起動は、共通の NodeRef activation が拒みます。
