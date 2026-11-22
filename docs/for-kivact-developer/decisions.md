# 設計の決定

この文書は Kivact の設計で採用した決定とその理由を記録する ADR である。現行の規則は[アーキテクチャ](architecture.md)、製品の範囲は[製品戦略](strategy.md)、現在の署名は rustdoc とソースを参照する。

各記録は日付、問い、決定、理由を持つ。同じ事項は後の決定が先の決定を置き換える。この文書は設計の決定だけを記録する。検証の結果、試験の実行条件、作業の進め方は PR の説明や issue に書く。

## 作者の決定

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-04 | Q-01 | Machine の純粋遷移だけ単体試験を例外にするか | 例外にせず App 全体の E2E で確かめる | 利用者の契約を本番経路で検証する方針を揃えるため |
| 2026-11-04 | Q-02 | custom の Experimental 表記を外すか | 外す | 独自 control/editor を公開 API で作れる正式な境界にするため |
| 2026-11-04 | Q-05 | interaction handle の接尾辞は何か | Api とする | 一つの語彙に統一するため |
| 2026-11-04 | Q-06 | mirror=off の E2E を App 全体の E2E と認めるか | 認める。戦略の文言は変更しない | 元から入力は注入であり、mirror の有無によらず本番 presenter/driver を通るため |
| 2026-11-04 | Q-07 | ICU4X の全 locale data を含めるか | 数 MB の増加を受け入れ compiled data を含める | locale ごとの独自 table を作らず標準書式を提供するため |
| 2026-11-04 | Q-08 | macOS/mobile/Web backend も実装するか | 実装しない | 実機と環境がなく検証できないため |
| 2026-11-10 | Q-09 | 既定の binary を小さくするため `gpu` を facade の既定 feature から外すか | 外さない。既定は `gpu` と `complex-scripts` で、GPU の device は必要時に起動する | 既定から外すと、CPU の raster が重い画面で GPU への自動の切替えが無くなり、user shader が代わりの見た目になる。製品の既定の性能と機能を binary の大きさのために変えない |

### APG と platform の慣習

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-10-09（2026-11-04再確認） | APG-01 | 右クリック menu をいつ開くか | Capabilities.context_menu_on に従い Windows/Linux は release、macOS は press。keyboard 起動は trigger の位置 | platform の慣習を優先するため。macOS backend が実装済みという意味ではない |
| 2026-10-09（2026-11-04再確認） | APG-02 | 閉じた Select の矢印をどう扱うか | 値を変更し、Alt+Down で開く | 作者が採用した desktop 操作を維持するため |
| 2026-10-09（2026-11-04再確認） | APG-03 | Tooltip の時間と並行表示は何か | 約500ms、skip-delay あり、一度に一つ | 遅延を共有する既存の操作 model を保つため |
| 2026-11-04 | APG-04 | behavior の外部基準を何にするか | 公式 WAI-ARIA APG の Keyboard Interaction と Roles, States, Properties。ただし上記作者決定を優先。時間など未規定の細部は InteractionPolicy | 独自の慣習を無自覚に増やさず、支援技術を含む一貫した操作にするため |

対象 pattern は Accordion、Alert/Alert Dialog、Breadcrumb、Button、Carousel、Checkbox、Combobox、Modal Dialog、Disclosure、Grid/Table、Listbox、Menu/Menubar/Menu Button、Radio Group、Slider/Multi-Thumb Slider、Spinbutton、Switch、Tabs、Toolbar、Tooltip、Tree View、Window Splitter。公式一次資料は [APG patterns](https://www.w3.org/WAI/ARIA/apg/patterns/) を参照する。pattern ごとの意図した差分は、その behavior の実装と E2E で確かめる。

### 非目標

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-04 | SCOPE-01 | OS の SPI があれば各 backend も作るか | Windows/Linux と virtual の境界を整備し、macOS/Android/iOS/Web は追加しない | 実機・検証環境・保守者のない対応を表明しないため |
| 2026-11-04 | SCOPE-02 | Path boolean 演算は Canvas/Path2D 対応の不足か | 不足ではなく追加しない。addPath 相当は既存 Path.append | Canvas/Path2D 自体に boolean 演算はなく、必要とする要求もないため |
| 2026-11-04 | SCOPE-03 | touch wheel のため rotateX を足すか | 追加しない。既存 2D affine による円筒近似を使う | 差は台形の遠近表現であり、操作・支援技術・behavior の欠落ではないため |
| 2026-11-07 | SCOPE-04 | 3D transform（rotateX 等）を足すか（SCOPE-03 を置き換える） | CSS Transforms 2 の 3D（perspective、preserve-3d、backface）を CPU と GPU で一致させて実装する | ゲーム風の UI と Web と同等の表現を目標にしたため |
| 2026-11-07 | SCOPE-05 | user shader をどう提供するか | WGSL をそのまま書く fragment shader（paint と filter）。GPU だけで実行し、CPU は宣言された代わりの見た目を描く | 表現の自由度と、Kivact が抱えるもの（CPU の解釈器）の少なさを優先した。描画以外（hit、a11y）には影響しない |
| 2026-11-07 | SCOPE-06 | 色空間をどこまで扱うか | CSS Color 4 の空間で指定・補間し、gamut mapping して sRGB で出力する | UI library として必要十分であり、ブラウザの UI の扱いと同じ |
| 2026-11-07 | SCOPE-07 | `matrix3d` の遠近の行を扱うか | 扱わない（最下行が (0, 0, 0, 1) のものだけ）。遠近は perspective() と perspective property で表す | 正準形で補間でき、遠近の利用はこの二つで足りるため |

## 改修の決定（2026-11-18）

API の抜本的な改修で行った決定。設計者が決め、作者が PR の review で覆せる（DEC-01）。ここに書いた決定と矛盾する先の行は削除した。

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-18 | DEC-01 | 未決の問いを作者に聞いてから進めるか | 聞かずに設計で決める。作者は PR の review で覆せる | 実装者は設計の判断ができないため、全部を前もって決める必要がある |
| 2026-11-18 | DEC-02 | theme 変数の型名 | `ThemeVar<T>` | 状態の `Var<T>`（useRef）と衝突させない |
| 2026-11-18 | DEC-03 | ui・kit の部品も `#[component]` にするか（CONTRACT-01・CONTRACT-08 を置き換える） | する。手書きの builder struct と `impl Element`/`impl IntoView` は core の基本要素だけに残す。`-> impl Element` の部品は呼出し側の modifier を `Node::absorb` で本体の root に移す | 一つの書き方。利用者の部品 library も同じ形で書け、`.style(..)` を付けられる |
| 2026-11-18 | DEC-04 | `Signal` と `Binding` | `Signal` に統合する（内部は Source と Lens の enum）。`binding(get, set)` と `bind!` も `Signal` を返す | 書ける handle を一つにする |
| 2026-11-18 | DEC-05 | 入力部品の見えない名前 | 位置引数をやめ `.label()` と `field()`。名前の欠落は mount の位置付きの診断（debug build と `kivact::test` では error） | JS の `<label>` と `<input>` の感覚。a11y の保証は compile 時から実行時と試験に移る |
| 2026-11-18 | DEC-06 | 部品の token（ADR-37 を置き換える） | theme 変数に統一し、`UiTokens` と kit の `Tokens` の値の field を消す。形（recipe）は `kivact::ui::theme` の `<Name>Theme` | 値の仕組みと形の仕組みを一つずつにする |
| 2026-11-18 | DEC-07 | style の名前 | CSS の名前（`font_weight`、`font_style`、`border_radius`、`flex_*`、`outline`、`active`、`FlexDirection`） | CSS の利用者が推測できる |
| 2026-11-18 | DEC-08 | GPU の device | Driver（一つの App の event loop）に一つ。window ごとには frame の状態だけ。同じ process の二つの埋め込み editor はそれぞれの Driver の device を持つ | 起動とメモリが window の数に比例しない。process 全体の仲介は持たない |
| 2026-11-18 | DEC-09 | SVG の asset | resvg・tiny-skia による今の描画を保つ。usvg に arvane-text の font を渡し、fontdb の system font の二度目の走査だけを消す（`RenderScene` への変換はしない） | mask・pattern・filter などの既存の SVG の能力を、性能の改修のついでに黙って落とさない |
| 2026-11-18 | DEC-10 | Python の DevTools client（ADR-51 の Python の部分を置き換える） | 消す。xtask の codegen も消し、JSON Schema は protocol から作る | 利用者がいない。MCP が同じ役目を持つ |
| 2026-11-18 | DEC-11 | SVG の書き出しで同等のものが無い機能（ADR-64 を置き換える） | SVG 1.1 と CSS に同等があれば書き（font-feature-settings、font-variation-settings、mix-blend-mode、文字の装飾、塗りと線）、無いものだけ Unsupported の error にする。書き出しは `kivact-offscreen` の `Offscreen::render_svg` と `kivact-svg` | plot の書き出し（`tnum` など）が失敗しない |
| 2026-11-18 | DEC-12 | 性能の必達 | 全 KPI で後退しない（≤ 1.00、時間の行は 1.05、大きさの行は 1.02 までの雑音を許す）。改善の目標は記録する | 未達で設計が止まらず、後退は確実に防ぐ |
| 2026-11-18 | DEC-13 | 遅延初期化 | `kivact_types::cell::Deferred<T>`（UI thread）と `SharedDeferred<T>`（static）だけ。std の lazy 型の禁止は続ける。ADR-17・ADR-21 の内部の OnceCell はこの二つに読み替える | 性能の規則が求める「使う時に作る」と、S-7（未完成の object を作らない）の区別を一つの道具で表す |
| 2026-11-18 | DEC-17 | 状態の印の機械的な変換 | 既存の印から機械的に作った印は `// xtask-allow(state): legacy: ...` と書いて区別する。CP1 で全件を台帳（`rework/state-ledger.md`）にし、設計者が一件ずつ閉じ方を決めて実装する。`legacy:` が 0 件で完了 | 印の件数 0 は状態機械の解消の証拠にならない |
| 2026-11-18 | DEC-18 | 依存の fork | しない。依存は二つの群（いま上げる群と、wgpu・vello・parley・skrifa・usvg などの連動する群）で更新する | 主な依存は活発に版が出ており、fork の追従の費用が性能の利益を上回る |
| 2026-11-18 | DEC-15 | `update` の戻り値 | 返さない（`Signal`・`Var` とも）。破棄された signal への `set` と `update` は位置付きの `WriteAfterDispose` を一度記録して何もしない | 監査の提案（`Option<R>`）では最も多い `on_press(move \|\| x.update(..))` が handler の結果の型に合わなくなる |
| 2026-11-18 | DEC-16 | payload のある handler の型 | `impl Fn(E) -> R`。`IntoHandler` は payload の無い handler だけ | closure の引数の型の推論は `Fn` の bound からしか働かない。`Callback<E>` は `Fn` を実装できない |
| 2026-11-18 | DEC-14 | 改修の途中の kit | style の改修から kit の移行までの間、5 つの kit・catalog・kit の試験を compile から外し、kit は最終の API に一度で移す | 途中の API への追従を二度するより安い |
| 2026-11-18 | RW-01 | handler の受け方（CONTRACT-10 を置き換える。DEC-16） | 二つだけ。payload の無い handler は sealed な `IntoHandler<(), M>`（closure と `Callback<()>`）、payload のある handler は `impl Fn(E) -> R + 'static`（共有の `Callback<E>` は `move \|e\| cb.call(e)`）。どちらも `Handler<E>`（`Rc<dyn Fn>`、arena の object を作らない）にする。`From<F> for Callback<()>`、`Callback::bind`・`to_fn` を消す | closure の引数の型の推論は `Fn` の bound からしか働かず、`Callback<E>` は `Fn` を実装できない。payload の無い press では `Callback<()>` を包み直す必要と press ごとの arena の object が消える |
| 2026-11-18 | RW-02 | context の読み出しの型 | `use_context::<T>() -> T`、`try_use_context -> Option<T>`。context の置き場は runtime と core の二つを一つにする | 定数の `Value` を二度 `.get()` する必要が無い。reactive にしたいときは中に `Signal` を入れる（Solid と同じ） |
| 2026-11-18 | RW-03 | prelude の範囲 | 設計で決めた約 120 の名前の明示の一覧。変換 trait（IntoValue、IntoText、IntoHandler）、kit 寄りの型、`scope` を入れない。`provide_theme`・`use_theme`・`color`・`space`・`radius`・`font` を入れる（CONTRACT-14 を置き換える） | 初めての利用者が見る名前を減らし、theme 変数を import なしで書けるようにする |
| 2026-11-18 | RW-04 | 非同期の action の形（CONTRACT-05 を置き換える） | `action(f)` は新しい実行が古いものを取り消す。順に実行するものは `queued_action(f)`。owner の破棄でその owner の action（queued を含む）を取り消し、画面をまたぐ保存は App の root の owner で作った action で行う。期限つきの drain は App の終了だけ（12 章 ST-08）。`ActionMode`・`ActionOutcome`・`Action::close` を消し、`on_complete` は `Result<T, UiError>` を受ける。`Retry` は `Callback<()>`。非同期の表示は `async_view` だけを教える | 既定の意味を一つにし、完了の結果を std の型で表す |
| 2026-11-18 | RW-05 | 他の thread から reactive な状態に値を入れる口 | `task::remote(initial) -> (Signal<T>, Remote<T>)` と `task::remote_trigger() -> (Trigger, RemoteTrigger)`。`Remote` は `set(T)` だけ（利用者の closure を他の thread で動かさない）。最新の値だけを `Open { latest }`／`Closed` の mailbox に保ち、runtime の driver が turn の始めに一度だけ届け、event loop を起こし、block しない。owner の破棄で閉じ、その後の `set` は値を捨てる（12 章 API-9〜11）。`Notifier` は `Trigger` に改名する | channel と `spawn` の定型を利用者に書かせない |
| 2026-11-18 | RW-06 | font の持ち主（ADR-25 の font の部分を置き換える） | theme が持つ（`Theme::with_fonts(impl IntoIterator<Item = impl Into<Asset>>)`：static bytes・`Vec<u8>`・path・共有 bytes を同じ入口で受け、`.leak()` は要らない。12 章 API-13）。`App::fonts` と `TestFonts::Only` を消し、`App::theme` と `RenderOptions::with_theme` が登録する | kit の theme と font を一緒に配れ、試験と本番が同じ経路を使う |
| 2026-11-18 | RW-07 | GPU の起動の希望の置き場（CONTRACT-11 を置き換える） | 5 つの型を `kivact::platform` に置き、`kivact::launch` を消す。`GpuStart::Never` を足し、`prefer_gpu` を消す。`ARVANE_RENDERER=cpu\|gpu\|auto` は `Never\|AtLaunch\|OnDemand` | 1 つの型のための module をなくし、CPU だけの起動を同じ型で表す |
| 2026-11-18 | RW-08 | close request の handler が効く window（CONTRACT-15 を置き換える） | `platform::on_close_request(f)` は呼んだ owner の window（main window、`window()`、foreign window）に効く。main 以外の window では handler が `Handled::Yes` を返さなければ `on_dismiss(DismissReason::WindowClose)` が続く。`AppHandle::close` と埋め込み App は handler を呼ばない | 別 window の未保存の確認も同じ口で書け、`on_dismiss` の規則と揃う |
| 2026-11-18 | RW-09 | 埋め込みの停止と名前（ADR-72 を置き換える） | `EmbedNotice::{ScaleChanged(f64), Stopped(EmbedStatus)}`。停止は `Closed`・`Failed`・`Panicked` を payload で一度だけ知らせる。`embed::Visibility` を `EmbedVisibility` に改名し、`kivact::embed` は全 OS で compile できる（未対応の環境の `App::embed` は `Unsupported`） | wrapper が通知の後に status を読み直す手間と、OS ごとの cfg を利用者から除く |
| 2026-11-18 | RW-10 | 埋め込みの表示・非表示の表し方（ADR-67 を置き換える） | `ShellEvent::Visibility(bool)` で表す。`Suspended`・`Resumed`・`SurfaceDestroyed`・`SurfaceCreated` などモバイルの event は残す | 表示の切替えを surface の喪失と別の事実にする。モバイルの設計境界は先送りしない（戦略 P3） |
| 2026-11-18 | RW-11 | theme の値（ADR-27 を置き換える） | theme の値は theme 変数 `ThemeVar<T>`（`color::ACCENT` など）。`Color::var`・`Length::var` で const の値に置ける。解決は `compile_style` で行い、theme の切替えは style の effect の再実行になる。`ThemeColor`、`ColorToken`、process 全体の `THEMED` 表、`Palette`・`Spacing`・`Radii` を消す。`Theme` は `Arc` で共有し、`PartialEq` は pointer を先に比べる | CSS の `var(--x)` と同じ一つの仕組みで、closure なしに theme に追従する。process の可変の表を持たない |
| 2026-11-18 | RW-12 | 状態の style | `.hover(..)`、`.active(..)`、`.focus(..)`、`.focus_visible(..)`、`.disabled(..)`、`.selected(..)`、`.group_hover(..)`、`.group_focus(..)`、`.enter(..)`、`.exit(..)` はすべての property を受け、layout の property が変われば layout をやり直す | CSS の擬似 class と同じ。paint だけという例外を利用者に覚えさせない |
| 2026-11-18 | RW-13 | layer の合成の指定 | `Layer::mix(Mix)`（`Mix::{Normal, Blend(..), Composite(..)}`）が `blend`・`composite` を置き換える。CSS の値の型は `kivact::style` だけに置く | 後から呼んだ方が黙って勝つ二つの setter をなくす。同じ型を二つの module から出さない |
| 2026-11-18 | RW-14 | window なしの描画の置き場 | `kivact-offscreen` を新設し、facade が `kivact::draw::Offscreen` として再公開する。core は kivact-svg に依存しない | 入力（theme・font・描画の closure）と出力（Pixels・SVG）が小さく、view と runtime を読まない明確な境界 |
| 2026-11-18 | RW-15 | 数値の plot の置き場（ADR-65 を置き換える） | 新しい crate `kivact-plot`。色は `kivact_plot::color` の theme 変数で受け、どの kit とも組み合わせられる。範囲の操作は ADR-65 のまま（PlotRange、Drag、EditSession、`on_edit(EditPhase)`）。basic の chart は basic に残し、目盛りと scale の計算を共有する | 分析ツールという変更理由は kit の見た目と別。kit なしでも使える |
| 2026-11-18 | RW-16 | 描画の protocol の置き場 | `arvane-protocol` を新設し、`kivact-types/src/render` を移す。`arvane` は window driver と presenter だけにして再公開をやめる | renderer との契約という一つの変更理由を持ち、作るのは engine、使うのは arvane-* だけ |
| 2026-11-18 | RW-17 | DevTools の client の crate | `kivact-devtools-client` を `kivact-devtools-protocol` の feature `client` に統合する | client と protocol は同じ理由で同時に変わり、client だけを使う利用者がいない |
| 2026-11-18 | RW-18 | 開閉の語彙（CONTRACT-12 を置き換える） | interaction の surface の Api は `use_x(.., open: Signal<bool>)` で open を必須の位置引数に取り、`show`・`hide`・`toggle`・`is_open`、`trigger()`・`content()` の Part、`surface(placement, content)` を持つ。`use_menu_with_open`、`on_open_change`、`DialogMode::Popover`、`while_open` を消す | 開閉の書き方を一つにする。`surface` が anchor・`show`・overlay・content の Part の配線をまとめる |
| 2026-11-18 | RW-19 | 部品の状態と recipe の形 | part の状態を読む唯一の方法は `Part::state()`。recipe の上書きは `Fn(&Theme, &Look<S>, Style) -> Style`（3 つ目は前の recipe の結果）。ui は入力で動く Machine を持たず、interaction は view を返さない（`surface()` を除く） | 状態の getter・slot・map の写しをなくし、上書きを前の結果の上に重ねられるようにする |
| 2026-11-18 | RW-20 | 状態の検査の単位（ADR-52・STATE-AUDIT・STATE-CLAIMS を置き換える） | 型の単位の検査 `state`：状態の field を二つ以上持ち、`&mut self` か内部可変の書込みを持つ型は、型の直前の印 `// xtask-allow(state): <閉じ方>` が無ければ error。名前の接尾辞や derive による除外は置かない。constructor・`Default`・値渡しの builder だけで書く field は状態ではない。field ごとの印、`--report-audit`、`state-audit.md` を消す（機械的に変換した印は DEC-17 の `legacy:`） | field ごとの印は受動的な record に 1000 行以上の定型文を書かせ、意味の誤りは見つけられなかった。名前を変えるだけで状態が見えなくなる除外も持たない |
| 2026-11-18 | RW-21 | 公開 API の E2E と文書の compile | E2E を `kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native` と、新しい package `kivact-kit-tests` の `e2e_kits` の 4 つの binary に分ける。文書の doctest は英語だけを compile し、日本語の Rust の code block は `docs-pairs` で英語と同一であることを確かめる。生成した図（`architecture/`）は git から外し、`cargo xtask diagrams` で作る | 変更した crate に合う binary だけを link でき、日本語の同じ例の二度目の compile を省く |
| 2026-11-18 | RW-22 | kit の骨格 | 5 つの kit を一つの骨格に揃える：`theme(mode)`、option があるときだけ `theme_with`、kit の theme 変数、`recipes_<group>.rs`、`variants.rs` の `<Component>Variant`、分野の部品は `src/<domain>/`。kit 名の接頭辞を型名に付けない。kit の README の移行表と recipe の一覧を消す | kit の間の差を見た目と分野の部品だけにする。公開していない kit に旧版からの移行表は要らない |
| 2026-11-18 | RW-23 | 診断の記録 | turn の trace・原因の文字列・node の provenance は DevTools・`kivact::test` の trace・debug の trace sink が有効なときだけ記録し、release で常に持つのは counter だけにする。DevTools は feature で外さず、実行時の `Option` の hook で費用を消す | 常時の記録がメモリと起動の大半を占めていた。機能の有無で API を分けない |
| 2026-11-18 | RW-24 | engine の配送の受理 | pointer の source の写しを消し、配送の受理（admission）は engine が持つ | 入力の route と modal の受理の正本を一つにする |
| 2026-11-18 | RV-01 | 部品の props の特別な形 | 型引数そのものの props（`control: C`）は `Into` で包まずに受け、関数の props は `Rc<dyn Fn(..) -> T>`（closure を受ける）。部品は型引数を持て、lifetime・const の引数は持たない | `field("Name", text_input(v))` の型を推論でき、formatter などを closure で渡せる |
| 2026-11-18 | RV-02 | 呼出し側の modifier を部品が読む方法 | `#[component]` の引数 `#[caller] caller: Caller` に macro が吸収した modifier の snapshot を渡す。最終の variant・disabled・押下の有無は `Node::authored()` 一つで求める。thread-local の stack（`custom::caller()`）は使わない | 部品ごとの読み方を一つにし、隠れた global の状態を持たない |
| 2026-11-18 | RV-03 | 共有の handler と callback の props | 共有の handler は `callback(move \|_\| ..)`（payload の無いものも同じ形）。`Callback` の props は body の実行の前に一度だけ owner に登録し、呼出しの `Result<Handled, UiError>` を保つ | ガイドの書き方を一つにし、inline の closure・共有の `Callback`・props の経由で同じ結果にする |
| 2026-11-18 | RV-04 | `Remote` の操作 | `set(T)` だけ（`update` は持たない）。mailbox は `Open { latest }`／`Closed` の二状態で owner の cleanup で閉じ、配送は runtime の driver が turn の始めに一度だけ行う | 利用者の closure を他の thread や lock の中で動かさない |
| 2026-11-18 | RV-05 | `tooltip` と `provide_direction` の形 | `tooltip(label, trigger) -> impl Element`（`Tooltip::attach` を消す）。`provide_direction` は定数のまま | React の `<Tooltip content><Button/></Tooltip>` と同じ形。読み方向は画面の途中で変わらない |
| 2026-11-18 | RV-06 | font の入口（RW-06 を補う） | `Theme::with_fonts(impl IntoIterator<Item = impl Into<Asset>>)`。static bytes・`Vec<u8>`・path・共有 bytes を受ける | `.leak()` を利用者に書かせない |
| 2026-11-18 | RV-07 | 非同期の寿命 | owner から切り離された action は publish しない。owner の破棄でその owner の action（queued を含む）を取り消し、画面をまたぐ保存は App の root の owner の action で行う。deferred の実行も turn の仕事量の予算に数え、超えたら `Cycle` を一度報告して次の turn に回す | 期限つきの drain は App の終了だけに使い、画面の破棄の後に仕事を残さない |
| 2026-11-18 | RV-08 | 寿命の掃除 | selector の node は最後の購読者が外れたときに消す。motion の登録は token を返し、token の `Drop` で全 map から消す。`devtools` feature の build は node の型と作成の位置を常に持つ | owner の cleanup や診断の呼出しを掃除の契機にしない |
| 2026-11-18 | RV-09 | `Runtime.app` の一度だけの設定 | App の構築の中（利用者のコードが動く前）に限る明記した例外。設定前の読出しは debug の assert で落とす。型での二段階の分離はしない | 公開 handle が thread-local の runtime を読む構造を変える範囲が大きく、得る安全が小さい |
| 2026-11-18 | RV-10 | theme の編集と変数 | `Theme` の書込みはすべて一つの編集の入口を通り、拡張の fallback の cache を空にする。すべての型の theme 変数は別の変数を値（別名）に持て、部品の変数の既定は中核の変数への別名にする。`Length` の `+`・`-` は theme 変数・`em`・`pct` を含む式として保つ。`cached_theme` は作らず、kit は theme を `thread_local!` の static で保つ | 共通の値の変更がすべての部品に届き、式を黙って `Auto` にしない。関数の address は同一性を保証しない |
| 2026-11-18 | RV-11 | system font の走査 | process の開始時に別 thread で始め、window・GPU の準備と重ねる。最初の文字の計測は残りを待つ | 走査を critical path から外しきれないので、重ねて待ちを最小にする |
| 2026-11-18 | RV-12 | 試験と検証の範囲 | 最適化の正しさを守る最小の不変条件の試験（cache の無効化、資源の寿命、状態の遷移）は、その変更の試験の file に足してよい。kivact-test の自己試験は `crates/kivact-test/tests/` へ移す。CPU だけの build は package ごとの `--no-default-features --lib` の check と、`cargo tree` に wgpu が無いことで確かめる。docs-pairs は英日のファイルの集合と、Rust の block の token の列（文字列の中身を除く）を比べる | 過去の不具合の再現の試験は足さないまま、性能の改修が壊しやすい不変条件を守る |
| 2026-11-18 | RV-13 | stepper の編集 | `stepper(items, current)` は `on_edit(EditPhase)` を持つ | 値を書き換える部品は `on_edit` を出す（戦略 5 章の編集の取消し） |

## 実装の決定

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | ADR-01 | 幾何の精度と、境界をまたぐ補足 API をどう定めるか | 幾何と時間の値を f64 に統一する。calendar の選択は Into<CalendarSelection>、theme は IntoValue<Theme> で受け、disclosure の animated を維持し、軸は Orientation、embed の終了は CloseSignal で表す | f32 への暗黙の縮小で値が変わる境界をなくすため。API は実際の情報と既存の振る舞いを保つ必要がある |
| 2026-11-05 | ADR-02 | Backend を新しい巨大な抽象にするか | native の event loop の抽象を Backend とし native が実装する。virtual は SteppedBackend。embed は free function。各 consuming backend が session shutdown を所有し、Send な MakeHandler と UI thread の ChildHooks を分ける | 既存の実行 model と借用 handler を保ち、非 Send backend 全体を Send factory に捕捉しないため |
| 2026-11-05 | ADR-03 | virtual の入力と観測をどう分けるか | Script が書込み、Probe が owned read を担う。Step は Input/Wake/Redraw、StepOutcome は idle 情報/exited/failed。frame 後の公開、GPU 非同期初期化、presenter は本番と同じ。mirror は native の UI thread に adapter を接続する | 二つ目の試験専用実行 model と raw 観測 facade を増やさず、実際の App 契約を検証するため |
| 2026-11-05 | ADR-04 | 入力正規化の持ち主は誰か | native port が Winit/VK/Keysym を写し、shell/pipeline が key、IME、DnD を正規化する。repeat が未指定なら導出。IME snapshot は UTF-8 byte、native が UTF-16 を変換し engine が excerpt origin を適用。virtual の raw 入力も同じ道 | OS の mapping と共通の入力規則を分離し、native と試験の挙動を一致させるため |
| 2026-11-05 | ADR-05 | OS service を window 前に取得するか | 完成した ShellWindow に UI thread 同期の perform を一つ置く。RegisterFont/ReducedMotion は先に engine HostRequest へ分離し、完了 effect 配送前に実 Environment を同期。virtual は応答を上書き可能 | parent dialog と複数 embed resize には window identity が必要であり、late binding を避けるため |
| 2026-11-05 | ADR-06 | embed の thread をまたぐ寿命を一つの enum にするか | 共有 ChildStatus と UI thread の ChildLifecycle/HandlerSlot を分ける。再入/drain guard と connection lifetime は別の事実。shutdown/module 保持は実 handler の破棄に従い、handle の早期 close では行わない | 異なる thread の所有権と再入を偽の単一状態にせず、panic/leak 時の安全を保つため |
| 2026-11-05 | ADR-07 | surface の失敗をどう数えるか | FrameHealth で skip と loss を分ける。GPU の連続 loss 二回で CPU fallback、任意 failure 三回で失敗。shown と受理された skip で数を reset | 正常な skip を故障と数えず、復旧条件を保つため |
| 2026-11-05 | ADR-08 | gpu feature 無効時の API を変えるか | GpuContext/GpuSurface は空 enum、GpuEnv は空値、GPU_BUILT=false として名前を保つ。NoAdapter/External と FailureKind::Unavailable で説明 | 中間層の cfg と利用者側の型分岐を不要にするため |
| 2026-11-05 | ADR-09 | embed close を exit turn に変換するか | CloseSignal を入口、client 後、effect 後と core の各 event で確認し追加の exit turn を作らない。HostPhase と presence/surface は別の事実で drawable は投影。SurfaceLost は診断。Launch は Pending/Starting/Running と実 waker を持つ | host 破棄後に App を動かさず、停止と描画 surface の喪失を混同しないため |
| 2026-11-05 | ADR-10 | lower を独立 crate に保つか | engine の lower module に統合し、Context.finish が消費可能な LowerReport を返す。latch getter を並べない | 計測・model・paint が内部情報を共有しており、独立した小さい契約が成立しないため |
| 2026-11-05 | ADR-11 | layout の pass を型状態に分割するか | 一つの Layout が columns/state を所有し、compute 内で changed を取り、settle は一つの関数。reason は bit flags。pass artifact を保存しない | 所有権を明確にしながら、一回の計算を表すだけの型と中間状態を増やさないため |
| 2026-11-05 | ADR-12 | display と hidden をどう表すか | Display は Flex/Grid/Contents。hidden は独立。公開 style に display を置き prelude の別義名と衝突させない | layout mode と表示可否は異なる事実だから |
| 2026-11-05 | ADR-13 | 出現・退場の layout をどう動かすか | layout hook が parent item の大きさを scale し、子の自然サイズと cache を保つ。EnteringPending/Animating は同じ phase index。ExitLayout::Pop は exit_size を使わない | 第二の layout model とキャッシュ破壊を避けるため |
| 2026-11-05 | ADR-14 | pointer の経路と modal の受入をどこに保存するか | capture と press は独立で四組合せを許す。touch Pending は payload。input 時の Track と dispatch 時の modal admission を分け、identity のない KeyUp は現在 focus に配送。primary hover chain と secondary over を保つ | 一つの phase にまとめると有効な経路情報を失うため |
| 2026-11-05 | ADR-15 | 状態の一覧を形式的に満たすべきか | 異なる事実は別に持ち、AcceptedTree は a11y 初期化/action 検証のため残す。focus や diff の複製には使わない | enum 化の件数より実際の情報と一つの正本を優先するため |
| 2026-11-05 | ADR-16 | runtime module の向きはどうするか | reactive を基底に task/motion、machine→reactive/task、host→reactive/task/motion。passive host_types と Runtime を借りる TurnGuard を実装から分離 | host 実装への逆依存をなくしながら正準 root path を保つため |
| 2026-11-05 | ADR-17 | runtime と host を後付け接続するか | constructor が clock、registry、root owner と task/motion extension を完成させる。内部 OnceCell はこの原子的構築に限る。UI wake と thread-safe host wake を分離し、Driver の late attach は作らない | 半完成 object と遅延初期化の公開 phase をなくすため |
| 2026-11-05 | ADR-18 | turn の順序はどこが持つか | core の AppClient が一つの turn 関数で clock、reports、入力、async、commit、mount、user effect、output を順序付ける。runtime は Stage/guard/部品を提供。async 1000、commit 100 round の上限を保つ。再入/setup、Commit 外 stamp、構築中 extension 参照を misuse とする | focus/error handler の正当な書込みを妨げず、配送順序を保つため |
| 2026-11-05 | ADR-19 | frame loop を future に置き換えるか | FrameTick の callback は同期。clock は一つ、MotionEnv は readonly input。FrameClock と AnimPhase を一つずつ持ち、InternalWatcher が自然完了を安定化中に通知。Sleep の cancel queue は期限確認前に処理 | future の poll に移すと同じ batch 内の入力と frame の順序が変わるため |
| 2026-11-05 | ADR-20 | reactive frame と Watcher をどう統一するか | Handler/Setup/Computation の kind と pure/tracking/owner/writer を分離し、callback/untrack/pure でも kind を保持。Watcher の初回は依存と queue だけを作り、以後 untracked callback。PartialEq 比較を加えない | 初回通知と比較を変えると reactive の意味が変わるため |
| 2026-11-05 | ADR-21 | 非同期の最新値と phase を統合するか | AsyncCell の phase と latest は異なる事実。pending/failed 中も成功値を保持し T に PartialEq を要求しない。待機 queue があれば失敗 callback 後も Pending。PoolShutdown は root cleanup 後、TaskCell の OnceCell は初 poll 前に実 key を設定 | 成功値保持、待機処理、取消と shutdown の契約を失わないため |
| 2026-11-05 | ADR-22 | App 全体の状態をどこへ置くか | constructor で固定する Extensions と root owner に属する遅延 App state を分ける。unsafe な増設 map AppStates を作らない | 寿命と構築時点の異なる情報を一つの可変 registry にしないため |
| 2026-11-05 | ADR-23 | 内部 handler を層ごとに定義するか | runtime に一つの Handler<E> を置き、frame/task/core で共有。unit payload も同じ model | 同義の callback slot を繰り返さないため |
| 2026-11-05 | ADR-24 | Machine の実装をどう小さくするか | Self を input snapshot、State/Event/Effect を関連型にし、純粋 transition が Step を返す。send は再入 FIFO、timer は event variant 単位、feed の初回は無通知、破棄後は Disposed。MachineCx は send の口だけ | 状態更新・timer・effect を一つの owner に保ち、小さい reducer として説明するため |
| 2026-11-05 | ADR-25 | 起動 API と option 束をどう統一するか | private AppOptions に launch 設定を集め App が委譲、RenderOptions はそれを包む。font の登録元 location は private、blocking は None、theme は IntoValue。戻り値が異なる render_with_handle は残す。内部 engine feature は testing のまま | 同義設定と入口を減らし、実際に異なる返却契約は消さないため |
| 2026-11-05 | ADR-26 | Inspect が何を読むか | core 所有の errors/pending/outstanding_blocking/motion_ticks、engine Reader の stats を公開投影にする。backend の E2E が使う private harness は残すが raw 公開観測 API は増やさない | 第二の tree/model を持たず、残る内部契約を試験するため |
| 2026-11-05 | ADR-28 | 地域書式を中間 table に変換するか | ICU4X compiled data を全 locale 分含め、Locale が一度だけ formatter を作る。CalendarDate の from/to days を共通計算にする | kit 固有の en-US table、二重変換、手書き calendar 計算を避けるため |
| 2026-11-05 | ADR-29 | 削除結果をいつ handler に渡すか | Removed を borrow 終了後に消費する。論理削除した node の handler は停止し、退場 paint は保つ。node でない ID は HashMap、on_exit は Handler の Option | borrow と callback の再入を分離し、退場中の二重入力を避けるため |
| 2026-11-05 | ADR-30 | fragment に専用 NodeKind を作るか | Display::Contents の実 node とし、すべての走査で共通 flat_children 判定を使う。Reader の論理親子は leaving を除外、最終退場まで透明 ancestor を保持、detached subtree も実 node | layout の透過性と論理 ownership を同時に保つため |
| 2026-11-05 | ADR-31 | Part が独自 hover を保存するか | engine の FocusTarget(node,region) をそのまま読み、正確に一致する region だけ hovered | node だけの比較や第二の hover state は region の意味を失うため |
| 2026-11-05 | ADR-32 | handler の返り値をどう揃えるか | HandlerResult 一つに ()、Handled、Result<(),UiError>、Result<Handled,UiError> の四実装。Handled の PreventDefault を合成でも保つ | 処理済みと既定動作の禁止を区別し、error も同じ入口で扱うため |
| 2026-11-05 | ADR-33 | Callback の構築は setup に限るか | payload なし Callback は現在の owner があれば handler/effect 下でも構築可能。owner がなければ misuse | 正当な owner-bound callback の利用を不必要に制限しないため |
| 2026-11-05 | ADR-34 | 外部 Element 実装を閉じるか | node_mut は公開だが doc(hidden)、trait は seal しない。with_node は普通の公開 API | 外部 builder が標準 setter を継承できる実装用の口が必要だから |
| 2026-11-05 | ADR-35 | 公開 API の改名をどう進めるか | 層をまたぐ署名は family 単位で一度に変え、改名と振る舞いの変更は別の変更に分ける | 同じ利用箇所を何度も書き直すことと、改名に紛れた振る舞い変更を避けるため |
| 2026-11-05 | ADR-36 | InteractionPolicy の所属と形はどうするか | interaction に inputs/menu/tooltip/button/slider/switch/toast/select/hover_card/drawer の nested policy を置く。ui が依存し、逆向きにしない。tooltip に第二の context config を残さない | behavior の設定の正本を一つにするため |
| 2026-11-05 | ADR-38 | Disclosure の既定 motion は何か | motion は animate=false、duration=200ms。部品ごとの animated reactive override は残す | 標準の既定と kit の明示指定を両立するため |
| 2026-11-05 | ADR-39 | overlay_margin の None を何に使うか | Theme.overlay_margin は常に数値で既定 0。必ず window 内に fit し、no-fit sentinel を作らない | 余白と fitting の有無を一つの Option に混ぜないため |
| 2026-11-05 | ADR-40 | すべての touch gesture を Drag に統一するか | DragApi に slop（既定0）、axis lock、100ms velocity sampling を追加。実 drag 部品は共用。touch scalar/switch/toast/select/month の Machine は既存の閾値と activation timing のまま | Drag のために三つの新概念や異なる gesture phase を強制しないため |
| 2026-11-05 | ADR-41 | APG への対応を後から別に行うか | behavior を変えるときは同じ変更で公式 APG の keyboard/roles/states に合わせ、作者の決定による差分を記録し E2E で確認する | 後から別に合わせると状態機械と behavior を二度作り直すため |
| 2026-11-05 | ADR-42 | toast/tree 行が独自の表示 phase を持つか | engine presence で出現・退場を描く。visibility 配列や animated height Machine を複製しない | 一つの表示 lifecycle に揃えるため |
| 2026-11-05 | ADR-43 | editor の syntax と装飾 API はどうするか | line_styles は content T と cross-line state S を純粋 callback へ渡す。StyledRange は byte range と色/装飾、decorations は全文対象。CodeBlock の highlight は既存 parser と palette を保つ | 行ごとの解析状態と編集内容を二重保存せず、tokenization を失わないため |
| 2026-11-05 | ADR-44 | text 色の Transition と未確認の option を増やすか | Paint が text color を担うため新 Color TransitionProp は不要。Combobox/Select の option は kit の利用者が実際に使っていた option の和集合だけを実装 | 同義の表現と要求のない knob を増やさないため |
| 2026-11-05 | ADR-45 | touch の日付入力は kit 固有か | DatePresentation Adaptive/Desktop/BottomSheet を構造 option とし、Adaptive は capabilities.touch。calendar/date/time/wheel は標準 UI と scalar に統合し kit の option をすべて維持 | 同じ日付入力の behavior と OS 判断を kit に重複させないため |
| 2026-11-05 | ADR-46 | Creative picker を全て標準 combobox に置換するか | 共通 body は標準 API を使うが、Creative 固有の column-follow picker は kit に残す | column に追従する domain の意味が一般の選択 field と異なるため |
| 2026-11-08 | ADR-48 | 複数の window を扱う shell の境界をどうするか | 一つの handler が loop 全体と複数の window を相手にし、backend の event を `EventTarget::{Loop, Window(WindowId)}` で分ける。handler の口は `ShellLoop`・`ShellWindow` とし、`open_window`・`close_window` を持つ | window ごとに backend と handler の組を増やさず、loop 全体の event（SaveState・Wake など）と window の event を一つの型で区別するため |
| 2026-11-08 | ADR-49 | engine の model を App 全体で一つにするか window ごとに分けるか | 一つの window の `Model` と App 全体の `AppModel` に分ける。`AppModel` が `windows`・`window_changes`・`AppClock`・`client_frames`・共有の `EventSink` と assets と writes を持つ | window ごとの tree・focus・surface を保ちながら、motion の時計と frame の要求を App に一つにし、ある window の surface が止まっても他の window の animated が進むようにするため |
| 2026-11-08 | ADR-50 | OS の window を公開 API でどう表すか | `window(title, open)` を `overlay` と同じ構造の view とし、content は宣言した場所の context を引き継ぐ（React の Portal と同じ）。閉じる操作は `on_dismiss` が取り消さない限り既定で window を閉じる（`dialog` と同じ） | open の binding で寿命を表し、overlay・dialog と同じ組み立てと dismiss の規則を共有するため |
| 2026-11-08 | ADR-51 | DevTools を target の中の UI にするか別プロセスにするか、protocol の定義をどこに置くか | DevTools は別プロセスのアプリとし、loopback TCP の JSON-RPC で target の agent に繋ぐ。protocol の型・method・通知は `kivact-devtools-protocol` を単一の正本とし、JSON Schema と MCP をそこから生成する | DevTools の不具合で target を止めず、同じ語彙を Rust と MCP に複製しないため |
| 2026-11-09 | ADR-53 | 性能の改善と後退をどう測るか | K1〜K6 を `dev/bench` で測り、比べる二つの commit を同じ機械で取った値の比で判定する。frame・turn・layout・a11y は「変わったものだけを処理する」（PERF-1）一つの規則にする | 機械ごとに絶対値が違い、全走査の後退は小さい試験では見えないため |
| 2026-11-09 | ADR-54 | GPU をいつ起動するか（Arvane の起動時の GPU の開始を置き換える） | `GpuStart::OnDemand` を既定にし、user shader の描画、`gpu_interop::adapter()` の読取り、CPU の frame が表示周期の半分を 8 frame 続けて超えたときに起動する。`AtLaunch` を選べる。初回の shader は device の準備まで CPU の代わりの見た目を描く | 業務画面では GPU のドライバのメモリが working set の大半を占め、CPU 合成は高速経路で十分に速い。device と pipeline は worker で作り、その間も CPU で描き続ける |
| 2026-11-10 | ADR-55 | GPU の需要をどう届け、誰が記録するか | shader の需要は composition の報告から、GPU 共有と GPU export の需要は core が `HostRequest::DemandGpu` で明示する。core は記録せず、window の寿命を持つ presenter の `Gpu` だけが起動の入口（begin、demand）と状態を持つ。需要の宣言は memo・draw の中でもよい | GPU 共有は GPU の準備の後でないと composition に現れない。記録を二か所に持つと window の close で食い違う |
| 2026-11-09 | ADR-56 | release の profile をどうするか | fat LTO、codegen-units 1、opt-level "s"、描画・shaping・layout の crate は 3、asset の worker だけで動く第三者の crate は "z"、strip、panic は unwind。雛形にも同じ profile を書く | 大きさと速度を両立する。embed（DAW の plugin）で panic を host の外へ出さない。profile は root の manifest からしか読まれない |
| 2026-11-09 | ADR-57 | authored `Style` をどう表すか | 8 つの群の疎な struct にし、操作の履歴を持たない | 履歴の再生は merge で指数的に増え、上書きで肥大する |
| 2026-11-09 | ADR-58 | ICU の分かち書きの辞書を任意にするか | facade の既定 feature `complex-scripts` にし、外せるようにする | 多言語の既定の挙動を保ちながら約 3 MB を外せるようにするため |
| 2026-11-10 | ADR-59 | hit test が paint の ink を読むとき依存の向きをどうするか | model が `HitInk` を定義し paint が実装する。3D の投影・drawable を含む subtree は ink の `beyond` の印で枝刈りと culling から外す | engine の中で hit と paint を分けたまま model が paint の内部を参照しないため。上界にならない subtree を誤って枝刈りしないため |
| 2026-11-10 | ADR-60 | a11y の構造の変化（Metadata の経路）を増分にするか | 全比較のまま残し、PERF-1 の例外として関数の doc comment に理由を書く | 構造の変化の正しさを writer の通知の完全さに頼らず全比較で守る。値と geometry の日常の更新には現れない |
| 2026-11-10 | ADR-61 | 公開の診断に a11y の走査の件数を出すか | `Stats::a11y_enumerated` を足す | E2E で a11y の全走査の後退を検出する唯一の観測点であるため |
| 2026-11-10 | ADR-62 | idle の window の CPU の frame state をいつ手放すか | 最後の present から 1 秒後 | 再描画のたびの再確保を避けながら、止まった window のメモリを返すため |
| 2026-11-10 | ADR-63 | catalog の Noto Sans JP をどう持つか | 使う文字の subset を commit し、`dev/tools/subset-catalog-font.py` で作り直す | 原本（約 9.6 MB）が catalog の binary の大半を占める。OFL は subset を許す |
| 2026-11-12 | ADR-66 | game と pop の sparkline をどこに置くか | kivact-ui の標準部品 `sparkline(values)` にし、線・塗り・余白は sparkline の theme 変数、大きさは recipe で kit が決める。kit の sparkline は削除する | 二つの kit が同じ意味の部品を持っていたため（strategy 6 章の 5）。外観だけが kit ごとに違う |
| 2026-11-12 | ADR-68 | 埋め込み App の panic を埋め込み側へどう渡すか | `Embedded::status` の `Panicked` と `App::embed` の `Err(Panicked)` で報告し、呼出し元で再開しない。`catch_unwind` は host から呼ばれる入口の二か所だけに置く（Win32 の wndproc と、X11 の埋め込み UI thread の `run`）。終了処理全体（`state.stop()`、`Session::close`、`handler.close/drop`、root の破棄）をその内側に置き、終了処理中の panic は `Panicked` にして残りを飛ばす。panic の payload は捕まえた直後に文字列化し、payload 自体は `mem::forget` する（重複 payload も同じ）。（2026-11-14 追記: 境界は上の二か所だけで、どちらも host の呼出しを受ける入口。X11 の `run` では start と event loop を一つの境界に、`Session::close` を別の境界に置く。毒された lock は `embed/child.rs` の `Shared::lock` と、X11 UI thread の join handle の slot（`x11/thread.rs` の `thread_slot`）だけで回復する。どちらも host の thread で panic の報告に使われ、unwrap すると panic が host へ出るため） | C ABI の host へ巻き戻すと process が abort する（G-01、ADR-56）。App は再び動かず、隔離ではない |
| 2026-11-12 | ADR-69 | 埋め込みの size 要求をホストの確定まで待つか | `ResizeAnswer { Accepted, Pending, Refused }`。Pending は次の host size で答え、再要求で Cancelled、2 秒で Timeout。`ShellWindow::request_size` と `ShellEvent::PlatformAnswer` の遅延応答を `RequestWindowSize` だけに置く（ADR-05 補足を置き換え） | VST3 の確定点は onSize。確定前に子を変えない |
| 2026-11-12 | ADR-70 | 未処理のキーを誰が判定し、どう host へ返すか | core が `HostRequest::UnhandledKey` で判定を知らせ、埋め込みは probe（Key の直後に Wake）で同期に判定し、`Parent::forward_unhandled_keys` で親 window へ返す。`Embedded::key_event` は保留のまま | 判定の正本を一つにし、native の message と判定を持つ層が返す |
| 2026-11-12 | ADR-71 | `UiError` に std Error を実装するか | 実装しない（blanket `From` と `From<T> for T` が重なるため）。`as_error`、`AsRef<dyn Error + Send + Sync>`、`Box<dyn Error>` / `Box<dyn Error + Send + Sync>` への `From` で相互運用し、`from_boxed` は box した `UiError` を downcast して元の error を返す（API8-03 の拡張） | `?` の推論（ADR-32）を保つため |
| 2026-11-12 | ADR-73 | 数値の値・位置の写像と表示の書式をどこに置くか | 写像（ScalarMapping）とステップの空間（ScalarStep）は kivact-interaction の scalar に置き、track・touch・Creative の Numeric が共有する（ui の Slider は保留）。値の文字列は部品の format（Fn(f64) → String）で、読み戻しは既存の text_parser。修飾キーの倍率とリセットクリックは InteractionPolicy.scrub（ScrubPolicy） | 対数ノブと fader の平方根という二つの経路を一つにし、値の単位を変えずにプラグインの曲線を扱うため。binding は常に値の単位で、位置は描画と操作だけに使う |
| 2026-11-12 | NUM-02 | ホイール 1 ステップの距離 | InteractionPolicy.scrub.wheel_notch（既定 60 論理 px = 3 行 × 20 px）。小さいデルタは同じ向きに積み上げ、逆向きで 0 に戻す。粗調整キーはホイールでは無視する | 精密タッチパッドで 1 イベント 1 ステップにならず、Shift がホイールを横に変える環境でも方向と量を保つため |
| 2026-11-12 | ADR-74 | App を外から閉じる口を何にするか | `platform::AppHandle`（Send + Sync + Clone。`App::handle` と `platform::app_handle`）。close は dismiss ではなく取消し不可。embed は Unsupported。ハンドルは 1 回の run に属し、run 後の close は何もしない（rearm しない） | 別 thread の session と main thread の UI を持つ standalone（vivace K-2）が `process::exit` に頼らず正常な cleanup で終わるため。同じことの二つ目の口（`platform::quit`）は作らない |
| 2026-11-12 | INPUT-01 補足 | probe のため turn を key ごとに進めてよいか | native の押下は配送直後に `Wake(CLIENT)` で turn を進め、`forward_unhandled_keys` の判定を message の戻りまでに得る。`key_event` は保留なので対象外 | 入力順序を保ち、ホストへ返す判定を message の戻りまでに決める |
| 2026-11-13 | ADR-75 | `foreign_window` を `window` の option にするか、別の宣言にするか | 別の宣言 `foreign_window(title, open)` とし、`WindowSlot` の開閉・`on_dismiss`・mode・placement・`multi_window` の検査を共通にする。content の有無は `SlotContent::{Views, Foreign}` で表す | content を持たない window に `.content(..)` が意味を持つ第二の mode を作らず、開閉の実装を一つに保つため |
| 2026-11-13 | ADR-76 | foreign window の子 window をいつ作るか | native の `open` の中で winit window の直後に作り、子の作成失敗はその open 全体の失敗として `WindowFailed` にする。engine は `PlatformRequest::ForeignChild` で handle を受け取り、`HostWindow.content` に保持する | 半端に開いた window や子のない foreign window を App に見せず、open の成否を一つの結果にするため |
| 2026-11-13 | ADR-77 | App の終了時に foreign window をどう閉じるか | 終了要求の turn で `ForeignWindows::close_all_for_exit` が各 foreign window の `open` binding に false を書き、同じ turn の安定化で owner を破棄する（`on_destroy` は子が生きている間に走る）。終了順を全 App で変える `ShellHandler::close` への移動はしない | 終了時だけ `on_destroy` が子の破棄後に走るのを、foreign window を使う App の局所的な変更で直し、既存の全 App と test の終了順を変えないため |
| 2026-11-13 | ADR-78 | `ParentWindow` の名前と `SurfaceSize` の所有者 | `ParentWindow` を `NativeWindow` に改名して `kivact-types::window` へ移し、別名を残さない。`SurfaceSize` も同所に移し、`kivact::view` から再公開する | 「生の native window 値」を embed 専用の名前から切り離し、`foreign_window` と `native_host` が同じ語彙を使えるようにするため（VOC-1） |
| 2026-11-16 | ADR-79 | 小さい文字、特に暗い背景の明るい文字が DirectWrite より細く薄く見える問題をどこで直すか | 文字色の相対輝度で決まる stem darkening に置き換える。`d = 0.21 × min(1, L_q / 0.8) × F(size)`（device px／片側、`L_q` は 8 段に量子化、塗りが単色でなければ 4/7、`F` は 48 px まで 1 で 96 px で 0）。暗い文字は 0。定数は Windows の Chrome の grayscale 描画（DirectWrite）に、Kanit・Kanit Mono・Inter の 10〜48 px、7 色の組で合わせた | grayscale の coverage を sRGB の値で線形に合成するため、明るい文字だけが細くなる。vello_cpu と vello_hybrid は coverage の補正口を持たず、GPU の atlas は GPU 上で描かれるので、両 backend が同じに扱える唯一の手段が輪郭の dilation である。輝度を量子化すれば glifo の atlas key（embolden 量を含む）で色ごとの glyph の増加を 8 倍以内に抑えられる。独自の rasterizer は作らない（戦略 N-03） |
| 2026-11-16 | ADR-80 | CPU の raster でも glifo の glyph atlas を使うか | 使わない。CPU は glyph を毎回輪郭から描く。vello_cpu が page の大きさを設定できるようになったら再検討する | 定常の frame は 22〜23 ms から 12〜13 ms に速くなるが、vello_cpu 0.2 は 4096 px 四方の page（64 MiB）に固定で、最初の frame は 24.5 ms から 34〜42 ms に遅くなる。CPU の raster の主な用途は短命な E2E と offscreen の書き出しなので、全体の費用（戦略 P-05）では損になる |

### 実装の決定の補足

次の細部も上表と同じ日付の決定に含む。問いと理由は対応する ADR を参照する。

- ADR-02: Backend は run だけで、事前 capabilities/open_child/shutdown は持たない。ShellOptions は native constructor の引数で寸法検証もそこで行う。embed 内部の同名 Backend は ChildPort とした。native への通常依存は持たず、native/embed の E2E のための arvane の dev-dependency は許す。embed Session は handler を drop した直後、DetachedThreads があれば module を保持する。window 作成前の capability は対応可能な上限で、native setup 後の実 ShellWindow の capability が正本である。
- ADR-03: SteppedBackend.start は window を作り handler.start を呼んでから返す。Idle は next_deadline/redraw_requested/woken を持つ。試験 host は handler を loop と共有し、engine の inject は試験専用で残す。node query は同じ handler から読む。a11y の公開は frame 後に揃え、turn 後に redraw 要求があれば必ず描き、見たときだけ描く経路は持たない。
- ADR-03: GPU の renderer 通知を待つ wake は最大5秒。frame の readback は keeps_frames の window だけで行い keep_frame へ渡す。test_font/gpu_unavailable は presenter/text を知る arvane root に所属し、arvane に test feature は置かない。arvane の E2E は handler 内で実行する Harness.with、Script、Probe で契約を保つ。virtual に winit を持ち込まない。
- ADR-04: raw は Key/Ime/Drop。PlatformKey は Vk（physical code付き）/Keysym/Code で winit 型を含めない。IME は Preedit/Commit/End/Reconvert、DropSession は Idle/Hovering/Dropped。Win32/winit は repeat を明示し、X11 の未指定だけ押下集合から導出する。Screen.raw_input から Script.raw を通す公開試験口を持つ。IME の詳細は IME-01 を参照する。
- ADR-05: CPU の native 提示形式は Xrgb8888。clipboard/dialog 等の個別 ShellWindow service method を並べず perform に統合する。`RequestWindowSize` だけは `ShellWindow::request_size` が `Later` を返し、`ShellEvent::PlatformAnswer` で答える。他の service は同期。CommandKey の既定は Ctrl、ContextMenuTrigger は Release。virtual は memory clipboard、dialog queue、記録を保ち、respond の request→optional result callback で上書きする。別の PlatformRequestKind は不要。
- ADR-06: ChildStatus は Running/Stopped(cause,window)、ChildLifecycle は Opening/Live/Disposed/Leaked、HandlerSlot は Unstarted/Running(handler,control)。Win32 の drain 中にも event callback が走るため ReentryGuard と DrainGuard は同時に有効となり得る。X11 の atomic guard は ConnectionAlive と名付ける。
- ADR-07: skip は GPU の連続 loss の数も reset する。Healthy/Skipping(frames)/Losing(losses) は相互排他で、Recover/FallbackToCpu/Fail を返す。
- ADR-08: GPU import 未対応の公開分類は Unavailable と具体的理由を使い、新しい FailureKind::Unsupported は足さない。core 単独 build の GPU 試験は gpu_unavailable で skip できる。
- ADR-09: HostPhase は Starting/Running/Exiting/Closed、AppPresence は Foreground/Suspended、Surface は Drawable(generation)/Unavailable(NoSurface/ZeroSize)、LastPresent は Settled/Skipped(retry_at_ns)。どの phase でも presence/surface は独立した事実。SurfaceLost は再作成後の一回の HostFailure であり Unavailable::Lost を作らない。
- ADR-09: Launch の Pending/Running payload は Box 所有。Starting は引数を移して実 Host/Model を構築中の状態で、その間の event は invariant 違反。waker が得られる start で本物の asset store を作り、仮の store を後から差し替えない。前回 Environment は Sent の借用で読み theme/motion override を保つ。
- ADR-10: lower は resolved style/DrawList/text run→scene の変換で tree/input を知らない。BoxShape、asset adapter/event、shape は lower 内に留め types へ広げない。一回の Context 内の Cell/RefCell は残し、finish が dropped 件数と failures を一度だけ返す。
- ADR-11: 変化がなくても overlay の再配置と geometry drain は可能でなければならない。LayoutReasons は PASS/STYLE/VIEWPORT、settle は sync→layout→lint→sync の唯一の順序を持つ。hit test/file drop/wheel/redraw の前に必要な settle を呼ぶ。columns/state と実施結果の Kept/Updated は private とする。
- ADR-12: authored display は optional、computed display は確定値。Contents は共通の flatten 処理で消費し Taffy へ渡さない。hidden は display mode と別で保持する。
- ADR-13: TransitionProp::Size を使い Orientation 方向に presence scale を適用する。公開 Presence は Entering/Present/Leaving、内部 Pending/Animating は両方 Entering。engine の通知と Reader から NodeRef.presence が初回 query で signal を作る。parent が直接読む item size/min/max/basis にも scale を適用し、child の natural style/cache は保つ。
- ADR-14: MouseGesture は Route/Scrollbar/SelectText、TouchGesture は Route/Pending/Pan。press/capture は listener の有無や副buttonでも異なるため struct Route のまま。KeyDownTargets は passive focus 宛先表で routing 実装へ逆依存しない。
- ADR-15: soft_keyboard の owner は専用の text-input/IME 処理であり一般 HostPhase へ複製しない。OS service の状態は window の perform（ADR-05）で扱う。
- ADR-16: handler/turn/time を reactive より上の独立 module にすると binding/callback が HandlerResult を使うことで循環する。Runtime は一つで、host module の inherent impl が上位の構築/settle/finish を提供する。motion は task に依存しない。
- ADR-17: Runtime の内部 extension OnceCell は constructor が返る前に埋まり外から空を観測できない。build の panic は registry から外す。build 中に signal/trigger/selector は作れるが ext は読めない。Driver、late waker 設定、optional root、turn_requested の保存は置かない。blocking threads の既定は min(4, CPU数)。
- ADR-18: TurnReport は timer/drain の最短 deadline と ExitState。exit は Running/Draining(deadline)/Exited、request_exit の再呼出しで deadline を延ばさない。eventなしでも mount boundary がなければ report_now で終わる。before revision と done revision は所定の位置で一回ずつ読む。
- ADR-18: mount 待ちは unstamped と revision付き awaiting の型付き queue。deliver_reports は開始 snapshot だけを配送し後の報告は次回へ残す。frame request は時刻と writer revision の write であり bool にしない。finish が user effect/ready task/later report/client pending から一度だけ wake を決める。commit 上限時は user effect を abandon して Cycle を報告する。
- ADR-19: MotionEnv は reduced_motion/surface_active/frame_period_ns の readonly Value を constructor の Extensions に置く。attach_env、optional env、reduced-motion setter の mirror は作らない。motion 有効化時の reduce_motion は保つ。Sleep は Unregistered(deadline)/Registered(id)/Fired、Drop は cancel queue に積む。
- ADR-20: effect_on の Watcher は User queue、Animated は Internal queue で最初に評価する。生成時へ評価を早めると setup 値になり最初の user round の値を失うため行わない。WriteInPureContext の misuse は変更しない。
- ADR-21: AsyncCell を書くのは start/finish/reset のみで、state/latest getter を memo にしない。latest の読み手を無関係な state change で起こさない。Drain は Queue/All runner だけから作り Weak registry で保持。shutdown 後の blocking は新しい UiError を返さず invariant panic とする。root 破棄と最後の poll の後なので正当な App の code からは届かない。
- ADR-22: use_app_state の初回 init は root setup で実行して root context に provide する。Extensions は固定 storage の借用を返すため、unsafe な後付け増設 map とは統合しない。
- ADR-23: task 完了 handler は Handler<Result<T,UiError>> で、各 handler へ結果を clone して渡す。on_done が持つ T:Clone の契約内で行う。
- ADR-24: dispatch 自体も Idle/Running(queue) 一つの enum。after は同 variant の前 task を cancel し、その場で Sleep を作ってから spawn する。effect は machine owner の handler frame で順に実行する。
- ADR-25: AppOptions は内部 launch module の設定で facade から再公開しない。RenderOptions だけの host_locale/dark/scale/mirror/gpu/frame period/from_env などはその builder に置く。font の登録元を保持し、拒否時の報告を失わない。
- ADR-26: LaunchHandle.inspect は起動済みのときだけ存在。core だけが答える root/app_view/位置/owner/why/runtime stats/turn record と engine Reader を分ける。semantics snapshot は Reader の host tree を辿る。
- ADR-29: Removed の ids は論理削除した subtree の前順。handler records は tree borrow が終わってから drop。退場 paint が残っていても core handler、asset failure と NodeRef の detach は論理削除で処理する。
- ADR-30: flow の show/dynamic/for_each/for_index/virtual-list行/error-boundary/provide は Contents の実箱を作る。箱自体にsemantics/hit/focusを付けず、未挿入の部分木も親なしの実箱として構築可能。a11y の論理 hierarchy は BOUNDARY-05 を参照する。
- ADR-32〜34: HandlerResult の変換 method と Element.node_mut は hidden な実装口。外部 builder の実装可能性は維持する。payloadなしcallbackをownerの種類で不当に制限しない。
- ADR-36: policy は Clone/Debug/PartialEq/Default と公開 field を持つ FRU 値。tooltip delay/skip は500/300ms、hover card open/close は600/300msで部品別上書きも残す。toast slop8px、select slop10px、menu drift context10/other13px。drawer close fraction0.3、flick speed0.6、distance4。現在の追加 field と正確な署名はrustdocを参照する。
- ADR-39: overlay margin は inset の数値で既定0。負数/NaN、未設定modeを no-fit sentinel として使わない。
- ADR-40: DragUpdate.velocity は px/ms、track_velocity 未指定は ZERO。axis_lock は slop を越えた最初の軸に固定し交差方向deltaを0にする。raw touch gesture は primary touch/pen filter、slop前の見た目、activation抑止時点を自分のMachineで持つ。switch10px/month6pxは名前付きprivate定数とし、閾値を変えない。
- ADR-41: APG の対応は E2E の apg_<pattern> と操作を表す名前の試験で契約を確かめる。
- ADR-42: toast は行の exit_size、tree は子行の enter_size/exit_size を使う。行ごとの slot の二重化、generation、高さの Animated、前回の開閉や高さを補修する状態を持たず、行の表示 phase を表す別の Machine も作らない。
- ADR-43: content T は Clone+PartialEq、cross-line S は PartialEq+Default。line_styles は (&T,line,&S)→(ranges,next S)、全文decorations は &T→decorations。StyledRange はoptional foreground/background/decoration、DecorationKind はUnderline(style)/Outline/Background。行番号だけを渡す形では content と block comment 等の状態を失うため採用しない。
- ADR-44: option 名は実際の kit 利用から決め、未確認の仕様一覧を先に作らない。calendar の option の和集合は日付入力の側に含める。Paint に text color が含まれることは E2E で確認する。

## 境界の決定

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | BOUNDARY-01 | Theme の値と source を二重に持つか | 固定 root theme は必要な Signal へ一度正規化し、動的 theme は元の Value を保つ | query のために別の source を同期しないため |
| 2026-11-05 | BOUNDARY-02 | Drag/Select に不足した口を alias で埋めるか | Drag.source_with は press 時 factory、active_payload は readonly、Generation.as_u64 は明示 query。Select.trigger_with_sheet/show_sheet/is_sheet_open は既存 owner の実 port | 新しい payload copy や open mirror を作らず実際の要求を表すため |
| 2026-11-05 | BOUNDARY-03 | editor drag と domain selection を混ぜるか | editor は zero slop。pointer/origin/payload は Drag、選択意図は domain、numeric payload は一度だけ保持 | 入力経路と選択の意味が異なるため |
| 2026-11-05 | BOUNDARY-04 | authored text decoration に backend 型を出すか | Color と TextLength を入口とし、Rgba は内部の解決値。Draw と呼び Canvas alias は置かない | API の値と backend 表現を分け一つの名前にするため |
| 2026-11-05 | BOUNDARY-05 | Contents と a11y の論理構造をどう両立するか | layout/paint/input の透過走査を共有しつつ、意味のある論理 hierarchy を a11y で保ち、bounds は実 children の合併にする | 透明 layout を理由に親子の意味や正確な bounds を失わないため |
| 2026-11-07 | BOUNDARY-COMPOSE | 合成をどこに置くか | `arvane-compose` を新設し、Vello は layer の 2D ラスタライズだけにする | 3D、cache、shader、外部 texture は、Vello の外で一つの合成として扱うほうが単純であるため（Chrome の Skia と cc の分担と同じ） |
| 2026-11-07 | BOUNDARY-SHADER | WGSL の検証をどこに置くか | `kivact-shader`（naga だけに依存） | style が構築時に検証して UiError を返せ、style に wgpu を持ち込まないため |
| 2026-11-05 | API8-01 | immutable bytes と shaping の共有を利用者へ露出するか | AssetBytes、ShapedTextSnapshot、GlyphRunSnapshot、SemanticsTextRuns で private Arc を隠す。owned Vec/iterator と参照 query を普通の入口にし、実 backend の shared allocation は理由付き hidden bridge で再利用 | allocation identity/caching/cheap clone を維持しながら ordinary API の Arc をなくすため。Vec 変換の allocation 不要は保証しない |
| 2026-11-05 | API8-02 | Draw の measurer をアプリが install するか | set_text_measurer は doc(hidden) と precise internal-contract を付す自動 mount/offscreen bridge | 普通の Draw 利用に手動 install の順序を要求するのは不適切であり、実際の呼出者は内部二経路だけのため |
| 2026-11-05 | API8-03 | 診断とRust Errorの情報まで同義wrapperで隠すか | UiError/ErrorKindとdiagnosticsのsource Location、UiError.from_boxedの標準Error interopは定義上の理由付き例外にする。普通の部品には広げない | 診断の出所と標準Errorの相互運用は実際の境界であり、同義wrapperを増やすと意味と使いやすさを失うため。逆向き（`UiError::as_error`、`AsRef<dyn Error + Send + Sync>`、`Box<dyn Error>` / `Box<dyn Error + Send + Sync>` への `From`、`from_boxed` の往復）も同じ理由の例外である。 |

## 公開 API の契約

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | CONTRACT-02 | ScrollSnap をどう表すか | ScrollSnap は None/Start/Center/End とし、ScrollAxes の方向で子を整列する。可変サイズの子を扱い、raw pixel の停止位置の別設定は増やさない。nearest_snap は types の一つの純粋実装を interaction が再公開する | snap の位置を子の配置から導き、同じ計算を二か所に持たないため |
| 2026-11-05 | CONTRACT-03 | window の寸法をどこまで受け入れるか | window の寸法は少なくとも 1 logical pixel、有限、unsigned の native extent 内とし、App の起動と resize が同じ判定を使う。native は scale 後の physical size も検証する | f64 で有限になったことを理由に不正な寸法を通さず、f32 の underflow に共有 API を依存させないため |
| 2026-11-05 | CONTRACT-04 | reactive な theme を thread をまたいで渡すか | 固定の theme と Send な起動用 data だけを UI thread に渡す。embed の reactive theme は root factory の中で provide_theme し、App 側の呼出し元で作った reactive theme は明示的に Unsupported とする | Rc や owner に束縛された Value は Send でなく、unsafe や snapshot で偽装しないため |
| 2026-11-05 | CONTRACT-06 | Pagination の総数をどう渡すか | pagination の総数は pages と items のどちらか一方で受け、内部の PageCount で排他にする。Stepper の collapsed は geometry から導く値とする | 二つの総数の不整合と、導ける値の別保存を避けるため |
| 2026-11-05 | CONTRACT-07 | 失敗の分類をどこまで増やすか | FailureKind は Content/Resource/Backend/Surface/Unavailable とし、任意の新分類を足さない。render/text adapter の error は Failure へ分類し、core の利用者向け診断は source location を持つ別の意味として保つ | 分類を増やしても境界は増えず、診断の出所を失わないため |
| 2026-11-09 | CONTRACT-09 | 名前の衝突をどう解くか | `NodeBuilder` を `Element` に、trait `Component` を削除して `impl Element` に、setup の境界の関数 `component()` を `scope()` に、試験の `Element` を `Locator` に改名する | 一つの名前に一つの意味を持たせ、component を `#[component]` の意味に一本化するため |
| 2026-11-10 | CONTRACT-13 | 公開の recipe state の形と名前 | 排他な事実は pub の enum の field、常にある事実は pub の field、一部の種類にだけある事実と導出する事実は accessor（bool は is_/has_）。外部の theme は `state.input.pressed` を `state.input.is_pressed()` のように読む | 無効な組合せを型で除き、読み方を部品の間で揃えるため |
| 2026-11-07 | CONTRACT-ISOLATION | blend の isolated group | CSS と同じ。stacking context（z-index による分離を除く）が isolated group になる | CSS の利用者の期待と一致させるため |
| 2026-11-07 | CONTRACT-FILTER-SURFACE | 色の filter を各色に掛ける最適化（Tint）を残すか | 残さない。filter はすべて合成後の layer の surface に掛ける | clamp を含む色の行列は合成と交換できず、CSS と結果が変わるため。経路を一つにして内部も単純になる |
| 2026-11-07 | CONTRACT-COLOR | 広色域の色をどこまで元の空間のまま運ぶか | gradient の stop（engine が CSS の補間と gamut mapping で展開する）、style の `Color::mix`、`Animated<Color>` は元の色を保ち、最後に一度だけ CSS の gamut mapping で sRGB へ写す。engine の state transition だけは、sRGB に写した端点から Oklab で補間する | wire の `PaintProps` の色の型をすべて変える複雑さを避けるため。広色域の出力を行わない範囲での制限である |
| 2026-11-07 | CONTRACT-SPRING-DURATION | `Spring::with_duration` の duration の意味 | 静止するまでの時間。振幅 500 以下では振幅によらない（静止の判定は、位置と速度/ω0 に同じ相対の閾値 `rest` を使い、振幅を 500 で頭打ちにする）。step は解析解 | Framer Motion の利用者の期待と一致させ、duration と frame の間隔によらず契約を守り、最後に跳ぶ量を 0.5 以下に抑え、すべての係数で安定させるため。振幅 500 を超えると静止が少し遅れる |
| 2026-11-07 | CONTRACT-LAYOUT-REPEAT | layout motion に反復の transition を与えたとき | 反復と往復を使わず、一回で goal へ動く（報告する） | layout motion は box を goal へ運ぶ動きであり、往復して元の位置で終わると layout と描画が食い違うため |
| 2026-11-07 | CONTRACT-RASTER-GRID | 動く layer の raster の格子 | 動く layer（`will_change(Transform)` の layer、または opacity・transform が動いている layer）は layer 空間に固定した格子で raster し、小数 px の位置は最後の合成で bilinear にする。止まっている layer は target の px に snap する | 動きの間に raster をやり直さないため（Chrome の合成 layer と同じ）。動いている間だけ、小数 px の位置で輪郭がわずかにぼける |
| 2026-11-07 | CONTRACT-GRADIENT-PRECISION | gradient の画素の精度 | engine の展開は式と 0.5/255 以内。最終の画素は Vello の LUT（最大 4096 項目）の量子化を含む | 独自の gradient の rasterizer を持たず、CPU と GPU で同じ Vello の経路を使うため |
| 2026-11-07 | CONTRACT-REPEAT-END | 反復した run の終わりの値 | 最後の反復の終わりの値（alternate で偶数回なら from）。`Animated`、state transition、Timeline で共通 | CSS の animation と Framer Motion と同じにし、経路で終わりの値が違わないようにするため |
| 2026-11-07 | CONTRACT-3D-CROWD | 参加面の多い 3D コンテキスト | 参加面が 256 を超えたら、描画も hit も DOM 順の平坦な重なりにする（engine と compositor が同じ定数で判定する） | 描画と hit の食い違いを作らずに、BSP の分割の量を有限にするため |
| 2026-11-07 | CONTRACT-EXPORT-ACK | GPU の export の開始 | consumer の import が共有の release fence を `SHARED_PREPARED`（1）に signal するまで、window は fresh な slot に書かない（frame の値は 2 から）。それまでの export と、slot が無いときの export は、backoff（上限 100 ms）で再試行する。公開 API に ACK も unsafe も持たない | 既存の `SharedTextures` の hand-off の契約を、確かめられない自己申告ではなく GPU の fence で守るため。import の完了を知らせる処理は kivact-gpu-share の safe な `Import::new` の中にあり、利用者は import するだけでよい。止まった consumer で window も CPU も止めない |
| 2026-11-07 | CONTRACT-BACKDROP-ROOT | backdrop が読む範囲 | CSS の Backdrop Root（effects を持つ layer）または root。effects が既定の layer（overlay、fixed、cache、3D、drawable）は透過として、その中身を重ねて読む | 実画素を読む合成で、CSS の Backdrop Root と同じ見え方にするため |
| 2026-11-05 | IME-01 | IME の選択範囲の offset を何で数えるか | 直前の surrounding-text snapshot に対する UTF-8 byte で数える。native は snapshot で UTF-16 から変換し、engine は excerpt の origin を全文の editor へ加える。custom session は全文を供給する | native と engine で同じ文字境界を使い、snapshot に重複した全文 offset を持たないため |
| 2026-11-05 | INPUT-01 | 遅延配送する key 入力は何か | 遅延配送するのは KeyPhase::Down だけとし、release は保留中の入力を先に送る | release まで遅らせると入力の順序と中間値が変わるため |
| 2026-11-05 | SEM-01 | Semantics の等価性をどう定めるか | Semantics<R> は PartialEq/Hash を実装し Eq は実装しない。text snapshot の Hash は符号付きゼロを正規化し、PartialEq と NaN の扱いは変えない | 宣言値と計算値の両方を持ち浮動小数の幾何を含むため Eq を満たせず、等価性を変えずに hash を使うため |
| 2026-11-05 | TEXT-01 | computed text が相対値をどう保つか | 継承を解決しても relative length と current color を保持し、pixel 化は利用時に行う。親 10px の 0.1em の spacing は子 20px で 2px になる | 第二の raw style snapshot を作らず、継承先の font で正しく解決するため |

## 状態の決定

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | STATE-JOBS | Creative jobs の一つの running を Machine にするか | 独立したrun/stop入力を保ちprogress/amountを測定値とする。effectは実FrameLoopのstart/stopのみ | 他Signalの補修がなく、phaseのためのboolではないため |
| 2026-11-05 | STATE-CHAT | catalog chat の text と typing をどう表すか | ChatContent::Message(text)/Typing。id/mine は passive identity/著者情報 | 空textとtypingの無効組合せを除き、時間と通知/選択ownerを保つため |
| 2026-11-05 | STATE-METER | integrated meter の latch をどこが持つか | 単一測定ownerがUnmeasured/Measuredと実transportのSample/Resetを処理 | playing mirror、時刻、epochを追加せず測定の事実を保つため |
| 2026-11-05 | STATE-SCROLL-SPY | scroll spy とnavigationが別のactive sectionを持つか | 一つのMachineへObserved/Requestedを送り、geometryがないとき最後の事実を保つ | navigation直後の選択を保ち、scroll/geometry/selectionのmirrorを除くため |

## 拡張の境界

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | EXT-01 | 拡張時の変更箇所 | text styleの公開語彙は所有するstyle crateの明示facade一覧で一元化し、Coreはその限定した一覧だけを転送する。virtualは既知serviceを処理し未知requestはUnsupported、capabilityは構築時の値と実GPU/native事実から投影する | 新しい型やserviceごとにCoreの重複一覧やvirtualの網羅matchを修正する第四のcrateを不要にし、型付きAPIと正直な未対応の契約を維持するため |

## ADR: term の参照と gauge の範囲

- 日付: 2026-11-05
- 問い: カスタムtermのラベル参照と、Game kit の gauge をどの公開契約に揃えるか。
- 決定: カスタムdescription termの実wrapperはGroupとして公開し、detailのlabelled_byが同じnamed termを指す。term/detailに同名Groupがあっても、detail内の操作と参照先を検証する。Game入門のgaugeも標準ProgressBarの0..1範囲を使い、4/10は0.4と検査する。
- 理由: unroledなGenericはnativeラベル参照先にならず、見えるtermと支援技術の関係が切れる。進捗のvariant間で役割と数値契約を統一し、別の目盛りを持つ Meter を公開APIへ戻さない。

## ADR: snapshot の取得境界

- 日付: 2026-11-05
- 問い: 同じasync処理でframeやPixelsが更新された直後のNodeRef::snapshotが何を取得するか。
- 決定: Hostのrequest/reply経路でlayoutとpaintがsettleした後のCPU描画を取得する。poll時のNodeIdを束縛し、応答時に寿命を再確認する。追加のFrameTickや実時間待ちを要求しない。
- 理由: Render再計算前のsceneを同期的に読むと、更新後の画像ではなく直前の画像を新しい寸法に合わせた描画を取得し、一部が透明になる。状態更新と描画確定の順序をhost境界で守り、表示とsnapshotに別のgeometryモデルを作らない。
