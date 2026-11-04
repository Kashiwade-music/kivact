# Kivact 基盤再設計（Foundation v3）設計書

- 発行: 2026-11-04（Claude Code が `main` @ `a3536ef` を調査して作成）。2026-11-05 改訂（19 章: W0・W1 の実装の結果と、W2 以降の unit を code に当てた調べによる）
- 実装担当: ChatGPT Codex（主エージェント = オーケストレータ）と、Codex が起動するサブエージェント
- 作業手順: [`REFACTORING_WORK_ORDER.md`](REFACTORING_WORK_ORDER.md)
- 付録:
  - [`refactoring/A-state-machines.md`](refactoring/A-state-machines.md) — 暗黙の状態機械の全一覧と、それぞれの解消後の形
  - [`refactoring/B-public-api.md`](refactoring/B-public-api.md) — 公開 API の最終形（module ごとの item、改名・移動の表、規則の例）
  - [`refactoring/C-kits.md`](refactoring/C-kits.md) — 公式 kit の整理（中核へ上げる部品、kit の規約、kit ごとの作業）
- 依拠する戦略: `docs/for-kivact-developer/strategy.md`（v3.0）。依頼文の `docs/for-kivact-developer/strategy/20261009_strategy.md`（v2.5）は b387290 でこの文書へ整理・統合された。本書は v3.0 の章番号で参照する。

---

## 目次

0. [この文書の読み方](#0-この文書の読み方)
1. [結論](#1-結論)
2. [現状の診断](#2-現状の診断)
3. [設計の規則](#3-設計の規則)
4. [目標のクレート構成](#4-目標のクレート構成)
5. [語彙の一本化（`kivact-types`）](#5-語彙の一本化kivact-types)
6. [ランタイム（`kivact-runtime`）](#6-ランタイムkivact-runtime)
7. [中核（`kivact-core`）と `kivact-test`](#7-中核kivact-core-と-kivact-test)
8. [エンジン（`kivact-engine`・`kivact-lower`・`kivact-layout`・`kivact-editing`）](#8-エンジン)
9. [Arvane（backend の注入と SPI）](#9-arvanebackend-の注入と-spi)
10. [公開 API](#10-公開-api)
11. [振る舞いの層（`interaction`）と標準部品（`ui`）](#11-振る舞いの層interactionと標準部品ui)
12. [公式 kit](#12-公式-kit)
13. [暗黙の状態機械の解消](#13-暗黙の状態機械の解消)
14. [検査（xtask）](#14-検査xtask)
15. [戦略の遂行性（変更前後の比較）](#15-戦略の遂行性変更前後の比較)
16. [やらないこと・リスク・要確認事項](#16-やらないことリスク要確認事項)
17. [根本解決の追加](#17-根本解決の追加2026-11-04-の改訂)
18. [第 4 回のレビューによる改訂](#18-第-4-回のレビューによる改訂2026-11-04)
19. [実装の結果による改訂](#19-実装の結果による改訂2026-11-05)

---

## 0. この文書の読み方

### 0.1 位置付け

本書は **何をどの形にするか** を決める。**どの順に、どう進め、何をもって完了とするか** は `REFACTORING_WORK_ORDER.md` が決める。実装者（Codex）は本書の決定を変えない。本書とコードが食い違ったときの扱いは 0.3 節。

### 0.2 用語

| 用語 | 意味 |
|---|---|
| 中核 | `kivact` facade から届く crate 全部（`kivact-core`、`kivact-runtime`、`kivact-style`、`kivact-interaction`、`kivact-ui` など） |
| 下層 | 中核の下で動く crate（`kivact-engine`、`kivact-lower`、`kivact-layout`、Arvane の各 crate） |
| 語彙 | 複数の crate が同じ意味で使う値の型（座標、色、役割、入力 event、semantics、style の計算値など） |
| 契約 | 二つの crate の間で、値以外（closure、trait、handle）を含めて取り決めたもの |
| 暗黙の状態機械 | 状態を enum で表さず、bool・`Option`・counter・呼び出し順・collection への所属の組合せで表しているもの（13 章で定義） |
| 正準パス | 公開 item の唯一の module path（10.2 節） |
| 構造の option | mount のときに一度だけ決まり、木の形や振る舞いの種類を選ぶ option（10.6 節） |

### 0.3 何を正とするか

1. **本書の決定が正。** 付録は本書の決定を具体化したもので、本書と付録が矛盾したら本書に従う。本書の中では **18 章 → 17 章 → 1〜16 章** の順に優先する（18.0 節）。古い記述が新しい章と食い違うときは新しい章のとおりに読み、選択肢として扱わない。
2. **現状の事実（`file:line`）はコードが正。** 本書と付録の `file:line` は `a3536ef` の値で、作業が進むとずれる。ずれていたら、その名前でコードを検索し直す。
3. **実装者は設計を決めない。** 実装者（Codex のオーケストレータとサブエージェント）が自分で決めてよいのは、次の **機械的な導出** だけである。
   - **M-1 private な名前:** 公開されない field・variant・関数・module の名前。現在の code の名前から作る（例: `pointer_pressed: Signal<bool>` → `Press::Pointer`）。
   - **M-2 状態機械の event・effect・遷移:** 付録 A が「解消後」に状態の型だけを書いた項目では、event は「現在その field を書き換えている入口（メソッド、handler、effect）」を一つずつ variant にし、effect は「その入口が今行っている副作用（signal の書き込み、focus の移動、task の起動、caller の callback の呼び出し）」を一つずつ variant にし、遷移は「その入口が今行っている field の書き換え」をそのまま写す。振る舞いを変えない（取りうる状態の組合せだけが減る）。
   - **M-3 variant の data:** 付録 A の「解消後」の variant が持つ field は、その状態のときに現在の code が読む field だけにする。
   - **M-4 file・関数・commit の分け方:** 1000 行の上限の範囲で module を葉に分ける位置、80 code 行を超える関数を同じ葉の module の private な helper 関数（か新しい葉の module）に分けること、commit の分け方。
   - **M-5 試験の書き直し:** 名前と形が変わった API に合わせて既存の試験の書き方を直すこと（確かめる内容は変えない）。
   - **M-6 下層の内部の署名:** facade から届かない型と関数（`kivact-lower`・`kivact-layout`・`kivact-engine` の内部、各 crate の `pub(crate)`）の引数を、現在渡している値を struct にまとめる形で決めること。本書が名前だけを決めた型（例: 8.3 節の `LowerCx`）の field もこれで決める。
   - **M-7 公開の builder の引数の型:** 本書と付録が名前と意味だけを書き、型を書いていない公開の builder のメソッドは、付録 C.2 の冒頭の「引数の型の表」で型を一意に決める。
   - **M-8 `UiTokens` の field:** 既定の recipe の literal と theme の読み出しの一つずつを、17.7 節の名前の規則（`<部品>_<部分>_<性質>`）と型の規則で `UiTokens` の field にすること。

   これ以外（**公開の名前・公開の型と署名・振る舞い・crate と依存・本書にない新しい概念**）を本書と付録が決めていないことに気づいたら、実装者は作業を止め、作者（Kashiwade）に聞く。聞いた結果は作業手順書 0.5 節の決定記録に書く。
4. **互換は残さない。** Kivact は未公開で、互換 wrapper・alias・旧 API・二重の経路を一切作らない（CONTRIBUTING.md）。workspace の中の呼び出し元（例・雛形・kit・試験・文書）は、同じ変更で新しい形へ移す。kit の外部利用者は追従するので、それを理由に変更を避けない。

## 1. 結論

Kivact は機能の点では戦略 5 章の多くの行を満たしているが、**同じ意味が層ごとに書き直され、状態が層ごとに複製され、公開 API が経路と形の点で揃っていない**。このため、一つの機能を足すのに 3〜6 crate・最大 4 層の構造体を書き換え、状態の食い違いを effect や呼び出し順で抑えている。今の形のままでは、戦略が求める「第二の backend」「地域書式」「テキスト編集の拡張口」「モバイル」が、どれも全層を貫く改修になる。

以下の 32 の決定で、これを解消する（D-13〜D-31 は 17 章、D-32 は 18 章）。

| ID | 決定 | 主な効果 |
|---|---|---|
| D-01 | **語彙は `kivact-types` 一つに置く。** `kivact-wire` の値の部分を `kivact-types` へ統合し、closure を含む契約は `kivact-engine::contract` へ移して `kivact-wire` を消す。同じ意味の型（`NodeId`×2、Scene×2、Semantics×3、Environment×3、Pointer×4、Pixels×3、FileDialog×3、Embed options×3、MouseSettings×2 など）を一つにする | 一つの性質を足す変更が 3 crate で済む（15 章） |
| D-02 | **App のランタイムを `kivact-runtime` 一つにする。** `kivact-reactive`・`kivact-task`・`kivact-motion` の実行部を統合し、turn の順序・時刻・timer・frame・task・waker・終了を一つの状態機械が持つ | 二つの waker、四つの timer の仕組み、後付けの初期化（`OnceCell`・`attach_env`・`set_waker`）、呼び出し順の規則が消える |
| D-03 | **`kivact::test` を `kivact-test` crate に分ける。** 中核の起動経路を `kivact_core::launch` に一つにまとめ、`App::run`・`App::embed`・試験が同じ経路で起動する | 中核から `test` feature と 6k 行が抜け、試験と本番の起動の差が消える |
| D-04 | **backend を注入できるようにする。** `arvane-shell` に `Backend` trait を置き、`arvane-native`（winit・Win32・X11）をその一実装にする。試験用の `VirtualWindow`／`VirtualShell` を、同じ SPI を実装する新しい crate `arvane-virtual` に置き換え、`arvane::Window` trait（二つ目の窓 trait）を消す | 戦略 P-10 の「第二の実装」が常に動き、試験が本番と同じ presenter・driver を通る。窓を開けない環境でも全 E2E が回る |
| D-05 | **OS サービスは `PlatformRequest` → `Services::perform` の一本の経路にする。** サービスの有無は `Capabilities` で事前に分かる | OS サービスの追加が 3 crate で済む |
| D-06 | **`kivact-text` を `kivact-engine` に統合し、`kivact-lower` の公開面を狭める。** 境界のはっきりしない crate は統合し、はっきりした境界（layout、lower、editing、runtime、test、virtual backend）だけを crate にする | crate の数は 28 → 26（kit 5 を含む。5 つ消し 3 つ足す）。全 crate に境界の根拠を書ける（4 章） |
| D-07 | **公開 API は正準パス一つ＋prelude にする。** module を `state`・`view`・`style`・`draw`・`motion`・`task`・`i18n`・`platform`・`diagnostics`・`interaction`・`ui`・`custom` に揃え、同じ item を複数の module から出さない | 「どこから import するか」の迷いが消える |
| D-08 | **handler・prop・slot・数値・文字列・bool の形を規則で一つに決める**（10.4〜10.9 節）。handler の戻り値の trait を一つにし、位置引数の callback をなくし、数値は `impl Into<f64>`、変わりうる option は `impl IntoValue<T>` にする | React・SolidJS の開発者が一つの規則で全部品を書ける |
| D-09 | **modifier の語彙を一つにする。** `NodeExt` と `Part` の 40 個の複製をなくし、`Node` の固有メソッド（全部）と `NodeBuilder`（呼び出し側に安全な部分）の二つにする。`Part` は「`Node` に modifier を当てる closure と状態」にする | semantics の性質の追加が builder 一か所で済む |
| D-10 | **振る舞いは明示的な状態機械にする。** `kivact::state::Machine`（状態 enum・event・遷移関数・effect）を用意し、`interaction` の全 behavior と `ui`・kit の独自の gesture をその上に作り直す | 13 章の全項目が解消され、新しい behavior も同じ形で書かれる |
| D-11 | **二つ以上の公式 kit が持つ同じ意味の部品を `kivact::ui` と `interaction` に上げる**（戦略 6 章の 5）。kit は token・recipe・分野固有の部品だけを持つ | kit の重複（Calendar・日付入力・Combobox 系・Collapsible・Hover card・Carousel・Tree・Toolbar・Segmented・Dropzone・Badge・Avatar など）が消える |
| D-12 | **規則を `cargo xtask check` で強制する。** 状態の bool、公開 API の形、名前の衝突、正準パス、kit の literal color を機械で検査する（CI は既定で無効なので、CI には何も足さない。検査は手元の `cargo xtask check` で回す） | 再設計後に同じ崩れ方をしない |
| D-13 | **木の構造は engine の木だけが持つ**（17.1 節）。engine に透明な `Fragment` の node を足し、core の木を side table にする | host の node の並びの二重の持ち主が消える。`display: contents` も同じ仕組みで書ける |
| D-14 | **arvane-native を OS の port と共有の pipeline に分ける**（17.2 節） | key・IME・DnD・CPU の提示・cursor の三重の実装が一つになる |
| D-15 | **feature は実装の crate と公開 API の crate だけが持つ**（17.3 節、DEP-4） | `gpu-interop` が 7 crate、`embed` が 5 crate を貫く状態が消える |
| D-16 | **関数は 80 code 行まで**（17.4 節） | 200 行を超える関数がなくなる |
| D-17 | **内部の callback の保存は `Handler<E>` 一つ**（17.5 節） | 内部の handler の型の重複が消える |
| D-18 | **handler に `Callback` をそのまま渡せる**（17.6 節） | `on_add.call(())` を包む closure が要らない |
| D-19 | **既定の recipe の値を `UiTokens` にする**（17.7 節） | 値が違うだけの kit の recipe（pop と game の写し）が消える |
| D-20 | **状態機械の網羅を全数調査と検査で保証する**（17.8 節） | 「見つけた分だけ」ではなく「全部」を解消する |
| D-21 | **振る舞いを WAI-ARIA APG に合わせる**（17.9 節） | 状態機械の移設で残る不自然な振る舞いを外部の基準で直す |
| D-22 | **出現と退場の動きは engine の presence 一つ**（17.10 節） | 退場の 5 種類の仕組みが一つになる |
| D-23 | **App の状態の置き場所は範囲で二つだけ**（17.11 節） | 状態の置き場所の重複が消える |
| D-24 | **`text_field` を公開 API だけで作れるようにする**（17.12 節） | 戦略 5 章の到達目標（コードエディタを作れる水準） |
| D-25 | **アプリが描いた絵の表示（`viewport`）を中核へ**（17.13 節） | 戦略 5 章の所有範囲に合う |
| D-26 | **地域書式を ICU4X で全 locale に**（17.14 節） | 地域書式の手書きの表が消える |
| D-27 | **kit の Tier 3 の重複も中核へ**（17.15 節） | kit の重複が全部消える |
| D-28 | **モバイルの境界を今作る**（17.16 節） | 文字入力の種類、keyboard の inset、状態の復元 |
| D-29 | **Canvas・Path2D の対応と明示の pointer の capture**（17.17 節） | 戦略 5 章の 2D 描画の対応 |
| D-30 | **soft kit が求めた中核の不足を全部入れる**（17.18 節） | SOFT_UI_WORK_ORDER の未実装の項目が消える |
| D-31 | **非同期の標準部品**（17.19 節） | 戦略 5 章の標準部品の範囲を満たす |
| D-32 | **第 4 回のレビューで API の規則・handler・`use_`・数値・非同期・prelude・`Machine`・設定の面・検査を直す**（18 章） | 15 の規則で公開 API 全体を説明でき、検査と設定の面が減る |

---

## 2. 現状の診断

依頼の 5 つの観点ごとに、調査で確かめた事実を示す（`file:line` は `a3536ef`）。

### 2.1 Rust 未経験の React・SolidJS 開発者が滑らかに書けるか

良い点は多い。部品は一度だけ実行される関数で、`signal`・`memo`・`effect` は Solid と一対一に対応し、`counter` の例は `move ||` だけで書ける（`examples/kivact/counter/src/main.rs`）。問題は「一つ覚えれば全部に通じる」状態になっていないことである。

- **import の場所が覚えられない。** `callback`・`selector`・`constant` は `kivact::view` に、型の `Callback`・`Selector`・`Value` は root にある。`Binding` は prelude にない（`docs/for-kivact-user/en/how-to-guide/custom-components.md:165`）。ガイド自身が「`callback`, `error_boundary`, `try_context` は `kivact::view` から、`effect_on` は `kivact` から」と注記している（`coming-from-react-or-solid.md:92-94`）。
- **同じ名前が別の型を指す。** `ui::List` と `interaction::List`、`ui::Toggle` と `interaction::Toggle`、`kivact::Selector` と soft kit の `Selector`、prelude の `text`/`stack`/`grid`/`icon`/`list` と soft kit の同名の関数。
- **handler の形が部品ごとに違う。** 戻り値の trait が `HandlerResult` と `KeyHandlerResult` の二つあり（`kivact-reactive/src/error.rs:242`、`kivact-core/src/view/input.rs:98`）、`on_exit_complete` だけ `Callback<()>` を取り、`on_image_error` は `&str` を、`ErrorBoundary::on_error` は戻り値なしを取る。`menu`・`command_palette`・`tag_input` は callback を位置引数で取る（`kivact-ui/src/menu.rs:61-65` ほか）。完了の名前は `on_complete`・`on_done`・`on_exit_complete` の三つ。
- **同じ種類の option が定数のときと reactive のときがある。** `focusable(impl IntoValue<bool>)` と `hit_testable(bool)`、`password(impl IntoValue<bool>)` と `wrap(bool)`（`kivact-core/src/view/node.rs:276-293`、`kivact-ui/src/text_input.rs:138,202`）。`kivact-ui/src/lib.rs:106-107` は「bool の option は定数でも reactive でもよい」と書いているが、実際は違う。
- **数値の型が揃っていない。** `Overlay::offset(impl Into<f64>)` と `viewport_margin(f32)`、`Style::opacity(f32)` とほかの `impl Into<f64>`、`split` の `Binding<Vec<f32>>` と `slider` の `Binding<f64>`。`f32` の引数には `4` と書けず `4.0` と書く必要がある。
- **slot の形が三つある。** `impl Fn() -> V`（ui）、`impl FnOnce() -> V`（kit）、`impl IntoView`。soft kit の `content` だけで三種類ある。
- **公開されてはいけないものが見える。** `Theme::install(kivact_reactive::Owner, …)`、`Style::compile`、`Draw::record`、`Transition::as_wire`、`Signal::id() -> NodeId`、`IntoValue::into_prop -> Prop<T>`（`Prop` は facade から名前を付けられない）。`Rc<dyn Fn()>` が `ErrorBoundary::fallback` と `NodeExt::accessible_name` の署名に現れる。

### 2.2 公開 API は分かりやすく統一的か

- **経路が一つでない。** 定数の prop は literal・`constant(x)`・`x.into_value()`・`Prop::constant`・`Reactive::new` の五通りに書ける。semantics の setter は `Node` の固有メソッド・`NodeExt`・`Part` に三重にある（`Part` の約 40 メソッドは「Same as `NodeBuilder::…`」と書かれた複製、`kivact-core/src/view/part.rs:390-742`）。grid は `grid(..)` と `Style::display_grid()`（後者は自分を「the one way」と書いている、`kivact-style/src/sheet.rs:912-914`）。
- **module の分け方が利用者の概念と合わない。** `control` は `view::node` の再公開で、doc は「text inputs を持つ」と書くが持っていない（`kivact-core/src/control.rs:6`）。`primitive` は「Experimental」なのに常に公開され、kit が広く使う。`ui` は「app が `interaction` を import しなくてよいように」interaction の型を再公開するが、`tree` と `splitter` は再公開されていない型を要る（`kivact-ui/src/lib.rs:171-173`）。
- **interaction の handle の名前が二系統。** `DialogApi`・`MenuApi` など 8 個と、`Combobox`・`List`・`Popup` など 21 個。
- **選択の形が三つ。** `select(.., Binding<Option<K>>)`、`List::multiple(Binding<Vec<K>>)`、`tree(.., ListSelection<K>)`（`ListSelection` は `ui` から出ていない）。
- **文脈の読み出しの名前と戻り値が揃っていない。** `current_theme() -> Theme`、`current_direction()`、`locale() -> Value<Locale>`、`mouse_position() -> Value<..>`（setup のみ）、`window_size() -> Size`（tracked snapshot）。

### 2.3 内部は統一的で単純な構造か

- **語彙の重複（D-01）。** 二つの `NodeId`（`kivact-types/src/geometry.rs:7` と `kivact-wire/src/ids.rs:8`）と約 21 か所の手書きの変換。wire の `Scene`/`SceneCmd`（17 variant）と protocol の `Scene`/`DrawCommand`（22 variant）。semantics の性質の集合が三回（core の `SemanticSources`、wire の `SemanticsProperties`（`Option`）、protocol の `SemanticsProperties`（bool））。`Environment`（protocol）と `HostEnv`（wire）と `VirtualShell` の field。pointer は 4 つの形（protocol の `PointerInput`、wire の `PointerEvent`、engine の `PointerSource`、shell の `Contacts::Live`）。file dialog の option は 3 回、embed の option も 3 回、同じ field を手で写している。frame 周期の `16_666_667` は 3 か所、960×720 は 5 か所、char 境界の切り下げは 5 か所に書かれている。
- **状態の重複。** マウスの位置は 6 か所（`Runner.cursor`、`NativeInput.pointer`、shell の `Mouse`、Win32 の `State.mouse`、engine の `Track.at` と `PointerState.mouse`）。modifier は 4 か所。focus は engine の `Focus`、`A11yState.focus`、shell の `AcceptedTree.focus`。pointer の capture は engine（`model/pointer/state.rs:175`）と core（`view/common.rs:227-249`）に二重にあり、`adopt_pointer`／`take_adopted_activation` で往復している。
- **App ごとの状態の置き場所が 4 種類**（runtime の `Extensions`、`app_state` の root 文脈、`Tree`/`ViewState` の中の遅延生成、owner の文脈）。**型の map が 3 種類**（runtime の `Extensions`、theme の `ThemeExtension`、owner の文脈）。**timer の仕組みが 4 種類以上**（task の `Sleep`、motion の frame、engine の paint transition、interaction の場当たりの `spawn(sleep)`）。**同じ `HostWaker` への waker が 2 つ**。**変化の通知が 3 形**（`Trigger`、`Signal<()>`、`Signal<u64>` の counter）。**一回だけの slot が 3 種類**。**退場のアニメーションが 5 種類**。**hover の情報源が `Part` の中だけで 2 つ**。
- **呼び出し順で守る規則。** core の turn は `client/phases.rs:9-21` の順序表と `app_client.rs:271-360` の呼び出しの並びだけで守られている。layout は「push → rebuild_style → topology_changed → take_changed → compute → drain_geometry_changes → reset」の順を doc で求める（`kivact-layout/src/lib.rs:11-23`）。mount の段階の順は `view/mount.rs:57-66` に書かれ、例外として glue の関数を認めている。
- **暗黙の状態機械が全層にある。** 調査で特定したものは中核側で 87 件、下層で 49 件、kit で 33 件（付録 A、計 169 件）。6be52b0 で多くを enum にしたが、enum の周りに bool や `Option` の field が残り、enum と同じ事実を別の field が二重に持つ例（`Animated` の `Phase` と `running: Signal<bool>`、`FrameLoop` の `run` と `rate`、engine の `Clock` と `env.surface_active`）が多い。

### 2.4 長期の保守性

- **一つの性質を足す費用が高い。**（tests/tooling 調査の実測）
  - 支援技術の性質を 1 つ: **5 crate（関係なら 6）、14〜18 file**。`Part` と `NodeExt` に builder を二重に書く。
  - text style の性質を 1 つ（`text_capsize`）: **6 crate、4 層の構造体**（style → wire `TextProps`/`TextMore` → engine の継承 → lower の変換 → protocol `TextStyle` → arvane-text）。
  - OS サービスを 1 つ: **6 crate、約 15 file**。`ShellWindow` の実装 4 つと `arvane::Window` の実装 2 つ、試験の台本の配管 3 層。
  - 環境の field を 1 つ: **5〜6 crate**。
- **feature が層を貫く。** `gpu-interop` は 7 crate、`embed` は 5 crate、試験は名前が層で変わる（`test` → `testing`）。
- **試験が本番と違う道を通る。** 試験の窓は `Driver::event` を通らず、GPU の `Background`／`Step` 状態を一度も通らない（`arvane/src/presenter/gpu.rs:246-253` の `Foreground` は試験専用）。Click の数え方、focus を失ったときの `Cancel`、accessibility の公開の時点が本番と違う。
- **kit の規約が守られていない。** `kivact-ui/src/lib.rs:88-103` の「kit は全標準部品の recipe を持つ」「色は `Color` enum」は 5 kit 中 4 kit で偽。同じ意味の部品が 3〜5 kit で別々に作られている（付録 C）。

### 2.5 戦略を簡単に遂行できるか

| 戦略の項目 | 現状の障害 |
|---|---|
| 5 章「編集の取消し」 | `EditPhase` が interaction（中核の上）にあり、中核の native text editing が報告できない。値を書く部品のうち checkbox・switch・select・tabs・toggle_button・tag_input に `on_edit` がない（`kivact-ui/src/lib.rs:37-39` の約束と矛盾） |
| 5 章「多言語対応」の地域書式 | 置き場所がなく、kit が英語の表・千の区切り・月名・`civil_from_days` を自前で持つ（`kivact-ui-soft/src/timestamp.rs:430-498`、`kivact-ui-basic/src/chart/scale.rs:56-70`、`kivact-ui/src/number_input.rs:149-161`） |
| 5 章「テキスト編集の拡張口」 | 目標（`text_field` を公開 API だけで作り直す）に未到達。`text_input` は engine の native editor leaf の上にあり、`ime_session` を使う部品も試験もない |
| 5 章「アプリが描いた絵の表示」 | `viewport_surface` が creative kit にある（戦略は中核と定める） |
| 5 章「移植の境界」・戦略 P-10 | backend を差し替えられない（`arvane/src/window.rs:352`、`kivact-engine/src/host/launch.rs:31`）。SPI に softbuffer の 0RGB 形式とデスクトップのサービスが焼き込まれ、サービスの有無が `Capabilities` にない |
| 8 章「macOS 向けの分岐を入れない」 | 中核と kit に 2 か所（`kivact-interaction/src/menu/handle.rs:295`、`kivact-ui-soft/src/kbd.rs:14`）。ほかの `target_os` の分岐は backend（`arvane-native`）、GPU 共有（`kivact-gpu-share`、`arvane-render/src/gpu/context.rs`、`kivact-types/src/gpu_interop.rs`）と試験にある |
| 8 章「モバイル」 | 状態の復元の口がない。文字入力の inset がない。suspend の状態が engine で bool（`host/pump.rs:66-71`） |
| 6 章の 5「共通の意味は中核へ」 | 守られていない（付録 C） |

---

## 3. 設計の規則

この章の規則は、本書が個別に決めていないことを決めるための基準でもある。規則には ID を振り、付録・作業手順書・xtask の検査から参照する。規則の ID の接頭辞（B・VOC・S・API・DEP・KIT）は、作業手順書の unit の ID（P・V・A・E・R・C・N・I・U・K・F）と重ならないようにしてある。

### 3.1 crate の境界（B 規則）

- **B-1 crate は境界がはっきりしたときだけ作る。** はっきりした境界とは、次の全部を満たすもの。
  1. 入力と出力を数個の型で言える（例: layout は「layout の性質の木」→「幾何」）。
  2. 相手の内部状態を読まない（`pub(crate)` の field を読むための再公開や、16 file にまたがる 12 種類の item の import をしない）。
  3. その crate を変える理由が一つの言葉で言える。
- **B-2 境界がはっきりしたら、module ではなく crate に分ける。** 依存の向き・可視性・feature・最適化の単位を crate で強制できるからである。
- **B-3 境界がはっきりしない crate は統合する。** 「将来の DAG のため」「最適化の単位のため」だけの crate は作らない（最適化は `[profile.dev.package]` を統合先にかければよい）。
- **B-4 crate の役割は crate root の `//! Role:` と、4 章の表の「境界の根拠」の一文で説明できること。** 説明できない crate は B-3 に従う。
- **B-5 `visible_to` は 4 章の表のとおりにする。** 表にない依存を足すときは本書を先に直す。

### 3.2 語彙（VOC 規則）

- **VOC-1 一つの意味には一つの型。** 同じ意味の struct・enum を二つの crate に定義して field を写す変換を書かない。層によって必要な情報が違うときは、generic の型引数（例: 関係の参照先の型）か、一方が他方を包む形にする。
- **VOC-2 値の語彙は `kivact-types` に置く。** closure・`Rc`・trait object を含むものは語彙ではなく契約で、使う側の crate に置く（D-01）。
- **VOC-3 名前の衝突を作らない。** workspace の中で、意味の違う公開型に同じ名前を付けない（`NodeId` と `SemanticsId`、`DrawList` と `RenderScene` のように分ける）。
- **VOC-4 定数（既定の大きさ、frame 周期、double-click の閾値など）は `kivact_types::defaults` に一度だけ書く。**

### 3.3 状態（S 規則）— 13 章で詳しく定める

> **S-2 は 18.14 節が置き換える。** 状態の field が二つ以上の struct だけを対象にし、単独の bool は許す。

- **S-1** 3 つ以上の状態、または状態によって有効な field が変わるものは、データを持つ variant の enum で表す。
- **S-2** 状態の field に `bool`（`Cell<bool>`、`AtomicBool`、`Signal<bool>`、`Var<bool>` を含む）を使わない。2 状態でも名前の付いた 2 variant の enum にする（例: `Modality::{Pointer, Keyboard}`）。option の束（`*Options`、`*Settings`、`*Policy`、`*Config`、`*Props`、`*Style`）の field は対象外。
- **S-3** ある field の `Some`/`None` が別の field の値に依存してはならない。依存するなら S-1 に従って一つの enum の variant の中に入れる。
- **S-4** 同じ事実を二つの場所に置かない。派生できる値（`running`、`is_open`、`drawable`）は保存せず、状態から計算するか memo にする。
- **S-5** 状態の書き換えは、その状態の型の遷移メソッド（または `Machine::transition`）だけが行う。effect で後から辻褄を合わせない（「閉じたら effect で `hovered` を戻す」をしない）。
- **S-6** 呼び出し順の要求は、型状態（前の段階を消費して次の段階の型を返す）か、ランタイムで検査する段階の enum にする。doc に「X の前に Y を呼ぶ」と書いて済ませない。
- **S-7** 後から一度だけ埋める初期化（`OnceCell`、後で `Some` にする `Option`、`set_waker`、`attach_env`）をしない。依存は構築時に渡す。構築時に渡せないなら、構築を遅らせる（`Launch::{Waiting, Started}` の `Started` で作る）。
- **S-8** counter を event として使わない（`Signal<u64>` を増やして読む側が差分を取る、など）。通知は `Trigger`、識別は id の型、計数は統計だけに使う。
- **S-9** 例外として許すもの（名前の付いた型で包み、`impl` の外から field を触らない）:
  - 一度だけ報告する latch（`ReportOnce`）。
  - 最後に送った値の重複除去の cache（`Sent<T>`）。
  - 世代番号による古い結果の判定（`Generation`）。
  - スレッドをまたぐ取り出し一回の mailbox（`Mailbox<T>`）。
  - 純粋な cache（無効化の規則が一つの関数にあるもの）。
  - 統計の counter。

### 3.4 公開 API（API 規則）— 10 章と 18 章で詳しく定める

公開 API の全体をこの 15 の規則で説明できるようにする（2026-11-04 の第 4 回のレビューで書き直した。番号の意味は初版の API-1〜API-12 と同じで、API-13〜API-15 を足した）。例外はこの節に書いたものだけにする。

- **API-1 正準パス:** 公開 item の module path は一つ。ほかに出してよいのは `kivact::prelude` だけ。型と、それを作る関数と、その builder の型は同じ module に置く。振る舞いと、その data は `interaction`、部品と、その `*State`・`*Variant`・`*Recipes`・`*Tokens` は `ui`。
- **API-2 値:** builder の option は全部 `impl IntoValue<T>` を取る。plain な値を取るのは次の三つだけ: (1) mount のときに一度だけ決まる引数なしの flag（10.6 節の一覧。名前は既定でない方の振る舞い）と `.mode(XMode)`、(2) `*Options` の束、(3) collection に入れる data の record（10.6 節の末尾）。
- **API-3 handler:** `on_` で始まる名前は event の handler だけに使う。payload のある event は `on_<event>(impl Fn(Payload) -> R + 'static)`（`R: HandlerResult`）、payload のない event は `on_<event>(impl Into<Callback<()>>)`（closure と `Callback<()>` の両方を受ける、18.2 節）。状態の変化の event は `on_<名詞>_change`。既定の層は `_default`。位置引数で handler を取らない。event でない関数（計算する関数）は、計算するものの名詞で名付け（`disabled_dates`、`key`、`highlight`）、引数を `&T` で受ける。
- **API-4 数値:** 数値の引数と、利用者が読み返す数値（`Point`・`Size`・`Rect`・`Insets`・`Transform` の field、event の位置、drag の値）は全部 `f64`（引数は `impl Into<f64>`、reactive なら `impl IntoValue<f64>`）。整数は個数・index・level・font の weight だけ。
- **API-5 文字列:** 表示する文字列は `impl IntoText`。識別子（test id、icon の名前、font の family）は `impl Into<String>`。message の key と既定の文は `&'static str`。部品の名前の語は三つだけ: `label`（control の支援技術の名前）、`title`（容器の見える見出し）、`Node::name`（semantics の生の名前）。
- **API-6 slot:** 一度だけ mount する内容は `impl IntoView`、開くたび・項目ごと・状態ごとに作り直す内容は `impl Fn(..) -> V + 'static`。項目ごとの slot は `.item(..)`、容器の slot の名前は `content`・`header`・`footer`・`media`・`actions`・`icon`・`trailing`・`loading`・`fallback`・`empty` の語彙だけ。全部の slot の種類は付録 B.9 の表にする。
- **API-7 `use_`:** `use_` は「今の owner に結び付く」関数にだけ付ける。二種類ある: (1) 振る舞いを付ける `use_<振る舞い>()`（`*Api` を返す。setup の中だけ）、(2) 文脈を読む `use_<値>()`（`use_context`、`use_theme`、`use_tokens`、`use_locale`、`use_direction`。setup と、その owner の下で作られた closure・handler・effect の中で呼べ、reactive な closure の中では tracked な読み出し）。reactive の部品を作る関数（`signal`、`memo`、`machine`、`resource`、`action`、`animated`、`frame_loop`、`node_ref`、`toaster`、`announcer`）と、App 全体の値の読み出し（`platform::window_size()`、`view::modifiers()` など）には `use_` を付けない。`provide_x` には必ず対の `use_x` がある。
- **API-8 署名:** 公開の署名に `Rc`・`Arc`・`Box<dyn ..>`・`&'static Location`・lifetime の引数を出さない（例外は `custom::Constraints<'_>` と `test::Element<'_>` だけ）。
- **API-9 下層の型:** winit・wgpu・vello・accesskit・parley・taffy・kurbo・ICU4X の型と、facade から名前を付けられない型を公開の署名に出さない。
- **API-10 kit の名前:** kit の公開名は `kivact::prelude`・`kivact::ui`・Rust の std prelude の名前と衝突させない。
- **API-11 診断:** 公開の署名の境界に現れる trait（`IntoView`、`IntoValue`、`IntoText`、`IntoReactive`、`IntoBinding`、`IntoSelection`、`IntoViewFn`、`HandlerResult`、`IntoChildren`、`Variant`、`ThemeExtension`、`ThemeColor`、`Animatable`、`Machine`、`ItemKey`、`PropValue`、`AsyncSource`、`Component`）には `#[diagnostic::on_unimplemented]` で、React・SolidJS の開発者が次に何を書けばよいかが分かる message を付ける（例: `IntoValue<MyEnum>` が足りないとき「`constant_value!(MyEnum)` を、その型を定義した crate に一度書く。外の crate の型なら `move || x.clone()` を渡す」）。
- **API-12 誤用:** setup の中だけで呼べる関数の誤用と、文脈がないときの `use_context::<T>()` は、`#[track_caller]` の位置と「何をどこに書けばよいか」を含む message で panic する（G-05）。
- **API-13 位置引数:** 位置引数は、部品がそれなしでは成り立たないものだけを、`label`/`title` → data → binding/selection → 子 の順で取る。ほかは全部 builder のメソッド。constructor の `snake_name(..)` は builder の型 `SnakeName` を返し、builder は `IntoView` を実装する（一つの root を持つ部品は `Component` も、18.9 節）。trigger に付ける overlay は `.attach(trigger)` が `View` を返す。
- **API-14 問い合わせと語:** bool か `Value<bool>` を返す読み出しは `is_*`・`has_*`、ほかの読み出しは名詞だけ（`get_`・`_value` を付けない）、変更は動詞か `set_*`、失敗しうる版は `try_*`。一つの概念に一つの語を使う（open、disabled、busy、selection、active、dismiss、exit、label、title。語の表は 18.12 節）。略語は `a11y`・`ime`・`id` だけ。
- **API-15 接尾辞:** `*Api`（`use_*` が返す handle）、`*State`（slot と recipe が受け取る状態）、`*Variant`（見た目の選択）、`*Recipes`・`*Tokens`（recipe の表、design token の束）、`*Options`（作る前に渡す設定の束）、`*Policy`（theme が運ぶ振る舞いの既定。`InteractionPolicy` だけ）、`*Mode`（構造の選択）、`*Kind`（値の分類）、`*Event`・`*Phase`・`*Reason`（handler の payload）。

### 3.5 依存と機能（DEP 規則）

- **DEP-1** feature の名前は全 crate で同じにする（`test` を下層で `testing` と呼ばない）。
- **DEP-2** feature は、その crate の振る舞いだけを変える（CONTRIBUTING.md の既存の規則）。
- **DEP-3** OS 固有の分岐は backend の crate（`arvane-native`）だけに置く。中核・kit で `cfg!(target_os = ..)` を使う必要があるときは、backend が `Capabilities` または `Environment` の field で事実を渡す（8 章「macOS 向けの分岐を入れない」の違反を解消する）。

---

## 4. 目標のクレート構成

### 4.1 一覧

| crate | 状態 | 責務（Role） | 境界の根拠（B-4） | 依存 | `visible_to` |
|---|---|---|---|---|---|
| `kivact-types` | 拡張（`kivact-wire` の値を統合） | 全層が共有する値の語彙と、std だけの小さな状態の部品（`cell`: S-9 の型） | 値と S-9 の部品だけを持ち、依存がない。全層の型の唯一の定義 | なし | 制限なし（全員） |
| `kivact-runtime` | 新設（`kivact-reactive`＋`kivact-task`＋`kivact-motion` の実行部） | App の UI スレッドの実行: reactive graph、owner、turn、時刻、timer、frame、task、animation、`Machine`、handler の戻り値（`HandlerResult`、`Handled`） | 入力は host の wake と turn ごとの時刻と event、出力は次の締切・frame の要求・終了の状態。host を知らない | `kivact-types`、`async-task`、`tracing` | `kivact-style`、`kivact-i18n`、`kivact-core` |
| `kivact-i18n` | 変更（文脈の設置を core へ移す） | locale、message の表、地域ごとの書式、暦 | 純粋な関数と値。reactive の文脈との接続（`provide_locale`、`message`）は `kivact-core` が持つ。`kivact-runtime` への依存は `constant_value!`（`Locale`・`Weekday`・`Messages`・暦の型を prop にするための impl。orphan rule のため型の crate に書く）だけに使う | `kivact-types`、`kivact-runtime` | `kivact-core` |
| `kivact-editing` | 維持 | 見た目のない文字編集の buffer（`EditState`、`Journal`、`TextSession`） | 入力は編集の命令、出力は文字列と選択と変更の記録。描画も host も知らない | `kivact-types`、`unicode-segmentation` | `kivact-style`、`kivact-engine`、`kivact-core` |
| `kivact-style` | 変更（host 側の公開を止め、`kivact-wire`・`kivact-motion` への依存を `kivact-types` へ） | `Style`・`Theme`・`Draw`・`Path`・色・長さの作成と、計算値への解決 | 入力は利用者の style の記述と theme、出力は `kivact-types` の計算値と `DrawList` | `kivact-types`、`kivact-runtime`（`IntoValue` の impl のため）、`kivact-editing`、`kurbo` | `kivact-core` |
| `kivact-core` | 変更（`test` を分離、起動経路を一本化） | App、view の木、mount、flow、event の配送、文脈、overlay、OS サービスの呼び出し、診断、embed、Offscreen | reactive な記述を engine の契約へ書き込む reconciler | `kivact-types`、`kivact-runtime`、`kivact-i18n`、`kivact-editing`、`kivact-style`、`kivact-engine` | `kivact`、`kivact-interaction`、`kivact-ui`、`kivact-test` |
| `kivact-interaction` | 変更（全 behavior を `Machine` に） | 見た目のない振る舞いと semantics | 入力は caller の binding と node の event、出力は `Part` と状態。描画しない | `kivact-core` | `kivact`、`kivact-ui` |
| `kivact-ui` | 変更（kit から上がる部品を追加） | 標準部品と recipe の表 | 振る舞いは interaction、見た目は theme の token と recipe | `kivact-core`、`kivact-interaction` | `kivact` |
| `kivact-test` | 新設（`kivact-core/src/test` から） | 実 window の E2E（Testing Library） | 入力は App の root と試験の操作、出力は問い合わせと検査の結果。中核の起動経路（`kivact_core::launch`）と engine の試験 host（`kivact_engine::testing`）だけを使う | `kivact-core`、`kivact-engine`（feature `test`） | `kivact` |
| `kivact` | 変更 | facade | 公開 API の唯一の入口 | `kivact-core`、`kivact-interaction`、`kivact-ui`、`kivact-test`（optional）、`arvane-render`・`arvane-native`（optional。feature を有効にするだけの anchor、17.3 節） | 制限なし |
| `kivact-engine` | 拡張（`kivact-text` を統合、`contract` を追加） | 保持する木、layout の統合、入力のモデル、paint、支援技術の木、host の pump | 入力は中核の書き込みと backend の event、出力は render scene と支援技術の更新と中核への event | `kivact-types`、`kivact-editing`、`kivact-layout`、`kivact-lower`、`arvane`、`arvane-native`、`arvane-virtual`（feature `test`） | `kivact-core`、`kivact-test` |
| `kivact-lower` | 変更（公開面を `Lowerer` と `shape` に絞る） | 解決済みの style と `DrawList` を `RenderScene` へ下ろす | 純粋な変換と cache。木も入力も知らない | `kivact-types`、`arvane` | `kivact-engine` |
| `kivact-layout` | 変更（pass を型状態にする） | Taffy による flex・grid の計算 | 入力は layout の性質の木と葉の測定、出力は幾何。Taffy を外に出さない | `kivact-types`、`taffy` | `kivact-engine` |
| `arvane-shell` | 変更（`Backend` trait を追加、サービスを分離） | backend の SPI と、入力の正規化 | OS を知らない。backend が実装する唯一の trait 群。engine は `arvane::spi`（この crate の再公開）から使う | `kivact-types`、`raw-window-handle` | `arvane`、`arvane-native`、`arvane-virtual` |
| `arvane-native` | 変更（port と pipeline に分ける、17.2 節） | winit・Win32・X11 の backend | `Backend` の実装。OS 固有の分岐はここだけ（DEP-3） | `arvane-shell`、`kivact-types`、OS の crate | `kivact-engine`（`native_backend()` の構築だけ）、`arvane-virtual`（mirror だけ）、`kivact`（anchor だけ） |
| `arvane-virtual` | 新設（`arvane::test_display` と `arvane-native::test_display` から） | 仮想の surface・台本の入力・仮想の時計を持つ backend。feature `mirror` で実 window に写す | `Backend` の第二の実装（戦略 P-10）。試験と移植の試作が使う | `arvane-shell`、`kivact-types`、`arvane-native`（feature `mirror`） | `kivact-engine`、`kivact-test`、`arvane`（自身の試験） |
| `arvane-text` | 維持 | parley による shaping | parley を外に出さない | `kivact-types`、`parley` | `arvane` |
| `arvane-render` | 維持 | vello_cpu・vello_hybrid による描画 | vello と wgpu を外に出さない | `kivact-types`、vello 系、wgpu | `arvane`、`kivact`（anchor だけ） |
| `arvane-asset` | 維持 | 画像・SVG・font の読み込み | 入力は bytes、出力は画素と font の情報 | `kivact-types`、image 系 | `arvane` |
| `arvane` | 変更（`Window` trait を消す） | 窓・presenter・描画・text の組み立て。`arvane::spi` で SPI を再公開する | backend を受け取り、render scene を画面に出す | `arvane-shell`、`arvane-text`、`arvane-render`、`arvane-asset`、`kivact-types` | `kivact-engine`、`kivact-lower` |
| `kivact-gpu-share` | 維持 | GPU texture の共有 | UI を知らない import・export の契約 | `kivact-types`、ash、windows、wgpu | 制限なし |
| `kivact-ui-{basic,pop,game,creative,soft}` | 変更（12 章） | 公式 kit | facade だけに依存する | `kivact` | 制限なし |

**消える crate:** `kivact-wire`（値は `kivact-types`、契約は `kivact-engine::contract`）、`kivact-reactive`・`kivact-task`・`kivact-motion`（`kivact-runtime`。純粋な motion の計算は `kivact-types::motion`）、`kivact-text`（`kivact-engine::text_input`）。

### 4.2 依存の図

```text
                         kivact (facade)
                 ┌──────────┼───────────┬───────────────┐
            kivact-ui ─▶ kivact-interaction   kivact-test(opt)
                 └──────────┼───────────────────────┤
                       kivact-core ◀────────────────┘
        ┌──────────┬────────┼─────────┬──────────────┐
  kivact-style  kivact-i18n │   kivact-editing   kivact-engine
        │                   │                ┌──────┼────────┐
        └──▶ kivact-runtime ◀─ kivact-i18n     kivact-lower kivact-layout
                                             │        │
                                           arvane ◀───┘(engine)
                         ┌────────┬──────────┼──────────┐
                  arvane-shell arvane-text arvane-render arvane-asset
                    ▲     ▲
         arvane-native  arvane-virtual
  （全員が kivact-types に依存。kivact-gpu-share は arvane-render の feature `gpu-interop` だけが使う）
```

### 4.3 統合・分割の判断の記録

| 対象 | 判断 | 理由 |
|---|---|---|
| `kivact-wire` | 解体 | 「独立した protocol」と書かれているが、closure（`LeafMeasure`、`EditingData` の `EditBehavior`）と `Rc` を含み、process の境界にならない（engine 調査 3.1）。facade から見える値は `kivact-types` の再公開で、自分の値は core と engine の間の契約だけ。値と契約を分ければ、それぞれの置き場所がはっきりする |
| `kivact-reactive`・`kivact-task`・`kivact-motion` | 統合 | owner の破棄が task・timer・frame を止めるので、三者は同じ owner の木と同じ turn を共有する。現状は core の client が三者の間の順序を呼び出しの並びで保ち（暗黙の状態機械）、waker が二つ、timer が二系統ある。一つの crate にすると turn の状態機械を一か所に書ける。純粋な motion の計算（`Transition::step`、easing、spring）は engine の host transition も使うので `kivact-types::motion` へ |
| `kivact-text` | engine へ統合 | 使うのは engine だけ（8 file で 3 item）。分かれている理由は「将来の crate DAG のため」（`kivact-engine/src/measure/editor.rs:9-10`）で、B-3 に当たる。lower への依存は一関数（`sanitize::color`）だけ |
| `kivact-lower` | 維持し、公開面を絞る | 入力（解決済みの style、`DrawList`、text の run）と出力（`RenderScene`）が値で言え、木と入力を知らない。現在は engine が 19 file で 12 種類の item を使い（B-1 の 2 に違反）、当たり判定と asset の診断も持つ。公開を `Lowerer`・`shape`・`LowerReport` に絞れば境界がはっきりする（8.3 節） |
| `kivact-layout` | 維持 | Taffy を閉じ込める明確な境界。呼び出し順の規則だけを型状態にする（8.4 節） |
| `kivact-editing` | 維持 | 純粋。style・engine・core が同じ値を使う |
| `kivact-i18n` | 維持し、文脈の設置を core へ移す | 地域書式（戦略 5 章、8 章 P1）が増える場所。reactive の文脈との接続（`provide_locale`、`message`）は core へ移す。`constant_value!` の impl は orphan rule のため i18n に残し、そのためだけに `kivact-runtime` へ依存する（`kivact-style` と同じ理由） |
| `kivact-core/src/test` | `kivact-test` へ分割 | 中核の起動経路と engine の試験 host だけを使う葉（core 調査 4）。障害は `AppClient`・`Tree`・`ViewState` を直接読むことで、7.2 節の `launch` と問い合わせの API で解く |
| `kivact-core/src/gpu_interop` | 維持（core の module） | client の dispatch と App ごとの状態に結び付き、分けると core の内部を公開することになる（B-1 の 2）。feature で閉じる |
| `kivact-core` の `view`・`control`・`primitive` | 一つの `view` にまとめる（module の整理） | 互いの内部を再公開し合っていて、境界がない（core 調査 4） |
| `arvane::test_display`、`arvane-native::test_display` | `arvane-virtual` へ分割 | `Backend` の第二の実装として明確な境界を持つ。現状は `arvane::Window` の第二の実装として本番と別の道を通る |
| `arvane` facade と `arvane-shell` | 維持 | shell は第三者の backend が実装する SPI で、facade はそれを組み立てる側 |
| 公式 kit 5 つ | 維持 | 外部の利用者がいる（kit を縮めない）。重複は中核へ上げて消す（12 章） |

### 4.4 crate の中の module の向き

各 crate の module の依存の向きは `dev/xtask/policy/workspace.toml` の `[module_rules.<crate>.uses]` の形（key の module が、値に挙げた module だけを使ってよい）で決める。新しい crate と、構成が大きく変わる crate の向きは次のとおり。ここにない crate は、作業手順書の各 unit が現在の向きを保って書く。

```toml
[module_rules.kivact-runtime.uses]
reactive = []                                   # graph, owner, frame, handles, props, setup, selector, keyed, error, registry …（旧 kivact-reactive）
handler = ["reactive"]                          # HandlerResult, Handled
turn = ["reactive", "handler"]                  # TurnState, Stage, ExitState, TurnClient, TurnInput, TurnItem, TurnReport（型だけ）
time = ["reactive", "turn"]                     # Scheduler: timers, frames, CancelQueue, sleep, next_frame
task = ["reactive", "handler", "turn", "time"]  # spawn, resource, action, blocking（旧 kivact-task）
motion = ["reactive", "handler", "time", "task"]  # Animated, frame_loop（旧 kivact-motion の実行部）
machine = ["reactive", "time", "task"]          # Machine, machine, MachineHandle, MachineCx
host = ["reactive", "handler", "turn", "time", "task", "motion", "machine"]  # Host, ThreadWaker, RuntimeConfig, Runtime::new, Runtime::turn（turn の本体は時刻・task を呼ぶのでここ）

[module_rules.kivact-core.uses]
state = []                                      # kivact::state の再公開と Owner
view = ["state"]                                # elements, node, part, flow, boundary, events, refs, overlays と非公開の木・mount
platform = ["state", "view"]
diagnostics = ["state", "view"]
gpu_interop = ["state", "view"]
draw = ["state", "view"]                        # Offscreen
client = ["state", "view", "platform", "gpu_interop"]
launch = ["state", "view", "platform", "diagnostics", "gpu_interop", "client"]
app = ["state", "view", "launch"]
embed = ["state", "view", "launch", "app"]
custom = ["state", "view", "platform"]          # 再公開だけ

[module_rules.kivact-engine.uses]
contract = []
tree = ["contract"]
mutation = ["contract", "tree"]
frame_log = ["contract"]
measure = ["contract", "tree"]
model = ["contract", "tree", "measure", "mutation"]
text_measure = ["contract", "tree", "measure", "model"]
text_input = ["contract", "tree", "measure"]   # 旧 kivact-text
paint = ["contract", "tree", "measure", "mutation", "model", "text_measure", "text_input"]
a11y = ["contract", "tree", "measure", "mutation", "model", "text_input"]
check = ["contract", "tree", "measure", "mutation", "model", "paint", "a11y"]
host = ["contract", "tree", "measure", "mutation", "model", "paint", "a11y", "check", "text_measure", "text_input", "frame_log"]
offscreen = ["contract", "text_measure"]
api = ["contract", "host", "offscreen", "text_measure", "mutation"]   # core が使う handle の再公開だけ（現在の bridge）
embed = ["contract", "host", "api"]
testing = ["contract", "host", "offscreen", "api"]

[module_rules.arvane-virtual.uses]
clock = []
script = ["clock"]
surface = []
window = ["clock", "script", "surface"]
backend = ["clock", "script", "surface", "window"]
mirror = ["surface", "window"]
```

公開 facade の再公開に限り、`state -> view::boundary`・`state -> view::owner`・`style -> view::boundary` を個別の例外とする。`provide`・`Owner`・`use_app_state`・`provide_theme` は view の所有と文脈を使う部品であり、実装の逆向き依存は許可しない。

kivact-core の `prelude` は消える（prelude は facade の `kivact::prelude` だけ。付録 B.2）。engine の既存の `except` の 4 行（`model::aggregate -> paint::state` など）は、`model` と `paint`・`a11y`・`check` の向きが変わらないので、そのまま残す。

## 5. 語彙の一本化（`kivact-types`）

### 5.1 module の構成

`kivact-types` の root には facade が再公開する値を、`render` には engine と Arvane の間の値を置く（現在の `protocol` を `render` と改名する）。

| module | 内容 | 吸収元 |
|---|---|---|
| `geometry` | `Point`、`Size`、`Rect`、`Insets`、`Edges`、`Transform`（2D affine） | types、wire `values` |
| `color` | `Rgba`（直線 alpha、0..1 の f32） | types、wire `values` |
| `ids` | `NodeId`（木の node）、`RequestId`、`Revision`、`FocusTarget`、`RegionId`、`SemanticsId`（支援技術の node、engine → backend）、`TextRunId`、`ExternalImageId`、`IdMap`/`IdHasher` | wire `ids`・`id_hash`、types の `NodeId`（→ `SemanticsId`） |
| `input` | `Key`、`KeyCode`、`Modifiers`、`MouseButton`、`PointerButtons`、`PointerId`、`PointerKind`、`PointerPhase`、`ButtonPress`、`PointerSample`（下の 5.4 節）、`KeyInput`、`WheelInput`、`ImeInput`、`FileDrop` | types `input`、wire `events` の入力部分、protocol `PointerInput` |
| `semantics` | `Role`、`CheckedState`、`CurrentItem`、`LiveRegion`、`Orientation`、`PopupKind`、`SortDirection`、`A11yAction`、`A11yRequest`、`DismissReason`、`Semantics<R>`（5.3 節）、`SemanticsText`、`SemanticsTextRun`、`TextCaret` | types `semantics`・`semantics_text`、wire `semantics`、protocol `semantics` |
| `layout` | `LayoutStyle`（現 `LayoutProps`）、`Dimension`、`Align`、`Justify`、`FlexDirection`、`Position`、`Overflow`、`ScrollAxes`、grid の型、`OverlayAlign`、`Placement`、`IntrinsicSize`、`AvailableSpace`、`Baseline`、`MeasureRequest`、`LeafSize`、`OverlayProps`、`OverlayFit`、`OverlayAt` | types `layout`、wire `style`・`grid`・`measure` の値 |
| `paint` | `PaintStyle`（現 `PaintProps`）、`Fill`、`Border`、`Ring`、`CornerRadius`、`Gradient`…、`Effects`、`Shadow`、`FilterOp`、`BlendMode`、`ClipShape`、`ImageFit`、`Cursor`、`StateStyles` | types `paint`・`effect`・`cursor`、wire `paint`・`style` |
| `text` | `TextStyle`（宣言。全 field が `Option` = 継承）、`ComputedTextStyle`（解決済み）、`TextAlign`、`TextWrap`、`TextOverflow`、`LineHeight`、`FontFeature`、`TextDecoration`、`TextTransform`、`TextMetrics`（`ShapedText` は `render`） | types `text`、wire `text`、protocol `text`・`shaped` |
| `text_shaping` | `TextShaper`（値だけを入出力する backend 中立の問い合わせ trait。style・engine・core が共有するためここに置く） | wire `measure` の trait |
| `scene` | `DrawList`・`DrawCmd`（作成側の描画の記録。現 wire `Scene`/`SceneCmd`）、`Path`/`PathCmd`、`Stroke`、`LineCap`、`LineJoin`、`FillRule`、`Pixels`（直線 alpha の RGBA8。全層で一つ）、`InlineBox` | wire `scene`・`drawing`、types `shape` |
| `render` | `RenderScene`・`RenderCmd`（現 protocol `Scene`/`DrawCommand`）、`ShapedText`、`ShapedInlineText`、`TextLine`、`CpuFrame`。変換は `geometry::Transform`、画素は `scene::Pixels` を使う（render 側の二つ目の `Transform` と `ImageData` は作らない） | protocol 全体 |
| `motion` | `Transition`、`Easing`、`Spring`、`SpringRun`、`Repeat`、`PlayDirection`、`MotionClock` と純粋な step の関数 | types `motion`、wire `motion` |
| `env` | `Environment`（5.5 節）、`Capabilities`、`ThemeHint`、`PointerSettings`（現 `MouseSettings` 二つを統合）、`LifecycleEvent` | types `platform`、protocol `window`、wire `HostEnv`、arvane-native の `MouseSettings` |
| `platform` | `PlatformRequest`、`PlatformResult`、`FileDialog`（三つを統合）、`Unsupported` | wire `node`（`PlatformRequest`、`FileDialogRequest`）、wire `events`（`PlatformResult`）、arvane-shell `ShellFileDialog` |
| `embed` | `ParentWindow`、`EmbedOptions`（三つを統合） | protocol `window`、engine・arvane・shell の options |
| `gpu` | GPU 共有の値（`GpuAdapter`、`SharedTextures`…） | types `gpu_interop` |
| `check` | `LayoutRule`、`LayoutLimit`、`LayoutFinding`（core の `test/screen/results.rs:189-220` の二つ目を消す） | wire `check` |
| `defaults` | `FRAME_PERIOD_NS`、`DEFAULT_WINDOW_SIZE`、`DOUBLE_CLICK`… | 各所の literal |
| `process` | `DetachedThreads`（process 全体の「切り離したスレッドがある」を数える唯一の static） | 四つの static: `kivact-task/src/detached.rs:4`、`arvane/src/shutdown.rs:9`、`arvane-native/src/detached.rs:3`、`arvane-asset/src/detached.rs:4` |
| `text_utf` | `floor_char_boundary`、`utf16_offset`、`byte_offset_from_utf16` | 5 か所の重複（engine 調査 3.2） |
| `cell` | `ReportOnce`、`ReportOnceSet`、`Sent<T>`、`Generation`、`Mailbox<T>`、`LocalMailbox<T>`、`Registry<T>`（S-9 の部品。付録 A の冒頭） | 各所の `Cell<bool>` の latch、重複除去の cache、Weak の表 |

### 5.2 契約は使う側へ

`kivact-wire` の次のものは語彙ではない（closure・trait・`Rc` を含む、または kivact-editing の型を field に持つ、core と engine の二者だけの取り決め）。`kivact-engine::contract` へ移し、`kivact-core` だけが使う。

- `NodeProperties`、`NodeKind`、`NodeType`、`BoxData`、`EditingData`、`ImeSession`、`RovingFocus`、`SceneLayer`、`NodeSemantics`、`NodeInput`
- `HostEvent`、`HostInfo`、`TurnOutput`、`HostError`
- `LeafMeasure`（struct）、`LeafTextMeasurer`
- `CloseSignal`（embed の「閉じて」を host へ伝える `Arc<AtomicBool>` の包み。8.2 節）
- `Routed<T>`、`Activation`（5.4 節）

`contract` は値と trait だけを持ち、engine のほかの module に依存しない（4.4 節の `contract = []`）。core が使う handle（`Writer`、`Reader`、`Client`、`Boundary`、`HostWaker`、`OffscreenRenderer`（現在の engine の `Offscreen`。core の公開の `draw::Offscreen` と名前を分ける）、`TextMeasurer`、`WriteStats`）は、それを定義する module（`host`、`offscreen`、`text_measure`、`mutation`）に置いたまま、`kivact_engine::api` が再公開する（現在の `bridge.rs` と同じ役割の、再公開だけの module）。`ImeSession` は core の公開の `custom::ImeSession` と名前を分けて `ImeSessionProps` と改名する（VOC-3）。

値の `TextData`、`ImageData`（node の props）、`OverlayProps`、`OverlayFit`、`OverlayAt`、`IntrinsicSize`、`MeasureRequest`、`LeafSize`、`AvailableSpace`、`Baseline`、`FileDialogRequest`、`PlatformRequest`、`PlatformResult` は `kivact-types` へ。`BoxData` は `measure: Option<LeafMeasure>` に closure を持つため contract に置く（2026-11-04、実コードに基づく訂正）。grid の反復は `GridRepeat` とし、motion の `Repeat` と名前を分ける。

`TextShaper` は `kivact-style::draw::Draw` も使っているため、core と engine だけの契約ではない。既存の一つの trait を `kivact-types::text_shaping` に置き、root から再公開する。入出力は共有の text・scene・render の値と std の型だけで、types に実装の状態や closure は置かない。style から engine への依存、trait の複製、engine contract からの互換再公開は作らない（2026-11-04、実コードに基づく訂正）。

`kivact-layout` の trait `LeafMeasure` は `LeafMeasurer` と改名し、struct と名前を分ける（VOC-3。作業手順書 V-1 で行う）。

### 5.3 semantics は一つの型

```rust
// kivact_types::semantics
/// The semantics of one node. `R` is how a relation names its target:
/// `FocusTarget` in core, `NodeId` in the engine, `SemanticsId` in the backend.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Semantics<R> {
    pub role: Option<Role>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub value_text: Option<String>,
    pub value_min: Option<Finite>,     // f64 は Hash を持たないので Finite で包む
    pub value_max: Option<Finite>,
    pub value_now: Option<Finite>,
    pub checked: Option<CheckedState>,
    pub selected: Option<bool>,        // 支援技術へ出す値（状態機械ではない。state-flags の対象外の型）
    pub expanded: Option<bool>,
    pub disabled: bool,
    // … 現在の三つの定義の和集合を、下の規則で一つずつ写す
    pub controls: Vec<R>,
    pub described_by: Vec<R>,
    pub labelled_by: Vec<R>,
}
impl<R> Default for Semantics<R> { /* 手で書く（derive は R: Default を要求するため） */ }
impl<R> Semantics<R> {
    pub fn map_relations<S>(self, f: impl FnMut(R) -> Option<S>) -> Semantics<S>;
}
/// A finite f64 that hashes by bits; NaN and infinities are rejected at construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Finite(f64);
impl Eq for Finite {}
impl Hash for Finite { /* self.0.to_bits() */ }
impl Finite { pub fn new(v: f64) -> Option<Finite>; pub fn get(self) -> f64; }
```

- **field の型の規則:** wire の `SemanticsProperties` にある field は wire の型（多くは `Option<T>` = 「設定していない」）。protocol にだけある field は protocol の型（bool は既定 `false`）。名前は wire を優先する。`f64` は `Finite`。この規則で三つの定義の和集合を作り、field の一覧を作業手順書 V-2 の報告に書く。
- **実コードに基づく訂正（2026-11-04）:** `text` は custom control も宣言できる `SemanticsText` で、geometry の float を含む。既存の `PartialEq` を保ち、集約の `Semantics<R>` に `Eq` は実装しない。`SemanticsText`・`SemanticsTextRun`・`TextCaret` に値による `Hash` を実装し、geometry の `+0.0` と `-0.0` は同じ hash にする（NaN の比較は既存のまま）。`Finite::new` も符号付きゼロを `0.0` に正規化し、`Eq` と bit hash の整合を守る。
- engine の差分の検出は `Hash`・`PartialEq` を使い、`a11y/hash.rs` の field ごとの手書きの hash を消す。
- backend（`arvane-native/src/a11y/conversion.rs`）は `Semantics<SemanticsId>` を AccessKit へ写す。field を足したら、ここに写し方を一つ書く。
- core の側は 7.4 節の「patch」で組み立て、`SemanticSources` を消す。

**semantics の性質を足すときに変える場所:** `kivact-types` の field、`kivact-core` の `Node` の builder 一つ、`arvane-native` の写し方一つ（関係なら engine の `map_relations` の呼び出しは変わらない）。

### 5.4 入力 event は一つの形

backend から core まで、同じ値を運ぶ。各層が足す情報は包む型で足す。

```rust
// kivact_types::input
pub struct PointerSample {
    pub id: PointerId, pub kind: PointerKind, pub phase: PointerPhase,
    pub position: Point,            // window の論理 px
    pub buttons: PointerButtons, pub button: Option<MouseButton>, pub press: Option<ButtonPress>,
    pub pressure: Option<f64>, pub tilt: Option<(f64, f64)>, pub time_ns: u64, pub modifiers: Modifiers,
    pub is_primary: bool,
}
```

- shell は `PointerSample` を作る（`Mouse`・`ClickCounter`・`Contacts` の正規化はここ）。
- engine は `Routed<PointerSample>`（`target: Option<NodeId>`、`region: Option<RegionId>`、local 座標、capture の有無、input）を core へ渡す。window 宛て key の target は `None`、pointer/wheel は `Some`。pressure の未取得と primary の区別を保持する。
- core は `PointerEvent`（公開型、`kivact::view`）として handler へ渡す。`PointerEvent` は `Routed<PointerSample>` を包むだけで、field を写さない。
- key も同じ: shell の `KeyInput` → engine の `Routed<KeyInput>` → core の `KeyEvent`。`KeyEvent.region` の field と `PointerEvent::region()` のメソッドの不揃いはなくし、全部メソッドにする。

### 5.5 環境は一つの型

```rust
// kivact_types::env
pub struct Environment {
    pub scale_factor: f64, pub window_size: Size, pub safe_area: Insets, pub keyboard_inset: Insets,
    pub theme_hint: ThemeHint, pub reduced_motion: bool, pub locale: Option<String>,
    pub fullscreen: bool, pub pointer: PointerSettings, pub fonts: Generation, // font の集合の世代（S-9。counter を event に使わない）
    // surface_active は持たない: engine の HostPhase から派生する（S-4）
}
```

backend → engine → core へ同じ値を送る。core は field ごとの memo に分けて `platform::*` の読み出しにする。`keyboard_inset`（文字入力の inset、戦略 8 章のモバイル）は今回 field だけ足し、`arvane-native` は 0 を返す（Android・iOS の backend が埋める）。

### 5.6 text style は宣言と計算値の二つだけ

- `TextStyle`（宣言）: 全 field が `Option`（`None` = 親から継承）。`Style` の builder が作る。
- `ComputedTextStyle`（計算値）: 全 field が値。`impl ComputedTextStyle { pub fn inherit(parent: &ComputedTextStyle, own: &TextStyle) -> Self }` が継承の規則を一か所に持つ（現在 engine の `measure/inherited_text.rs` にある）。
- `arvane-text` は `ComputedTextStyle` を直接受け取る。`kivact-lower/src/conversion.rs:47-141` の写しと、wire の `TextProps`/`TextMore`、protocol の `TextStyle` は消える。
- 計算値は継承を解決するが、`TextLen`・`LineHeight`・`TextDecoration` の相対長・current color の意味を保ち、利用時に px を解決する。意味としての不在（無制限の行数など）は `Option` を保つ。親の相対 spacing は子の font size で解決する現行挙動を変えない。

**text style の性質を足すときに変える場所:** `kivact-types` の二つの struct と `inherit`、`kivact-style` の builder、`arvane-text` の使い方。

layout の性質（`kivact-types::layout` ＋ `kivact-style` ＋ `kivact-layout` の変換）と paint の性質（`kivact-types::paint` ＋ `kivact-style` ＋ `kivact-lower`）も同じ 3 crate になる。

---

## 6. ランタイム（`kivact-runtime`）

### 6.1 host との境界

```rust
// kivact_runtime::host
/// What the runtime needs from whoever runs it (kivact-core's client).
pub trait Host: 'static {
    /// Ask for `Runtime::turn` soon. The runtime calls it only on the
    /// `TurnState::Quiet → WakeSent` edge.
    fn wake(&self);
}

/// A wake usable from any thread (blocking jobs, GPU frames). It calls the
/// engine's thread-safe waker directly and never touches `TurnState`; extra
/// wakes are merged by the engine (one `Wake` per loop iteration).
#[derive(Clone)]
pub struct ThreadWaker(Arc<dyn Fn() + Send + Sync>);

pub struct RuntimeConfig {
    pub blocking_threads: usize,
    pub reduced_motion: bool,   // 初期値。以後は Runtime::set_reduced_motion
}

impl Runtime {
    /// Builds a complete runtime (S-7). The root owner exists before `build`
    /// runs, so `build` can create the extensions' signals and triggers under
    /// it; nothing is filled in later.
    pub fn new(
        host: Rc<dyn Host>,
        waker: ThreadWaker,
        config: RuntimeConfig,
        build: impl FnOnce(&Runtime) -> Extensions,
    ) -> Rc<Runtime>;
    pub fn turn<C: TurnClient>(&self, client: &mut C, input: TurnInput<C::Event>) -> TurnReport;
    pub fn request_exit(&self);
    pub fn set_reduced_motion(&self, on: bool);
}
```

- 時刻は `TurnInput.now` だけが運ぶ（`Host` は時刻を持たない。同じ事実を二か所に置かない、S-4）。turn の外で時刻を読む口はない（handler・effect・task は turn の中でだけ走る）。`clock_now()` は現在の turn の `now` を返す。
- 現在の `Driver` trait（`kivact-reactive/src/data.rs:411-417`）、core の `HostLink`（`client/link.rs`）、`TaskState::set_waker`、`MotionState::attach_env`、`driver: OnceCell` は消える。
- **ランタイムは host が始まってから作る。** core の `Launch::Waiting(..)` → `Launch::Started(Started { runtime, .. })` の `Started` を作るときに `Runtime::new` を呼ぶ（現在は `AppClient::new` で先に作り、waker と env を後で入れている）。
- **handler の戻り値の型は runtime に置く。** `HandlerResult`（10.4 節）と `Handled` は `kivact-runtime::handler` に定義する（`Animated::on_complete`、`Action::on_complete`、`spawn`、`frame_loop` が使うため）。facade は `kivact::view` から再公開する（正準パスは `kivact::view`。定義の crate と公開の module が違ってよい）。

### 6.2 turn は一つの関数が順序を持つ

```rust
pub trait TurnClient {
    type Event;
    /// Run the handlers of one event (core: routing and handler layers).
    fn dispatch(&mut self, rt: &Runtime, event: Self::Event);
    /// Write the tree's pending changes to the engine; returns the new revision.
    fn commit(&mut self, rt: &Runtime) -> Revision;
}
pub struct TurnInput<E> {
    pub now: u64,
    pub frame: Option<FrameInfo>,
    pub items: Vec<TurnItem<E>>,
}
pub enum TurnItem<E> { Event(E), Acknowledged(Revision) }
pub struct TurnReport {
    pub next_deadline: Option<u64>,
    pub wants_frame: bool,
    pub exit: ExitState,
}
pub enum ExitState { Running, Draining { deadline: u64 }, Exited }
```

`Runtime::turn` は、現在の `AppClient::turn`（`kivact-core/src/client/app_client.rs:271-360`）と `client/phases.rs`・`client/settle.rs` の順序を**動作を変えずに一つの関数へ移したもの**である。順序の表は作らない。順序は `Runtime::turn` の本体の上から下への並び（と、そこにある二つの loop: settle の 1000 round、commit の 100 round）そのもので、`Stage` は「いまどこを実行しているか」の記録である。各段階の中で許されない操作（例: `Stage::Commit` の中の signal の書き込み）を `Stage` で検査し、misuse にする。

```rust
enum TurnState {
    Quiet,                       // turn も要求もない
    WakeSent,                    // host に wake を送った
    Running(Stage),              // turn の実行中
    RunningRequested(Stage),     // 実行中に次の turn が要求された
}
enum Stage { Clock, Reports, Stabilize, Dispatch, Async, Commit, Mounts, UserEffects, Output }
```

`Stage` の値と現在の code の対応（作業手順書 R-2 はこの対応で `Stage` を書く）:

| 現在の code（`app_client.rs` の `turn`） | `Stage` |
|---|---|
| `advance_clock`、境界だけで mount もないときの早い return（`:278-284`） | `Clock` |
| `Phase::NextTurn`（`deliver_later`） | `Reports` |
| `stabilize`（どこで呼ばれても） | `Stabilize` |
| `dispatch_all` の中の各 event の `dispatch` | `Dispatch` |
| `Phase::AfterCommit { before }`、`run_acked`、mounts due（turn の始め、境界ごと、commit の後） | `Mounts` |
| `check_draining`、`settle_async` | `Async` |
| `phases::commit` | `Commit` |
| user effects の round | `UserEffects` |
| `output()` | `Output` |

- `turn_requested: Cell<bool>`（`kivact-reactive/src/data.rs:74`）と `HostLink.in_turn: Cell<bool>`（`client/link.rs:31-33`）の二つの bool は `TurnState` 一つになる。
- `Phase::AfterHandler { settle: Option<&TurnRecorder> }`、`Phase::AfterCommit`、`Phase::NextTurn` は `Stage` に吸収する。
- `ExitPhase`（core の `client/state.rs:19-30`）と、その条件の `TaskState.queues_busy()` はランタイムに移り、`ExitState` として報告する。終了の締切（drain の deadline）の持ち主はランタイムの `ExitState::Draining` だけで、engine の `HostPhase::Exiting` は締切を持たない（8.2 節）。

### 6.3 時間の仕組みは一つ

ランタイムの時間は **task（`async`）と、二つの待ち（`sleep(duration)` と `next_frame()`）だけ** にする。

| 現在 | 再設計後 |
|---|---|
| `kivact-task` の `Sleep` と `TaskState.time` | `Scheduler.timers`（締切の heap） |
| `kivact-motion` の `MotionState.frames`（`requested: Option<u64>`、`last_tick`、`ticks`、`applied_offset_ns`） | `Scheduler.frames`: `FrameClock`（13 章の SM-R23） |
| `FrameLoop`（`LoopCell{run, rate, frames}`） | `frame_loop(f)` = `next_frame().await` を繰り返す task。状態は `Signal<LoopState>` 一つ |
| `Animated`（`Phase` と `running: Signal<bool>` と `completed: Signal<u64>`） | `Signal<AnimPhase>` 一つ。`running()` は memo。完了の通知は `on_complete` の callback 列 |
| interaction の場当たりの `spawn(sleep)`（`delayed.rs:214-220`、`menu/press.rs:50-51`、`hover_corridor.rs:115-120`、`toaster.rs:593-595`） | `MachineCx::after(timer, delay, event)`（6.5 節） |
| 二つの waker（`HostLink.waker` と `TaskState::set_waker`） | 役割で二つに分ける。`Host::wake`（UI スレッド）は `TurnState` の `Quiet → WakeSent` の遷移だけが呼ぶ。`ThreadWaker`（ほかのスレッド）は engine のスレッド安全な waker を直接呼び、`TurnState` に触らない（重複した wake は engine が一つにまとめる） |

`Runtime::turn` の `TurnReport.next_deadline` は `timers` の最小の締切、`wants_frame` は「frame を待つ task があるか」で決まる。

### 6.4 reactive の内部

付録 A の SM-R01〜SM-R12 に従う。要点:

- `Computation { state: Color, running: bool }` と sched の集合への所属を、`CompState { Clean, Stale(Staleness), Queued(Staleness), Running }` 一つにする。queue に key があること ⇔ `Queued` を不変条件にする。
- `Ctx { owner, setup, pure, effect, tracking }` の 5 つの独立した field を `Frame { owner, kind: FrameKind }`、`FrameKind::{Setup, Tracking(Collector), Effect{..}, Handler, Callback, Untracked, Pure}` にする（具体的な variant は現在の `frame.rs:32-95` の組合せから一つずつ作る）。
- **`effect_on` と `Animated` の「初回を飛ばす」** は、bool ではなく graph の新しい node の種類 `Watcher` で実装する。`Watcher` は生成時に source を tracking の frame で評価して依存を取る（`f` は実行しない）。以後、source の再評価の結果が前の値と違うときだけ、`f` を untracked で実行する。closure の source でも、依存は生成時の評価で取れる。
- owner は `OwnerPhase::{Live, Disposing { round }}` を保存し、arena にないことを `Gone` とする。

### 6.5 状態機械の部品 `Machine`

> **18.13 節が置き換える。** この節の `Input`・`Timer` の関連型、`MachineCx::after`・`cancel`、`TimerKey` はない。

interaction・ui・kit・利用者の独自部品が、同じ形で状態機械を書くための小さな部品。公開する（`kivact::state`。prelude には入れない）。

```rust
// kivact_runtime::machine（kivact::state で公開）
pub trait Machine: 'static {
    type State: Clone + PartialEq + 'static;
    type Event: 'static;
    type Effect: 'static;
    /// What the transition reads besides the state: settings, `disabled`,
    /// policy timings, the current rows. A snapshot taken (untracked) when
    /// the event is handled.
    type Input: 'static;
    /// Names the machine's timers; a later `after` with an equal key replaces the earlier one.
    type Timer: Eq + Hash + Clone + 'static;
    /// Pure: the next state (None = unchanged) and the effects to run.
    fn transition(state: &Self::State, event: Self::Event, input: &Self::Input) -> Step<Self::State, Self::Effect>;
}
pub struct Step<S, E> { pub next: Option<S>, pub effects: Vec<E> }
impl<S, E> Step<S, E> {
    pub fn stay() -> Self; pub fn to(next: S) -> Self;
    pub fn with(self, effect: E) -> Self;
}

/// Setup only.
/// - `input` builds the `Input` snapshot; it runs untracked each time an event is handled.
/// - `run` executes the effects after the state is written.
pub fn machine<M: Machine>(
    initial: M::State,
    input: impl Fn() -> M::Input + 'static,
    run: impl Fn(MachineCx<M>, M::Effect) + 'static,
) -> MachineHandle<M>;

pub struct MachineHandle<M: Machine> { /* runtime arena key（state の Signal と effect の runner は arena に置く） */ }
impl<M: Machine> Clone for MachineHandle<M> { .. }   // 手で書く（derive は M: Clone を要求する）
impl<M: Machine> Copy for MachineHandle<M> {}
impl<M: Machine> MachineHandle<M> {
    pub fn send(&self, event: M::Event);             // untracked。turn の中で直列に処理する
    pub fn state(&self) -> Value<M::State>;
    pub fn get(&self) -> M::State;                   // tracked read
    /// Turn changes of an input into events: whenever `source` changes
    /// (not for its initial value), send `to_event(new value)`. Setup only.
    pub fn feed<T: Clone + PartialEq + 'static>(&self, source: impl IntoValue<T>, to_event: impl Fn(T) -> M::Event + 'static);
}
pub struct MachineCx<M: Machine> { /* send, after, cancel */ }
impl<M: Machine> MachineCx<M> {
    pub fn send(&self, event: M::Event);
    /// Send `event` after `delay` unless `cancel(timer)` runs first; a later
    /// `after` with an equal timer replaces the earlier one.
    pub fn after(&self, timer: M::Timer, delay: Duration, event: M::Event);
    pub fn cancel(&self, timer: M::Timer);
}
```

- **入力の扱い（S-5 との関係）:** 遷移が読むだけの入力（timing、settings、行の一覧）は `Input` の snapshot で読む。入力の変化そのもので状態を変える必要があるもの（例: `disabled` になったら `Press::Idle`、行が消えたら highlight を外す）は、`feed` で event にする（`feed(disabled, |d| Event::DisabledChanged(d))`）。これは「入力の変化を event として machine に渡す」ことで、S-5 が禁じる「effect が別の状態を後から直す」ではない。状態の signal を machine の外から書く経路はない。
- `send` の中で `send` が呼ばれたら、queue に積んで現在の遷移の後に処理する（再入しない）。
- timer は `MachineCx::after` だけで作る。owner が破棄されると timer も消える。
- 状態の値は arena の `Signal<State>` に一つだけ持ち、派生する bool は memo にする（S-4）。
- `MachineHandle` を field に持つ interaction の `*Api` も、`Clone`・`Copy` を手で書く（11.1 節）。

### 6.6 非同期の状態

`Resource` と `Action` が別々に持つ `state: Signal<AsyncState<T>>` と `latest: Signal<Option<T>>` の組（`kivact-task/src/resource.rs:16-21`、`action.rs:196-197`）を、`AsyncCell<T>`（`Signal<AsyncSnapshot<T> { state, latest }>` 一つ）にまとめる。`Action` は `ActionPhase::{Open(Worker), Closed { worker: Worker, waiting: VecDeque<Submitted<I>> }, Detached(Drain)}` を持ち、`Detached` は `Runner::Queue|All` からしか作れない型にする（SM-R13）。

### 6.7 拡張（App ごとの状態）

App ごとの状態の置き場所を二つにする。

| 置き場所 | 何を置くか |
|---|---|
| `Runtime` の `Extensions`（型 map） | 中核の subsystem の App ごとの状態（`ViewState`、`gpu_interop::State`、`LifecycleState`）。構築時に全部入れる |
| owner の文脈（`provide`/`use_context`） | 部分木ごとの値（theme、locale、messages、reading direction、control size、利用者の文脈） |

`app_state(api, init)`（root の owner の文脈に遅延で置く、`kivact-core/src/view/owner.rs:147`）は、App 全体の状態を部品が持つための口として残すが、名前を `use_app_state::<T>(init)` にする（API-7）。tooltip の hub と toaster はこれを使う。`Tree`・`ViewState` の中の遅延生成の trigger（`view/tree.rs:373-391`、`view/hover.rs:32-42`）は構築時に作る。

---

## 7. 中核（`kivact-core`）と `kivact-test`

### 7.1 module の構成

```text
kivact-core
├── state      kivact::state の中身（runtime の handle・setup の関数・Machine・Trigger の再公開と、core の Owner）
├── app        App（builder）
├── launch     起動の唯一の経路と AppSettings（7.2 節）。kivact-test だけが使う公開 module
├── client     TurnClient の実装、event の配送（dispatch/*）、engine への commit
├── view       記述と mount の全部
│   ├── elements   node, text, rich_text, span, image, icon, canvas, column, row, stack, grid, scroll, spacer
│   ├── node       Node, NodeBuilder, Modifiers（共通の field の記録）
│   ├── part       Part（7.4 節）
│   ├── flow       show, show_some, dynamic, for_each, for_index, virtual_list, error_boundary
│   ├── boundary   provide_theme/locale/messages/direction と、文脈の設置（i18n・style から移る）
│   ├── events     KeyEvent, PointerEvent, …（HandlerResult・Handled は runtime の再公開）
│   ├── refs       NodeRef
│   ├── overlays   overlay, Overlay
│   ├── focus, hover, input, tree, mount, state, requests …（非公開）
├── platform   OS サービス、環境、lifecycle
├── diagnostics
├── draw       Offscreen（作成の型は kivact-style）
├── embed      （feature embed）
├── gpu_interop（feature gpu-interop）
└── custom     独自部品の作者向けの公開面（10.10 節）
```

- `control` と `primitive` は消える（`view::elements` と `custom` へ）。
- `prelude` は core から消え、facade の `kivact::prelude` だけになる（付録 B.2）。
- `test` は `kivact-test` へ移る。

### 7.2 起動の経路を一つにする

```rust
// kivact_core::launch（kivact-test だけが使う。facade は再公開しない）
pub struct AppSettings {  // App の builder が作り、embed・試験も同じものを使う
    pub title: String, pub window_size: Option<Size>, pub min_window_size: Option<Size>,
    pub theme: Theme, pub locale: Option<Locale>, pub messages: Messages,
    pub fonts: Vec<Asset>, pub blocking_threads: usize, pub gpu_preference: GpuPreference,
}
pub struct Launch { /* enum: Waiting { settings, root, mode } | Started(Started) */ }
impl Launch {
    pub fn new(settings: AppSettings, mode: ClientMode, root: impl FnOnce() -> View + 'static) -> Self;
    /// The engine client, and a handle that reaches the started App afterwards.
    pub fn into_engine_client(self) -> (kivact_engine::api::Client, LaunchHandle);
}
pub struct LaunchHandle { /* Rc の共有。Launch::Started を指す */ }
impl LaunchHandle {
    pub fn inspect(&self) -> Option<Inspect<'_>>;   // Started のときだけ Some
}
pub enum ClientMode { Window, Embedded, Test(TestTiming) }
/// The test clock hook the test host gives the client (today's `Option<Rc<dyn Fn(u64)>>`, `client/app_client.rs:170`).
pub struct TestTiming(pub Rc<dyn Fn(u64)>);
```

- `App::run` は `Launch::new(settings, ClientMode::Window, root).into_engine_client()` を engine の `run(backend, ..)` に渡す。
- `App::embed` は `ClientMode::Embedded` で同じことをする。embed の「閉じて」は engine だけが読む（`CloseSignal` は engine の `EmbeddedHost` が持ち、host は `HostEvent::CloseRequested` を core へ送る。core はそれを受けて `Runtime::request_exit` を呼ぶ）。現在の `AppClient.embed_closing: Option<Arc<AtomicBool>>`（core が毎 event 読む）は消える（SM-C03）。
- `TestTiming` は現在 `into_client(Some(timing))`（`kivact-core/src/test/screen/driver.rs:89-91`）に渡している `Rc<dyn Fn(u64)>` を包む型。`kivact-test` は `LaunchHandle::inspect` で core の読み出しを得る（client を engine に渡した後でも使える）。
- `kivact-test` の `render(root)` は `AppSettings` を `RenderOptions` から作り、`ClientMode::Test` で同じ経路を通る（現在の `Screen::start` は `App` を通らず、`window_size`・`min_window_size`・`threads` を無視し、blocking のスレッドを 2 に固定している。これをなくす）。
- `test::RenderOptions` は `AppSettings` を包み、試験だけの設定（台本の時計、GPU の要否、mirror の有無）を足す。title・theme・locale などを別に持たない。

### 7.3 `kivact-test`

- 中身は現在の `kivact-core/src/test` の公開 API（`render`、`Screen`、`Element`、問い合わせ、`LayoutCheck` など）。
- core の内部（`AppClient`、`Tree`、`ViewState`）を直接読まない。必要な問い合わせは次の二つだけで得る。
  1. engine の試験 host の API（`kivact_engine::testing`、feature `test`: semantics の木、layout、screenshot、台本による入力の注入、embed の `TestParent`）。
  2. `kivact_core::launch::Inspect<'_>`（`LaunchHandle::inspect` で得る）: core だけが答えられる**読み出し**。現在 `kivact-core/src/test` が core の内部から読んでいる値（`ViewState` の 7 か所、`Tree`、`register_app_fonts`、`overlay`、`input` の読み出し）ごとに、読み出しだけのメソッドを一つずつ作る（名前は読む値の名前。書き込みのメソッドは作らない）。作業手順書 C-1 の報告にメソッドの一覧を書く。（core と engine の木の一致の検査は、17.1 節で木が一つになるので作らない。C-5 までの間だけ、今の debug の検査を `Inspect::check_trees` として残し、C-5 で消す。）
- `kivact-core/src/test/screen/*_tests.rs`（10 file、37 試験。real_window、gpu、host_sync、scene_cache、disposal など）は core の内部を試す試験である。公開 API で書き直せるものは `dev/tests/kivact-public-api/tests/e2e/` へ移し、内部の不変条件を試すもの（core と engine の木の一致など）は `kivact-test` の debug の自己検査に入れる。どちらにもできないもの（`real_window_tests.rs:398` の `#[ignore]` の測定）は消す。core の中の試験は `kivact-test` を dev-dependency にできない（kivact-core が二重になる）ので、core の中には残さない。
- 入力の注入は `arvane-virtual` の台本（9.3 節）を通す。`ShellEvent` を host の入口に直接入れない。`Screen::script()` が返すのは `kivact::test::Script`（`kivact-test` の型。engine の試験 host の台本を包み、`arvane-virtual` の型を公開の署名に出さない、API-9）。
- embed の E2E（`TestParent` を使うもの）は native の親 window が要る。embed の試験は native の backend（`kivact_engine::native_backend()`）の `open_child` で、`TestParent`（native の試験用の親 window）の中に App を開く（`arvane-virtual` は使わない。`VirtualBackend::open_child` は `ShellError::Unsupported` を返す）。`kivact::test::native_unavailable() -> Option<&'static str>` を足し、native の window を開けない環境では理由を返して試験を skip する（`gpu_unavailable()` と同じ扱い）。
- `#[cfg(feature = "test")]` は `kivact-core` から消える。

### 7.4 modifier の語彙を一つにする（D-09）

現状は `NodeBuilder`（`view/node.rs:180-436`）、`NodeExt`（`primitive/node_ext.rs:32-487`）、`Node` の固有メソッド（「Same as `NodeExt::…`」）、`Part` の約 40 メソッド（`view/part.rs:390-742`）、`Scroll::role`、`Button::selected/expanded` に同じ modifier が散っている。`NodeExt` の設計意図（app が `checkbox("x", on).checked(false)` を書けないこと）は正しいので保つ。

**再設計後:**

| 型 | 持つメソッド | 使う人 |
|---|---|---|
| `Node`（固有メソッド、by value） | modifier の**全部**: `NodeBuilder` の全部と、既定の層（`base_style`、`on_key_default`、`on_pointer_default`、`on_wheel_default`）、semantics の全部（`role`、`checked`、`selected`、`expanded`、`value_text`、`value_min`、`value_max`、`value_now`、`level`、`controls`、`described_by`、`labelled_by`、現在の `NodeExt` の semantics のメソッドの全部）、`part`、`draw_over`、`at`、`ime_session`、`on_ime`、`semantics_text`、`a11y_action`、`on_mount`、`accessible_name`、現在の `NodeExt` のそのほかのメソッドの全部 | 部品の作者（自分の root の `Node` を作るとき） |
| `trait NodeBuilder` | 呼び出し側に安全な部分だけ。**現在の `NodeBuilder` のメソッドと同じ集合**: `test_id`、`node_ref`、`style`、`name`、`description`、`disabled`、`invalid`、`exit_layout`、`on_exit_complete`、`hidden`、`hit_testable`、`focusable`、`focusable_when_disabled`、`roving_focus`、`on_key`、`on_key_up`、`on_pointer`、`on_wheel`、`on_press`、`on_file`、`draw`、`intrinsic_size`。これに `with_node(self, f: impl FnOnce(Node) -> Node) -> Self` を足す。必須の実装は `fn node_mut(&mut self) -> &mut Node`（今のまま公開。kit と外部の部品が自分の builder に実装する、戦略 P-07） | `Node` と全部品の builder |
| `Part<S>` | `Part::new(state: Value<S>, apply: impl FnOnce(Node) -> Node + 'static)`。modifier を自分では持たず、`Node` の固有メソッドを呼ぶ closure を持つ | interaction の behavior と独自の behavior |

- `NodeExt` は消える。`node().role(..)` は import なしで書け、部品の builder には `role` がないので `checkbox(..).checked(false)` は今までどおりコンパイルできない。`with_node` は明示の抜け道（使えば何でも書ける）として残す。
- 部品の作者が builder の後のメソッドで自分の root を変えるときは `with_node` を使う（`Node` のメソッドは by value なので、`node_mut` からは呼べない。`node_mut` は `NodeBuilder` の既定のメソッドが使う実装の口）。
- **`Part` を当てる時点:** `node.part(p)` は、`Node` の modifier の列（`Modifiers.layers`）のその位置に part を記録するだけで、`apply` はまだ呼ばない。mount のとき（owner の文脈が読める時点）に、列を記録の順に当てる。だから層の規則（「part の後に書いた semantics の modifier が勝つ」「caller の入力の handler が part の既定より先に走る」）は今と同じで、part は mount の時点の文脈（`provide` したもの）を読める（付録 C.1 の `provide((), ..)` の回避策が要らなくなる）。`PartState` は `Part` の状態、`PartMount`（`on_bind`）は `Node::on_mount` の引数になる。

**semantics の組み立て（`SemanticSources` を消す）:** `Node` の semantics のメソッドは、`Modifiers.semantics: Vec<SemanticsPatch>` に `(層, |&mut Semantics<FocusTarget>| ..)` の patch を積む。mount のとき、node ごとに一つの render effect が patch を層の順に当てて `Semantics<FocusTarget>` を作り、前回と違えば engine へ書く。reactive な値を読む patch はその effect を購読させる。これで、semantics の性質を足す変更は `kivact-types` の field と `Node` のメソッド一つと backend の写し方一つになる（5.3 節）。

### 7.5 状態の唯一の持ち主

core と engine が同じ事実を二重に持たないように、事実ごとに持ち主を決める（S-4）。

| 事実 | 持ち主 | もう一方 |
|---|---|---|
| keyboard focus の node | engine（`Focus`） | core は engine の event で受け取り、`NodeRef::focused()` の signal にだけ写す |
| node 単位の pointer capture | engine | core は持たない。`PointerEvent::captured()` で読む |
| node の中の region・part の capture | core（`PartCapture`、SM-C17） | engine は「この pointer をこの node へ送り続ける」だけを知る（`adopt_pointer`）。`take_adopted_activation` の往復は、engine が activation を `Routed` の field として core へ渡す形にしてなくす |
| hover の chain | engine | core の `Part` の hover は engine の hover chain だけを使い、pointer の Enter/Leave の observer による二つ目の hover（`view/part.rs:222-223`）を消す |
| 支援技術の focus | engine の `Focus` から publish のときに派生 | `A11yState.focus`、shell の `AcceptedTree.focus` は保存しない |
| modifier の押下状態 | shell の `ModifierState`（正規化） | `Runner.shift`、X11 の xkb mask、Win32 の `GetKeyState` は shell の正規化へ入力するだけ |
| マウスの位置 | shell の `Mouse` | `Runner.cursor`、`NativeInput.pointer`、`PointerState.mouse` を消す。engine は pointer ごとの `Track` だけ |
| 描画できるか（drawable） | engine の `HostPhase`（8.2 節） | `Frame.clock` と `env.surface_active` は派生 |

### 7.6 i18n と style の純粋化

- `kivact-i18n` の文脈の設置（`install_locale`・`install_messages`・`install_layered_messages`（`kivact-i18n/src/context.rs:40-56`）と `LocaleCtx`/`MessagesCtx`）を core の `view::boundary` へ移す。`message(key, default)` は `kivact::i18n` の公開関数のまま、core が実装する。`kivact-i18n` は `kivact-runtime` への依存を `constant_value!`（`Locale`、`Weekday`、`Messages` と、足す暦の型）のためだけに残す（orphan rule: `IntoValue` は runtime、型は i18n にあるので、impl はどちらかの crate に要る。runtime は i18n を知らないので i18n に書く）。
- `kivact-i18n` に地域書式の置き場所を作る: `Locale::format_number`、`parse_number`、`format_date`、`parse_date`、`format_time`、`month_name`、`weekday_name`、`hour_cycle`、`first_day_of_week`（既存）。`NumberStyle` は小数の桁（`fraction_digits: FractionDigits::{Exact(u8), Range { min: u8, max: u8 }}`）と桁の区切り（`grouping: Grouping::{None, Locale}`）を持つ（`kivact-ui/src/number_input.rs:149-161` の step の精度の書式をこれで表す）。今回の再設計では、kit が自前で持つ表（soft の `timestamp.rs` の英語の表、basic の chart の千の区切り、`number_input` の書式、basic・soft の月名の表）を `kivact-i18n` の en-US の表へ移すところまでを行い、ほかの地域は 17.14 節（ICU4X、unit L-1）で全 locale に広げる。
- `kivact-style` の host 側の口（`Theme::install`、`Theme::of`、`Style::compile`、`Style::merge`、`Draw::record`、`Transition::as_wire`）を facade から見えなくする: crate の中の `pub` は `kivact-core` だけが使う `kivact_style::internal` module にまとめ、facade はこの module を再公開しない。`Theme` の文脈への設置は core の `view::boundary` が行う。
- `kivact-style/src/theme/value.rs:527` の `static THEMED: Mutex<Vec<&'static dyn Themed>>`（縮まない interning の表）は、`Theme` の値の中の `Arc` の表にする（process の global をなくす）。

## 8. エンジン

### 8.1 構成

```text
kivact-engine
├── contract     core との契約（5.2 節）。Writer、Reader、Client、Boundary もここ
├── tree         保持する木
├── mutation
├── measure
├── model        入力・focus・modal・IME・編集・frame・presence の状態
├── text_input   （旧 kivact-text の EditLines）
├── paint        paint list と RenderScene の組み立て（lowering は kivact-lower を呼ぶ）
├── a11y
├── check
├── host         HostPhase と pump（8.2 節）
├── offscreen
├── embed        （feature embed）
└── testing      （feature test）試験 host の公開 API
```

- `bridge`・`native`・`layout` の再公開だけの module（`bridge.rs` 17 行、`native.rs` 7 行、`layout.rs` 2 行）は消える（`contract` に入る）。
- engine は `W: arvane::Window` の generic をやめる（`arvane::Window` trait が消えるため。9.2 節）。

### 8.2 host の状態機械

現在の `Host`（`kivact-engine/src/host/pump.rs:45-81`）は、`phase: Phase{Starting, Running, Exiting}` の外に `suspended: bool`（「phase と独立」）、`embedded_closing: Option<Arc<AtomicBool>>`、`last_frame`、`next_wake`、`soft_keyboard: bool`、`accessibility_noted: bool`、`reduced_motion_override`、`environment`、`adapter` を持ち、`self.closing()` を 14 か所で問い合わせる。

```rust
enum HostPhase {
    Starting,
    Running(Run),
    Exiting,                         // client の ExitState::Exited を待つ（締切は runtime が持つ）
    Closed,
}
struct Run {
    presence: AppPresence,
    last_present: LastPresent,       // 既存の `LastFrame` を改名（model の `last_frame` と名前を分ける）
}
enum AppPresence {
    Foreground(Surface),
    Suspended(Surface),              // surface の状態を保ったまま描かない（resume で Foreground に戻す）
}
enum Surface {
    Drawable { generation: u64, size: Size },
    Unavailable(Unavailable),        // NoSurface | ZeroSize | Lost
}
```

- `drawable()` は `HostPhase` から計算する唯一の関数（`Running(Run { presence: Foreground(Surface::Drawable { .. }), .. })` のときだけ true）。model の `Clock { Running, Stopped }` と、core への event の `surface_active` はこれから派生し、保存しない（現在は `debug_assert_eq!` で一致を確かめているだけ、`model/frame/ticks.rs:82-86`）。
- embed の閉じる要求は `CloseSignal`（`kivact_engine::contract`。`EmbeddedHost::drop` が立てる `AtomicBool` の包み）にする。pump の入口で一度だけ読み、立っていれば `HostEvent::CloseRequested` を core へ送る。core はそれで `Runtime::request_exit` を呼ぶ。engine は client の `ExitState` が `Exited` になったら `HostPhase::Closed` へ進む。14 か所の `closing()` は消える。
- `SurfaceLost` で `set_surface(false)` → `HostFailure` → `set_surface(true)` と状態を往復して event を作る処理（`events.rs:182-195`）をやめ、`Surface::Unavailable(Lost)` → 再生成の遷移で表す。
- `soft_keyboard`、`environment`、`adapter`、毎回の `set_ime_allowed` は `Sent<T>`（S-9）にする。`accessibility_noted` は `ReportOnce`。
- `PlatformRequest::RegisterFont` が effects の queue に入りうるのに `unreachable!`（`pump.rs:175-177`）なのは、型を分けて表せないようにする。

### 8.3 `kivact-lower` の公開面

> **18.17 節が置き換える。** `ArvaneAssets` と `shape` は engine へ移る。

```rust
// kivact_lower
pub struct Lowerer { /* SceneCache、ArvaneAssets（現在の asset の表）、text engine への参照 */ }
impl Lowerer {
    pub fn new(assets: ArvaneAssets) -> Self;
    pub fn lower_draw_list(&mut self, list: &DrawList, cx: &LowerCx<'_>, out: &mut RenderScene) -> LowerReport;
    pub fn lower_box(&mut self, paint: &PaintStyle, geometry: &BoxShape, cx: &LowerCx<'_>, out: &mut RenderScene) -> LowerReport;
    pub fn lower_text(&mut self, text: &ShapedText, style: &ComputedTextStyle, cx: &LowerCx<'_>, out: &mut RenderScene) -> LowerReport;
}
/// What one lowering call needs besides its input; the fields are the values
/// the engine passes to lower's functions today (M-6).
pub struct LowerCx<'a> { /* .. */ }
pub struct LowerReport { pub dropped: u32, pub failures: Vec<String>, pub asset_events: Vec<AssetEvent> }
pub enum AssetEvent { /* 現在 lower が作る asset の HostEvent の variant の中身 */ }
pub mod shape { /* BoxShape, hits, in_clip_path, in_child_clip: 当たり判定が使う幾何 */ }
```

- `Context` の `error: RefCell<Option<String>>`、`dropped: Cell<u32>`、`failures: Cell<u32>`（「non-reentrant」、`kivact-lower/src/context.rs:44-57`）は、呼び出しごとに返す `LowerReport` にする（S-9 の latch をなくす）。
- 公開は `Lowerer`、`LowerCx`、`LowerReport`、`AssetEvent`、`ArvaneAssets`、`shape` だけ。`SceneOut`、`Tint`、`glyphs`、`conversion::spans`、`sanitize` は `pub(crate)` にする。engine が今それらを直接使っている箇所は `Lowerer` のメソッドへ置き換える。
- asset の診断（`ArvaneAssets` の `reported_images`、`reported_families`、`pending: Vec<HostEvent>`）は、`LowerReport.asset_events` として返し、engine が `HostEvent` にする（lower が `HostEvent` を知らないようにする）。

### 8.4 layout の pass を型状態にする

`kivact-layout/src/lib.rs:11-23` の呼び出し順の規則を型にする。

```rust
impl LayoutState {
    pub fn edit(&mut self) -> Edit<'_>;            // push, rebuild_style, topology_changed
}
impl Edit<'_> {
    pub fn finish(self) -> Pending<'_>;            // take_changed をここで行う
}
impl Pending<'_> {
    pub fn compute(self, tree: &impl LayoutTree, measure: &mut impl LeafMeasurer) -> Computed<'_>;
}
impl Computed<'_> {
    pub fn geometry_changes(self) -> GeometryChanges;  // drain
}
```

engine の `Changes` の `needs_layout`・`layout_changed`・`viewport_chain`・`geometry` の組と、`MEASURE` の bit が `dispatch` を越えて残る扱い（`model/changes.rs:212-232`、`:285-287`）は、`LayoutPending::{Clean, Dirty(Reasons), Computed(GeometryChanges)}` 一つにする（SM-E27）。

### 8.5 そのほかの engine の状態

付録 A の SM-E01〜SM-E33 に従う。要点:

- pointer: `Track` を `MouseTrack`・`TouchTrack`・`PenTrack` に分け、variant ごとに取りうる gesture を型で制限する（現在は doc で「Scrollbar は mouse だけ」「Pan は touch だけ」と書くだけ、`model/pointer/state.rs:96-103`）。`Route { capture: Option, press: Option, touch: Option }` は enum にする。文字の選択の drag を `Gesture::SelectText { editor }` として明示する。
- key: `key_up` を「今 focus がある node」ではなく「その key の `key_down` を受けた node」へ送る（`KeyDownTargets`）。これは振る舞いの変更で、E2E を足す。
- IME: 二つの経路（IME session の node と native editor）を `ImeTarget::{None, Native(slot), Session(node)}` に、focus が変わったときに一度だけ決める。`EditorNode.pending_controlled`（変換中だけ意味がある）を `ControlledValue::{InSync, HeldDuringComposition(Rc<str>)}` にする。
- 支援技術: `Changes` の `a11y_structure: bool` などの複製をやめ、`Dirt` に直接積む。`root: Option<NodeId>`（`None` が「未受理」と「支援技術が動いていない」の二つを意味する）を `A11yPublish::{Inactive, AwaitingFirstTree, Published { root }}` にし、毎 pump の `accessibility_active()` の問い合わせを、backend の event（`AccessibilityActivated`/`Deactivated`）にする。
- presence: node ごとの `Presence { Mounted, Leaving }` と、別の集合 `Presences { arriving, leaving, notify_exit }` を、node ごとの `NodePresence::{Arriving, Mounted, Leaving(LeavingRole)}` 一つにし、集合は同じ遷移関数だけが保つ索引にする（名前は host の `AppPresence`、interaction の `DisclosurePresence` と分ける。VOC-3）。

---

## 9. Arvane（backend の注入と SPI）

### 9.1 `Backend` trait

```rust
// arvane_shell（engine は arvane::spi の再公開から使う）
pub trait Backend: 'static {
    /// Supported upper bounds before a window exists; the created ShellWindow
    /// reports the authoritative capabilities after native setup succeeds.
    fn capabilities(&self) -> Capabilities;
    /// Run the event loop until the handler exits (blocks on desktop).
    fn run(self: Box<Self>, options: ShellOptions, handler: Box<dyn ShellHandler + '_>) -> Result<(), ShellError>;
    /// Open a child window inside a foreign parent (embed). Default: `Err(ShellError::Unsupported)`.
    fn open_child(&self, parent: ParentWindow, options: EmbedOptions, hooks: ChildHooks, make: MakeHandler) -> Result<Box<dyn ShellChild>, ShellError>;
    /// Called once when the shell shuts down; `detached` tells whether threads
    /// were left running (`DetachedThreads::any()`), so a backend loaded as a
    /// plug-in module can keep its module loaded (`retain_module`).
    fn shutdown(&self, detached: bool);
}
/// A backend whose loop the caller steps (tests). Same `Driver` and `Presenter` as `run`.
pub trait SteppedBackend: 'static {
    fn capabilities(&self) -> Capabilities;
    fn start(self: Box<Self>, options: ShellOptions, handler: Box<dyn ShellHandler>) -> Box<dyn SteppedLoop>;
}
pub trait SteppedLoop {
    /// Deliver due script items and wakes once; report what the handler asked for.
    fn step(&mut self) -> StepOutcome;
}
pub enum StepOutcome { Idle { next_deadline: Option<u64> }, Exited }
pub trait ShellWindow {
    /* surface・入力・IME・支援技術・cursor・title・size */
    /// Services bound to this fully created window, on its UI thread.
    fn services(&self) -> Rc<dyn Services>;
}
pub trait Services {
    /// One entry point for every OS service (D-05). `reply` runs on the UI thread.
    fn perform(&self, request: PlatformRequest, reply: Box<dyn FnOnce(PlatformResult)>);
}
```

**A-0 の実コードとの照合による補足:** embedded handler は `Rc` を持つため、既存の `MakeHandler = Box<dyn FnOnce() -> Box<dyn ShellHandler> + Send>` で UI thread 内に作る。handler 自体を thread 間で送らない。`run` は backend を消費するので、各実装が session の終了通知を所有する。native の終了 guard は handler/presenter より後に破棄し、成功・起動失敗・unwind の順序を保つ。embedded child は返却 handle の Drop 時でなく実 UI handler の終了時に同じ順序で通知する。既存の意図した leak/panic 境界の module 保持は維持する。軽量な native factory は非 Send な backend 自体を UI factory に capture しない。window 生成前の `capabilities` は上限値で、実際の subclass 設置結果を含む生成後の `ShellWindow::capabilities` が確定値。

**A-2 の所有権の訂正:** service は生成済み window ごとの resource とする。生成前の引数なし `Backend::services` では、親 handle を必要とする dialog や複数の embedded child の resize を識別できず、後埋め window slot が必要になるため採用しない。`ShellWindow::services` は UI thread 内の window 構築で完成した同じ service を返し、request ごとに別の current-window 状態を作らない。`RegisterFont` と `ReducedMotion` は OS request ではないため A-2 で engine の `HostRequest` に分離し、後者の override は Host が一か所で持つ。OS request 完了時は実 window 環境の同期を先に行い、その後で `EffectResult` を core に届ける。reply は UI thread で処理する。

- `arvane::Shell::new` が `arvane_native::Shell::new` を直接作る（`arvane/src/window.rs:352`）のをやめ、`arvane::Shell::new(backend: Box<dyn Backend>)` と `arvane::Shell::start_stepped(backend: Box<dyn SteppedBackend>) -> SteppedShell` にする。どちらも同じ `Driver` で handler を包む。`arvane_native::backend()` が既定の backend を返す。
- engine の `run_native` は `run(backend: Box<dyn arvane::spi::Backend>, client, options)` にする。`kivact-core` の `App::run` は `kivact_engine::native_backend()`（`arvane_native::backend()` を返すだけ。core が arvane-native の名前を知らないため）を渡す。
- `arvane` は arvane-native に依存しなくなる。現在 arvane が arvane-native から使っている `retain_module`・`detached_threads`（`arvane/src/lib.rs:74`、`shutdown.rs:18,25`）は `Backend::shutdown` と `DetachedThreads`（5.1 節）に、`embed::open`（`arvane/src/embed.rs:96`）は `Backend::open_child` に、`TestParent`（`test_display.rs:58`）は engine の `testing` の再公開に移る。
- `ShellWindow` からサービス（clipboard、4 つの dialog、open、reveal）を外し、`Services::perform` 一つにする。`ShellError { unsupported: bool }` は `ShellError::Unsupported` と `ShellError::Failed(String)` にし、サービスの失敗は `PlatformResult::Unsupported`・`PlatformResult::Failed` で返す。
- `Capabilities` は現在の field（`touch`、`pen`、`soft_keyboard`、`safe_area`、`memory_warnings`、`ime_reconversion`、`native_accessibility`、`gpu`）に、次の field を足したものだけにする: `clipboard_text`、`file_dialogs`、`open_path`、`reveal_path`、`fullscreen`、`min_size`（bool、各サービスと機能の有無）、`command_key: CommandKey::{Ctrl, Command}`、`context_menu_on: ContextMenuTrigger::{Press, Release}`。embed の子 window が no-op で済ませているもの（X11 の `set_min_size`・`set_soft_keyboard`・`set_ime_text`、Win32 の `set_min_size`・`set_soft_keyboard`、両方の `set_fullscreen`）は `false` を報告する。足す field の根拠: サービスの有無は 戦略 P-09（未対応は Unsupported として示す）、`command_key`・`context_menu_on` は DEP-3（中核と kit の OS の分岐を事実の field に置き換える）。
- CPU の提示の契約（`present_with(width, height, &mut dyn FnMut(&mut [u32]) -> bool)`、0x00RRGGBB）は、backend が `ShellWindow::cpu_format() -> CpuFormat` で形式を報告し、renderer がその形式で書く形にする（今は `CpuFormat::Xrgb8888` だけを定義する）。softbuffer の形式は `arvane-native` だけが知る。

### 9.2 `arvane::Window` trait を消す

`arvane::Window` trait の実装は `NativeWindow` と `test_display::VirtualWindow` の二つで、それぞれ約 30 の転送メソッドを持つ。`VirtualWindow` が `arvane-virtual` の `ShellWindow` の実装になれば、`arvane` の窓は一つの具体型でよい。

- `arvane::Window`（trait）を消し、`NativeWindow` を `arvane::Window`（struct）と改名する。
- `Driver`、`Presenter` は一つの経路だけを持つ。GPU の `State::Foreground`（試験専用、`presenter/gpu.rs:246-253`）は消える。
- `defers_accessibility`（native は true、試験は false）は消え、どちらも frame の後に公開する。

### 9.3 `arvane-virtual`

```rust
// arvane_virtual
pub struct VirtualBackend { /* clock, script, surfaces, capabilities, services */ }
impl VirtualBackend {
    pub fn new(options: VirtualOptions) -> Self;   // size, scale, capabilities, gpu: GpuMode, mirror: MirrorMode
    pub fn script(&self) -> Script;                 // 試験から入力・時刻・サービスの応答を入れる口（Clone）
}
impl Backend for VirtualBackend { .. }        // run: 仮想の時計を次の締切へ進めながら、handler が終わるまで step を繰り返す
impl SteppedBackend for VirtualBackend { .. } // 試験は step で一段ずつ進める
pub struct Script { /* pointer, key, ime, wheel, file drop, resize, suspend/resume, surface lost, memory warning, advance(ns), respond(kind, f) */ }
```

- 窓の surface は CPU の frame（`CpuFrame`）を持ち、screenshot はそれを読む。
- **GPU の試験は mirror の実 window の surface を使う。** window のない GPU の surface は作れないので、`GpuMode::Required` は mirror の window（feature `mirror` と `MirrorMode::Required`）を要し、本番と同じ `Presenter` の GPU の経路（背景での起動、swapchain）を通る。mirror の window を開けない環境では、GPU の試験は `gpu_unavailable()` が理由を返して skip する（現状と同じ扱い）。
- feature `mirror` は、frame と支援技術の木を `arvane-native` の実 window に写す（現在の `arvane-native/src/test_display.rs` の役割を移す）。OS の入力は無視する（現状どおり）。`kivact-engine` の feature `test` が `arvane-virtual/mirror` を有効にする。
- サービスの応答は request の種類ごとに台本へ登録する（`script.respond(PlatformRequestKind::PickFile, |_| PlatformResult::Files(..))`）。登録がない種類は `Unsupported` を返す。これで、OS サービスを足しても試験の配管を足す必要がない。
- `kivact-test` の環境変数（`ARVANE_TEST_HEADLESS`、`KIVACT_TEST_HEADLESS`、`KIVACT_TEST_GPU`、`KIVACT_TEST_LIVE`、`KIVACT_TEST_HOLD_MS`）は `KIVACT_TEST_MIRROR`（`off`/`auto`/`required`）、`KIVACT_TEST_GPU`、`KIVACT_TEST_HOLD_MS` の三つにする。既定は `auto`（窓が開ければ写す）。
- **現在の「実 window の E2E」との関係:** 現在の試験の window も mirror で、入力は host の入口へ注入している（`arvane-native/src/test_display.rs:1-30`、`kivact-engine/src/host/testing.rs:22-27`）。`arvane-virtual` は同じ性質を保ち、注入の位置を backend の台本に下げて、本番と同じ `Driver`・`Presenter` を通るようにする。`KIVACT_TEST_MIRROR=off` は同じ E2E を写さずに回すだけで、E2E 以外の試験を増やすものではない（16.3 節の Q-06）。
- **native の backend の自動の確認:** `arvane-native` の run loop は E2E が通らなくなるので、`crates/arvane-native/tests/smoke.rs` に native の window を開き、frame を一つ出し、閉じる試験を一つ置く（手元と WSL で `window-lock.sh` を通して回す。native の window を開けない環境では `native_unavailable()` で skip）。

### 9.4 `arvane-native` の整理

- embed の停止の表現（`ChildState`、`HandlerState`、Win32 の `Phase`、X11 の `opening: bool`、engine の `closing`）を、共有の `ChildLifecycle::{Opening, Live, Stopping(Stop), Stopped(Stop), Leaked}` 一つにし、Win32 と X11 が同じ enum を使う（SM-A01）。
- Win32 の再入の guard（`in_callback: Cell<bool>`、`draining: Cell<bool>`）は `Reentry::{Idle, InCallback, Draining}` にする。
- `ShellEvent::Wake` の多重の意味（client の waker、asset の worker、支援技術の action、環境の変化、GPU の起動の段階）は `ShellEvent::Wake(WakeReasons)`（bit の集合）にし、`Driver` が GPU の準備を全部の `Wake` に相乗りさせるのをやめる。
- `cfg!(target_os = "macos")` の分岐は `arvane-native` の中だけに残し、中核・kit の分岐（`kivact-interaction/src/menu/handle.rs:295`、`kivact-ui-soft/src/kbd.rs:14`）は `Capabilities.context_menu_on`（`Press`/`Release`）と `Capabilities.command_key`（`Ctrl`/`Command`）の事実の field にする（DEP-3）。

---

## 10. 公開 API

詳細な item の一覧と改名の表は付録 B。この章は規則と主要な決定を定める。

### 10.1 module の構成（D-07）

| module | 内容 | 主な item |
|---|---|---|
| `kivact`（root） | App と error だけ | `App`、`Result`、`UiError`、`ErrorKind` |
| `kivact::prelude` | 入門のガイドで使う名前の全部（付録 B.2 の固定の一覧） | — |
| `kivact::state` | 状態と効果 | `Signal`、`Value`、`Binding`、`Var`、`Callback`、`Selector`、`Trigger`、`signal`、`memo`、`effect`、`effect_on`、`var`、`binding`、`callback`、`selector`、`trigger`、`untrack`、`on_mount`、`on_cleanup`、`provide`、`use_context`、`try_use_context`、`use_app_state`、`IntoValue`、`IntoText`、`IntoReactive`、`constant_value!`、`Machine`、`machine`、`Step`、`MachineHandle`、`MachineCx` |
| `kivact::view` | 木に置くもの、制御の流れ、event、参照、overlay、semantics の値 | `IntoView`、`View`、`ViewFn`、`IntoChildren`、`Node`、`NodeBuilder`、要素の関数、flow の関数、`NodeRef`、`node_ref`、`overlay`、event の型、`Handled`、`HandlerResult`、`Role` など、`EditPhase`、`use_direction`、`provide_direction`、`modifiers`、`pointer_position` |
| `kivact::style` | style と theme | `Style`、`style`、`Theme`、`ThemeMode`、`ThemeExtension`、`ThemeColor`、`ColorToken`、`Color`、`rgb`…、`Length`、`pct`、`em`、`auto`、`TransitionProp`、`use_theme`、`provide_theme`、`tokens`、`Ext`、`Asset`、`Icon`、`IconSet`、`StandardIcon` |
| `kivact::draw` | 描画 | `Draw`、`Path`、`Paint`、gradient、`Stroke`、`TextParagraph`、`DrawText`（現 `draw::TextStyle`）、`Pixels`、`Offscreen`、`Point`、`Rect`、`Size`、`Transform` |
| `kivact::motion` | アニメーション | `Animated`、`animated`、`Animatable`、`FrameLoop`、`frame_loop`、`FrameInfo`、`FrameRate`、`Easing`、`Spring`、`Repeat` |
| `kivact::task` | 非同期 | `spawn`、`Task`、`resource`、`Resource`、`action`、`Action`、`ActionPolicy`、`AsyncState`、`blocking`、`CancelToken`、`sleep`、`timeout` |
| `kivact::i18n` | 言語と地域 | `Locale`、`Messages`、`message`、`message_with`、`use_locale`、`provide_locale`、`provide_messages`、暦の型、書式の関数 |
| `kivact::platform` | OS と window | 環境の読み出し、サービス、lifecycle、`Capabilities`、`Insets`、`FileDialog` |
| `kivact::diagnostics` | 調査 | `why`、`errors`、`Stats`、`Turn`、`TurnRecord` … |
| `kivact::interaction` | 見た目のない振る舞い | `use_*` と `*Api` |
| `kivact::ui` | 標準部品 | 部品の関数・builder・recipe・look |
| `kivact::custom` | 独自部品の作者向け（10.10 節） | `Part`、測定する葉、文字編集、IME、node の問い合わせ |
| `kivact::test`、`kivact::embed`、`kivact::gpu_interop` | feature | — |

- **API-1 の徹底:** root の `Signal` などの再公開、`ui` による `interaction` の型の再公開、`draw` と `primitive` の両方からの `TextParagraph` などをなくす。正準パスと prelude のほかから出さない。
- `kivact::control`、`kivact::primitive` は消える。

### 10.2 正準パスの決め方

型と、それを作る関数は同じ module に置く（`Callback` と `callback` はどちらも `kivact::state`）。ある型を複数の module の署名が使うときは、意味の持ち主の module に置く。

| 型 | 正準パス | 理由 |
|---|---|---|
| `ListItem`、`MenuEntry`、`Command`、`TreeNode`、`Tone`、`Toast`、`ToastOptions`、`ScalarRange` | `kivact::interaction` | behavior の data。`ui` の署名も使うが再公開しない（prelude に入っているものは prelude から使える） |
| `EditPhase` | `kivact::view` | 戦略 5 章: 中核が `on_edit` を統一する。native text editing（core）も報告するので core に置く |
| `Selection<K>` | `kivact::interaction` | 10.8 節 |
| `ControlSize`、`provide_control_size` | `kivact::ui` | 標準部品の寸法の文脈 |

### 10.3 状態の API

- **定数の prop は literal だけ。** `constant(x)` を消す。利用者の型を prop にするには `constant_value!(MyType)`（`kivact::state`）を一度書く。`Reactive<T>`（owner のない保存された prop）は `kivact::custom` に移し、`ListItem` のような data の型の作者だけが使う。`Prop<T>` と `IntoValue::into_prop` は facade から見えなくする（`IntoValue` の公開メソッドは `into_value` だけ）。
- **文脈:** `context::<T>()` → `use_context::<T>()`、`try_context::<T>()` → `try_use_context::<T>()`。ガイドは `let user: User = use_context();` の形（turbofish なし）を先に教える。
- **ambient の値（API-7）:** setup の中で文脈に結び付く読み出しは `use_theme() -> Value<Theme>`、`use_locale() -> Value<Locale>`、`use_direction() -> Value<ReadingDirection>`。App 全体の値は `use_` を付けず、どこでも呼べる: `view::modifiers() -> Value<Modifiers>`、`view::pointer_position() -> Value<Option<Point>>`、`platform::window_size() -> Value<Size>` など（現在 setup だけの `mouse_position` は、`ViewState` の構築時に signal を作るので、どこでも呼べるようになる）。戻り値は全部 `Value<T>`（snapshot を返す読み出しはなくす）。
- `Trigger` と `trigger()` を公開する（interaction が `Signal<()>` で代用している、`roving.rs:65`）。
- `Signal::id() -> NodeId` を消す（facade から名前を付けられない）。

### 10.4 handler（API-3）

```rust
// kivact::view
/// What a handler returns. `()` lets the event continue (like DOM handlers
/// that do not call stopPropagation); return `Handled::Yes` to consume it.
pub trait HandlerResult: Sealed { #[doc(hidden)] fn into_outcome(self) -> Result<Handled, UiError>; }
impl HandlerResult for () {}                   // = Handled::No
impl HandlerResult for Handled {}
impl HandlerResult for Result<(), UiError> {}  // Ok = Handled::No
impl HandlerResult for Result<Handled, UiError> {}
```

- `KeyHandlerResult` は消える（`HandlerResult` 一つ）。伝播しない event（`on_press` など）では `Handled` の値は無視され、error だけが意味を持つ。
- **名前の語彙**（付録 B.4 に全部品の表）:

| event | 名前 | payload |
|---|---|---|
| 押下（activate） | `on_press` | なし |
| 値の編集の段階 | `on_edit` | `EditPhase` |
| 確定（Enter など） | `on_submit` | なし（text input は値を binding で持つ） |
| 項目の実行（menu の項目、command palette、tree・list の Enter・double click） | `on_activate` | `K` |
| gesture の段階（`DragApi` だけ） | `on_start`、`on_update`、`on_end` | `DragUpdate<T>` |
| 非表示になった（toast） | `on_hide` | なし |
| ファイルを受け取った（`ui::dropzone`） | `on_files` | `Vec<FileInfo>` |
| 閉じる要求 | `on_dismiss` | `DismissReason` |
| 終わった（animation、action、退場） | `on_complete`（`Animated`、`Action`）、`on_exit_complete`（node の退場） | `Action` は `&Result<T, UiError>` ではなく `ActionOutcome<T>`（Clone） |
| 失敗 | `on_error` | `UiError` |
| 変換中の文字の有無 | `on_composition` | `Composition`（`Idle`/`Composing`） |
| preview（選ぶ前の強調） | `on_preview` | `Option<K>` |
| 追加・削除（tag input） | `on_add`、`on_remove` | `String`、`usize` |
| 並べ替え | `on_reorder` | `Reorder<K> { key, to }` |

- **位置引数の callback をなくす:** `menu(label, entries, choose)` → `menu(label, entries).on_activate(choose)`、`command_palette(open, commands, run)` → `.on_activate(run)`、`tag_input(label, tags, on_add, on_remove)` → `.on_add(..).on_remove(..)`。
- 選択は binding が持つ（10.8 節）ので、「選んだ」の event（`on_select`）は作らない。calendar の policy の `on_commit` は日付の実行なので `on_activate(Fn(CalendarDate))` にする。
- 支援技術の action は二つの引数を取るので `on_` を付けない: `on_a11y(action, Fn())` と `on_a11y_event(action, Fn(A11yEvent))` を `a11y_action(action, Fn(A11yEvent) -> R)` 一つにする（action を宣言して支援技術へ提供する、という意味）。
- 層の規則に合わせる: `Overlay::on_dismiss_after`（既存の dismiss の後に足す）は既定の層なので `on_dismiss_default` と改名し、`Node::on_key_default` などと揃える。
- `on_exit_complete(Callback<()>)` → `on_exit_complete(impl IntoAction<M>)`（closure と `Callback<()>` の両方を受ける。17.6 節）。`on_image_error(Fn(&str))` → `on_image_error(Fn(UiError))`。`ErrorBoundary::on_error(Fn(UiError))` と `ToastOptions::on_hide(Fn())` は `-> R` にする。`List::on_leave(Fn(&K) -> Option<K>)` は event ではないので `leave_to(..)` と改名する。
- `Action::on_done(&self, ..)` は builder の `on_complete(self, ..) -> Self` にする（Copy handle）。

### 10.5 prop の値（API-2、API-4、API-5）

- 実行中に変わりうる性質は全部 `impl IntoValue<T>`。`hit_testable`、`focusable_when_disabled`、`TextInput::wrap`、`ImeSession::password` などを `impl IntoValue<bool>` にする（付録 B.5 に全部の表）。構造の option（10.6 節。`Overlay::modal` など）と data の record（10.6 節の末尾。`ListItem::disabled` など）は例外。
- 数値は `impl Into<f64>`・`impl IntoValue<f64>`。`4` と書けるように、`kivact-runtime` に `impl IntoValue<f64> for i32`・`for u32`・`for f32`（定数の prop）を足す（現在は `Length` だけが整数を受ける、`kivact-style/src/length.rs:170`）。`Overlay::viewport_margin(f32)`、`Menu::offset(f32)`、`Tooltip::offset(f32)`、`VirtualList::overscan(f32)`、`Style::opacity(f32)`、`DataColumn::width(f32)`、`split` の `Binding<Vec<f32>>` などを揃える。内部の格納は f32 でよい。
- 範囲は `impl IntoValue<ScalarRange>`。`IntoScalarRange`（独自の sealed trait、`kivact-interaction/src/scalar/range.rs:15`）を消し、`ScalarRange` に `constant_value!` と、`impl IntoValue<ScalarRange> for RangeInclusive<f64>`・`for RangeInclusive<i32>`（`0.0..=1.0` と `0..=100` を書けるように。`kivact-interaction` に置く: `ScalarRange` の crate なので orphan rule を満たす）を付ける。`number_input(.., ScalarRange)` も同じ形にする。
- 文字列は API-5。`span(impl Into<String>)` → `span(impl IntoText)`、`test_id(impl AsRef<str>)` → `test_id(impl Into<String>)`、`Theme::with_icon(impl Into<Cow<..>>)`・`Icon::named(..)` → `impl Into<String>`。

### 10.6 構造の option

mount のときに一度だけ読み、木の形や振る舞いの種類を選ぶ option だけが plain な値を取る。名前で区別がつくように、構造の option は次の一覧に限る（増やすときは本書を直す）。

| option | 部品 | 現在の既定 |
|---|---|---|
| `kind(DialogKind)` | `dialog` | — |
| `keep_mounted()`（引数なし） | `disclosure`、`tabs` | しない（`kivact-ui/src/disclosure.rs:82`、`tabs.rs:65`） |
| `selection(..)` の single/multiple の別 | 一覧系（10.8 節） | — |
| `mode(ComboboxMode)` | `use_combobox`、`ui::combobox`（`Inline`/`Popup(binding)`） | — |
| `mode(CarouselMode)` | `use_carousel`、`ui::carousel`（`Translate`/`Scroll`） | — |
| `row_height(RowHeight)` | `virtual_list` | — |
| `variant(impl Variant)` | 全部品（look は theme が reactive に解決するが、variant の型自体は mount で決まる） | — |
| `retain_focus()`（引数なし） | `use_list` | しない |
| `modal()`（引数なし） | `overlay`、`Node`（Part closure からも呼ぶ。mount 時の modal 範囲） | しない |
| `region_events()`（引数なし） | `Node` の canvas region 入力の構築 | しない |
| `located_at_caller()`（引数なし） | `Node` の診断用構築位置の操作。runtime flag ではなく metadata | constructor の位置 |
| `release_offscreen_focus()`（引数なし） | `virtual_list`（現在の `keep_focused(false)`。既定は focus のある行を残す、`virtual_list.rs:98`） | 残す |
| `multiline()`、`report_text()`（引数なし） | `custom::TextEditing` | しない（`text_editing.rs:152,154`） |
| `expand_all()`（引数なし） | `data_table` の tree の行（初期の展開） | しない（`data_table/builder.rs:108`） |
| `keep_anchor_clear()`、`at_least_anchor_width()`（引数なし） | `overlay`。`Menu::at_least_trigger_width` は `at_least_anchor_width()` と改名して揃える | しない（`overlays.rs:201`） |
| `italic()`（引数なし） | `view::Span`（rich text の span は静的な記述） | しない |
| `essential()`（引数なし） | `motion::Transition`（現在の `decorative(false)`。既定は reduced motion で飛ばす装飾、`kivact-motion/src/transition.rs:23`） | 装飾 |
| `looping()`（引数なし） | `ui::carousel` | しない |
| `multiple()`、`pick_on_press()`（引数なし） | `ui::dropzone` | しない |
| `free_text()`、`tokens()`（引数なし） | `ui::combobox`（任意の文字を値にする、複数の選択を token で見せる） | しない |
| `always_open()`（引数なし） | `ui::disclosure`、`use_disclosure`（soft の「無効なら常に開く」） | しない |
| `announce()`（引数なし） | `ui::empty_state` | しない |
| `side(Edge)` | kit の Drawer（DialogMode::Drawer の構築） | 既存の side |
| `collapsible(SidebarCollapsible)` | basic Sidebar の Offcanvas/Icon/None という構築 mode | Offcanvas |
| `groups(..)`・`pattern(..)` | InputOtp の固定入力 format・validation schema | 既存の format |
| `side(..)` | Sidebar provider の構築 geometry | 既存の side |
| `open_delay(..)`・`close_delay(..)` | HoverCard の構築 timing policy | 既存の timing |
| `delay_ms(..)`・`hide_delay_ms(..)` | TopNavMenu/TopNavMegaMenu の Delays 構築 policy | 既存の timing |
| `indicator(..)` | StepIndicator の slot 構築選択 | 既存の indicator |
| `index(..)` | Skeleton の初期 stagger（Transition.delay の構築） | 既存の index |

custom renderer を持つ `StepIndicator` は slot の構築設定として plain な enum にする。slot の Value 化だけを目的に equality を導入しない。既存 Toast の public field 比較には Callback の app/key identity と ViewFn の shared-factory identity の PartialEq を使い、追加の識別子を持たせない。Calendar の number_of_months/caption/week_start は表示の入力として reactive にし、再構築が必要な場合も navigation/selection binding は外側で保つ。

bool の構造の option は作らず、引数なしのメソッドにする（`keep_mounted(true)` ではなく `keep_mounted()`）。既定が「する」の option は、上の表のとおり意味を反転した名前にした（`release_offscreen_focus`、`essential`）。

**data の record は例外。** `ListItem`、`MenuEntry`、`Command`、`TreeNode`、`DataColumn`、`ToastOptions`、`CalendarPolicy`、`StepperItem` のような、collection に入れる data の値の setter は plain な値（`disabled(bool)` など）を取る。reactive なのは collection の側（`Signal<Vec<ListItem<K>>>`）で、record の変化は新しい collection として渡す。表示する文字列だけは `impl IntoText`（言語の切替に追従するため）。

**option の束は例外。** `RenderOptions`、`AppSettings`、`VirtualOptions`、`ThemeOptions`、`InteractionPolicy`、`UiPolicy` のような、作る前に渡す設定の束の builder（`RenderOptions::reduced_motion()` など）は plain な値を取る。

### 10.7 子と slot（API-6）

- 容器の子: `impl IntoView`（tuple、`Vec`、`Option`、`()`）。
- 子を一つずつ包む部品（`split`、`button_group`、`toolbar` など）: `impl IntoChildren`（`kivact::view`。`kivact-ui/src/children.rs` から移し、kit の `GroupItems`・`GroupMembers`・`InputGroupItems`・`IntoListItems`・`IntoMetadataItems` を消す）。
- slot: 一度だけ mount する内容は `impl IntoView`、開くたび・項目ごとに作る内容は `impl Fn(..) -> V + 'static`。部品の中での保存は `View` と `ViewFn<A>`（`kivact::view`。`Rc<dyn Fn(A) -> View>` を包む Copy でない型、`From<impl Fn(A) -> V>`）。kit の `slot.rs`（5 つの写し）は消える。
- `disclosure(title, open, content)` の位置引数の content は `disclosure(title, open).content(..)` にし、`dialog` と揃える。

### 10.8 二方向の値と選択

- 二方向の値は `impl Into<Binding<T>>`（`Signal<T>`、`Signal::field(..)`、`Signal::item(..)`、`binding(get, set)`）。
- 選択は一つの型にする。

```rust
// kivact::interaction
pub enum Selection<K> { Single(Binding<Option<K>>), Multiple(Binding<Vec<K>>) }
impl<K> From<Signal<Option<K>>> for Selection<K> { .. }
impl<K> From<Signal<Vec<K>>> for Selection<K> { .. }
impl<K> From<Binding<Option<K>>> for Selection<K> { .. }
impl<K> From<Binding<Vec<K>>> for Selection<K> { .. }
```

**位置引数か builder か:** 選択が部品の本質で、選択なしでは意味をなさない部品（`select`、`radio_group`、`tabs`、`combobox`、`toggle_group`、`disclosure_group`）は、選択を位置引数で取る（`combobox(label, options, selection)`）。選択が任意の部品（`list`、`tree`、`data_table`）は builder の `.selection(..)` で取る。

`list(items).selection(selected)`、`tree(nodes).selection(selected)`、`data_table(..).selection(selected)`、`combobox(label, options, selected)`、`toggle_group(label, options, selected)`、`disclosure_group(selected)` が、渡した signal の型で single と multiple を選ぶ。`List::multiple(..)`、`ListSelection`、`tree(.., selection)` の位置引数は消える。`select`・`radio_group` は single だけなので `selected: impl Into<Binding<Option<K>>>` のまま、`tabs` は常に一つ選ばれているので `Binding<K>` のまま。

### 10.9 error と回復

- `ErrorBoundary::fallback(Fn(UiError, Rc<dyn Fn()>))` → `fallback(Fn(UiError, Retry) -> V)`、`Retry` は Copy の handle で `retry()` を持つ。
- `ErrorKind` は root（`kivact::ErrorKind`）に置く。`UiError` の隣で使うからである。
- `raise(kind, message, at)` は `kivact::custom` へ移す。`&'static Location` を引数に取る公開関数（`NodeExt::at`、`Owner::report`、`Owned::get_at`、`NodeRefExt::tracked_node`）は `#[track_caller]` で位置を取る形にし、引数から消す（API-8）。

### 10.10 `kivact::custom`（旧 `primitive`）

独自部品の作者の公開面。「Experimental」の表記はやめ、ほかの module と同じ安定度で扱う（kit と外部部品が同じ API で作る、戦略 P-07）。

- `Part`、`PartState`、`part`
- `Owner`（core の型だけ。`kivact_reactive::Owner` が `Theme::install` の署名に出るのをなくす）
- 測定する葉: `Primitive`、`primitive`、`Constraints<'_>`、`Measurement`、`AvailableSpace`
- 文字編集: `TextEditing`（17.12 節の `line_styles`・`decorations` と、17.16 節の `purpose`）、`Node::draw_with_layout`、`Decoration`、`LineIndex`、`LineRange`（`StyledRange` は `kivact::style`、`InputPurpose` は `kivact::view`）、`TextSession`、`TextChange`、`EditState`、`EditContext`、`EditRequest`、`EditOutcome`、`TextHighlight`、`TextLayout`、`EditLine`、`EditCaret`
- IME: `ImeSession`、`ImeEvent`
- node の問い合わせ: `NodeRefExt`、`NodeToken`、`focused_node`
- 段落: `shape_paragraph`、`shape_inline_paragraph`、`inline_view`
- `Reactive`、`raise`、`require_setup`、`require_effectful`、`clock_now`、`view_is_empty`

### 10.11 API-8 の例外

`kivact::custom::Constraints<'_>`（測定の引数を借りる）と `kivact::test::Element<'_>`（`Screen` を借りる）だけ、公開の署名に lifetime を出してよい。それ以外の `Rc`・`Arc`・`Box<dyn>`（`Theme::get -> Arc<T>`、`Recipe<S> = Arc<dyn Fn ..>`、`TestFonts::Only(Vec<Arc<[u8]>>)`、`Owner::store(name, Rc<T>)`）は、kivact の型で包むか、引数を値にする（付録 B.6）。

診断と標準 Rust Error の境界は明示の例外にする。UiError/ErrorKind、diagnostics の Trace*/NodeInfo/Snapshot の source-location field・constructor は `std::panic::Location` を保ち、UiError::from_boxed は Rust Error interop の入口として保つ。通常の部品や behavior の引数・field に例外を広げない。診断情報を同じ意味の wrapper に複製せず、定義の inline 理由で範囲を明示する。

`__private` namespace と cross-crate 実装用 bridge は利用者向け facade surface から分ける。`#[doc(hidden)]` と `// xtask-allow(internal-contract): <具体的な理由>` の両方がある実装 edge だけで到達性を止める（片方だけでは除外しない）。IntoValue::into_prop/IntoText::into_text、Prop の実装操作、Value/Binding の runtime/location bridge、TextParagraph::from_shaped の backend adapter が対象。通常の public method から runtime Owner 等を漏らさない。NodeBuilder::node_mut のような外部部品用の契約は単なる doc(hidden) だけで除外しない。

---

## 11. 振る舞いの層（`interaction`）と標準部品（`ui`）

### 11.1 behavior の形

> **18.13・18.15 節が置き換える部分がある**（`Machine` の形、`UiPolicy` の削除、`Settings<S>` → `OptionsCell<S>`）。

全 behavior は次の形にする。

```rust
// kivact-interaction の各 behavior（例: popup）
pub(crate) struct PopupMachine;
impl Machine for PopupMachine {
    type State = PopupState;          // 付録 A の SM-I02 の enum
    type Event = PopupEvent;          // 設計書 0.3 節の M-2 で現在の入口から作る
    type Effect = PopupEffect;        // focus の移動、caller の binding への書き込み …
    type Input = PopupInput;          // Settings<PopupSettings> と InteractionPolicy の snapshot
    type Timer = PopupTimer;          // enum PopupTimer { Close, .. }
    fn transition(state: &PopupState, event: PopupEvent, input: &PopupInput) -> Step<PopupState, PopupEffect> { .. }
}

pub struct PopupApi { machine: MachineHandle<PopupMachine>, settings: Settings<PopupSettings>, .. }
impl Clone for PopupApi { .. } impl Copy for PopupApi {}   // 手で書く
pub fn use_popup(open: impl Into<Binding<bool>>) -> PopupApi;
impl PopupApi {
    pub fn trigger(&self) -> Part<TriggerState>;   // 既存の part の名前の規約を保つ
    pub fn content(&self) -> Part<ContentState>;
    pub fn shown(&self) -> Value<bool>;            // 状態から派生（memo）
    // option の builder: self を取り Self を返す（Copy handle）
    pub fn disabled(self, v: impl IntoValue<bool>) -> Self;
}
```

- **handle の名前は全部 `*Api`**（`ComboboxApi`、`ListApi`、`PopupApi`、`TreeApi` …）。`ui` の builder（`List`、`Toggle` …）との衝突がなくなる。`use_switch`・`use_toggle_button` が `Toggle` を、`use_checkbox` が `CheckboxApi` を返す不揃いもなくなる（`SwitchApi`、`ToggleButtonApi`、`CheckboxApi`）。
- **option の保存は一つの形:** `Settings<S>`（`kivact-interaction` の crate private の型）。
  ```rust
  pub(crate) struct Settings<S: Clone + 'static>(Signal<S>);   // Clone・Copy は手で書く
  impl<S: Clone + 'static> Settings<S> {
      pub(crate) fn new(initial: S) -> Self;
      pub(crate) fn update(&self, f: impl FnOnce(&mut S));     // builder の setter が使う（任意の順に呼べる）
      pub(crate) fn get(&self) -> S;                           // untracked（Input の snapshot を作るとき）
      pub(crate) fn value(&self) -> Value<S>;                  // part が mount のときに読む・feed に渡す
  }
  ```
  `Var<Settings>`・`Signal<Settings>`・個別の `Signal<Option<Value<..>>>` の混在をなくす。setting の field が `Value<T>`（reactive な option）のときは、`Settings` の中に `Value<T>` を持ち、変化で状態を変える必要があるものは `MachineHandle::feed` で event にする（6.5 節）。
- **effect で辻褄を合わせない（S-5）:** 「閉じたら `hovered` を戻す effect」（select）、「`opened == None ⇔ closed` を保つ reset の effect」（menu）、「highlight を有効に保つ二つの effect」（combobox）は、状態の enum の variant と遷移の規則に入れて消す。
- **timer は `MachineCx::after` だけ。**
- **gesture は一つの部品:** `DragApi` に `slop(impl Into<f64>)`（slop を越えるまで drag を始めない）、`axis_lock()`（構造の option。slop を越えた最初の向きの軸に固定する）、`velocity()`（構造の option。`DragUpdate.velocity` を埋める）を足し（SOFT_UI_WORK_ORDER の C-09）、`Drag` の状態に `Pending { origin }`（slop を越える前）を足す。離したときの投げ（decay）と snap は既存の公開の関数（`project_release`、`nearest_snap`、`pull_to_snap`、`resist_overshoot`）を使い、`DragApi` には足さない。slop の既定は 0（今と同じく押した瞬間に始まる）。scalar の touch、switch の drag、toast の swipe、select の touch sheet、drawer、bottom sheet、month swipe、carousel はこの上に作り直す。**閾値の値は変えない:** 現在の literal（`menu/press.rs:96-99` の 10.0・13.0 px、`ui/select.rs:258` の 10 px、`ui/toast_swipe.rs:11` の 8.0 px、soft の `month_swipe.rs:86-93` の 6 px、`selector_surface.rs:62-91` の 10 px）は、それぞれ下の `Policy` の field（中核の部品）か kit の名前の付いた定数（kit の部品）の既定値にし、同じ値を使う。OS の `PointerSettings.drag_slop` は新しく足す `ui` の部品の既定にだけ使う。
- **policy は二つの struct に集める:** 現在 theme の型 map に散っている振る舞いと動きの設定と、文脈で渡している `TooltipConfig` を、次の二つの struct（`ThemeExtension`、`Default`、FRU で部分的に上書き）にまとめる。field の型は旧い型そのもの（`ThemeExtension` の impl だけを外す）。

  | struct | field（旧い型） |
  |---|---|
  | `kivact::interaction::InteractionPolicy` | `menu_press: MenuPress`、`menu_press_drift: PressDrift`(新)、`submenu_timing: SubmenuTiming`、`disabled_menu_focus: DisabledMenuFocus`、`tooltip_config: TooltipConfig`、`hover_card_timing: HoverCardTiming`(新) |
  | `kivact::ui::UiPolicy` | `button_busy_motion: ButtonBusyMotion`、`disabled_button_focus: DisabledButtonFocus`、`dialog_center: DialogCenter`、`progress_motion: ProgressMotion`、`slider_touch_motion: SliderTouchMotion`、`switch_drag: SwitchDrag`、`toast_row_motion: ToastRowMotion`、`toast_swipe: ToastSwipe`、`toast_swipe_slop: f64`(新、既定 8.0)、`toggle_label_reservation: ToggleLabelReservation`、`select_sheet_slop: f64`(新、既定 10.0)、`drawer_swipe: DrawerSwipe`(新)、`disclosure_motion: DisclosureMotion`(新) |

  新しい field の型（全部 `Clone`、`Debug`、`PartialEq`、`Default` は下の既定値）:

  ```rust
  pub struct PressDrift { pub context: f64, pub other: f64 }                 // 10.0, 13.0（menu/press.rs:96-99）
  pub struct HoverCardTiming { pub open_delay: Duration, pub close_delay: Duration } // 600 ms, 300 ms（basic の hover_card）
  pub struct DrawerSwipe { pub close_fraction: f64, pub flick_speed: f64, pub flick_distance: f64 } // 0.3, 0.6 px/ms, 4.0 px（basic の drawer.rs:97,99,160）
  pub struct DisclosureMotion { pub animate: bool, pub duration: Duration } // true, 200 ms（現在の ui::Disclosure::animated の既定と時間）
  ```

  field の名前は旧い型の名前の snake_case（新しい field はこの表の名前）。二つの struct の名前は、どちらも `Policy` とすると prelude と glob の import で衝突するので、`InteractionPolicy` と `UiPolicy` にする（VOC-3）。

  この 11 型（ui の theme 拡張は `UiRecipes` を除いて 11）のほかに、見た目の値の `DataTableStickyPaint`・`DataTableRowStatusPaint`（`kivact-ui/src/data_table/sticky_paint.rs:19`、`row_status.rs:56`）は `UiRecipes` の `data_table` の表の field にする。`OverlayViewport` は `Theme` の中核の metric（`overlay_margin`）にする。`UiRecipes` は今のまま theme の拡張。

### 11.2 `on_edit` の網羅

値を書く全部の標準部品と behavior が `on_edit(EditPhase)` を出す。今ないもの: `checkbox`、`switch`、`select`、`tabs`、`toggle_button`、`tag_input` と、対応する `use_checkbox`、`use_switch`、`use_toggle_button`、`use_select`、`use_tabs`。core の native text editing も `EditPhase` を出し、`text_input` が橋渡しする処理を消す。

### 11.3 中核へ上がる部品（D-11）

付録 C の表に従い、二つ以上の kit が持つ意味の部品を `kivact::ui`（見た目は recipe）と `kivact::interaction`（振る舞い）に作る。kit は自分の look の recipe を足し、自分の版を消す。新しい標準部品は、現在の kit の版の機能の和集合を持つ（kit を縮めない）。

---

## 12. 公式 kit

付録 C に kit ごとの作業を書く。規約は次のとおり。`kivact-ui/src/lib.rs:88-103` の kit の契約の文はこの規約に書き換える。

- **KIT-1 theme:** `pub fn theme(mode: ThemeMode) -> Theme` を必ず持つ。option があるときだけ `pub fn theme_with(mode: ThemeMode, options: ThemeOptions) -> Theme` と、`ThemeOptions`（option の束、`Default`）を持つ。`Theme` への拡張 trait（game・creative の `ThemeOptions` trait）は作らない。
- **KIT-2 token と色:** `Tokens`（`ThemeExtension`）と、theme の色の名前の field のない enum `ColorName`（`ThemeColor` を実装）を持つ。現在の kit の `Color` enum は prelude の `kivact::style::Color`（RGBA の型）と名前が衝突するので `ColorName` と改名する（API-10）。soft も `Colors` struct に加えて `ColorName` enum を持つ。token の読み出しは中核の `kivact::style::tokens::<Tokens>()` を使い、kit は `tokens()` を公開しない。色の literal（`rgb(`・`rgba(`・`hsl(`）は `tokens.rs`・`theme.rs`・`color.rs` の外に書かない（xtask が検査する）。
- **KIT-3 recipe と token:** 値だけが違う look は `UiTokens`（17.7 節）で渡し、recipe は形が違うときだけ書く。kit の theme が埋める recipe の表と設定する token を、kit の README の表に列挙する（少なくとも、その kit が以前独自の版を持っていた部品の表と、現在埋めている表）。埋めない表は中核の既定の recipe（theme の token から作る）を使う。`UiRecipes::filled() -> Vec<&'static str>` を中核に足し、`theme_conformance` の E2E が kit ごとに README の列挙と一致することを確かめる。`kivact-ui/src/lib.rs` の「kit は全標準部品の recipe を持つ」という偽の文はこの規約に書き換える。
- **KIT-4 名前:** kit の公開名は API-10 に従う。衝突している名前は付録 C.4 の表で改名する。
- **KIT-5 slot と子:** API-6 と 10.7 節。kit 独自の children の trait と `slot.rs` を消す。
- **KIT-6 variant:** kit の部品の見た目の種類も `variant(impl Variant)` にする（標準部品と同じ）。一つの部品に複数の variant の型を渡す（soft の `Segmented`＋`SegmentedLayout`＋`SegmentedIconOnly`）のは、variant の型の field にまとめる。
- **KIT-7 hook:** kit が `use_*` を持つのは、その kit の分野固有の振る舞いだけ。汎用のもの（soft の `use_carousel`、`use_collapsible`、`use_resizable`、`use_scroll_state`、`use_scroll_overflow`、`use_overflow`）は interaction か core へ移すか消す（付録 C.3）。kit の hook も `Machine` で状態を持つ。
- **KIT-8 message の key:** 標準部品は接頭辞なし（`dialog.close`）。kit の部品は `<kit>.<component>.<name>` の 3 段だけ（component は snake_case）。同じ意味の文は中核へ上がった部品の key を使う（"Loading" の 4 つの key を一つに）。
- **KIT-9 試験:** kit の E2E は一つの test binary（`tests/consumer.rs` と `#[path]` の module）にする（soft の 33 binary を一つに）。kit の試験の helper の重複（`evidence()`、`options()`、`assert_clean`/`clean`）は `kivact::test` の公開 API（`Screen::evidence`、`RenderOptions::with_theme`、`Screen::assert_layout_clean`）にする。
- **KIT-10 catalog:** catalog の font（NotoSansJP が 4 つの catalog で同じ bytes、Inter、GeistMono）は `examples/shared-assets/fonts/` に一つだけ置き、各 catalog は相対 path で `include_bytes!` する。`main.rs` の font の loop は `App::fonts(iter)` で書く。

---

## 13. 暗黙の状態機械の解消

### 13.1 定義

次のどれかに当たるものを「暗黙の状態機械」とする。

1. 状態を、二つ以上の bool・`Option`・counter の組合せで表している（S-1、S-2）。
2. ある field の有効・無効が別の field の値で決まる（S-3）。
3. 同じ事実を二か所に持ち、effect・`debug_assert`・呼び出しの規則で一致させている（S-4、S-5）。
4. 段階を呼び出し順で表している（S-6）。
5. 後から一度だけ埋める field で「始まる前」と「始まった後」を表している（S-7）。
6. counter を event として使っている（S-8）。
7. collection への所属で状態を表し、値が二つの collection に同時に入りうる。値を一方から取り出して他方へ移す（同時に二つに入れない）設計なら、型が状態を表すので暗黙ではない。
8. 時刻と比べて状態を決めているが、その状態に名前がない（typeahead の 500 ms の窓、tooltip の skip-delay の窓）。

### 13.2 解消の型

| 現在の形 | 解消後の形 |
|---|---|
| `a: bool, b: bool, c: Option<X>` | `enum S { A, B { x: X }, .. }`（取りうる組合せだけを variant にする） |
| `Signal<bool>` ×N（part の入力状態など） | `Signal<PartInput>`（struct の中の `Press::{Idle, Pointer(PointerId), Key}` などの enum） |
| `enum Phase` ＋ 同じ事実の `Signal<bool>` | `Signal<Phase>` ＋ memo |
| `Cell<Option<T>>` の後付けの初期化 | 構築時の引数 |
| `Cell<bool>` の edge の検出（`was_composing`、`previously_focused`） | 遷移を起こす event（IME の event、focus の in/out の event）を受け取る |
| `Signal<u64>` の counter の event | `Trigger`、または `MachineHandle::send` |
| 一度だけの報告の `Cell<bool>` | `ReportOnce` |
| 最後に送った値の cache | `Sent<T>` |
| doc の「X の前に Y」 | 型状態か `Stage` の enum |
| `spawn(sleep)` の task を `Option<Task>` で持つ | `MachineCx::after(key, delay, event)` |
| effect で別の signal を後から直す | 遷移関数の中で一度に決める |

### 13.3 一覧

全項目は付録 A にある。ID の接頭辞と件数:

| 接頭辞 | 範囲 | 件数 |
|---|---|---|
| SM-R | `kivact-runtime`（旧 reactive・task・motion） | 25 |
| SM-C | `kivact-core` | 28 |
| SM-I | `kivact-interaction` | 28 |
| SM-U | `kivact-ui` | 6 |
| SM-E | `kivact-engine`、`kivact-lower`、`kivact-layout`、`kivact-editing` | 33 |
| SM-A | Arvane（`arvane`、`arvane-shell`、`arvane-native`、`arvane-render`、`arvane-asset`、`kivact-gpu-share`） | 16 |
| SM-K | 公式 kit | 33 |

付録 A の各項目は、現在の場所・field・取りうる状態・表せてしまう不正な組合せ・解消後の型を持つ。**付録 A にない暗黙の状態機械を作業中に見つけたら、同じ形で解消し、付録 A に追記する**（作業手順書 0.5 節）。13.1 節の定義に当たるものは全部解消する。

### 13.4 機械による検査

14 章の `state-flags` の検査が、S-2 の違反（状態の bool の field）を検出する。S-1・S-3〜S-8 は機械で完全には検出できないので、作業手順書の各 unit の「状態の点検表」で確かめる。

---

## 14. 検査（xtask）

### 14.1 `cargo xtask check` に足す検査

> **検査の一覧は 18.20 節が置き換える。** この節の表のうち `state-flags`・`state-options`・`late-init`・`counter-state`・`memo-purity`・`fn-length`・`anchor-deps`・`policy-coverage` は作らない。

`dev/xtask` に足す（`vendor/xtask-core` は変えない）。すべて error で、移行中の警告扱いは作らない。対象の crate は `dev/xtask/policy/workspace.toml` の各検査の `crates` に書き、作業手順書の unit がその crate を規則に合わせたときに足す（現在の `module_rules` と同じ、列挙した crate だけを検査する方式）。最後の phase で全 crate が入る。

| 検査 | 内容 | 例外の書き方 |
|---|---|---|
| `state-flags` | struct・enum の variant の field の型が `bool`、`Cell<bool>`、`RefCell<bool>`、`AtomicBool`、`Signal<bool>`、`Var<bool>`、`Rc<Cell<bool>>` なら error。**対象外**（状態機械ではないもの）: (1) 名前が `*Options`・`*Settings`・`*Policy`・`*Config`・`*Props`・`*Style`・`*Look`・`*Request` の struct（option の束）、(2) `NodeBuilder` を実装する型と、`*Api`・`*Builder` の builder の struct（mount の前に集める option）、(3) 名前が `*Event`・`*Input`・`*Sample`・`*Update`・`*Result`・`*Report` の struct（event と結果の payload）、(4) policy の `record_types` に列挙した data の record（`ListItem`、`MenuEntry`、`Command`、`TreeNode`、`DataColumn`、`ToastOptions`、`CalendarPolicy`、`StepperItem`）、(5) `kivact-engine::contract` と `kivact-types` の値の module（`semantics`、`env`、`layout`、`paint`、`text`、`scene`、`render`、`input`、`platform`、`embed`、`check`）の型（公開する値で状態ではない。ただし `kivact-types::cell` は対象） | `[[state_flags.exceptions]] item = ".." reason = ".."`（理由は状態でないことの説明に限る） |
| `api-shape` | **facade の公開 module から届く item**（`kivact::*` の再公開をたどって届くもの。crate の中の `pub` で facade から届かないものは対象外）と kit の公開 item について: (a) 引数の型が `f32` なら error（API-4）、(b) `on_` で始まるメソッドが、closure（`impl Fn(..) -> R`）と `impl IntoAction<M>` 以外を取る・二つ以上の引数を取るなら error（API-3）、(c) 公開の署名に `Rc<`・`Arc<`・`Box<dyn`・`&'static Location` が出たら error（API-8、10.11 節の例外を policy に列挙）、(d) 名前が slot（`content`、`header`、`footer`、`media`、`image`、`icon`、`trailing`、`actions`、`item`、`option`、`fallback`、`start`、`center`、`end`）の公開メソッドが `impl FnOnce` を取ったら error（API-6。setup の root を受け取る `App::run` などや、その場で一度だけ呼ぶ `update`・`untrack`・`with` は slot でないので対象外） | `[[api_shape.exceptions]]` |
| `canonical-path` | 同じ item が、正準パスと `prelude` 以外の公開 module から再公開されていたら error（API-1） | なし |
| `name-collision` | kit の公開名が `kivact::prelude`・`kivact::ui`・std prelude の名前と同じなら error（API-10） | なし |
| `kit-literal-color` | kit の `tokens.rs`・`theme.rs`・`color.rs` の外で `rgb(`・`rgba(`・`hsl(` を呼んだら error（KIT-2） | なし |
| `state-options`・`late-init`・`counter-state`・`memo-purity`・`effect-writes` | 17.8 節の表 | `state-flags` と同じ |
| `fn-length` | 関数の本体の code 行が 80 を超えたら error（17.4 節） | 一行ずつの写像の表の `match` だけ、policy に path と理由 |
| `anchor-deps` | facade の anchor の依存（17.3 節）の crate を、facade の code が `use` したら error | なし |
| `policy-coverage` | 次の対応のとおり、各検査の `crates` に crate が入っていなければ error: `state-flags`・`state-options`・`late-init`・`counter-state`・`memo-purity`・`effect-writes`・`fn-length`・`os-branch` は全 library crate、`api-shape`・`canonical-path` は facade から届く crate と kit、`name-collision`・`kit-literal-color` は kit | なし |
| `os-branch` | `arvane-native` の外の `cfg(..)`・`cfg!(..)`・`cfg_attr(..)`（`any`・`all`・`not` の中を含む）に `target_os`、`target_family`、`windows`、`unix` が現れたら error（DEP-3）。**対象外**: `kivact-gpu-share`、`arvane-render/src/gpu/`、`kivact-types/src/gpu_interop.rs`（GPU 共有の API は OS ごと）、`arvane-asset` と `arvane-text` の font の探索、`tests/` の下と `testing`・`test` の module（試験の環境の判定） | `[[os_branch.exceptions]] path = ".." reason = ".."` |

検査の CLI は `cargo xtask check --only <検査> [--crate <crate> [--module <module path>]]` で、一つの検査を一つの crate（と module の部分木）だけに回せるようにする（作業手順書の unit が、自分の触った module だけを確かめるため）。`--crate` を付けたときは、その crate が policy の `crates` に入っていなくても検査する。

既存の検査の更新:

- `visible_to` を 4.1 節の表にする。
- `module_rules` を 4.4 節と各 unit の決定にする。interaction の未登録の module（`announcer`、`calendar`、`content_reveal`、`drag_settle`、`drag_velocity`、`follow_end`、`hover_corridor`、`menubar`、`tone`）を全部登録する。
- `non-leaf-implementation` の glue の例外（`client::dispatch::dispatch`、`view::mount::mount_host_with`、`mount_host`）は、7 章の再構成で不要になったら消す。
- `api_packages`・`options_structs` を新しい crate 名にする。

### 14.2 CI

CI は既定で無効にしてある（作者の方針）。この再設計は `.github/workflows/` を変えない。検査と試験はすべて手元の gate（作業手順書 0.6 節）で回す。
---

## 15. 戦略の遂行性（変更前後の比較）

| 作業 | 現在 | 再設計後 |
|---|---|---|
| 支援技術の性質を 1 つ足す | 5〜6 crate、14〜18 file、builder を 2 重 | 3 crate（`kivact-types` の field、`kivact-core` の `Node` のメソッド、`arvane-native` の写し方） |
| Role を 1 つ足す | 2 crate | 2 crate（変わらない） |
| text style の性質を 1 つ足す | 6 crate、4 層の構造体 | 3 crate（`kivact-types` の宣言・計算値・継承、`kivact-style` の builder、`arvane-text`） |
| layout・paint の性質を 1 つ足す | 3 crate | 3 crate（wire がなくなり、型は一つ） |
| OS サービスを 1 つ足す | 6 crate、約 15 file、`ShellWindow` の実装 4 つ、`arvane::Window` の実装 2 つ、試験の配管 3 層 | 3 crate（`kivact-types` の request・result・capability、`arvane-native` の `perform` の分岐、`kivact-core` の `platform` の関数）。試験は台本の応答を登録するだけ |
| 環境の field を 1 つ足す | 5〜6 crate | 3 crate（`kivact-types` の `Environment`、`arvane-native`、`kivact-core` の読み出し） |
| 第二の backend（モバイル、Web の試作、別の窓の library） | 不可能（fork が要る） | OS の backend は `arvane-native/port/<os>` を一つ足す（入力の pipeline は `arvane-shell` で共有。18.18・18.19 節）。`arvane-virtual` が常に二つ目の実装として SPI を検証する |
| 地域書式（数値・日付・時刻・月名） | 置き場所がなく kit が自前 | `kivact-i18n` の `Locale` の書式の関数に足す。kit は中核の関数を使う |
| `text_field` を公開 API で作り直す | native editor leaf に依存 | `kivact::custom` の文字編集と IME session は今回の再設計で形を揃える。編集する node の制約（子と独自の描画が `Unsupported`）を外し、`kivact::custom` だけで code editor を作れることを E2E で証明する（17.12 節） |
| 新しい behavior を足す | 書き手ごとに bool と effect の組合せ | `Machine` の状態 enum・event・遷移関数を書く。timer・effect・状態の公開の仕方が決まっている |
| 新しい標準部品を足す | kit ごとに別々に作られがち | 規則（10 章）と kit の規約（12 章）で一つの形。xtask が名前の衝突と API の形を検査する |
| 実 window の E2E | 試験は本番と別の道 | 本番と同じ presenter・driver を通る。窓を開けない環境でも同じ E2E が回る |
| モバイルの lifecycle | engine で bool | `HostPhase::Running(Run { presence: Suspended | Foreground(..) })` と `Surface::Unavailable(..)` で明示。文字入力の inset の field を用意 |

---

### 15.1 戦略 10 章の問いによる判定

| 問い（戦略 10 章） | 判定 | 理由 |
|---|---|---|
| 1. 誰のどの仕事が改善するか。実アプリの要求はあるか | 適合 | 公式 kit の利用者（作者の dogfooding を含む）が kit で同じ部品を別々の API で使っている（付録 C）。soft kit の作業で中核の不足 39 件（SOFT_UI_WORK_ORDER の C-01〜C-39）が出た。新しい API は、kit の重複を中核へ上げるもの、戦略 5 章・8 章が中核の担当とした不足、soft kit が求めた不足だけ（17 章） |
| 2. 層は 5 章の所有範囲に合っているか | 適合 | 振る舞いは中核の `interaction`、共通の部品は `ui`（Tier 3 を含む、17.15 節）、OS の事実は backend、地域書式は `kivact-i18n`（ICU4X、17.14 節）、アプリが描いた絵の表示は中核の `viewport`（17.13 節） |
| 3. React・SolidJS 開発者に少ない概念で説明できるか。二つの経路がないか | 適合 | 正準パス一つと prelude（API-1）、handler の型一つ、prop の規則（API-2〜API-6）、選択の型一つ。`Machine` は新しい概念だが prelude に入れず、独自の control を書く人だけが使う |
| 4. G-01〜G-07 を満たすか | 適合 | G-01: owner の破棄で timer・task・frame が止まる（6.3 節）。G-02: turn の状態機械（6.2 節）。G-03: IME の経路を一つに（8.5 節）。G-05: misuse の message（API-12）。G-07: 事実の持ち主（7.5 節） |
| 5. E2E で確かめたか。文言と地域書式を差し替えられるか | 未検証（作業で確かめる） | 振る舞いを変える unit ごとに E2E を足す（作業手順書）。地域書式は ICU4X で全 locale（17.14 節） |
| 6. 性能と保守の全体の費用は釣り合うか | 適合 | 一つの性質を足す費用が 3 crate に下がる（15 章）。build 時間は phase ごとに記録する（16.2 節） |
| 7. backend の型やデスクトップ固有の前提を公開契約に入れていないか | 適合 | `Backend` の注入と `arvane-virtual`（9 章）、OS の分岐を事実の field へ（DEP-3）、`keyboard_inset`（戦略 8 章: モバイルの境界を P3 へ先送りしない） |
| 8. 非目標を暗黙に広げていないか | 適合 | 新しい backend（モバイル・Web・macOS）は作らない。新しい部品は kit の重複を中核へ上げるものだけ（N-04 の「要求のない部品」に当たらない） |

## 16. やらないこと・リスク・要確認事項

### 16.1 この再設計でやらないこと

16 章の初版で「やらないこと」にしていた項目（新しい機能、ほかの地域の書式、`text_field` の作り直し、kit の Tier 3、`viewport_surface` の移動、SOFT_UI_WORK_ORDER の未実装の項目）は、作者の指示（見つけた課題は全て根本的に解決する）で 17 章に入れた。行わないのは 17.20 節の backend（戦略が外したもの）だけである。

### 16.2 リスクと対策

| リスク | 対策 |
|---|---|
| 公開 API の全面の変更で、文書・例・kit の更新が膨らむ | 公開 API の変更は Phase N にまとめ、一つの unit が一つの規則を全 crate に適用する（作業手順書）。文書の doctest（`dev/tests/docs`）がコンパイルで漏れを検出する |
| 状態機械への書き換えで振る舞いが変わる | 状態機械の unit は振る舞いを変えない移設とし、既存の E2E（public-api 289、kit 184、arvane 33）を回帰の網にする。意図して変える振る舞い（`key_up` の宛先、`on_edit` の追加、`Selection`）は個別の unit にして E2E を足す |
| `arvane-virtual` への移行で試験の挙動が変わる | 試験は本番に近づく方向に変わる（click の数え方、focus を失ったときの `Cancel`、支援技術の公開の時点）。差が出た試験は、本番の挙動を正として試験を直す |
| crate の統合で build 時間が変わる | `[profile.dev.package]` を統合先（`kivact-runtime` は debug のまま、`kivact-engine` は opt-level 3）に付け直し、phase ごとに `cargo build` の時間を記録する |
| kit の外部利用者の追従の量 | 公開 API の変更を Phase N〜K に集中させ、kit の README と `docs/for-kivact-ui-user` に、新しい書き方だけを書く（移行の手引きは作らない: CONTRIBUTING.md の方針） |
| 作業量が大きく、途中で止まる | checkpoint ごとに workspace の全 gate を通し、integration branch を push する。phase の途中でも main に入れられる単位にはしない（全部が終わってから、作者が main への PR を立てて squash merge する） |

### 16.3 作者の決定（2026-11-04 に全部決まった）

| ID | 問い | 決定 |
|---|---|---|
| Q-01 | `Machine` の遷移関数（純粋な関数）に、window を開かない単体の試験を例外として認めるか（CONTRIBUTING.md は E2E だけ） | **決定（2026-11-04）**: 認めない。状態機械も E2E で確かめる |
| Q-02 | `kivact::custom` から「Experimental」の表記を外してよいか | **承認済み（2026-11-04）**: 外す |
| Q-05 | interaction の handle の接尾辞を `Api` に揃えてよいか（zag.js の用語。`Handle` も候補） | **承認済み（2026-11-04）**: `Api` |
| Q-07 | ICU4X の compiled data を全 locale 分含めてよいか（binary が数 MB 大きくなる。17.14 節） | **決定（2026-11-04）**: 含める |
| Q-08 | 17.20 節の macOS・モバイル・Web の backend を、この再設計でも行わないままでよいか | **決定（2026-11-04）**: 行わない。実機と環境がなく、確かめる手段がないため |
| Q-06 | `KIVACT_TEST_MIRROR=off`（窓を開かない）で回す E2E を、戦略 5 章の「実 window 上の `kivact::test`（E2E）」の範囲と見なしてよいか（9.3 節: 現在の試験の window も mirror で、入力は注入している） | **承認済み（2026-11-04）**: 見なす。戦略の文は変えない |

---

## 17. 根本解決の追加（2026-11-04 の改訂）

16 章までで「緩和」や「後の作業」として残していたもの（作者の指示: 発見した課題は全て根本的に解決する）を、この章で設計に入れる。作業手順書の対応する unit は各節の末尾に書く。

### 17.1 一つの木（D-13）

**現状:** core の `Tree`（`kivact-core/src/view/tree.rs:281-293`。`TNode` が parent・children を持ち、`kind: None` が fragment）と engine の `NodeTree`（`kivact-engine/src/tree/node_tree.rs`）が、host の node の並びを二重に持つ。試験の driver は debug で二つの木の一致を確かめている。

**解決:** 木の構造は engine の木だけが持つ。

- engine の `NodeKind` に `Fragment` を足す。fragment は自分の箱を持たず、layout・paint・支援技術・当たり判定・focus の移動では、その子が「fragment でない一番近い祖先」の子として順に並ぶ（透明な node）。透明の扱いは `NodeTree::flat_children(id) -> impl Iterator<Item = NodeId>`（topology が変わったときに無効になる親ごとの cache）一つに置き、`layout/adapter.rs` の `impl LayoutTree`、paint の walker、a11y の build、hit test、focus の navigation はこれだけを使う。
- core の flow（`show`、`dynamic`、`for_each`、`virtual_list`、`error_boundary`、`provide*`）は、自分の位置に engine の fragment を一つ作り、その中へ子を入れる。core の側の fragment は消える。
- core は構造を持たない。`CoreNodes(IdMap<CoreNode>)` は core だけが持つ値（owner、`at`、handler、退場の callback、observation、`why` の origin、参照・presence の lifecycle source）を持つ。`ViewNodes` がこの side table と writer/root/id allocator/通知/文字編集 counter/退場 callback を所有する。parent・children・順序の問い合わせ（event の bubbling の `find_up`、挿入の位置）は `Reader::parent(id)`、`Reader::children(id)` で engine に聞く。`Writer::insert` が再配置も一元的に扱い、detach 前の継承値と旧・新親の layout/a11y を更新する。
- 二つの木の一致の検査（`Inspect::check_trees`）は不要になり、消す。SM-C07 の削除の flag も、engine の削除の通知（`Writer::remove_with_exit` が返す id の列）で置き換わる。
- `Style::display(Display::Contents)`（SOFT_UI_WORK_ORDER の C-38）は、box を layout・paint・当たり判定で fragment と同じに扱う（支援技術では自分の semantics を出さず、子を親へ上げる）。同じ `flat_children` を使う。
- **unit:** E-8（engine）、C-5（core）。

### 17.2 arvane-native の入力の一本化（D-14）

> **18.18 節が置き換える部分がある。** OS を知らない pipeline は `arvane-shell` に置く。

**現状:** winit・Win32 の埋め込み・X11 の埋め込みの三つの経路が、key の写像（`key_code.rs:162`、`embed/windows/input.rs:278`、`embed/x11/keyboard.rs:311,370`）、IME（winit＋Windows subclass、IMM32、XIM）、DnD、CPU の提示（softbuffer と GDI と試験の blit）、cursor の写像を別々に持つ。OS の機能を一つ足すと三か所を直す。

**解決:** arvane-native を「OS ごとの port」と「共有の pipeline」に分ける。

```text
arvane-native/src
├── port/            OS の API を呼び、OS の message を RawInput に訳すだけ
│   ├── winit/       （トップレベルの window）
│   ├── win32/       （埋め込みの子 window）
│   └── x11/         （埋め込みの子 window）
└── pipeline/        RawInput を ShellEvent にする処理と、OS へ出す要求の状態。三つの port が共有する
    ├── keys         一つの正規化（modifier は shell の ModifierState、repeat、text）と、三つの写像表（`from_winit`、`from_vk`、`from_keysym`）を一 file に
    ├── ime          ImeDriver（Composition::{Idle, Composing { preedit, caret }}、Sent<CursorArea>、Sent<Allowed>、reconversion）と、port が実装する ImePort（送受信だけ）
    ├── dnd          DropSession（Idle / Hovering { files, at } / Dropped）と DropPort
    ├── pointer      shell の Mouse・ClickCounter・Contacts を呼ぶだけ
    ├── cursor       Cursor → OS の cursor の写像表（三つの表を一 file に）
    ├── present      CPU の提示は三つとも softbuffer（Win32 の子 window の HWND も softbuffer で描く。GDI の `embed/windows/cpu.rs` を消す）
    └── a11y         AccessKit の三つの adapter が共有する conversion（既に共有。ここへ移す）
```

- `RawInput` は arvane-native の中の enum（`Key { code: PlatformKey, text, repeat, pressed }`、`PlatformKey::{Winit(KeyCode), Vk(u16), Keysym(u32)}`、`Ime(..)`、`Pointer(..)`、`Wheel(..)`、`Drop(..)`、`Focus(..)` …）。
- key を一つ足す変更は `pipeline/keys` の一 file の三行、IME・DnD の振る舞いの変更は pipeline の一か所になる。port は OS の呼び出しの差（API の名前と引数）だけを持つ。
- 1000 行に近い `runtime.rs`（1093 行、`window_event` 256 行）と `embed/x11/ime.rs`（757 行）は、port と pipeline に分かれて小さくなる。
- **unit:** A-6。

### 17.3 feature の閉じ込め（D-15、規則 DEP-4）

**現状:** `gpu-interop` は 7 crate（kivact → kivact-core → kivact-style ＋ kivact-engine → kivact-lower ＋ arvane → arvane-render）、`embed` は 5 crate、`gpu` は 5 crate を貫き、`cfg(feature)` が arvane に 50、engine に 41、core に 37 ある。

**規則 DEP-4:** feature は「実装を持つ crate」と「公開 API を出す crate」の二つだけが持つ。間の crate は `cfg(feature)` を持たず、値と経路を常に compile する（実装がないときは実行時に `Unsupported` を返す）。facade `kivact` は、実装の crate への optional の直接の依存（feature を有効にするためだけの依存。code からは使わない）で feature を有効にする。

| feature | 実装の crate | 公開 API の crate | facade の feature |
|---|---|---|---|
| `gpu` | `arvane-render`（wgpu・vello_hybrid） | なし（renderer の選択だけ） | `gpu = ["dep:arvane-render", "arvane-render/gpu"]` |
| `gpu-interop` | `arvane-render`（と `kivact-gpu-share`） | `kivact-core`（`kivact::gpu_interop`、`view::viewport(..).gpu_frame(..)`） | `gpu-interop = ["gpu", "arvane-render/gpu-interop", "kivact-core/gpu-interop"]` |
| `embed` | `arvane-native`（`port/win32`・`port/x11`） | `kivact-core`（`App::embed`、`kivact::embed`） | `embed = ["dep:arvane-native", "arvane-native/embed", "kivact-core/embed"]` |
| `test` | `kivact-engine`（試験 host）、`arvane-virtual`（feature `mirror`: 実 window への写し）、`arvane-native`（feature `test`: `TestParent` と mirror の窓の UI スレッド） | `kivact-test` | `test = ["dep:kivact-test"]`（`kivact-test` が `kivact-engine/test` を、`kivact-engine/test` が `arvane-virtual/mirror` を、`arvane-virtual/mirror` が `arvane-native/test` を有効にする。この連鎖は全部実装の crate の中） |

`arvane`（facade）は A-1 の後 `test` の feature を持たない（試験の窓の code は `arvane-virtual` と `arvane-native` へ移る）。

- `kivact-style` の `draw::GpuFrame`、`kivact-engine`・`kivact-lower`・`arvane` の GPU 共有の経路の `cfg` は消える（値と経路は常にある）。creative kit の `gpu-interop` の feature も消える: K-8 で `viewport_surface` が中核へ移ると、creative に GPU 共有の code と試験は残らない（creative の GPU frame の試験は `dev/tests/kivact-public-api` へ移る）。
- `visible_to` に、facade の anchor の依存（`arvane-render`・`arvane-native` → `kivact`）を足し、xtask の新しい検査 `anchor-deps` が「anchor の依存の crate を facade の code が `use` していない」ことを確かめる。
- **unit:** X-1。

### 17.4 関数の大きさ（D-16）

> **18.20 節が置き換える。** xtask の `fn-length` ではなく clippy の `too_many_lines` を使う。

- xtask の新しい検査 `fn-length`: 関数の本体の code 行（`max_code_lines` と同じ数え方）が 80 を超えたら error。例外は「一つの `match` の arm が一行ずつの写像の表」（key・cursor・role の写像）だけで、policy に path と理由を書く。
- 分け方は設計書 0.3 節の M-4（同じ葉の module の helper 関数か、新しい葉の module）。振る舞いを変えない。
- 調査で見つかった大きい関数（`data_table/runtime.rs:29` 347 行、`select.rs:176` 240 行、`virtual_list.rs:229` 233 行、`slider.rs:283` 201 行、`part_pointer.rs:150` 193 行、`runtime.rs:758` 256 行、`x11/dnd.rs:153` 250 行、`painter.rs:161` 246 行、`events.rs:124` 238 行、`a11y/build/dirty.rs:33` 235 行 ほか）は、状態機械の unit・17.1・17.2 で形が変わったあとに残ったものを、この検査が全部見つける。
- **unit:** 18.20 節（clippy の `too_many_lines` を crate ごとに有効にし、F-2 で workspace へ）。

### 17.5 内部の handler の型を一つに（D-17）

- `kivact_runtime::handler::Handler<E>`（`Rc<dyn Fn(E) -> Result<Handled, UiError>>` を包む `Clone` の型、`Handler::from_fn(impl Fn(E) -> R + 'static) where R: HandlerResult`、`call(&self, E) -> Result<Handled, UiError>`）を、内部で callback を保存する唯一の型にする。`view/tree.rs:64-99` の `Handlers` の slot、`PartActions`（`:145-152`）、`EventHandlers<E>`（`handler_layers.rs:57-59`）、`SessionHandler`（`editing_data.rs:105`）、motion の frame loop の関数（`state.rs:296`）、task の `DoneFn`（`action.rs:66`）を置き換える。facade は `Handler` を出さない（公開の保存する callback は `Callback<E>` だけ）。
- **unit:** R-7（runtime の中）、C-3（core の slot）。

### 17.6 開発者の使いにくさ（D-18）

> **18.2 節が置き換える。** `IntoAction` は作らず、payload のない handler は `impl Into<Callback<()>>`。

- **handler に `Callback` をそのまま渡す:** payload のない event（`on_press`、`on_exit_complete`、`on_hide`、`on_submit` など）の引数を `impl IntoAction<M>` にする。`IntoAction<M>` は sealed の trait で、`impl<F: Fn() -> R + 'static, R: HandlerResult> IntoAction<(R,)> for F` と `impl IntoAction<CallbackMarker> for Callback<()>` の二つの impl を持つ（`M` は impl を区別するための marker で、利用者は書かない。`R` を trait の引数に出さないと unconstrained の型引数になるため）。`button("Add").on_press(on_add)` と書ける（`on_add.call(())` を包む closure が要らない）。payload のある event（`on_key(|e| ..)` など）は `impl Fn(E) -> R + 'static` のまま: Rust は closure の引数の型を `Fn` 系の bound からしか推論しないので、独自の trait で受けると `|e: KeyEvent|` と型を書く必要が出るためである。payload のある handler に `Callback<E>` を渡すときは既存の `Callback::to_fn()` を使う（`.on_key(cb.to_fn())`）。`IntoAction` には API-11 の `on_unimplemented` を付ける。
- **`constant_value!` は残す（Rust の制約の下限）。** 「定数の blanket impl」と「closure の blanket impl」は stable の Rust では一つの trait に共存できない（coherence。特殊化は nightly）。利用者の型を prop にする一行は消せないので、(1) A-11 の message で「`constant_value!(MyEnum)` を一度書く」と示し、(2) 中核・kit の公開の enum には全部付けておき、(3) ガイドの custom component の最初の例で教える。
- **`'static` と `move`（Rust の制約の下限）:** Copy の handle（`Signal`、`Callback` …）で clone の連鎖をなくしてあるので、これ以上は言語を変えない限り減らせない。ガイドの「Rust の 5 つの詳細」の節を新しい API で書き直す（F-1）。
- **`Resource` と `Action` をそのまま view に渡す:** `impl IntoValue<AsyncState<T>> for Resource<T>`（と `Action`）を足す（現在は `dynamic(move || user.state(), ..)` と包む必要がある）。17.19 節の `ui::async_view` が受け取る。
- **unit:** N-2（`IntoAction`）、G-5（`Resource` の `IntoValue`）。

### 17.7 recipe の token 化（D-19）

> **18.15 節が置き換える部分がある。** `UiTokens` は部品ごとの入れ子の struct で、token にするのは kit が上書きする値と theme から読む値だけ。

**現状:** 中核の既定の recipe は値を literal と theme の数個の metric から作る。kit は値が違うだけでも recipe の関数を丸ごと書く。pop と game の recipe の file はほぼ同じ（`recipes_field` 31 行中 12 行だけ違うなど）。

**解決:** 中核の既定の recipe が読む値を、全部 `kivact::ui::UiTokens`（`ThemeExtension`）の field にする。kit は値の違いを `UiTokens` で渡し、recipe は「形が違う」（game の欠けた角、soft の影の重ね方など）ときだけ書く。

- `UiTokens` の field は、現在の `kivact-ui` の既定の recipe（`look*.rs`、`recipes.rs`）が使っている literal と theme の読み出しの一つずつ。名前は `<部品>_<部分>_<性質>`（例: `button_padding_x`、`checkbox_indicator_radius`、`menu_item_height`）。型は値の型（長さは `f64`。色は、既定の recipe が theme の palette から読んでいるものは `ColorToken`、literal の色は `Color`）。状態ごとに違う値は `StateTokens<T> { rest, hover, pressed, disabled, selected, focus }` で持つ。field の作り方は機械的なので、設計書 0.3 節の M-8（下に足す）で作る。
- pop と game は、recipe の file を `UiTokens` の値と、形の違う recipe だけにする。KIT-3 の README の表に「上書きした recipe」と「設定した token」を分けて書く。
- **unit:** K-0（18.15 節で token にする範囲を狭めた）。

（0.3 節に足す M-8: `UiTokens` の field の名前と型を、既定の recipe の literal と theme の読み出しから上の名前の規則で作ること。）

### 17.8 状態機械の網羅の保証（D-20）

> **18.14・18.20 節が置き換える部分がある。** 状態の検査は `state-fields`・`effect-writes` と clippy・runtime の検査。

付録 A の 169 項目は調査で見つけたもので、全部であることは保証されていない。次の三つで「全部」を保証する。

1. **全数調査（unit P-1）:** 作業の始めに、新しい検査（下の 2）を全 crate に report の mode で回し、さらに次の検索の結果を一件ずつ見て、13.1 節の 8 種類に当たるものを付録 A に足す: `Cell<bool>`、`Cell<Option`、`RefCell<Option`、`OnceCell`、`OnceLock`、二つ以上の `Option<` の field、`Signal<bool>`、`Var<bool>`、`Signal<u64>`、`Var<u64>`、`: bool,` の field、`spawn(sleep`、`memo(`・`effect(` の中の `.set(`、`debug_assert`、doc の「must」「before」「after」「first」「only when」「until」。足す項目の「解消後」は 13.2 節の表の型で決める。表に当たらない（公開の型が変わる、振る舞いが変わる）ものだけ作者に聞く。
2. **検査の追加（14.1 節に足す）:**

   | 検査 | 内容 |
   |---|---|
   | `state-options` | 対象外（`state-flags` と同じ (1)〜(5)）でない struct が、`Option` の field を二つ以上持つか、`Option` の field と `*State`・`*Phase`・`*Mode` の名前の enum の field を一緒に持ったら error |
   | `late-init` | 対象外でない struct の field の型が `OnceCell`・`OnceLock`・`LazyCell`・`Cell<Option<_>>`・`RefCell<Option<_>>` なら error（`kivact-types::cell` の型と、`static` は対象外） |
   | `counter-state` | 対象外でない struct の field の型が `Cell<u64|u32|usize>`・`Signal<u64|u32|usize>`・`Var<u64|u32|usize>` なら error（`Generation` と、名前が `*Stats` の struct は対象外） |
   | `memo-purity` | `memo(`・`selector(` の closure の中の `.set(`・`.update(`・`.replace(`・`send(` は error |
   | `effect-writes` | `effect(`・`effect_on(` の closure の中の、`Signal`・`Var` の `.set(`・`.update(` は error（状態の変更は `Machine::send` か handler で行う。例外は policy に理由と一緒に列挙） |

3. **終わりの確認（F-3）:** 最後にもう一度 1 の全数調査を回し、残りが 0 であることを `PROGRESS.md` に記録する。

S-9 の例外（`ReportOnce`、`Sent<T>`、`Generation`、`Mailbox<T>`、cache、統計）は、それぞれ中で名前の付いた 2 状態の enum か単一の値を持つ明示の型で、状態の意味が一つのメソッドの集合に閉じている。暗黙の状態機械ではない。

- **unit:** P-1（全数調査）、P-0（検査の実装に追加）、F-3（終わりの確認）。

### 17.9 振る舞いの正（D-21）

状態機械の unit（I-1〜I-6、U-1〜U-2）は振る舞いを変えない移設なので、今の振る舞いの不自然さは残る。その後に、振る舞いを外部の基準に合わせる unit を置く。

- **基準:** WAI-ARIA Authoring Practices Guide（APG）の各 pattern の「Keyboard Interaction」と「WAI-ARIA Roles, States, and Properties」。対象の pattern: Accordion、Alert・Alert Dialog、Breadcrumb、Button、Carousel、Checkbox、Combobox、Dialog（Modal）、Disclosure、Grid・Table、Listbox、Menu・Menubar・Menu Button、Radio Group、Slider・Multi-Thumb Slider、Spinbutton（`number_input`）、Switch、Tabs、Toolbar、Tooltip、Tree View、Window Splitter。
- **APG より優先する作者の決定（2026-10-09）:** 右クリックメニューは platform の慣習で開く（Windows・Linux は離したとき、macOS は押したとき。`Capabilities.context_menu_on`）、keyboard で開いたら trigger の位置。閉じた Select の矢印は値を変え、Alt+Down で開く。Tooltip は約 500 ms、skip-delay あり、一度に一つ。
- 違いを見つけたら APG の記述どおりに直し、E2E を足す（振る舞いの変更）。違いの一覧（pattern、現在、APG、直した内容）を unit の報告に書く。APG が決めていない細部（時間など）は今の値を `InteractionPolicy` に残す。
- **unit:** I-8。

### 17.10 退場と出現の一本化（D-22）

**現状:** 退場の動きが 5 種類（engine の `Style::exit`＋`ExitLayout`＋`on_exit_complete`、`ContentReveal::leaving`、`Disclosure::leaving`、`Delayed::Closing`、ui の toast の `Visual::Dismiss`＋`frame_loop`）。

**解決:** 出現と退場の動きは engine の presence（`NodePresence::{Arriving, Mounted, Leaving}`）一つにする。

- `Style::enter`・`exit` に大きさの動きを足す: `Style::enter_size(Axis)`・`exit_size(Axis)`。規則: (1) node の自然な大きさは通常の layout で決まる（二度目の layout はしない）。(2) 動きの間、その軸の外形の大きさ = 自然な大きさ × 進み具合（出現は 0→1、退場は 1→0）で、親の layout はこの大きさを使う。(3) 子は自然な大きさで layout し、node は動きの間その軸を clip する。(4) 動きの途中で内容が変わったら、次の layout の自然な大きさに同じ進み具合を掛ける。(5) 時間と easing は、その node の style の `TransitionProp::Size`（新しく足す transition の対象）の transition。標準部品の recipe は `UiPolicy.disclosure_motion` からこれを作る。これで内容の高さへ開く・閉じる（C-06）が node の出現と退場で書ける。
- `interaction::use_content_reveal` を消す。`disclosure` は閉じたら内容の node を unmount し、動きは recipe の `exit_size(Axis::Vertical)` で描く。`Disclosure::leaving` の外からの入力を消す。
- toast の swipe で閉じる動きは、drag の位置からの node の退場（退場の transition が現在の offset から始まる）で描き、`Visual::Dismiss` と `frame_loop` を消す。
- `Delayed::Closing` は動きではなく閉じるまでの時間の待ちなので、そのまま（`Machine` の timer）。
- 公開: `kivact::view::Presence::{Entering, Present, Leaving}` と `NodeRef::presence() -> Value<Presence>`（部品が退場の間の見た目を変えるため）。付録 A の SM-I01 の `DisclosurePresence` はこれに置き換わる。
- 残る仕組みは「node の presence（出現・退場）」と「`Animated`（その場の値の補間）」の二つで、役割が重ならない。
- **unit:** E-9（engine）、I-7（interaction・ui。付録 A の SM-I01・SM-U02・SM-U06 の presence の部分も I-7 が行う）。

### 17.11 App の状態の置き場所（D-23）

- 置き場所は範囲で二つだけ: **App 全体** は runtime の `Extensions`（構築時に全部入れる）と `use_app_state::<T>(init)`（型ごとの slot を runtime の `AppStates` に置く。最初の呼び出しで作る型の表で、S-9 の cache）、**部分木** は owner の文脈（`provide`・`use_context`）。
- theme の拡張の型 map は `Theme` という値の中の data で、状態の置き場所ではない（policy は 11.1 節の二つの struct）。`TooltipConfig` の文脈、`Tree`・`ViewState` の中の遅延の trigger、`ViewState.Started.adapter`（gpu-interop の状態の二つ目の置き場所）は消す（`gpu_interop::State` だけにする）。
- **unit:** C-3（`adapter`）、R-7（`AppStates`）。

### 17.12 text_field を公開 API で作れること（D-24）

**現状:** `ui::text_input` は公開の `interaction::use_text_edit` の上にあるが、編集する node は「native の葉」で、子と独自の描画が `Unsupported`（`kivact-core/src/primitive/text_editing.rs:12-14`）。戦略 5 章の到達目標（コードエディタを作れる水準）に届かない。

**解決:**

- 編集する node の制約を外す: `draw`・`draw_over`（行番号、波線、独自の caret）と、行の中の inline の子（`inline_view`）を編集する node に許す。engine の editor の paint は、自分の文字と選択と caret を描いた後に node の draw layer を重ねる。
- 行ごとの見た目を一般化する: 現在の `LineColors` を `TextEditing::line_styles(impl Fn(LineIndex, &str) -> Vec<StyledRange> + 'static)`（色、背景、下線の種類、太さ）にし、`TextEditing::decorations(impl Fn(LineRange) -> Vec<Decoration> + 'static)`（範囲の波線・枠・背景）を足す。
- 幾何の問い合わせは `TextLayout`（行の箱、caret の矩形、選択の矩形、当たり判定、行の範囲）を、編集する node でも描画の closure の中から読めるようにする（`Draw` の closure に `&TextLayout` を渡す `draw_with_layout`）。
- **証明:** `dev/tests/kivact-public-api/tests/e2e/custom_text_field.rs` に、`kivact::custom` と `kivact::interaction` だけで作った code editor の部品（行番号を draw で、構文の色を `line_styles` で、inline の部品を `inline_view` で、IME の変換、選択、undo（`EditPhase`）、支援技術の text）を置き、`ui::text_input` と同じ操作の E2E が通ることを確かめる。
- **unit:** T-1（engine と custom の制約の解除と API）、T-2（証明の E2E）。

### 17.13 アプリが描いた絵の表示を中核へ（D-25）

- creative の `viewport_surface` を `kivact::view::viewport(pixels: impl IntoValue<Pixels>)` として中核に移す。`.fit(impl IntoValue<ImageFit>)`、`.gpu_frame(..)`（feature `gpu-interop`）、`.transform(ZoomPanApi)`（`use_zoom_pan` の表示の変換）、`ViewportApi::to_content(Point) -> Point`・`to_view(Point) -> Point`（座標の変換）、pointer の capture（17.17 節）、`draw_over` の重ね描き、canvas の region による支援技術の領域。creative は look と編集の部品だけを持つ。
- **unit:** K-8。

### 17.14 地域書式（D-26）

- `kivact-i18n` の書式を ICU4X（2 系。`icu_decimal`、`icu_datetime`、`icu_calendar`、`icu_locale_core`、`icu_plurals`、`icu_collator`、compiled data）で実装する。ICU4X の型は公開しない（API-9）。手書きの CLDR の表（`kivact-i18n/src/locale.rs:120-140` の `SUNDAY_START`・`SATURDAY_START`）と、C-2 で作る en-US の表を消す。
- 足す公開の API: `Locale::plural_category(f64) -> PluralCategory`、`Locale::compare(&str, &str) -> Ordering`（collation）、`Messages::with_plural(key, PluralForms { zero, one, two, few, many, other })` と、`message_with` の plural の選択。C-2 の書式の関数（`format_number`、`format_date`、`format_time`、`month_name` …）は ICU4X で全 locale に対応する。
- **解析（parse）:** ICU4X 2 系には数値と日付の解析がない。`parse_number` は ICU4X の locale の数値の記号（小数点、桁の区切り、数字の体系）を使って kivact-i18n が解析する。`parse_date` は ISO 8601 と、locale の短い日付の pattern（ICU4X の datetime の pattern）の年・月・日の順の数字の入力を受ける（月名の入力は受けない）。
- 全 locale の data を含める（16.3 節の Q-07 の作者の決定）。実行ファイルが数 MB 大きくなる。増える大きさを L-1 で測り、`PROGRESS.md` に記録する。
- **unit:** L-1。

### 17.15 kit の Tier 3 も中核へ（D-27）

付録 C.2 の Tier 3 を全部上げる。

| 部品 | 中核の API | 元の kit |
|---|---|---|
| chat | `ui::chat_thread(children)`（`Role::Log`、`use_follow_end`）、`ui::chat_message(author)`（`.content(..)`、`.time(..)`、`.status(..)`、`.variant(..)`）、`ui::typing_indicator()`、`ui::chat_composer(value: impl Into<Binding<String>>)`（`.on_submit(..)`） | basic `chat.rs`、pop `messaging.rs`、soft `soft_chat*.rs` |
| code block | `ui::code_block(code: impl IntoValue<String>)`（`.line_numbers()`、`.highlight(impl Fn(&str) -> Vec<StyledRange> + 'static)`、`.wrap(impl IntoValue<bool>)`、copy の button は `use_copy_feedback`）。言語ごとの構文の色の関数は kit に残す（basic の `rust_lines` は `highlight` に渡す関数になる） | basic `code_block/`、soft `code_block.rs` |
| progress ring・gauge | `ui::progress(..).variant(..)` の `Ring`・`Gauge`（標準の variant） | pop `progress_ring`、game `ring`・`gauge` |
| stat・KPI | `ui::stat(label, value: impl IntoText)`（`.delta(..)`、`.trend(impl IntoValue<Trend>)`、`.icon(..)`） | pop `stat_card`、game `kpi` |
| heading | `ui::heading(level: u8, text: impl IntoText)`（`Role::Heading`、`level`） | pop `section_heading`、game `section_heading`、soft `heading`・`section` |
| key value | `ui::description_list(children)`、`ui::description(term: impl IntoText, detail: impl IntoView)` | pop `key_value`、game `property_list`、soft `metadata_list`、creative `property_row`（creative の編集できる行は creative に残し、表示の部分を中核の部品で組む） |
| aspect ratio | kit の部品を消し、`Style::aspect_ratio` を使う | basic、soft |

- **unit:** K-9。

### 17.16 モバイルの境界（D-28）

戦略 8 章「モバイルと移植の設計境界は P3 へ先送りしない」に合わせ、口と型を今作る（desktop の backend は使わない値を無視するか `Unsupported` を返す。`arvane-virtual` が台本で試す）。

- **文字入力の種類:** `InputPurpose::{Text, Number, Decimal, Email, Url, Phone, Password, Search}`。`ui::text_input(..).purpose(impl IntoValue<InputPurpose>)` → `TextEditing::purpose` → engine → `ShellWindow::set_ime_purpose`（winit は Normal・Password だけなので、ほかは Normal に写す）。`password(..)` は `purpose(InputPurpose::Password)` に統合する（二つの経路を作らない）。統合するのは四つ: `ui::TextInput::password`、`interaction::TextEditApi::password`（`text_edit/handle.rs:129`）、`custom::TextEditing::password`（`kivact-core/src/primitive/text_editing.rs:185`）、`custom::ImeSession::password`。どれも `purpose(impl IntoValue<InputPurpose>)` になる。
- **keyboard の inset:** `ShellEvent::KeyboardInset(Insets)` → `Environment.keyboard_inset` → `platform::keyboard_inset() -> Value<Insets>`。
- **状態の復元:** `LifecycleEvent::SaveState`、`platform::save_state(key: &'static str, bytes: Vec<u8>)`、`platform::restored_state(key: &'static str) -> Option<Vec<u8>>`、`Capabilities.state_restoration`。desktop の backend は `false`（保存しない）、`arvane-virtual` は台本で保存と復元を試せる。
- **unit:** G-3。

### 17.17 Canvas・Path2D・pointer の capture（D-29）

- `Path::contains(Point, FillRule) -> bool`（`isPointInPath`）、`Path::stroke_contains(Point, &Stroke) -> bool`（`isPointInStroke`）、`Path::arc_to(p1: Point, p2: Point, radius: f64)`（canvas の `arcTo`。現在の SVG 形式の `arc_to` は `svg_arc_to` と改名する）、`Path::parse_svg(&str) -> Result<Path, UiError>`（`Path2D(svgString)`）。
- 窓の中の node の画素の読み出し（`getImageData` に当たる）: `NodeRef::snapshot() -> impl Future<Output = Result<Pixels, UiError>>`。node の部分木の scene を Offscreen と同じ CPU の rasterizer で描く（GPU から読み戻さない）。部分木に GPU 共有の frame（`gpu_frame`、`RenderCmd::External`）があり CPU の画素の代わりがないときは、`UiError`（`ErrorKind::Unsupported`）を返す（黙って透明にしない、G-05）。`viewport` は自分の `Pixels` を CPU の代わりとして描く。
- 明示の pointer の capture: `PointerEvent::capture()`・`release_capture()`（handler の中で、その node にこの pointer を送り続ける。engine の `Route` の capture を使う。今の「押している間の暗黙の capture」は既定のまま）。
- **unit:** G-1（Path と snapshot）、G-2（capture）。

### 17.18 soft kit が求めた中核の不足（D-30）

SOFT_UI_WORK_ORDER の 7 章のうち未実装のもの:

| ID | 中核の API |
|---|---|
| C-05 | `TransitionProp::Text`（文字色の補間） |
| C-08 | `Style::layout_transition(Transition)`（layout の結果の位置と大きさの変化を、engine が前の位置から補間する FLIP） |
| C-10 | `Transition::retarget(Retarget::{Restart, Proportional})`（途中で目標が変わったとき、残りの距離に比例した時間にする） |
| C-20 | `platform::theme_hint() -> Value<ThemeHint>` と `ThemeMode::System`（OS の light・dark に追従する） |
| C-21 | `PointerSettings.primary: PointerClass::{Fine, Coarse}`。backend の判定: Windows は `GetSystemMetrics(SM_MOUSEPRESENT) == 0` かつ `SM_DIGITIZER` が `NID_READY` を含むとき `Coarse`、それ以外は `Fine`。X11・Wayland は `Fine`（OS に確かな問い合わせがない）。`arvane-virtual` は台本で指定する。値は環境の変化（`Environment` の更新）で変わる |
| C-25 | `TextDecorationStyle::{Solid, Dashed, Dotted, Wavy}` |
| C-34 | `Style::visibility(Visibility::Hidden)`（layout に参加し、paint・支援技術・focus・当たり判定から外れる） |
| C-35 | `use_combobox`・`use_select`・`use_calendar_grid` の option（soft-ui-work-order の付録 02 の「Kivact 実装方針」の節が列挙したもの。名前と型はその付録の記述のとおり） |
| C-37 | `Style::scroll_snap(ScrollSnap)`、touch の fling の慣性、`ScrollBehavior::Smooth(Easing)`。慣性は、指を離したときの速度（`DragApi::velocity` と同じ計算）から、`PointerSettings.fling_deceleration`（既定 0.998 / ms の減衰率）で減速し、速度が `PointerSettings.fling_min_velocity`（既定 0.05 px/ms）を下回ったら止める。snap がある scroll では、止まる位置を `nearest_snap` で snap の位置に寄せる |

- **unit:** G-4。

### 17.19 非同期の標準部品（D-31）

- `ui::async_view(state: impl IntoValue<AsyncState<T>>, ready: impl Fn(T) -> V + 'static)`（`.loading(impl Fn() -> V)`、`.error(impl Fn(UiError, Retry) -> V)`。既定は `ui::spinner` と、retry の button を持つ `ui::alert`）。`Resource<T>` と `Action` は 17.6 節の `IntoValue<AsyncState<T>>` で直接渡せる。戦略 5 章「標準部品: 非同期の状態」を満たす。
- **unit:** G-5。

### 17.20 この再設計でも行わないもの（戦略による）

次は「見つかった課題」ではなく、戦略が意図して外したもの。この再設計でも行わない（16.3 節の Q-08。作者の決定: 実機と環境がなく、確かめる手段がないため）。

| 項目 | 根拠 |
|---|---|
| macOS・Android・iOS・Web の backend | 戦略 8 章（macOS は実機と保守者がそろったとき、Android・iOS は設計と実機の検証の対象）、N-01（Web）、P-09。`Backend` の SPI（9 章）と 17.16 節の境界までは作る |

**課題ではなかったもの（初版の 17.20 節の表から外した。2026-11-04）:**

- **path の boolean 演算:** 調査は「HTML Canvas・Path2D との対応で足りないもの」に挙げたが、Canvas・Path2D にも boolean 演算はない（`addPath` だけで、これは `Path::append` として既にある）。戦略 5 章の「HTML Canvas と Path2D に対応させる」の不足ではなく、ほかに必要とする部品・要求も見つかっていない。作者の 2026-10-09 の見送りとも一致する。
- **3D の `rotateX`（SOFT_UI_WORK_ORDER の C-24）:** 使う場所は soft の日付の touch の wheel だけで、soft は既に 2D の affine の変換で円筒の見え方を近似している（`kivact-ui-soft/src/date_touch.rs:514-515`: 行ごとに `scale_xy(scale, scale * cos(角度))`）。C-24 の「代替（wheel を作らない）」の記述は古い。残る差は行が台形にならないこと（遠近の効果）だけで、操作・支援技術・振る舞いの問題ではない。

---

## 18. 第 4 回のレビューによる改訂（2026-11-04）

「Rust 未経験の React・SolidJS 開発者が書けるか」「API は統一的か」「内部は統一的で単純か」「長期の保守性」の四つの観点のレビューで決めたことを、この章にまとめる。

### 18.0 優先の順

**19 章 → 18 章 → 17 章 → 1〜16 章 → 付録 の順に優先する**（19 章は 2026-11-05 に足した）。新しい章と食い違う古い記述は、新しい章のとおりに読む（選択肢ではない）。付録 B の名前は B.1〜B.6 と B.10 が正である（B.8 は B.1〜B.6 に反映済み）。作業手順書の unit の節は、18 章と 19 章を反映した最終の形で書いてある。

### 18.1 API の規則

3.4 節を 15 の規則に書き直した。初版との主な違い: 位置引数の順（API-13）、問い合わせと語（API-14）、接尾辞の役割（API-15）、`use_` の意味（API-7、18.3 節）、利用者が読み返す数値も `f64`（API-4、18.4 節）。

### 18.2 handler と callback の型（17.6 節を置き換える）

- **payload のない event:** 引数は `impl Into<Callback<()>>`。`kivact-runtime` に `impl<F, R> From<F> for Callback<()> where F: Fn() -> R + 'static, R: HandlerResult` を足す。変換は今の owner（setup、handler、effect の中のどこでも）の下に `Callback` を作る。owner がないところでは API-12 の panic。data の record（`Toast::action(label, ..)` など、handler の中でも作るもの）も同じ型で受ける。`Fn` は `#[fundamental]` で `Callback` は自分の型なので、`From<T> for T` と重ならない（yew の `Callback` と同じ形）。`R` は `Fn() -> R` の projection で拘束されるので marker の型引数は要らない。
- **`IntoAction<M>` は作らない。** 17.6 節の `IntoAction`・marker・`CallbackMarker` を消す。
- **payload のある event:** 今までどおり `impl Fn(E) -> R + 'static`（Rust は closure の引数の型を `Fn` 系の bound からしか推論しないため）。`Callback<E>` を渡すときは `cb.to_fn()`。
- **独自部品の callback の prop の書き方（ガイドで最初に教える形）:**
  ```rust
  fn toolbar(on_add: impl Into<Callback<()>>) -> impl Component {
      let on_add = on_add.into();                  // Copy の Callback<()>
      row((button("Add").on_press(on_add), button("Add again").on_press(on_add)))
  }
  // 呼ぶ側は closure をそのまま渡す
  toolbar(move || hits.update(|h| *h += 1));
  ```
  payload のある callback の prop は `on_pick: impl Fn(K) + 'static` を取り、二か所で使うなら `let on_pick = callback(on_pick);`。

### 18.3 `use_` の意味

API-7 のとおり。`use_theme()`・`use_tokens::<T>()`・`use_context::<T>()`・`use_locale()`・`use_direction()` は、setup と、その owner の下で作られた closure（style の closure、handler、effect）の中で呼べる（現在の `current_theme()` が owner の文脈ならどこでも呼べるのと同じ。初版の「setup だけ」は退行だった）。17.7 節の `tokens::<T>()` は `use_tokens::<T>()` と改名する。振る舞いを付ける `use_popup()` などは setup の中だけ。

### 18.4 数値は読み返す値も f64

facade から届く型の公開の `f32` の field は全部 `f64` にする（約 74。`kivact-types/src/geometry.rs:11-35` の `Point`・`Size`・`Rect`、`Insets`（`kivact-types/src/platform.rs`）、`Transform`、`Spring` の `stiffness`・`damping`・`mass` など。一覧は V-4 が `api-shape` の report で作る）。event の位置、`DragUpdate`、`NodeRef::rect()`、`Draw` の大きさも同じ型なので、`value.set(origin + update.delta.x)` がそのまま書ける。f32 は taffy の adapter（`kivact-layout`）と GPU の code（`arvane-render`）の中だけで、境界で変換する。xtask の `api-shape` は公開の field の `f32` も error にする。

### 18.5 非同期

- `ui::async_view(source: impl AsyncSource<T>, ready: impl Fn(T) -> V + 'static)`。`AsyncSource` は sealed で、`Resource<T>`（retry = `refetch`）と `Action<T, I>`（retry = `retry`）が実装する。既定の失敗の表示（retry の button を持つ `ui::alert`）は `AsyncSource` の retry を呼ぶ。`.loading(impl Fn() -> V)`、`.fallback(impl Fn(UiError, Retry) -> V)`（`ErrorBoundary::fallback` と同じ名前）。17.6・17.19 節の `IntoValue<AsyncState<T>> for Resource` は作らない（重なるため）。`AsyncState` だけを持つときは `dynamic` を使う。
- **tokio の future を待つ:** facade の feature `tokio`（既定は無効。実装の crate は `kivact-runtime`、DEP-4）。`kivact::task::on_tokio<T: Send + 'static>(fut: impl Future<Output = T> + Send + 'static) -> impl Future<Output = T>` は、App が持つ一つの tokio の runtime のスレッドで `fut` を動かし、終わったら UI を起こす。返した future が drop されたら tokio の task を abort する（`spawn` した task は owner の破棄で drop されるので、owner の破棄で止まる）。ガイドに「HTTP で JSON を読む」の手順（`?` で `UiError` にする）を足す。
- `Retry::retry()` は `Retry::run()` と改名する。

### 18.6 prelude

**規則:** prelude に入れるのは、(1) 入門のガイド（getting-started）と付録 B.7 の例が import なしで使う名前、(2) prelude の関数の位置引数の型。builder の option の型（`ControlSize`、`Edge`、`*Variant`、`style()` の enum など）は、その module から import する（全部を入れると prelude が大きくなり、名前の衝突も増えるため）。付録 B.2 はこの規則の一覧。足すもの: `constant_value!`、`Component`、`ItemKey`、`PropValue`、`bind!`、`InputPurpose`、`async_view`、`toaster`、`Toast`、`toast_region`、`Tone`、`DialogMode`、`DismissOn`、`Placement`、`RowHeight`、`Point`、`Size`。

### 18.7 prop の型と binding

- **外の crate の型:** `std::time::Duration` と `std::path::PathBuf` に定数の prop の impl を `kivact-runtime` で付ける。それ以外の外の型（`chrono::NaiveDate` など）は orphan rule で `constant_value!` を書けないので、`move || x.clone()` を渡す（API-11 の message でそう示す）。
- **bound の別名:** `kivact::state::ItemKey`（`Clone + Eq + Hash + 'static` の blanket の supertrait）と `PropValue`（`Clone + PartialEq + 'static`）を足し、公開の署名の bound を全部これにする。`on_unimplemented` は「`#[derive(Clone, PartialEq, Eq, Hash)]` を付ける」。
- **binding と選択の trait:** `Into<Binding<T>>` を `IntoBinding<T>`、`Into<Selection<K>>` を `IntoSelection<K>`（どちらも sealed、`on_unimplemented` 付き。例: 「slider は f64 を binding する: `signal(40.0)` と書く」）にする。impl の組は、`IntoBinding<T>` が `Signal<T>` と `Binding<T>`、`IntoSelection<K>` が `Signal<Option<K>>`・`Signal<Vec<K>>`・`Binding<Option<K>>`・`Binding<Vec<K>>` と正準 `Selection<K>` の恒等変換。合成部品の呼出し側に Single/Multiple の分岐を要求しない。今の `From<Signal<T>> for Binding<T>` と `From<..> for Selection<K>` は消す（経路を一つにする）。`.optional()` は残す（`Signal<K>` を `Option<K>` の binding にすると None の書き込みを捨てることになり、G-05 に反するため）。
- **struct の field の binding:** `Signal::field(get_ref, get_mut)` を `Signal::project(get_ref, get_mut) -> Binding<U>` と改名し（`ui::field` と名前が衝突しないように）、`Binding::project(get_ref, get_mut) -> Binding<U>` も同じ形で持つ。macro `bind!(draft.name)`（`bind!(draft.address.city)` のように入れ子も可）を足す。`bind!(x.a.b)` は `x.project(|v| &v.a.b, |v| &mut v.a.b)` に展開し、`x` が `Signal` でも `Binding` でも `Binding<U>` を返す。
- **一覧の行の binding:** `for_each` の source が `Signal<Vec<T>>` のとき、行の closure は `Binding<T>` を受け取る。`trait ListSource<T> { type Row; }`（`T` は trait の型引数、行の型は関連型）。impl の組は全部で五つ: `Signal<Vec<T>>` と `Binding<Vec<T>>` は `Row = Binding<T>`、`Value<Vec<T>>`、`Vec<T>`、`F: Fn() -> Vec<T>` は `Row = Value<T>`（memo は `Value`）。`IntoValue<Vec<T>>` への blanket の impl にはしない（`Signal` の impl と重なるため）。`Var` は reactive でないので一覧の source にできない。`todos.item(|t| t.id, id)` が要らなくなり、`row.get()` は今までどおり書ける。`Signal::item` は消す。
- **closure を子にする:** `impl<F, R> IntoView for F where F: Fn() -> R + 'static, R: Into<String>` を足す（reactive な文字の子。Solid の `{n()}` と同じ: `row(("Count: ", move || n.get().to_string()))`）。closure の view を返す動的な子は今までどおり `dynamic`。
- `IntoText` の読み出しの名前は `into_text_value` のまま（`IntoValue::into_value` と同じ名前にすると、両方を実装する `&str`・`String` で曖昧になるため）。

**API-11 の受け入れの基準:** blanket の impl を持つ trait（`ItemKey`、`PropValue`、`IntoViewFn`、`Component`）では、rustc が内側の bound（例: `Hash`）の error を先に出し、`on_unimplemented` の message が出ないことがある。その trait は、message を付けたうえで、rustdoc の先頭と guide に同じ hint を書く（message が出ることまでは求めない）。

### 18.8 slot の種類の表

付録 B.9 に、全部の標準部品と kit の部品の slot の種類（`IntoView` か `Fn`）の表を置く。`impl Fn() -> V` の slot は `impl IntoViewFn<V>`（sealed、`Fn() -> V` の blanket）で受け、`on_unimplemented` は「この内容は開くたびに作り直す: `|| view` と書く」。

### 18.9 部品の戻り値

`pub trait Component: IntoView + NodeBuilder {}` と blanket の impl を `kivact::view` に足す。規則: 一つの root を持つ独自部品は `impl Component` を返す（呼ぶ側が `.style(..)` などを続けられる）。root が一つでない部品だけ `impl IntoView`。付録 B.7 の例と custom-components の guide をこれに揃える。

標準・kit の constructor が固有の fluent option を公開する場合は具体的な builder 型を返す。`impl Component` で `.variant(..)`・`.size(..)`・`.content(..)` などを隠さない。primitive は `Node`、behavior は `*Api` のまま。Action の完了 payload は `#[derive(Clone)] enum ActionOutcome<T> { Succeeded(T), Failed(UiError) }` とし、キャンセルの通知は増やさない。

### 18.10 Part が host の node の性質を読む

7.4 節で `Part` は mount のときに当てるので、part は host の node に caller が書いた `disabled`・`invalid`・`name`・`labelled_by`・`described_by` を読み、machine の入力にする（作者が `accessible_disabled()` を手で渡す処理が要らない）。`*Api` は状態の読み出しを直接持つ（例: `SwitchApi::control_state() -> Value<ToggleState>`）。「part を node に移す前に状態を読む」（`custom-controls.md:56`）は要らなくなる。

### 18.11 theme

- `Theme::get::<T>()` を `Theme::ext::<T>()` と改名する（`use_theme().get().ext::<X>()` と読める）。`Ext<T>` は `ThemeRef<T>` と改名する（`NodeExt` などの `Ext` と紛れないように）。
- `App::theme(impl IntoValue<Theme>)` は reactive な値を受け、root の owner の下で評価する。OS の light・dark に追従するのは `App::new(..).theme(move || basic::theme(platform::theme_hint().get().into()))` と書く（`ThemeHint` から `ThemeMode` への `From`）。17.18 節の `ThemeMode::System` は作らない。
- embed は UI thread に startup を送るため固定 App theme のみ転送する。caller-side の reactive Prop は Unsupported として明示し、embedded root factory 内で provide_theme を作る。private Send startup data と Prop theme を分けて所有し、同じ設定を二重保持しない。unsafe Send や dynamic theme の暗黙 snapshot は使わない。
- 別の kit の theme の下で kit の variant を使ったら、中核の既定の recipe で描き、`ReportOnce` の diagnostic を一度出す。

### 18.12 改名と語の統一

付録 B.8 の表のとおり（API-13〜API-15 を当てた結果）。主なもの:

| 語 | 統一 |
|---|---|
| 開いているか | `is_open()`（`shown()` を消す） |
| 押せない | `disabled` だけ（`use_tooltip_when(enabled)` → `use_tooltip().disabled(..)`） |
| 処理中 | `busy`（`Button::loading` → `busy`） |
| 消える | `exit`（`Presence::Leaving` → `Exiting`、`while_shown` → `while_present`） |
| 選ぶ前の強調 | `active`（`on_preview` → `on_active_change`） |
| 見た目の選択 | `*Variant`（`DialogLook` → `DialogVariant`、`ui::Segmented` → `RadioGroupVariant::Segmented`、`ProgressVariant::{Bar, Ring, Gauge}`、`AlertVariant::Banner`） |
| 構造の選択 | `.mode(XMode)`（`DialogKind` → `DialogMode`、`ActionPolicy` → `ActionMode`） |
| 作る前の設定の束 | `*Options`（`CalendarPolicy` → `CalendarOptions`、`ListPolicy` → `ListOptions`、`AppSettings` → `AppOptions`、behavior の `*Settings` → `*Options`、`Settings<S>` → `OptionsCell<S>`） |
| 時間の束 | `*Timing`（`TooltipConfig` → `TooltipTiming`） |
| 通知 | `Notifier`・`notifier()`（`Trigger` は `PopupApi::trigger()` と紛れるため改名。`notify()`・`track()`） |
| 端 | `Edge`（`Side`・`ToastSwipeEdge` を一つに） |
| 向き | `Orientation`（17.10 節の `Axis` を作らない） |
| node に付いた | `Node::on_attach(Fn(PartMount))`（`state::on_mount` と分ける） |

### 18.13 `Machine` を小さくする（6.5 節を置き換える）

```rust
pub trait Machine: Sized + 'static {     // Self が、遷移が読む入力の snapshot
    type State: Clone + PartialEq + 'static;
    type Event: 'static;
    type Effect: 'static;
    fn transition(&self, state: &Self::State, event: Self::Event) -> Step<Self::State, Self::Event, Self::Effect>;
}
impl<S, Ev, Ef> Step<S, Ev, Ef> {
    pub fn stay() -> Self; pub fn to(next: S) -> Self;
    pub fn with(self, effect: Ef) -> Self;
    /// Send `event` after `delay`, replacing a pending timer of the same event variant.
    pub fn after(self, delay: Duration, event: Ev) -> Self;
    /// Cancel the pending timer of `event`'s variant (the value is only used for its variant).
    pub fn cancel(self, event: Ev) -> Self;
}
/// Setup only. `input` builds the machine value (its input snapshot) for each event, untracked.
pub fn machine<M: Machine>(initial: M::State, input: impl Fn() -> M + 'static, run: impl Fn(MachineCx<M>, M::Effect) + 'static) -> MachineHandle<M>;
```

- `Input` と `Timer` の関連型、`MachineCx::after`・`cancel`、`TimerKey` を消す。timer は遷移が `Step::after` で出し、一つの machine の中で event の variant ごとに一つ（`std::mem::discriminant`）。遷移を読めば timer の動きも分かる。item ごとに timer が要るもの（toaster の toast ごとの期限など）は、item ごとに machine を一つ持つ（toaster は toast ごとの machine と、列を持つ machine に分ける）。
- `machine(..)` の `M` は `input` の closure の戻り値から推論されるので、turbofish は要らない。
- interaction の machine の型は、behavior の module の private な `machine` 子 module に `Machine`・`State`・`Event`・`Effect` の名前で置き、公開の `*State`（part の状態）と名前が衝突しないようにする。
- guide の「React・SolidJS からの対応表」に「`Machine` は `useReducer` に effect と timer を足したもの」と書く。

### 18.14 状態の規則（S-2・13.1 節・17.8 節を置き換える）

- 暗黙の状態機械の検出は、13.1 節の定義（1）と同じにする: **状態の field（bool、`Option`、counter）を二つ以上持つ struct**、または `Option` と `*State`・`*Phase` の enum を一緒に持つ struct を検出する。単独の bool の field（2 状態の独立した事実）は許す。初版の S-2（bool を全部 enum にする）はやめる（現在 bool の field は約 705 あり、大半は独立した事実か option の束）。
- 状態の検査の対象外は、役割で決まるものだけを policy に書く: API-15 で役割の決まった接尾辞（`*Options`・`*Event`・`*Phase`・`*Reason`・`*Input`・`*Report`・`*Tokens`・`*Recipes`）の struct、`NodeBuilder` を実装する builder と `*Api`、policy の `record_types` の record、policy の `value_modules`（`kivact-types` の値の module と `kivact-engine::contract`）の型。それ以外の例外は、field か struct の直前の inline の目印 `// xtask-allow(<検査>): <理由>` で書く（`//! Role:` と同じ流儀。file の分割や改名で壊れない）。
- 接尾辞は役割を一つだけ表す（API-15）。

### 18.15 設定の面を減らす（11.1 節・17.7 節を置き換える）

- `UiPolicy` を消す。振る舞いの knob（閾値、時間、disabled の focus、drag と swipe）は全部 `InteractionPolicy` に、behavior ごとの入れ子の struct で置く。ui の部品の振る舞いの knob もここ（ui は interaction を読める）。field の名前と型は次で決まり（全部 `pub`、`Default` は今の値）:
  ```rust
  pub struct InteractionPolicy {
      pub menu: MenuPolicy,          // { press: MenuPress, drift: PressDrift, submenu: SubmenuTiming, disabled_focus: DisabledMenuFocus }
      pub tooltip: TooltipTiming,
      pub hover_card: HoverCardTiming,
      pub button: ButtonPolicy,      // { disabled_focus: DisabledButtonFocus }
      pub slider: SliderPolicy,      // { touch: SliderTouchMotion }
      pub switch: SwitchPolicy,      // { drag: SwitchDrag }
      pub toast: ToastPolicy,        // { swipe: ToastSwipe, swipe_slop: f64 (8.0) }
      pub select: SelectPolicy,      // { sheet_slop: f64 (10.0) }
      pub drawer: DrawerPolicy,      // { swipe: DrawerSwipe }
  }
  ```
  `MenuPress(pub bool)` は今の形のまま field の型にする（`PressDrift` は 17 章の 11.1 節の表の型）。
- 見た目の動きと配置は `UiTokens` へ: `button.busy_motion: ButtonBusyMotion`、`progress.motion: ProgressMotion`、`toast.row_motion: ToastRowMotion`、`toggle_button.label_reservation: ToggleLabelReservation`、`dialog.center: DialogCenter`、`disclosure.motion: DisclosureMotion`、skeleton の shimmer。
- `UiTokens` は部品ごとの入れ子の struct（`UiTokens { button: ButtonTokens, menu: MenuTokens, .. }`）にし、各 struct はその部品の recipe の隣の file で定義する。入れ子の struct の field の名前は `<部分>_<性質>`（`ButtonTokens.padding_x`、`CheckboxTokens.indicator_radius`。17.7 節の `<部品>_` は付けない）。M-8 で token にするのは「現在いずれかの kit が上書きしている値」と「theme から読んでいる値」だけ（ほかの literal は既定の recipe の中に残す。後で kit が上書きしたくなったら field を一つ足す）。
- kit の作者が触る面は四つになる: `Theme`（palette と metric）、kit 自身の `Tokens`、`UiTokens`（値）と `UiRecipes`（形）、`InteractionPolicy`（振る舞い）。

### 18.16 語彙の規則 VOC-5

**VOC-5:** 依存を持たない純粋な値の型は、builder のメソッドも含めて `kivact-types` に置き、facade はそれをそのまま再公開する（包む型を作らない）。包む型を作ってよいのは、reactive な値を持つとき（`Style`）と、外の crate の依存が要るとき（kurbo を使う `Path` の幾何）だけ。

- `kivact::motion::Transition` は `kivact_types::motion::Transition` の再公開にする（`essential`・`retarget` も types に）。
- `Path` は kurbo の幾何（bounds、length、trim、stroke outline、contains、parse）が要るので `kivact-style` の型のまま。types の側の記録は `PathData`（`PathCmd` の列）と名付け、`types::scene::Path` という名前は作らない（VOC-3）。

### 18.17 `kivact-lower` の境界（8.3 節を置き換える）

`ArvaneAssets`（node ごとの報告の履歴、font の登録、asset の store を持つ。`kivact-lower/src/assets.rs:48`）と当たり判定の `shape` を engine へ移す。`BoxShape` は値なので `kivact-types::paint` へ。`kivact-lower` の公開は `Lowerer`・`LowerCx`・`LowerReport`・`SceneCache` だけにし、asset は `LowerCx` の lookup の関数で受け取る。これで lower は「木も入力も asset の状態も知らない純粋な変換」になる。

### 18.18 入力の正規化は `arvane-shell` に（17.2 節を変える）

OS を知らない正規化（`ImeDriver`、`DropSession`、key の正規化、VK と keysym の数値の写像表、cursor の写像表、`RawInput`）は `arvane-shell` の `pipeline` に置く（`Mouse`・`ClickCounter`・`Contacts`・`ModifierState` と同じ crate）。`arvane-native/port` には OS の呼び出しと、winit の型からの写像（winit に依存するため）だけを残す。`arvane-virtual` の台本に `Script::raw(RawInput)` を足し、E2E が同じ pipeline（key の写像、IME、DnD）を通るようにする。`RawInput` は OS を知らない値なので SPI に入れてよい。

### 18.19 backend の選び方（15 章の行を直す）

OS の backend は全部 `arvane-native/port/<os>` に置き、cfg で選ぶ（新しい OS は port を一つ足す。pipeline は共有）。OS でない backend（`arvane-virtual`）だけが別の crate。`App::run` は native を使い、試験は virtual を使う。外から backend を差し込む公開の口は作らない（P-10 の第二の実装は `arvane-virtual`）。

### 18.20 検査を減らす（14.1 節・17.3 節・17.4 節・17.8 節を置き換える）

| 検査 | 実現 | 理由 |
|---|---|---|
| 関数の長さ | clippy の `too_many_lines`（`clippy.toml` の `too-many-lines-threshold = 80`）。例外は `#[expect(clippy::too_many_lines, reason = "..")]` | 既存の lint で足り、例外の書き方も既存の規則（reason 必須）と同じ |
| 後から埋める初期化 | clippy の `disallowed_types`（`OnceCell`、`OnceLock`、`LazyCell`、`LazyLock`。`static` と `kivact-types::cell` は `#[expect]`） | 既存の lint で足りる |
| memo の中の書き込み | runtime の misuse（`FrameKind::Pure` の中の signal の書き込みは panic。6.4 節） | 構文の検査より正確（helper 関数の中の書き込みも見える） |
| `state-fields` | xtask。18.14 節の規則（状態の field が二つ以上） | `state-flags`・`state-options`・`counter-state`・`late-init` の残りを一つに |
| `effect-writes` | xtask。中核と kit の crate の `effect(`・`effect_on(` の中の signal の書き込み | 利用者の code は対象にしない（Solid と同じく、利用者は effect で書いてよい） |
| `api-shape`、`canonical-path`、`name-collision`、`kit-literal-color`、`os-branch` | xtask（14.1 節のとおり。ただし `api-shape` に「facade から届く型の公開の `f32` の field は error」を足す） | — |
| `structural-options` | xtask。引数なしで `Self` を返す公開のメソッドは、policy の `structural_options` の一覧に載っていなければ error | 10.6 節の一覧がずれない |

- 消す検査: `anchor-deps`（`visible_to` で足りる）、`policy-coverage`、`fn-length`、`late-init`、`memo-purity`、`state-flags`・`state-options`・`counter-state`（`state-fields` に統合）。
- 検査の対象は「全 crate から policy の `skip` の一覧を除いたもの」にする（一覧は作業の進みで空へ減る。新しい crate は自動で対象になる）。各 unit は自分の crate を `skip` から外す。
- 例外は inline の目印（18.14 節）。xtask の各検査の module の rustdoc に、規則とその理由を書く。

### 18.21 そのほかの内部の統一

- **失敗の型を一つに:** `ShellError::Failed(String)`、`LowerReport.failures: Vec<String>`、`PlatformResult::Failed` を `kivact_types::Failure { kind: FailureKind, detail: String }` にする。
- **App 全体の型の map を一つに:** `Extensions`（構築時に中核の entry を入れる）と `AppStates`（17.11 節）を一つの map にし、`use_app_state::<T>(init)` は同じ map に最初の呼び出しで入れる。
- **`*Api` の option の builder は setup の中だけ:** `disabled(self, ..) -> Self` などは共有の `OptionsCell` を書き換えるので、mount の後に呼んだら API-12 の misuse にする。
- **semantics の patch:** mount のとき定数の patch は畳み、reactive な patch があるときだけ render effect を作る。patch の `#[track_caller]` の位置を `why` の origin に記録する。
- **fragment の透明性:** engine の `tree` の生の `children` を module の外に出さず、`flat_children` だけを出す（`module_rules` で強制）。
- **ICU4X:** L-1 で実行ファイルの大きさに加えて build の時間も測り、`PROGRESS.md` に書く。

### 18.22 知識を残す（作業手順書 F-1・F-3 を変える）

- `docs/for-kivact-developer/decisions.md`（新規、日本語）: 作者の決定の記録（ADR の形: 日付、問い、決定、理由）。16.3 節の Q-01〜Q-08、17.9 節の APG より優先する作者の決定、17.20 節（行わないものと、課題でなかったもの）、`DECISIONS.md` の全部を移す。消さない。
- `docs/for-kivact-developer/architecture.md` に足す: 17.3 節の feature の表、6.2 節の `Stage` の対応の表と 6.3 節の時間の仕組み、15 章の表を「X を足すときに変える場所」の check list に書き直したもの、10.6 節の構造の option の考え方（一覧そのものは policy）。
- xtask の各検査の rustdoc に規則と理由を書く（18.20 節）。

### 18.23 15 章の表の行を足す

| 作業 | 再設計後 |
|---|---|
| 標準部品を一つ足す | `kivact-interaction`（`use_*` と machine）、`kivact-ui`（builder、recipe の表、`UiTokens` の入れ子の struct）、E2E 一つ、guide の en・jp |
| paint・layout の性質を一つ足す | 3 crate（`kivact-types`、`kivact-style`、`kivact-lower` か `kivact-layout`） |
| 地域書式を一つ足す | `kivact-i18n` の `Locale` のメソッド一つ（ICU4X を呼ぶ） |
| 振る舞いを一つ変える | その behavior の machine の遷移（`transition` 一つ）と E2E |
| 新しい OS | `arvane-native/port/<os>` を一つ（pipeline は `arvane-shell` で共有） |

作業手順書 12 章の試し（8 番）に、paint の性質と標準部品の追加を足し、crate の数に加えて file の数も記録する。

---

## 19. 実装の結果による改訂（2026-11-05）

W0・W1 を実装した結果と、W2 以降の各 unit を現在の code に当てた調べ（2026-11-05）から、設計の不足と、code の事実に合わない所を決め直した。**この章は 1〜18 章と付録より優先する**（優先の順は 19 > 18 > 17 > 1〜16 > 付録）。各節の冒頭に、置き換える節を書いた。作業手順書の unit の節は、この章の節を題で参照する。

主な変更:

- **turn の順序は core に残す**（19.17 節）。runtime は turn の状態・段・exit と部品だけを出す。`TurnClient` は作らない。
- **`kivact-lower` は engine の module にする**（19.9 節）。crate の数がさらに一つ減る。
- **`Backend` は既存の `ShellEventLoop` の改名**（19.1 節）。`arvane-virtual` は `SteppedBackend` だけを実装する。
- **振る舞いを保つための形の直し**: frame loop は同期のまま（19.18 節）、`Watcher` は比較しない（19.19 節）、pointer の経路は capture と press を別に持つ（19.13 節）、fragment は `Display::Contents` の箱（19.29 節）。
- **公開 API の名前の正は付録 B**（B.1〜B.6 が最終の表、B.10 が旧→新の一覧）。
- **APG への合わせは各 behavior の unit の中で行う**（19.40 節）。独立の unit はない。

### 19.0 そのほかの決定

（16.2 節・付録 B・3.4 節に足す）

- **trace の比較をやめる**（16.2 節の「振る舞いを変えない」の確かめ方を置き換える）: 振る舞いを保ったことの確かめは、既存の E2E が全部通ることだけで行う。`KIVACT_TRACE` の trace は、失敗した試験の原因を調べる道具として残す（作業手順書 0.6 節）。理由: f32→f64 の統一で全部の数値と型名が変わり、trace の厳密な一致は原理的に作れない（CP1 で 58,176 件の差が残った）。
- **API-15 の補足:** `InteractionPolicy` の入れ子の field の型（`MenuPolicy`、`ButtonPolicy` など、18.15 節）は `*Policy` の名前を使ってよい。
- **`calendar` の選択:** `calendar(selection: impl IntoBinding<CalendarSelection>)`。`CalendarSelection` は値の enum（一つの日付、範囲、複数）で、`IntoSelection`（集合の選択）は使わない。
- **`AppOptions::theme(impl IntoValue<Theme>)`**: `App::theme`（18.11 節）と同じ形。
- **`Disclosure::animated(bool)` は残す**: `UiTokens.disclosure.motion` の、一つの部品の上書き（19.37 節）。
- **軸の型は `Orientation`**: 出現と退場の大きさ（19.12 節）も `Orientation` を使う。`Axis` は作らない。
- **`ClientMode::Embedded(CloseSignal)`**（7.2 節の `ClientMode` を置き換える）: core は engine の `CloseSignal` を持ち、event ごとの中断を保つ（19.8 節）。`HostEvent::CloseRequested` による embed の exit の turn は作らない。

### 19.1 `Backend` は既存の `ShellEventLoop` の改名にする

既存の native/embed E2E のための `arvane` から `arvane-native` への dev-dependency は残す（通常依存は消す）。policy は dev-dependency も検査するので、この用途だけ `arvane` を consumer に含める。

（9.1 節の `Backend` trait の code、「A-0 の実コードとの照合による補足」、9.1 節の箇条の一つ目と二つ目、4.1 節の `kivact-engine` の行の依存 `arvane-virtual（feature test）` 以外を置き換える）

```rust
// arvane_shell
pub trait Backend {
    /// Run the event loop until the handler exits (blocks on desktop).
    fn run(self: Box<Self>, handler: &mut dyn ShellHandler) -> Result<(), ShellError>;
}
// arvane_native
pub fn backend(options: ShellOptions) -> Result<Box<dyn Backend>, ShellError>;
pub mod embed { pub fn open(parent: ParentWindow, options: EmbedOptions, hooks: ChildHooks, make: MakeHandler) -> Result<Box<dyn ShellChild>, ShellError>; }
// arvane
impl Shell { pub fn new(backend: Box<dyn Backend>) -> Shell; /* prefer_gpu, gpu_preference, on_renderer_change, run は今のまま */ }
pub mod embed {
    pub fn driver<H: Handler + 'static>(options: &EmbedOptions, make: impl FnOnce() -> Embedder<H> + Send + 'static) -> MakeHandler;
    impl Embedded { pub fn new(child: Box<dyn ShellChild>) -> Embedded; }
}
pub mod spi { pub use arvane_shell::{Backend, ShellWindow, ShellHandler, ShellError, ShellEvent, ShellChild, MakeHandler, ChildHooks, SteppedBackend, SteppedLoop, Step, StepOutcome}; }
```

- `Backend` は `capabilities`・`open_child`・`shutdown` を持たない。生成前の capability は誰も読まない（`HostInfo` は窓の `capabilities()` を読む、`kivact-engine/src/host/events.rs:94`）。`shutdown(&self)` は、embed の handler が backend より長く生きるので呼べない。module の保持は今も embed だけで、native が持つ（`arvane/src/shutdown.rs:8-15`、`arvane-native/src/embed/session.rs:99`）ので、`RetainOnDrop` を native の embed の `Session` へ移す（handler を drop した直後に `DetachedThreads::any()` なら `retain_module()`）。
- 同じ形の trait が既にある（`arvane-shell/src/window.rs:194` の `ShellEventLoop`）ので、新しい trait を作らずに改名する。`ShellOptions` は native の構築の引数にし、窓の大きさの検査を今の場所（native の構築）に保つ（`arvane/src/window.rs:347`）。
- embed は backend の trait に入れず、native の自由関数のまま engine が呼ぶ。外から backend を差し込む口を作らない（18.19 節）ので、embed の第二の実装はない。
- arvane-native の中の同名の `trait Backend`（`arvane-native/src/embed/child.rs:300`）は `ChildPort` と改名する。
- `VirtualBackend` は `SteppedBackend` だけを実装する（`Backend::run` を virtual で使う所がない）。

### 19.2 `arvane-virtual` の台本・読み出し・段

（9.3 節の code と最初の二つの箇条、9.1 節の `SteppedBackend`・`SteppedLoop`・`StepOutcome` の code、9.2 節の三つ目の箇条を置き換える）

```rust
// arvane_shell
pub trait SteppedBackend {
    /// Create the window and call `handler.start` before returning.
    fn start(self: Box<Self>, handler: Box<dyn ShellHandler>) -> Result<Box<dyn SteppedLoop>, ShellError>;
}
pub trait SteppedLoop { fn step(&mut self, step: Step) -> StepOutcome; }
pub enum Step { Input, Wake, Redraw }
pub enum StepOutcome { Idle { next_deadline: Option<u64>, redraw_requested: bool, woken: bool }, Exited, Failed(ShellError) }
pub trait ShellWindow { /* .. */ fn keeps_frames(&self) -> bool; fn keep_frame(&mut self, frame: CpuFrame); }
// arvane
impl Shell { pub fn start_stepped(backend: Box<dyn SteppedBackend>, setup: ShellSetup, handler: impl Handler + 'static) -> Result<SteppedShell, ShellError>; }
pub struct ShellSetup { pub prefer_gpu: bool, pub gpu_preference: GpuPreference, pub observer: Option<Observer> }
// arvane_virtual
pub struct VirtualOptions { pub title: String, pub size: Size, pub scale: f64, pub frame_period_ns: u64, pub reduced_motion: bool, pub dark: bool, pub locale: Option<String>, pub gpu: GpuMode, pub mirror: MirrorMode }
pub enum GpuMode { Off, Required }
pub enum MirrorMode { Off, Auto, Required }
impl VirtualBackend { pub fn new(options: VirtualOptions) -> Self; pub fn script(&self) -> Script; pub fn probe(&self) -> Probe; }
```

- `Script` は今の `VirtualWindow` の書き換えのメソッドを、`Probe` は読み出しのメソッドを同じ名前で持つ（`Probe` は所有の値を返す）。試験は窓の状態を読まずには書けない（`kivact-engine/src/host/testing/test_host.rs:388-653` が clipboard・frame・木などを読む）ので、書く口と読む口を分ける。
- `step(Step)` を三つに分ける: core の driver は turn・wake・redraw を別々に進めている（`kivact-core/src/test/screen/driver.rs:131-179`）。失敗は `StepOutcome::Failed` で返す（今の試験 host の `failure()`）。
- 試験 host は handler を `Rc<RefCell<..>>` で loop と共有し、node の問い合わせはその `Rc` から読む（今は host と窓を直接持つ: `test_host.rs:45-52`）。`inject(HostEvent)` は試験専用の engine の口として残す。
- 支援技術の公開は本番と同じく frame の後（`defers_accessibility` を消す、`kivact-engine/src/host/pump.rs:391`）。そのため core の driver は turn の後、redraw の要求があれば毎回描く（今の「見たときだけ描く」と `KIVACT_TEST_LIVE` をやめる）。
- GPU の試験も本番と同じ背景の起動を通る（試験だけの同期の起動 `State::Foreground` を消す、`arvane/src/presenter/gpu.rs:372-381`）。試験 host は renderer の通知が来るまで wake を回す（5 秒まで）。GPU の frame の screenshot は、`keeps_frames()` の窓にだけ presenter が read back して `keep_frame` で渡す。
- mirror の実 window と process に一つの UI スレッドは `arvane-native`（feature `test`、`arvane_native::mirror`）に残し、`arvane-virtual` の feature `mirror` は写しの adapter だけを持つ（17.3 節の表のとおり。winit を arvane-virtual に入れない）。
- `test_font()`・`gpu_unavailable()` は `arvane` の root へ移す（`Presenter` と text engine を使うため）。arvane は feature `test` を持たず、arvane-native に依存しない。
- arvane の e2e（33 試験、`crates/arvane/tests/e2e/support.rs:32` が窓を直接操作する）は、`Harness::with(|w: &mut arvane::Window| ..)`（handler の中で closure を走らせる）、`script()`、`probe()` で書き直す。

### 19.3 入力の pipeline の置き場所

（17.2 節の module の図と `RawInput` の箇条、18.18 節を置き換える）

```text
arvane-shell/src/pipeline/   raw（RawInput）、keys（KeyPipeline と VK・keysym・X keycode の表）、cursor（cursor の表）、ime（ImeDriver、ImePort）、dnd（DropSession）
arvane-native/src/
├── port/winit/   winit の型からの写像（Key・KeyCode・Win32 の scancode は winit の表を使う）
├── port/win32/   埋め込みの子 window
├── port/x11/     埋め込みの子 window（XIM の protocol と handshake を含む）
├── port/embed/   child・session・geometry（Win32 と X11 が共有）
├── present.rs    CPU の提示（三つの port とも softbuffer）
├── a11y/         AccessKit の conversion（今の共有のまま）
├── services.rs、display/、mirror.rs
```

```rust
pub enum RawInput { Key(RawKey), Ime(RawIme), Drop(RawDrop) }
pub struct RawKey { pub key: PlatformKey, pub pressed: bool, pub repeat: Option<bool>, pub text: Option<String> }
pub enum PlatformKey {
    Vk { vk: u16, chr: Option<char>, code: KeyCode },
    Keysym { keysym: u32, chr: Option<char>, keycode: u8 },
    Code { key: Key, code: KeyCode },
}
pub enum RawIme { Preedit { text: String, caret: Option<(usize, usize)> }, Commit(String), End, Reconvert { selection: (usize, usize) } }
pub trait ImePort { fn allow(&mut self, allowed: bool); fn cursor_area(&mut self, area: Rect); fn surrounding(&mut self, text: Option<&ShellImeText>); }
pub enum RawDrop { Hover { files: Vec<PathBuf>, at: Option<Point> }, Move(Point), Drop, Leave }
pub enum DropSession { Idle, Hovering { files: Vec<PathBuf>, at: Option<Point> }, Dropped { files: Vec<PathBuf>, at: Option<Point> } }
```

- arvane-shell は winit・softbuffer・AccessKit に依存しない（4.1 節）ので、`PlatformKey::Winit(KeyCode)`（17.2 節）、`present`、`a11y` は shell に置けない。winit の写像は port に残し、`PlatformKey::Code` で渡す。Win32 の物理の code は今も winit の scancode の表で作っている（`arvane-native/src/embed/windows/input.rs` の `keyboard`）ので、port が `code` を作って `Vk` に入れる。
- `repeat: None` は pipeline が押下中の集合から決める（今の X11 の規則）。Win32 と winit は今の値を `Some` で渡す（振る舞いを変えない）。
- policy: `[module_rules.arvane-shell.uses]` に `pipeline = ["events", "types"]`。`[module_rules.arvane-native.uses]` は `port = ["present", "a11y", "services", "display"]`、`mirror = ["port", "present", "a11y"]`、ほかは `[]`。
- 公開の E2E は `Screen::raw_input(RawInput)`（`kivact::test` の試験専用のメソッド。型は `kivact::test::raw` に再公開）から `Script::raw` を通す。

### 19.4 OS のサービスは窓の同期のメソッド一つ

（9.1 節の `Services` trait と `ShellWindow::services`、「A-2 の所有権の訂正」、9.1 節のサービスの箇条を置き換える）

```rust
pub trait ShellWindow {
    /// Every OS service (D-05), answered on the UI thread.
    fn perform(&mut self, request: PlatformRequest) -> PlatformResult;
    fn cpu_format(&self) -> CpuFormat;
    /* copy_text・read_text・pick_*・save_file・open_path・reveal_path・request_size・set_fullscreen は消える */
}
pub enum CpuFormat { Xrgb8888 }
// kivact_engine::contract
pub enum HostRequest { Platform(PlatformRequest), RegisterFont(Asset), ReducedMotion(Option<bool>) }
```

- 今の実装は全部同期で答える（native の `services.rs`、二つの embed、virtual）。非同期の reply が要るのは mobile・Web で、これらは対象外（16.3 節）。同期にすると、reply から `Host` と窓へ戻る経路と、結果の順の問題がなくなる（今の順は `host/pump.rs:132-178` の一回の loop のまま）。window 一つに一つの service という A-2 の所有の考えは、窓のメソッドにすることでそのまま満たされる。
- `RegisterFont`・`ReducedMotion` は OS の request ではないので `PlatformRequest` から外し、engine の契約 `HostRequest` で core から渡す（`Writer::effect(request, HostRequest)`）。
- `Capabilities` の追加の field は 9.1 節のとおり。`CommandKey` の既定は `Ctrl`、`ContextMenuTrigger` の既定は `Release`（`Capabilities` は `Default` を derive している）。
- virtual は今の既定の振る舞い（memory の clipboard、dialog の queue、記録）を持ち、`Script::respond(f: impl FnMut(&PlatformRequest) -> Option<PlatformResult> + 'static)` で上書きする。種類の enum（`PlatformRequestKind`）は作らない。

### 19.5 embed の子 window の lifecycle は thread ごとに二つ

（9.4 節の一つ目と二つ目の箇条、付録 A の SM-A01・SM-A02 の「解消後」を置き換える）

```rust
// thread をまたぐ（Shared の Mutex の中。host の thread が failure() で読む）
enum ChildStatus { Running, Stopped { cause: Stop, window: NativeWindow } }
// UI スレッドだけ（embed の Session）
enum ChildLifecycle { Opening(HandlerSlot), Live(HandlerSlot), Disposed, Leaked }
enum HandlerSlot { Unstarted, Running { handler: Box<dyn ShellHandler>, control: ShellControl } }
// Win32 の再入: 二つの guard は重なる
struct ReentryGuard(Cell<Held>);
struct DrainGuard(Cell<Held>);
enum Held { Free, Held }
```

- `Live { handler }` を一つの enum に入れると、`Send` でない handler（`arvane-native/src/embed/session.rs:13`）が host の thread と共有する `Mutex` の中（`embed/child.rs:44`）に入ってしまう。
- Win32 では command の drain の中で event が callback を走らせる（`embed/windows/window.rs` の `commands` が `self.event(ShellEvent::Resized)` を呼ぶ）ので、`Reentry::{Idle, InCallback, Draining}` の一つの enum では表せない。
- X11 の `exists: AtomicBool` は thread の間の guard なので `ConnectionAlive(AtomicBool)` に名前を付けるだけにする。

### 19.6 surface の失敗の数え方

（付録 A の SM-A11 の「解消後」の一つ目の文を置き換える）

```rust
enum FrameHealth { Healthy, Skipping { frames: NonZeroU32 }, Losing { losses: NonZeroU8 } }
enum LossAction { Recover, FallbackToCpu, Fail }
impl FrameHealth { fn lost(&mut self, renderer: Renderer) -> LossAction; }
```

- 続けての失敗が GPU で 2 回なら CPU へ、どの renderer でも 3 回なら error（今の二つの閾値: `arvane/src/presenter/gpu.rs:63` の `SURFACE_FAILURES = 2`、`presenter/health.rs:5` の `LOSSES = 3`）。数え直しは `shown`（と `skipped`）だけ。今は GPU の数は GPU の成功でだけ戻る（`gpu.rs` の `present` の `*failures = 0`）ので、「GPU の失敗の間の skip が数を戻す」点だけが変わる。この差は受け入れる。

### 19.7 GPU の型は feature がなくても同じ名前で出す

（17.3 節に足す）

- `arvane-render` は feature `gpu` がなくても `GpuContext`・`GpuSurface`・`GpuEnv` を公開する。feature がないときは `pub enum GpuContext {}`・`pub enum GpuSurface {}`（値がない型。メソッドは `match *self {}`）で、`GpuContext::new` は `Err(RenderError::NoAdapter(..))`。`pub const GPU_BUILT: bool`。外部の texture の import も、feature がないときは `Unsupported` を返す。arvane が使う arvane-render の GPU の item はこの三つだけ（`arvane_render::{GpuContext, GpuSurface, GpuEnv}`）なので、arvane の 28 か所の `cfg(feature = "gpu")` を stub の型なしに消せる。
- `kivact-core` の feature `gpu` を消す（core は公開 API の `gpu-interop`・`embed` だけを持つ）。core の GPU の試験は、workspace で facade の `gpu` と同時に build されるときに GPU を使う（core だけを build したときは `gpu_unavailable()` で skip する）。
- GPU import の未対応は render の既存 `RenderError::External`、arvane の公開境界では既存 `FailureKind::Unavailable` と具体的理由で示す。`FailureKind::Unsupported` は作らない。
- `anchor-deps` の検査は作らない（18.20 節）。

### 19.8 embed の閉じる要求は exit の turn にしない

（8.2 節の `CloseSignal` の箇条、`HostPhase` の code、`SurfaceLost` の箇条、付録 A の SM-E02・SM-E03 と SM-C の `ClientMode::Embedded` の項の「閉じる要求は…`Runtime::request_exit` を呼ぶ」を置き換える）

```rust
pub struct CloseSignal(Arc<AtomicBool>);          // kivact_engine::contract。Clone、new、close、is_set
enum HostPhase { Starting, Running, Exiting, Closed }
enum AppPresence { Foreground, Suspended }
enum Surface { Drawable { generation: u64 }, Unavailable(Unavailable) }
enum Unavailable { NoSurface, ZeroSize }
enum LastPresent { Settled, Skipped { retry_at_ns: u64 } }
enum Launch { Pending(Box<LaunchArgs>), Starting, Running(Box<Host>) }   // arvane::Handler の実装。start で Model を本物の store で作る
```

- `Host` は `phase`・`presence`・`surface`・`last_present` を並べて持つ（`presence` と `surface` はどの phase でも事実）。`drawable()` はこれらから計算する唯一の関数。
- 閉じる flag は host の thread から（X11 は別の thread、Win32 は再入の dialog の中から）いつでも立つ。今の規則は「flag が立ったら、新しい turn と効果を始めない。走っている turn は今の規則で終わる」（`kivact-engine/src/host/pump.rs:58-61` の comment）で、core も event ごとに中断する（`kivact-core/src/client/app_client.rs:208`）。pump の入口で一度だけ読むと、この規則が崩れ、embedder が drop した後に App の code が turn と exit の turn を走らせる。そこで `Host::observe_close()` を入口と、client の呼び出しの後と、効果ごとの後で呼び、立っていれば `Closed` へ進んで何もしない（今の振る舞い）。`HostEvent::CloseRequested` と、そのための `Runtime::request_exit` の呼び出しは作らない。core は `Option<CloseSignal>` を持ち、event ごとの中断を保つ。
- `Presented::SurfaceLost` は窓が既に surface を作り直した後の報告なので、状態を持たずに `HostFailure` を一度積むだけにする（`set_surface(false)`→`(true)` の往復を消す）。`Unavailable::Lost` は作らない。
- 起動: asset の store は shell の waker を要し、waker は `start` でしかない（`host/events.rs:55-57`）。`Launch` の enum が `start` で `Host` と `Model` を作るので、`AssetStore::inline()` で作って後で差し替える処理（`host/launch.rs:62`、`ArvaneAssets::replace_store`）が消える。
- `Launch::Starting` は mutable な `start` から Pending の引数を移して Host を構築する間の状態。dummy Client/Engine を使わず、Starting の event は invariant 違反。環境の前回送信値は `Sent::get(&self) -> Option<&T>` で読み、theme override と reduced motion の挙動を保つ。

### 19.9 `kivact-lower` を engine の module にする

（8.3 節、18.17 節、4.1 節の `kivact-lower` の行、4.3 節の `kivact-lower` の行、8.1 節の構成の図、4.4 節の `[module_rules.kivact-engine.uses]` を置き換える。付録 A の SM-E33 の「`LowerReport` を呼び出しごとに返す」はこの節の形で行う）

- **4.1 節:** `kivact-lower` の行を消す。`kivact-engine` の行の状態を「拡張（`kivact-text` と `kivact-lower` を統合、`contract` を追加）」、依存から `kivact-lower` を消す。`arvane` の行の `visible_to` を `kivact-engine` だけにする。「消える crate」に `kivact-lower`（`kivact-engine::lower`）を足す。
- **4.3 節:** `kivact-lower` の行を「engine へ統合 | 使うのは engine だけで、engine は lower の 12 種類の item を `tree`・`measure`・`model`・`paint`・`check`・`text_measure`・`offscreen` から使う（文字の測定の `glyphs::shape_run`・`measure_run`、当たり判定の `shape::hits`、`sanitize::paint`、ink の `filter_reach`・`backdrop_reach`、font の `text_style`・`family`）。`Lowerer` の三つのメソッド（8.3 節）ではこの使い方を覆えず、`paint/frame_scene.rs` は layers・backdrop・border・影を自分の木の walk の中で順に呼ぶ。境界は module の向きで守る」にする。
- **8.1 節:** 図に `lower`（解決済みの style・`DrawList`・text の run を render scene へ下ろす。木・入力を知らない）を足す。
- **形:** `crates/kivact-lower/src` → `kivact-engine/src/lower/`。中の `pub` は `pub(crate)`。latch は、`Context` の dropped・failures を外から読む getter を消し、`Context::finish(self) -> LowerReport`（`LowerReport { dropped: u32, failures: Vec<Failure> }`）で一回の lowering の報告として返す。Context の中の `Cell`・`RefCell` は一回の lowering の範囲の値なので残す（今の型は既に `failures: RefCell<Vec<Failure>>`、`kivact-lower/src/context.rs:51-56`）。`ArvaneAssets`・`AssetEvent`・`BoxShape`・`shape` は `lower` の中に残す（`BoxShape` を `kivact-types` へ移さない）。
- **4.4 節の engine の向き:** 今の行に `lower = []`、`text_input = []` を足し、`tree`・`measure`・`model`・`text_measure`・`paint`・`check`・`offscreen` に `"lower"`、`measure`・`model`・`host` に `"text_input"` を足す。lower を `paint` の下に置かないのは、`tree` と `model` が lower を使い、`paint` を使えない向きだから。

### 19.10 layout は一つの持ち主、型状態の pass は作らない

（8.4 節、付録 A の SM-E27・SM-E28 の「解消後」を置き換える）

```rust
// kivact_layout
pub struct Layout { /* NodeLayoutColumns と LayoutState（どちらも crate の中だけ） */ }
impl Layout {
    pub fn push(&mut self); pub fn reset(&mut self, slot: usize);
    pub fn rebuild_style<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId); pub fn rebuild_all<T: LayoutTree + ?Sized>(&mut self, tree: &T);
    pub fn topology_changed(&mut self); pub fn invalidate_chain<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId);
    pub fn compute<T: LayoutTree + ?Sized>(&mut self, tree: &T, available: Size, measure: &mut dyn LeafMeasurer) -> bool;  // 変化があったときだけ計算する
    pub fn compute_overlay<T: LayoutTree + ?Sized>(&mut self, tree: &T, id: NodeId, available: Size, limit: Option<Size>, measure: &mut dyn LeafMeasurer);
    pub fn drain_geometry_changes(&mut self) -> GeometryChanges<'_>;
    pub fn set_presence_scale(&mut self, slot: usize, scale: Option<(Orientation, f64)>);   // 19.12 節
}
```

- 8.4 節の `LayoutState::edit()` などは、今の `push`・`rebuild_style`・`compute`・`drain` が `LayoutState` ではなく `NodeLayoutColumns` のメソッドである（`kivact-layout/src/lib.rs:11-23`）ことに合わない。また engine は「変化がなくても tooltip を合わせ直し（`compute_overlay`）、そのときも geometry を drain する」（`kivact-engine/src/model/interact/geometry.rs:69-92`）ので、`Edit → Pending → Computed` の一本の順では表せない。呼び出し順の規則の誤りは「columns と state を別々に渡す」と「`take_changed` を呼び忘れる」の二つなので、二つを一つの持ち主にし、`take_changed` を `compute` の中へ入れれば型で防げる。
- engine の `Changes` の `needs_layout`・`layout_changed`・`viewport_chain` は `LayoutReasons`（bit の集合: `PASS`・`STYLE`・`VIEWPORT`）にする。`Computed(GeometryChanges)` の状態は作らない（geometry はその場で drain する）。
- `Model::settle()` は今の `relayout` の順（sync → layout → lint → sync）を持つ唯一の関数。入力の時点の幾何が要る所（hit test の前、file drop、wheel、redraw）はそれぞれ `settle()` を呼ぶ（`host/events.rs:141-150`。何もなければすぐ返る）。
- engine が必要とする既存の geometry/query/diagnostic は Layout に移す。旧二型は crate private、`Settled` は private の Kept/Updated で pass の実施を表す。

### 19.11 `Display`

（17.1 節の `Style::display(Display::Contents)` の箇条に足す）

```rust
// kivact_types::layout
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display { #[default] Flex, Grid, Contents }
// kivact_style
impl Style { pub fn display(self, display: Display) -> Style; }   // display_grid() を置き換える
```

- 今は display の型がない（`LayoutProps.hidden: bool`、`kivact-types/src/layout.rs:413`、grid は `Style::display_grid()`、`kivact-style/src/sheet.rs:914`、と grid の container の有無）。`grid_box: Option<bool>` を `display: Option<Display>` にし、解決済みの `LayoutProps` に `display: Display` を足す。`hidden`（display: none）は表示の切り替えなので別に残す。`Contents` は engine の `flat_children` が平らにするので Taffy に届かない。`Display` は prelude に入れない（`std::fmt::Display` との衝突を避ける）。

### 19.12 出現と退場の大きさは layout の hook で倍にする

（17.10 節の `Style::enter_size`・`exit_size` の箇条に足す）

- 軸は既存の `Orientation` を使う（`Axis` は作らない。付録 B.1）。`TransitionProp::Size` を足す（今の `TransitionProp` は `Paint` だけ、`kivact-style/src/sheet.rs:150`）。
- 「二度目の layout をせず、子は自然な大きさ、親は倍にした大きさ」は、`kivact-layout` の `compute_cached_layout` の hook（`kivact-layout/src/tree.rs:207`）で、その node の出力の大きさの軸だけに倍率を掛けて実現する（倍率を cache の依存に含め、変更時の無効化でもよい）。倍率は engine の presence の遷移が毎 frame `Layout::set_presence_scale` で渡す。`ExitLayout::Pop` の node では `exit_size` を無視する（退場の始めに場所を手放すため）。
- 公開の値: `kivact_types::Presence { Entering, Present, Leaving }`（正準パス `kivact::view::Presence`）。engine は `HostEvent::PresenceChanged { node, presence }` と `Reader::presence(id) -> Presence` を出し、core の `NodeRef::presence() -> Value<Presence>` は最初の呼び出しで signal を作る。
- `NodePresence::Arriving(ArrivalPhase)` は private な `Pending`（未開始）と `Animating`（出現中）を区別する。公開値は両方 Entering。arriving の索引は一つだけとし、出現完了で Mounted にする。
- Taffy が親の計算で直接読む item style の size/min/max/basis にも倍率を反映する private adapter を使う。子を計算する natural style は保持し、自然出力の cache から親の known size を復元する。二度目の layout は行わない。

### 19.13 pointer の経路と modal の受け入れ

（8.5 節の pointer の箇条、付録 A の SM-E10・SM-E15 の「解消後」を置き換える）

```rust
struct Route { capture: Option<NodeId>, press: Option<Press> }
enum MouseGesture { Route(Route), Scrollbar(ScrollDrag), SelectText { editor: NodeId, route: Route } }
enum TouchGesture { Route(Route), Pending { route: Route, candidate: TouchCandidate }, Pan(TouchPan) }
struct KeyDownTargets(HashMap<KeyCode, FocusTarget>);
```

- `Route` の `capture` と `press` は独立の事実で、四つの組み合わせが全部ある（副 button の押下は capture だけ、listener のない所の押下は press だけ）。touch の候補は両方と同時にある（`kivact-engine/src/model/pointer/state.rs:114-122`）。SM-E10 の enum ではこれが表せないので、`Route` は struct のまま、touch の候補だけを `TouchGesture::Pending` に移す。
- `Admission`（`model/modal/state.rs:17-29`）は client の dispatch の順で問われる表で、`Track` は入力の時点の状態なので、`Track` へ入れない（SM-E15 は行わない）。
- `key_up` の宛先がないとき（記録がない、`KeyCode::Unidentified`、node がもうない）は、今と同じく focus のある node へ送る。
- `HostEvent::HoverChanged { target: Option<FocusTarget> }`（canvas の region の hover を運ぶ）。
- `Hover` は primary の chain だけを持ち、各 Track の `over` が hit target を一つだけ保存する。secondary touch/pen の Enter/Leave/非 capture Move の既存配送も保つ。

### 19.14 付録 A の直し

（付録 A に足す）

- SM-A01・A02 → 19.5 節。SM-A11 の数え方 → 19.6 節。SM-A13 の「shell は検証と適用だけ」は「`AcceptedTree` を残し、差分の計算だけをやめる」（支援技術の起動の木と action の検査に要る、`arvane-shell/src/a11y/shared.rs:62-80`）。SM-A16 → 19.1 節（native の embed の `Session`）。
- SM-E02・E03 → 19.8 節。SM-E10・E15 → 19.13 節。SM-E27・E28 → 19.10 節。SM-E01 の `soft_keyboard` は SM-E22（E-4）だけが扱う。SM-E18 は A-2 で済む（19.4 節）。

### 19.15 `kivact-runtime` の module の向き

（4.4 節の `[module_rules.kivact-runtime.uses]` を置き換える）

```toml
[module_rules.kivact-runtime.uses]
reactive = []                      # 旧 kivact-reactive。handler（HandlerResult, Handled, Handler<E>）と、Host, ThreadWaker, TurnState, Stage, ExitState, TurnReport, TurnGuard の型もここ
task = ["reactive"]                # 旧 kivact-task。timer の heap（Scheduler）もここ
motion = ["reactive"]              # 旧 kivact-motion。FrameClock, LoopState, AnimPhase
machine = ["reactive", "task"]     # Machine, Step, machine, MachineHandle, MachineCx
host = ["reactive", "task", "motion"]  # RuntimeConfig, Runtime::new, TurnGuard の settle_async・check_draining・plan_frame・finish（inherent impl）
```

- `handler`・`turn`・`time` の module は作らない。理由: reactive の `callback`・`binding` が既に `HandlerResult` を使う（`kivact-reactive/src/setup.rs:72,120`）ので、`handler = ["reactive"]` では循環する。`Runtime` は reactive の中の struct（`kivact-reactive/src/data.rs:47-80`）で、host・turn の状態を field に持つので、その型も reactive に置く。motion は task を使わない（現在の motion の src に `kivact_task` の import はない）。
- `Runtime` は一つ。上の層の処理（`Runtime::new`、`TurnGuard` の一部）は `host` の module に `impl Runtime` / `impl TurnGuard<'_>` として書く（同じ crate なので書ける）。

### 19.16 host との境界と構築

（6.1 節を置き換える）

```rust
// kivact_runtime（定義は reactive、root から再公開）
/// What the runtime needs from whoever runs it (kivact-core's client).
pub trait Host: 'static {
    /// Ask for a turn soon. Called only on the `TurnState::Quiet → WakeSent` edge.
    fn wake(&self);
}
/// A wake usable from any thread (executor, blocking jobs). Never touches `TurnState`.
#[derive(Clone)]
pub struct ThreadWaker(Arc<dyn Fn() + Send + Sync>);
impl ThreadWaker {
    pub fn new(f: impl Fn() + Send + Sync + 'static) -> Self;
    pub fn wake(&self);
}

// kivact_runtime（定義は host）
pub struct RuntimeConfig {
    /// The host clock when the App starts (`HostInfo::now_ns`).
    pub start_ns: u64,
    /// Workers of the blocking pool (`None`: `min(4, CPUs)`).
    pub blocking_threads: Option<usize>,
    /// Where the App was started (`App::run`, `test::render`): the root owner's location.
    pub at: &'static Location<'static>,
}
impl Runtime {
    /// Registers the App, creates the root owner, runs `build` as setup of the root
    /// owner, installs the returned extensions (and the runtime's own: task, motion),
    /// and returns. `ext()` called before `build` returned is a misuse panic.
    pub fn new(
        host: Rc<dyn Host>,
        waker: ThreadWaker,
        config: RuntimeConfig,
        build: impl FnOnce(&Rc<Runtime>, Owner) -> Extensions,
    ) -> Rc<Runtime>;
    pub fn root(&self) -> Owner;                 // Option ではない
    pub fn now_ns(&self) -> u64;                 // runtime の時計（clock_now も同じ値）
    /// Move the clock forward to `ns` (a turn's time, a `FrameTick`); it never goes back.
    pub fn advance_clock(&self, ns: u64);
    /// The host asked to close: `Running → Draining { now + DEFAULT_DRAIN_NS }`; later calls keep the deadline.
    pub fn request_exit(&self);
}
```

- 構築の順: (1) registry に `Rc<Runtime>` を登録する（`TurnState::Running(Stage::Setup)`、`ext` は空の内部の `OnceCell<Extensions>`）。(2) root owner を作る。(3) root の setup の frame で `build(&rt, root)` を呼ぶ。(4) `build` の戻り値に runtime 自身の entry（`TaskState::new(blocking_threads, waker)`、`MotionState`）を足して `ext` に入れる。(5) `TurnState::Quiet` にして返す。`build` が panic したら registry から外す。
- `ext` の `OnceCell` は `Runtime::new` が返る前に必ず埋まり、外から空の状態は見えない（S-7 に反しない）。`build` の中で signal・trigger・selector（root の下）は作れる。理由: signal・trigger・selector の作成は registry と arena だけを使い、`ext` を読まない（`kivact-reactive/src/owner_handle.rs:140-155`、`selector.rs` の Internal effect は `try_runtime` と source だけを読む）。
- `Driver` trait、`driver: OnceCell`、`root_owner: Cell<Option<Key>>`、`turn_requested: Cell<bool>`、core の `HostLink`、`TaskState::set_waker` は消える。core の `ClientHost(HostWaker)` が `Host` を実装する。`ThreadWaker` も同じ `HostWaker` を呼ぶ。
- 時刻の持ち主は runtime の時計一つ（`RuntimeConfig.start_ns` で始まり、`advance_clock` で進む）。理由: 現在 root の setup が既に host の時刻を読み（`kivact-core/src/client/app_client.rs:236`）、`FrameTick` が batch の途中で時計を進める（`client/dispatch/env.rs:35`）ので、turn の入力一つでは運べない。
- runtime は `Launch::Started` を作るとき（core の `start` の中）に作る。

### 19.17 turn の順序は core の一つの関数が持つ

（6.2 節を置き換える）

turn の順序は `kivact-core` の `AppClient::turn` の一つの関数の上から下への並びが持つ（`client/phases.rs`・`client/settle.rs` はその関数と runtime に吸収して消す）。runtime は turn の状態（`TurnState`・`Stage`）・mount・報告・async の settle・exit を部品として出す。理由: 現在の turn は core だけが知るものと交互に進む。要求された activation と、その後の条件付きの stabilize と settle（`phases.rs:57-63`）、`before`/`done` の revision（`app_client.rs:296,312`）、batch 全体の `TextEditsAhead::count`（`:192`）、embed の close の打ち切り（`:208`）、commit の中の frame → focus → 削除 → commit → mount の stamp の順（`phases.rs:73-104`）、setup の turn（`:228-273`）。`TurnClient` の二つのメソッドでは表せず、hook を足すと順序が二つの crate に分かれる。

```rust
// kivact_runtime（定義は reactive、root から再公開）
pub enum Stage { Setup, Begin, Reports, Dispatch, Async, Commit, Mounts, UserEffects, Output }
enum TurnState { Quiet, WakeSent, Running(Stage) }       // 非公開
pub enum ExitState { Running, Draining { deadline: u64 }, Exited }
pub struct TurnReport {
    /// The earliest timer or drain deadline (ns).
    pub next_deadline: Option<u64>,
    pub exit: ExitState,
}
#[must_use] pub struct TurnGuard<'a> { /* &Runtime, 記録中の turn */ }

impl Runtime {
    pub fn begin_setup(&self) -> TurnGuard<'_>;               // Quiet → Running(Setup)
    pub fn begin_turn(&self, events: usize) -> TurnGuard<'_>; // Quiet|WakeSent → Running(Begin)
    /// Exit transition and deadline without a turn (the early return of a batch with nothing to do).
    pub fn report_now(&self) -> TurnReport;
    pub fn mounts_due(&self, acked: Revision) -> bool;
    pub fn pending(&self) -> Pending;                          // turn_requested = (TurnState == WakeSent)
}
impl TurnGuard<'_> {
    pub fn stage(&self, stage: Stage);              // core が Dispatch・Commit を記す
    pub fn cause(&self, cause: String);
    pub fn stabilize(&self);                        // Stage を変えない
    pub fn deliver_reports(&self);                  // Stage::Reports。始めの snapshot だけ
    pub fn run_mounts(&self, acked: Revision);      // Stage::Mounts
    pub fn stamp_mounts(&self, revision: Revision); // Stage::Commit の中だけ（ほかは misuse）
    pub fn check_draining(&self);                   // Stage::Async
    pub fn settle_async(&self);                     // Stage::Async。1000 round
    pub fn plan_frame(&self) -> Option<u64>;        // Writer::request_frame に渡す時刻
    pub fn run_user_effects(&self) -> bool;         // Stage::UserEffects。一 round
    pub fn abandon_user_effects(&self) -> Vec<NodeId>;
    /// Stage::Output: exit transition, deadline, the turn record; then
    /// WakeSent (and `Host::wake`) if user effects, ready tasks, later reports
    /// or `client_pending` remain, else Quiet.
    pub fn finish(self, ops: usize, client_pending: bool) -> TurnReport;
    /// Setup: WakeSent if user effects or ready tasks remain, else Quiet.
    pub fn finish_setup(self);
}
```

- **misuse は三つだけ:** turn の入れ子（`Running` の間の `begin_turn`・`begin_setup`）、`Stage::Commit` の外の `stamp_mounts`、`Runtime::new` の `build` の中の `ext()`。段階ごとの signal の書き込みの禁止は作らない。理由: commit の中の `flush_focus` が失敗した focus を `rt.report` に送り、利用者の `on_error` が走って書き込む（`kivact-core/src/view/requests.rs:64-77,180-188`）。
- `RunningRequested` はない。turn の中の要求は wake しない（現在の `client/link.rs:73-82` と同じ）。turn の後に wake するかは `finish` の pending の判定だけで決まる（`app_client.rs:366-376`）。
- `wants_frame` はない。frame の要求は時刻を持ち、revision の write でもある（`Writer::request_frame` は write の command を足す。`kivact-engine/src/host/write/writer.rs:315-319`）ので、core が commit の中で `plan_frame()` の値を `request_frame` に渡す。`TurnInput`・`TurnItem`・`TurnClient` は作らない。`FrameTick` は今までどおり batch の中の順序のある event。
- `on_mount` の待ちは二つの型付きの queue（`unstamped: Vec<Mount>`、`awaiting: VecDeque<(Revision, Mount)>`、SM-R05）。後の報告は `deliver_reports` が届ける（SM-R06）。`ExitPhase` は runtime の `ExitState` になり、遷移は `request_exit` と `finish`・`report_now` だけ（SM-C02）。
- core の `AppClient::turn` の並び（この順を変えない）:
  1. `advance_clock(now)`。event がなく、最後の boundary で mount も出ないなら `report_now()` を返す。
  2. `before` を求め、`begin_turn`、`run_mounts(before)`、`deliver_reports`、`stabilize`。
  3. event ごとに（embed の close なら打ち切り）`stage(Dispatch)`・dispatch・要求された activation（あれば `stabilize`）・`stabilize`・boundary ごとに `run_mounts` と `stabilize`。最後に残りの boundary を全部。
  4. `done` を一度だけ読む。`check_draining`、`settle_async`。
  5. loop: activation（あれば `stabilize` と `settle_async`）→ commit（`stage(Commit)`、`plan_frame` → `request_frame`、focus、削除と asset の失敗の掃除、`Writer::commit`、`stamp_mounts`）→ `mounts_due(done)` も user effect もなければ抜ける → 100 round を越えたら `abandon_user_effects` と `Cycle` の報告で抜ける → `run_mounts(done)`・`stabilize`・`run_user_effects`・`stabilize`・`settle_async`。
  6. `finish(writes, has_activations)`。

### 19.18 frame と animation の時間

（6.3 節の表の `MotionState.frames`・`FrameLoop`・`Animated` の行と、`TurnReport.wants_frame` の文を置き換える）

```rust
// kivact_runtime::motion（非公開）
struct FrameClock { phase: FramePhase, offset: ClockOffset, ticks: u64 }
enum FramePhase {
    Idle,
    Requested { at: u64, last_tick: Option<u64> },
    Ticked { last: u64 },
}
enum LoopState { Stopped, Running(LoopRun) }   // LoopCell { owner, f: Handler<FrameInfo>, state: Signal<LoopState>, frames: Cell<u64> }
pub(crate) enum AnimPhase { Idle, Running { start_ns: u64 }, Paused { start_ns: u64, at_ns: u64 }, Completed }

// kivact_runtime（公開。core が build の中で Extensions に入れる）
#[derive(Clone, Copy)]
pub struct MotionEnv { pub reduced_motion: Value<bool>, pub surface_active: Value<bool>, pub frame_period_ns: Value<u64> }
```

- frame loop は今までどおり `FrameTick` の event の中で同期に走る（`MotionState::on_frame_tick`）。`next_frame()` と「`next_frame().await` を繰り返す task」は作らない。理由: task にすると loop の callback が tick の位置（`kivact-motion/src/state.rs:179-221`）から batch の後の async の settle へ移り、順序が変わる。
- `FramePhase` は要求と最後の tick を一緒に持つ。理由: 要求中も前の tick の時刻を読む（`state.rs:239,248`）。
- `MotionEnv` は core が `Runtime::new` の `build` の中で（環境の signal を作った後で）`Extensions` に入れ、motion は `ext::<MotionEnv>()` で読む。`attach_env`、`env: Cell<Option<MotionEnv>>`、`RuntimeConfig.reduced_motion`、`Runtime::set_reduced_motion` は作らない・消す。理由: `MotionEnv` は reduced motion のほかに surface と frame の周期も読み（`state.rs:41-48`）、`set_reduced_motion` は `Environment.reduced_motion` の写しになる（S-4）。reduced motion が有効になったときの `reduce_motion()` の呼び出しは今のまま。
- `Animated` は `phase: Signal<AnimPhase>` 一つ（`running: Signal<bool>` と `phase: Cell` と `completed: Signal<u64>` を消す）。`is_running()` は `matches!(phase, Running { .. })` の memo。自然に終わった run は `Completed`（止まっている状態。`Idle` と同じく `to` にある）へ移る。`on_complete` は `phase` の Internal の Watcher（19.x の reactive）で、見た値が `Completed` のときに callback を handler の frame で呼ぶ。理由: 遷移の中で callback を呼ぶと、利用者の code が `rt.enter` の中の step の loop で走る（`state.rs:222-229`）。現在の通知は stabilize の中の Internal effect（`kivact-motion/src/animated.rs:368-392`）。同じ stabilize の中で完了と再開の両方が起きたら通知しない（現在は通知する。記録して受け入れる）。
- 初回の skip（`first_cell`、`animated.rs:262-270`）は Watcher（`explicit_start` のときは普通の Internal effect）。
- timer（`Sleep`）: `SleepState { Unregistered { deadline }, Registered { id }, Fired }`、`Drop` は `CancelQueue` に id を積む。`CancelQueue` は timer の heap を読むところ（`take_due`、`next_deadline`）の前に必ず空にする。理由: 消した timer が締切に残ると余分な wake と turn になる（`kivact-task/src/state.rs:120-124`）。

### 19.19 reactive の frame と Watcher

（6.4 節の `Ctx`→`Frame` の項と `Watcher` の項を置き換える）

```rust
struct Frame { owner: Option<Key>, kind: FrameKind, tracking: Option<CollectorSlot>, pure: bool, writer: Option<Key> }
enum FrameKind { Handler, Setup(Key), Computation { node: Key, effect: bool } }
```

- `pure_section`・`untracked_frame`・`callback_frame` は親の frame の `kind` を写し、`pure`・`tracking`・`owner` だけを変える。理由: 現在この三つは親の文脈を受け継ぎ（`kivact-reactive/src/frame.rs:68-89` の `..self.context()`）、setup の中で呼んだ callback は setup の関数を使え、effect の中の untracked は `on_cleanup` の effect を保つ。一つの enum の variant にすると失われる。
- `Watcher`（graph の node の種類）は、自分の queue（`effect_on` は User、`Animated` は Internal）の最初の実行で source を pure・tracking で評価して依存を取るだけで `f` を呼ばない。以後の再実行のたびに source を評価して `f` を untracked で呼ぶ。前の値との比較はしない。理由: `effect_on<T: Clone>` は `PartialEq` を求めず（`setup.rs:166`）、生成時に評価すると初期値が最初の user の round の値から setup の時の値に変わる。
- `FrameKind`・pure の中の書き込みの misuse は既にある（`ErrorKind::WriteInPureContext`、`graph.rs:252`）。変えない。

### 19.20 非同期の状態

（6.6 節を置き換える）

```rust
struct AsyncCell<T> { state: Signal<AsyncState<T>>, latest: Signal<Option<T>> }
impl<T: Clone> AsyncCell<T> {
    fn start(&self);                          // state = Pending
    fn finish(&self, result: Result<T, UiError>); // Ready なら latest も書く。Failed は latest を保つ
    fn reset(&self);                          // Idle
}
enum Latest { Idle, Running { task: Owned<TaskCell>, generation: Generation } }
enum ActionPhase<I> { Open { worker: Worker, waiting: VecDeque<Submitted<I>> }, Closed { worker: Worker, waiting: VecDeque<Submitted<I>> }, Detached(Drain) }
enum PoolState { NotStarted, Running(Arc<Pool>), ShutDown }
```

- `AsyncCell` は二つの signal を持ち、書くのは上の三つのメソッドだけ。`state()`・`latest()` は memo にしない。理由: memo は `PartialEq` を求めるが、`Resource`・`Action` の `T` は `Clone` だけ（`kivact-task/src/resource.rs:47-52`）。一つの signal にすると `latest` の読み手が state の変化のたびに再実行される。
- queue に待機仕事が残れば、完了が失敗でも公開 state と完了 handler 内の読み出しは `Pending` を保つ。個別の失敗は completion の結果で通知し、`latest` は直前の成功値を保つ。旧 failure 保持の rustdoc・E2E より新版の Pending と busy の一致を優先する。
- `Drain` は `Runner::Queue`・`Runner::All` からだけ作れる型。`queues` の Weak の表は `kivact_types::Registry<dyn Drain>`。
- `PoolState::ShutDown` の後の `blocking` は invariant の panic（`expect("invariant: no App code runs after the blocking pool shut down")`）。`ErrorKind::Shutdown` は足さない。理由: `blocking` は `impl Future<Output = R>` を返し（`blocking.rs:187-193`）`UiError` を返せない。pool の shutdown は `AppClient::drop` が root の破棄と task の最後の poll の後に呼ぶので、App の code から届かない。
- `TaskCell.task: OnceCell` は残す（SM-R17 は行わない）。理由: task の future は owner に保存した後の object の key を持つ必要があり（`spawn.rs:48-62`）、key は cell を保存して初めてできる。cell は同じ関数の中で最初の poll の前に埋まり、空の状態は外から見えない。

### 19.21 App 全体の状態

（6.7 節の `use_app_state` の文、17.11 節の `AppStates`、18.21 節の「App 全体の型の map を一つに」を置き換える）

- App 全体の状態の置き場所は二つ: `Extensions`（`Runtime::new` の `build` が全部入れ、以後変わらない）と、root の owner の文脈（`use_app_state::<T>(init)`。最初の呼び出しで root の setup として `init` を実行し、root に `provide` する。現在の `app_state` と同じ、`kivact-core/src/view/owner.rs:147-155`）。`AppStates` は作らない。理由: `ext() -> &T` は変わらない storage を前提にし（`kivact-reactive/src/ext.rs:1-9`）、`#![forbid(unsafe_code)]` の下で後から entry を足せない。名前の変更（`use_app_state`）は Phase N。

### 19.22 内部の handler の型の置き場所と unit

（17.5 節の定義の場所と unit の行を置き換える）

- `Handled`（core の `view/input.rs` から移す）と `Handler<E>`（`Rc<dyn Fn(E) -> Result<Handled, UiError>>` を包む `Clone` の型、`from_fn`、`call`）は `kivact_runtime` の reactive の `handler.rs` に置く（R-3）。`HandlerResult` は既に reactive（`error.rs`）にあり、`handler.rs` への移動と `KeyHandlerResult` との統合は C-3a。
- runtime の中の置き換え: frame loop の関数は R-4、task の `DoneFn` は R-5（`Handler<Result<T, UiError>>`。handler ごとに結果を clone して渡す。`on_done` は既に `T: Clone` の impl の中）。core の slot は C-3a。

### 19.23 `Machine` の実装の規則

（18.13 節に足す）

```rust
pub struct MachineCx<M: Machine> { /* MachineHandle<M> */ }
impl<M: Machine> MachineCx<M> { pub fn send(&self, event: M::Event); }
pub struct MachineHandle<M: Machine> { /* Owned<MachineCell<M>> */ }
// Step<S, Ev, Ef> の field は非公開（stay・to・with・after・cancel だけで作る）
```

- `send` は、その machine の遷移の実行中なら queue に積む。そうでなければその場で: `input()` を untracked で作る → `transition` → `next` があれば state の signal に書く → timer の操作 → effect を順に `run` で実行（machine の owner の handler の frame）→ queue の event を同じ順で処理する。実行中かどうかと queue は `enum Dispatch { Idle, Running(VecDeque<Ev>) }` 一つで持つ。
- timer は event の variant（`std::mem::discriminant`）ごとに一つの `Task`。`after` は同じ variant の前の task を `cancel` してから、`let wait = sleep(delay);` を先に作り（締切は `after` の時刻から数える）、`spawn_in(owner, async move { wait.await; handle.send(event) })` で作る。owner の破棄で task が消えるので timer も消える。
- `feed(source, to_event)` は `effect_on(source, move |v| handle.send(to_event(v.clone())))` と同じ。
- 破棄された machine への `send` は、破棄された `Signal` への書き込みと同じ misuse（`Disposed`）。

### 19.24 `AppOptions` と起動の経路

（7.2 節の `AppSettings` の struct と、18.12 節の `AppSettings` → `AppOptions` の行を置き換える）

```rust
// kivact_core::launch（kivact-test だけが使う公開の module。facade は再公開しない）
pub struct AppOptions { /* 全部 private */
    title: String, window_size: Option<Size>, min_window_size: Option<Size>,
    theme: Theme, locale: Option<Locale>, messages: Messages,
    fonts: Vec<(Asset, &'static Location<'static>)>,
    blocking_threads: Option<usize>,   // None: min(4, CPU の数)
    gpu_preference: GpuPreference,     // feature gpu-interop がないときは読まない
}
impl AppOptions {
    pub fn new(title: impl Into<String>) -> Self;
    pub fn title(self, title: impl Into<String>) -> Self;
    pub fn window_size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self;
    pub fn min_window_size(self, width: impl Into<f64>, height: impl Into<f64>) -> Self;
    pub fn theme(self, theme: Theme) -> Self;
    pub fn locale(self, locale: Locale) -> Self;
    pub fn messages(self, messages: Messages) -> Self;
    #[track_caller] pub fn font(self, font: impl Into<Asset>) -> Self;
    pub fn blocking_threads(self, n: usize) -> Self;
    #[cfg(feature = "gpu-interop")] pub fn gpu_preference(self, p: GpuPreference) -> Self;
    pub fn check_window_sizes(&self) -> Result<()>;
    pub fn host_options(&self) -> kivact_engine::api::HostOptions;
    pub fn window_title(&self) -> &str;
}
```

- `App` は `AppOptions` を一つ持ち、builder のメソッドは同名の `AppOptions` のメソッドを呼ぶ。`kivact::test::RenderOptions` は `AppOptions` を包む builder（field は private）で、同名のメソッドと試験だけのメソッド（`host_locale`、`reduced_motion()`、`dark()`、`scale`、`fonts`、`mirror`、`gpu()`、`frame_period_ns`、`title`、`from_env()`、`with_theme(theme)`）を持つ。
- **理由:** 7.2 節の公開の field の形では事実が落ちる。font は拒まれたときの報告のために登録した位置を持ち（`kivact-core/src/client/app_client.rs:67`）、blocking の数は「指定なし」を持ち（`app_client.rs:117`、`threads: Option<usize>`）、`RenderOptions` は 15 の公開の field を別に持っていた（`kivact-core/src/test/options.rs:36-78`）。field を private にすれば `Location` が facade から届く署名に出ない。
- `render_with_handle(options, || (handle, view)) -> (Screen, H)` は残す（付録 B.3 の「`render_with` に統合」の行を消す）。**理由:** 戻り値の形が違うので、新しい trait なしには一つの関数にできない（`kivact-core/src/test/render.rs:61`）。
- engine の試験の feature の名前は `testing` のまま（`kivact-engine/Cargo.toml:33`。7.3 節の「feature `test`」を読み替える）。

### 19.25 `Inspect` のメソッド

（7.3 節の「`Inspect` は core の内部の値ごとに読み出しのメソッドを一つ」を置き換える）

```rust
impl LaunchHandle { pub fn inspect(&self) -> Option<Inspect<'_>>; }   // Started のときだけ Some
impl Inspect<'_> {
    pub fn reader(&self) -> kivact_engine::Reader;   // engine の事実は全部これで読む
    pub fn root(&self) -> NodeId;
    pub fn app_view(&self) -> Option<NodeId>;
    pub fn located_at(&self, node: NodeId) -> Option<&'static Location<'static>>;
    pub fn contains(&self, node: NodeId) -> bool;
    pub fn why(&self, node: NodeId) -> Why;
    pub fn stats(&self) -> Stats;                    // core_ns は kivact-test が埋める
    pub fn turns(&self) -> Vec<TurnRecord>;
    pub fn errors(&self) -> Vec<UiError>;
    pub fn pending(&self) -> bool;
    pub fn outstanding_blocking(&self) -> usize;
    pub fn motion_ticks(&self) -> u64;
    #[cfg(debug_assertions)] pub fn open_trace(&self, name: &str);
    #[cfg(debug_assertions)] pub fn check_trees(&self) -> Result<(), String>;   // C-5 で消す
}
```

- **理由:** 試験が `Tree` から読んでいる値（`content`、`role`、`name`、`value`、`checked`、`selected`、`expanded`、`disabled`、`hidden`、`input_value`、`placeholder`、`scene`、`test_id`）は、どれも engine の `Reader` を包んでいるだけである（`kivact-core/src/view/tree.rs:494-575` の `with_node`・`reader()`）。core だけが答えられるのは、node の位置・owner・`why`・runtime の統計（`kivact-core/src/test/screen/state.rs:38-66`）と turn の記録だけである。semantics の snapshot は `reader()` の host の木を辿る（C-5 までは、`check_trees` が core の木と同じであることを保証する）。
- 既存 Screen の idle/error/blocking/tick 契約を保つため上の Inspect 読出を持つ。engine の統計は `Reader::stats() -> WriteStats`。`AppOptions::title(self, impl Into<String>) -> Self` に RenderOptions の title 更新を委譲する。
- C-1 限定で、App 公開 API へ移せない既存 backend E2E は kivact-test 内の private 試験として既存名/assert を保持する。新たな facade 観測 API は作らない。必要な既存 writer 契約は engine testing の `ArvaneTestHost::writer() -> Writer` から読む。

### 19.26 style の内部の口と `THEMED`

（7.6 節の第 3・第 4 項目と、付録 B.1 の `kivact::style` の「公開しない」の文を置き換える）

```rust
// kivact_style::internal（#[doc(hidden)] pub mod。kivact-core だけが使う）
pub fn install_theme(owner: Owner, theme: Value<Theme>);
pub fn theme_of(owner: Owner) -> Option<Value<Theme>>;
pub fn compile_style(style: &Style, theme: &Theme, state: Interactive) -> CompiledStyle;
pub fn compile_span_style(style: &SpanStyle, theme: &Theme) -> CompiledSpanStyle;
pub fn record(size: Size, theme: &Theme, f: impl FnOnce(&mut Draw)) -> Recording;
pub fn with_scroll_axes(style: Style, axes: ScrollAxes) -> Style;
// kivact::style（公開）
impl Style {
    pub fn merge(self, other: &Style) -> Style;          // 公開のまま
    pub fn display(self, display: Display) -> Style;     // Display::{Flex, Grid, Contents}。display_grid を置き換える
}
```

- 固有メソッドは module に移せないので、host 側の口は自由関数にして、同名の固有メソッドを消す（`Theme::install`・`Theme::of` は `kivact-style/src/theme/value.rs:450,459`、`Style::compile` は `resolve.rs:240`、`Draw::record` は `draw/recorder.rs:48`）。
- **`Style::merge` は公開する。** kit が recipe を合わせるのに使っている（`kivact-ui-basic/src/recipes_controls.rs:85` ほか basic・game・soft の約 10 file）。kit は小さくしない。
- **`display_grid` は `display(Display)` にする。** soft は style の closure の中で条件付きで grid にしている（`kivact-ui-soft/src/form_layout.rs:73,79`、`aspect_ratio.rs:81`、`toolbar.rs:242`）。`view::grid` だけでは書けない。E-8 が足す `Display::Contents` も同じメソッドで選ぶ。`with_scroll_axes` の利用者は core の `scroll` だけなので内部にする（`kivact-core/src/control/container.rs:123`）。
- **`static THEMED` は残す**（7.6 節の第 5 項目を消す）。`Color::of(color)` は theme なしで Copy の `Color(Repr::Themed(u32, alpha))` を作るので（`kivact-style/src/theme/value.rs:577-586`）、表を `Theme` の値に入れられない。表は `ThemeColor` の値ごとに一つの、縮まない interning の cache である（rustdoc にそう書き、`#[expect]` の理由にする）。

### 19.27 地域書式は ICU4X で一度だけ作る

（7.6 節の第 2 項目の「en-US の表へ移す」と、17.14 節の「C-2 で作る en-US の表を消す」を置き換える）

- 書式の API（付録 B.1 の `kivact::i18n`）は最初から ICU4X で実装する（unit L-1）。en-US だけの手書きの表は作らない。**理由:** その表は同じ wave の L-1 ですぐ消える。
- 暦の計算の根は `CalendarDate::from_days(i64)`・`CalendarDate::to_days(self) -> i64`（`kivact-i18n/src/date.rs:78,89` の `days_from_civil`・`civil_from_days` を公開する。soft の写し `kivact-ui-soft/src/timestamp.rs:489,498` は K の unit で消す）。

### 19.28 削除の結果の型

（付録 A の SM-C07、SM-C14 の「`Tree::remove` が id の列を返す」と、17.1 節の「`Writer::remove_with_exit` が返す id の列」を一つにする）

```rust
#[must_use = "drop the removed records after the tree borrow ends"]
pub(crate) struct Removed { pub(crate) ids: Vec<NodeId>, records: Vec<CoreNode> }   // ids: 消した部分木の全 node（前順）
pub(crate) fn remove(&mut self, id: NodeId) -> Removed;
// engine
impl Writer { pub fn remove_with_exit(&self, id: NodeId, pop_layout: bool, notify: bool) -> Vec<NodeId>; }
```

- C-3a はこの署名を core の木で作り（`ids` は core の記録から）、C-5 は `ids` を `Writer::remove_with_exit` の戻り値にする。commit の `asset_failures` の掃除（SM-C07）と `NodeRef` の `Detached`（SM-C14）は `ids` だけを読む。
- `Writer::remove_with_exit` が返すのは**論理的に**消した id で、退場の動きのために engine に残る node も含む。core の記録（handler など）は論理的な削除のときに消す。**理由:** 退場中の node は engine の木に残る（`kivact-engine/src/model/apply.rs:405-411` の `begin_exit`）ので、物理的な削除まで待つと handler が退場中も生きる。現在の戻り値は `()`（`kivact-engine/src/host/write/writer.rs:126`）。
- SM-C12 の `Leaving.on_exit` は `Option<Handler<()>>`（17.5 節の「内部の callback は `Handler` だけ」に合わせる。付録 A の `Rc<dyn Fn()>` を置き換える）。`IdMap<V>` は `NodeId` の key だけなので（`kivact-types/src/ids.rs:37`）、`RequestId` など別の key は `HashMap` にする。

### 19.29 fragment は `Display::Contents` の箱

（17.1 節の第 1・第 2 項目を置き換える。unit E-8・C-5）

- engine に `NodeKind::Fragment` を足さない。core の flow（`show`、`dynamic`、`for_each`、`for_index`、`virtual_list` の行、`error_boundary`、`provide*`）は、自分の位置に箱を一つ作り、style は `display(Display::Contents)`、semantics なし、`hit_testable(false)`、focus しないものにする。
- 透明の扱いは一つの判定 `is_transparent(node) = style の display が Contents` と、それを使う `NodeTree::flat_children(id)` だけ。layout の adapter、paint の walker、a11y の build（semantics を出さず子を親へ上げる）、hit test、focus の移動はこれだけを使う。
- engine の読み出し: `Reader::parent(id) -> Option<NodeId>`、`Reader::children(id) -> Vec<NodeId>`（退場中の node を含まない。core の挿入の位置と event の bubbling は退場中の node を見ない）。透明な箱は、退場中の子孫がいる間は残り、最後の子孫の退場が終わったときに消える。
- 組み立て途中の部分木（まだ挿入していない list の行）は engine で親なしの箱として作り、その下へ子を挿入する（`NodeTree::insert` は親が未接続でも受ける。`kivact-engine/src/tree/node_tree.rs:256`、Writer は即時に model を書く `writer.rs:93-108`）。
- **理由:** fragment を別の種類にすると、透明の扱いが `Display::Contents`（soft の要求 C-38）と二つになる。一つの判定にすれば E-8 の walker の変更は一度で、core の木の連結の code（`kivact-core/src/view/tree.rs:680-900` の `host_parent`・`top_hosts`・`next_host_after`・`attach_writes*`・`move_writes_before`・`set_children`）は全部消える。C-5 は CP4 の後へ遅らせてよい（Phase N は C-5 に依らない）。

### 19.30 part の hover は engine の hover の target から

（7.5 節の表の「hover の chain」の行に足す。unit E-3・C-4）

- engine の `HostEvent::HoverChanged { node: Option<NodeId> }` を `HoverChanged { target: Option<FocusTarget> }`（node と canvas の region）にする。canvas の region の part の hover は `target == FocusTarget { node, region: Some(region) }` で決まる。
- **理由:** 現在 region の part の hover の情報源は pointer の Enter/Leave の observer だけ（`kivact-core/src/view/part.rs:222-223` が作り、`view/mount/regions.rs:217-222` が region へ渡す）で、engine の event は node しか持たない（`kivact-engine/src/contract/events.rs:57`）。observer を消すと region の hover がなくなる。

### 19.31 `HandlerResult` は四つの impl を持つ一つの trait

（10.4 節に足す。unit C-3a）

```rust
// kivact_runtime::handler（正準パスは kivact::view）
pub enum Handled { Yes, No, PreventDefault } // and による既存の伝播・既定動作の合成も保つ
pub trait HandlerResult: Sealed { #[doc(hidden)] fn into_outcome(self) -> Result<Handled, UiError>; }
impl HandlerResult for () {}
impl HandlerResult for Handled {}
impl HandlerResult for Result<(), UiError> {}
impl HandlerResult for Result<Handled, UiError> {}
```

- `KeyHandlerResult` を消す（`kivact-core/src/view/input.rs:98`。core の中だけが使っている）。四つの impl でも `?` は `UiError` を推論する: closure の `Ok(..)` の値の型で impl が一つに決まる。現在の trait は二つの impl（`kivact-reactive/src/error.rs:242`）。
- 移動は C-3a で行い（R-7 はなくなった）、core の内部の slot を `Handler<E>` にするのと同じ unit にする。

### 19.32 payload のない handler の `Callback` はどの owner の下でも作れる

（18.2 節の第 1 項目に足す）

- `impl<F, R> From<F> for Callback<()> where F: Fn() -> R + 'static, R: HandlerResult` は、今の owner の種類（setup、handler、effect）を問わずその下に `Callback` を作る。owner がないときだけ API-12 の panic。
- **理由:** 現在の `callback()` は setup の文脈を要求する（`kivact-reactive/src/setup.rs:120-122` の `setup_scope("callback", ..)`）。そのままでは handler の中で作る data の record（`Toast::action(label, ..)` など）に closure を渡せない。

### 19.33 `NodeBuilder::node_mut` は公開で文書に出さない

（7.4 節の表の `trait NodeBuilder` の行の「必須の実装は `node_mut`（今のまま公開）」を補う。作業手順書の旧 N-3 の「`node_mut` を sealed にする」を消す）

```rust
pub trait NodeBuilder: Sized {
    #[doc(hidden)] fn node_mut(&mut self) -> &mut Node;
    fn with_node(self, f: impl FnOnce(Node) -> Node) -> Self;
    // 既定のメソッドは 7.4 節の表の集合
}
```

- **理由:** kit が約 190 の builder で `NodeBuilder` を実装している（basic 43、pop 18、game 9、creative 22、soft 100）。sealed にすると kit と外部の部品の作者（戦略 P-07）が実装できない。`#[doc(hidden)]` で、文書に出る口は `with_node` だけになる。

### 19.34 公開 API の phase の進め方

（作業手順書の旧 N-1〜N-7・N-9 を置き換える。設計の形は変えない）

- 名前と path の変更は付録 B.10（旧→新の表）一つにまとめ、新しい trait と型は定義の段（N-D1・N-D2）で一度に作る。その後、crate の組ごとの unit（N-F1〜N-F6、N-G1〜N-G2）が、自分の file に Phase N の全部の変更を一度に当てる。
- **理由:** 旧 N-1〜N-7 は、同じ ui・interaction・kit の builder の file を一つずつ最大 6 回書き直す順だった。旧 N-3・N-4 の「kit も直す」と、0.10 節の「kit は N-9 で一度に」が食い違っていた。
- 関数の長さ（`too_many_lines`）と状態の検査の有効化は公開 API の検査ではないので、N-8 から Phase K の H-G1〜H-G6 へ移す。今の code では、ui・interaction・kit を除く library crate に、80 行を超える関数が約 90 ある（engine 22、arvane-native 19、core 18 ほか）。

### 19.35 `InteractionPolicy` の形と置き場所

（18.15 節の `InteractionPolicy` の field の一覧を置き換える）

```rust
pub struct InteractionPolicy {           // theme の拡張。Clone・Debug・PartialEq・Default。全 field が pub。FRU で一部を上書きする
    pub menu: MenuPolicy,                // { press: MenuPress, drift: PressDrift, submenu: SubmenuTiming, disabled_focus: DisabledMenuFocus }
    pub tooltip: TooltipTiming,          // 旧 TooltipConfig { delay: 500 ms, skip_delay: 300 ms }
    pub button: ButtonPolicy,            // { disabled_focus: DisabledButtonFocus }
    pub slider: SliderPolicy,            // { touch: SliderTouchMotion }
    pub switch: SwitchPolicy,            // { drag: SwitchDrag }
    pub toast: ToastPolicy,              // { swipe: ToastSwipe, swipe_slop: f64 = 8.0 }
    pub select: SelectPolicy,            // { sheet_slop: f64 = 10.0 }
    pub hover_card: HoverCardTiming,     // K-2a が足す。{ open_delay: 600 ms, close_delay: 300 ms }
    pub drawer: DrawerPolicy,            // K-2a が足す。{ swipe: DrawerSwipe { close_fraction: 0.3, flick_speed: 0.6, flick_distance: 4.0 } }
}
pub struct PressDrift { pub context: f64, pub other: f64 }   // 10.0, 13.0
```

- **interaction へ移す型**: `DisabledButtonFocus`、`SliderTouchMotion`、`SwitchDrag`、`ToastSwipe` の定義を `kivact-ui` から `kivact-interaction` へ移す。公開の path は `kivact::interaction` になる。
  - 理由: `InteractionPolicy` は `kivact-interaction` にあり、interaction は ui に依存しない（`kivact-interaction/Cargo.toml` の依存は `kivact-core` だけ）。今の定義の場所は `kivact-ui/src/button.rs`、`slider.rs`、`switch_drag.rs`、`toast_swipe.rs`。
- **`TooltipTiming` は policy の field**: 文脈で渡す経路（`provide(TooltipConfig { .. })`）は消す。
  - 理由: 今は `try_context::<TooltipConfig>()`（`kivact-interaction/src/tooltip.rs` の `use_tooltip_when`）で読み、crate の外に使う所がない。theme と文脈の二つの経路を残さない。
- **`hover_card` と `drawer` の field は K-2a が足す**: その時まで、読む部品がない。
- **hover card の遅延の上書き**: hover card は、部品ごとの `.open_delay(Duration)`・`.close_delay(Duration)` も持つ。basic の今の builder の option で、kit を縮めないため。

### 19.36 `UiTokens` の作り方と範囲

（18.15 節の `UiTokens` の段落、17.7 節、付録 C.6 の「pop と game の recipe」の行、11.1 節の `DataTableStickyPaint`・`DataTableRowStatusPaint` の行き先を置き換える）

- **I-0 が作る**: `UiTokens` は I-0 が作り、最初の field は次の七つだけにする。
  ```rust
  pub struct UiTokens {
      pub button: ButtonTokens,               // { busy_motion: ButtonBusyMotion }
      pub progress: ProgressTokens,           // { motion: ProgressMotion }
      pub toast: ToastTokens,                 // { row_motion: ToastRowMotion }
      pub toggle_button: ToggleButtonTokens,  // { label_reservation: ToggleLabelReservation }
      pub dialog: DialogTokens,               // { center: DialogCenter }
      pub disclosure: DisclosureTokens,       // { motion: DisclosureMotion }
      pub data_table: DataTableTokens,        // { sticky: DataTableStickyPaint, row_status: DataTableRowStatusPaint }
  }
  ```
  入れ子の struct は、その部品の file に置く。`UiTokens` は `kivact-ui/src/tokens.rs` に置く。
- **K の各 unit が足す**: 中核へ上げた部品の分を、上げた unit が足す。token にするのは、元の kit が上書きしていた値と、theme から読む値だけ。
- **今ある recipe を後から token にしない**: pop と game の recipe の file を「値だけが違う recipe を token に」減らす作業もしない（file の構成を揃えるだけ）。
  - 理由: 5 kit の recipe（約 8.8k 行）と既定の recipe を全部比べる調査が要るが、kit の作者が触る面（`Theme`、kit の `Tokens`、`UiTokens`、`UiRecipes`、`InteractionPolicy`）は、これをしなくても揃う。後で kit が上書きしたくなった値は、field を一つ足せばよい（18.15 節の規則のまま）。
- **data table の二つの paint は `UiTokens.data_table`**: `UiRecipes` には入れない。
  - 理由: どちらも kit が上書きする値で（soft の `theme.rs` の `.with(kivact::ui::DataTableStickyPaint { .. })`）、style の関数ではない。

### 19.37 `DisclosureMotion` の既定と `animated`

（18.15 節の `disclosure.motion` と 11.1 節の `DisclosureMotion` の既定、付録 A の SM-U06 を置き換える）

```rust
pub struct DisclosureMotion { pub animate: bool, pub duration: Duration }  // 既定 animate: false, duration: 200 ms
```

- **既定は `false`**: 理由は、今の `ui::disclosure` の既定が `animated: false`（`kivact-ui/src/disclosure.rs` の `disclosure(..)` の初期値）だから。11.1 節の「true」は事実と違う。
- **`ui::disclosure(..).animated(bool)` は残す**: その一つの部品の `motion.animate` の上書きとして使う。
  - 理由: basic の catalog（`examples/kivact-ui-basic/src/docs/accordion.rs`）と試験が、部品ごとに `.animated(true)` を付けている。theme の値だけにすると、部品ごとの選択ができなくなる。

### 19.38 `Theme::overlay_margin`

（11.1 節の `OverlayViewport` の行き先を具体にする）

- `kivact-core/src/view/overlays.rs` の `OverlayViewport` の theme の拡張を消し、`Theme` の metric にする: `Theme::with_overlay_margin(f64) -> Theme`、`Theme::overlay_margin(&self) -> f64`。既定は今の `OverlayViewport` の fallback の値。
- 理由: ほかの metric（`with_focus_ring`、`with_control_height` など）と同じ形にする。

### 19.39 `DragApi` の追加と touch だけの gesture

（11.1 節の「gesture は一つの部品」の段落を置き換える）

```rust
impl<T: Clone + 'static> DragApi<T> {
    pub fn slop(self, distance: impl Into<f64>) -> Self;  // 既定 0。越えるまで Pending
    pub fn axis_lock(self) -> Self;      // 構造の option。slop を越えた最初の向きの軸に固定し、交わる軸の delta を 0 にする
    pub fn track_velocity(self) -> Self; // 構造の option。DragUpdate.velocity を埋める（窓 100 ms）
}
pub struct DragUpdate<T> { /* 今の field */ pub velocity: Point }  // px/ms。track_velocity がなければ Point::ZERO
```

- **`DragVelocity` は公開のまま残す**: `DragApi` は中でそれを使う。
  - 理由: soft の `date_touch.rs` と interaction の `scalar/touch.rs` が、drag なしで使っている。
- **`DragApi` を使う範囲**: `slop`・`axis_lock`・`track_velocity` を使うのは、今 `use_drag` の上にある部品だけ: basic の drawer（`drawer.rs`）、soft の bottom sheet（`bottom_sheet.rs`）、creative の drag（`editors/*`、`bezier`、`gradient`、`numeric` など）、carousel。
- **touch だけの gesture は `DragApi` の上に作り直さない**: 対象は switch の drag、toast の swipe、select の touch の sheet、scalar の touch の slider、soft の month swipe。
  - これらは、今の raw pointer の handler を `Machine` の event にする。slop の判定は `DragAxis::past_slop`、速度は `DragVelocity`。
  - 理由: これらは全部、`PointerKind::Touch`（と pen）だけを受け、primary の pointer だけを受け、押した時から `suppress_activation` を呼ぶ時を自分で決めている。
    - 場所: `kivact-ui/src/switch_drag.rs`、`toast_swipe.rs`、`select.rs` の touch の部分、`kivact-interaction/src/scalar/touch.rs`、`kivact-ui-soft/src/month_swipe.rs`。
    - `use_drag` には pointer の種類の filter も、slop の前の見た目の状態も、activation の抑止もない。作り直すには `DragApi` に新しい概念を三つ足す必要がある。状態を machine にするという目的は、raw pointer のままで果たせる。
- **閾値の値は変えない**: policy の field にするのは `toast.swipe_slop`（8.0）と `select.sheet_slop`（10.0）と `menu.drift` だけ。switch の 10 px と soft の month swipe の 6 px は、名前の付いた private の定数にする。

### 19.40 APG への合わせを振る舞いの unit に入れる

（17.9 節の「unit: I-8」と、作業手順書の I-8 を置き換える）

- **I・U の unit の二つ目の commit**: I・U の各 unit は、一つ目の commit で状態を `Machine` に移し（振る舞いを変えない）、二つ目の commit で、その behavior に当たる APG の pattern に合わせる（振る舞いの変更と E2E）。I-8 の unit はない。
  - 理由: 同じ file を二度読み直さない。移設と変更は commit で分かれるので、回帰の切り分けは今と同じにできる。
- **pattern と unit の対応**:

  | pattern | unit |
  |---|---|
  | Disclosure、Dialog (Modal)、Alert Dialog、Tooltip | I-1 |
  | Combobox、Listbox（select） | I-2a |
  | Menu・Menubar・Menu Button | I-2b |
  | Button、Checkbox、Switch | I-3 |
  | Listbox（list）、Tree View（behavior）、Tabs、Radio Group | I-4a |
  | Grid・Table | I-4b |
  | Slider・Multi-Thumb Slider | I-5a |
  | Window Splitter | I-5b |
  | Spinbutton | U-2 |
  | Carousel | K-1b |
  | Accordion | K-2a |
  | Toolbar | K-3a |
  | Breadcrumb | K-3b |
  | Tree View（`ui::tree` の役割と状態） | K-4 |
  | Alert | K-6b |

  Carousel、Accordion（disclosure group）、Toolbar、Breadcrumb、Alert の部品は Phase K で中核にできるので、その K の unit で合わせる。
- 作者の決定（右クリックのメニュー、閉じた select の矢印、tooltip の時間）と、試験の名前と置き場所（`apg_<pattern>.rs`、`<pattern>_<APG の操作>`）は 17.9 節のまま。

### 19.41 toast の行と tree の行の出現と退場も presence で描く

（17.10 節に足す。付録 A の SM-U04 と SM-U903 の「解消後」を置き換える）

- **toast の行**: 行の node の `exit_size(Orientation::Vertical)` で縮める。`kivact-ui/src/toast_rows.rs` の、退場中を含む `slots` と live の `live_slots` の二重の列、世代の counter、`animated` の高さを消す。
- **tree の行の reveal**: 子の行の node の `enter_size`・`exit_size` で描く。`kivact-ui/src/tree.rs` の memo の中の `previous_open` の書き込みと `last_height` を消す。`RowReveal`・`ToastRowPhase` の enum は作らない。
- 理由: どちらも「行が出る・消える間の高さの動き」で、17.10 節の presence の規則（自然な大きさ × 進み具合）と同じもの。先に machine にしてから presence に置き換えると、作った machine を捨てることになる。

### 19.42 編集する node の行の見た目の API

（17.12 節の `line_styles` と `decorations` の署名を置き換える）

```rust
impl TextEditing {
    pub fn line_styles<T, S, F>(self, content: impl IntoValue<T>, f: F) -> Self
    where T: Clone + PartialEq + 'static, S: PartialEq + Default + 'static,
          F: Fn(&T, &str, &S) -> (Vec<StyledRange>, S) + 'static;          // 旧 highlight
    pub fn decorations<T: Clone + PartialEq + 'static>(self, content: impl IntoValue<T>, f: impl Fn(&T) -> Vec<Decoration> + 'static) -> Self;
}
pub struct StyledRange { pub range: Range<usize>, pub color: Option<Color>, pub background: Option<Color>, pub decoration: Option<TextDecoration> }  // 旧 TextHighlight
pub struct Decoration { pub range: Range<usize>, pub kind: DecorationKind, pub color: Color }   // range は文書の byte
pub enum DecorationKind { Underline(TextDecorationStyle), Outline, Background }
```

- 理由: 今の `TextEditing::highlight`（`kivact-core/src/primitive/text_editing.rs`）は、content の値 `T` と行をまたぐ状態 `S`（block comment など）を受け渡している。basic の `code_block` がそれを使っている。17.12 節の `Fn(LineIndex, &str)` の形では、この二つが落ちる。
- `ui::code_block(..).highlight(..)`（K-9a）の引数も同じ形にする。

### 19.43 設計書 17.18 節の C-05 と C-35

（17.18 節の表の C-05 と C-35 の行を置き換える）

- **C-05**: 新しい `TransitionProp` の variant は足さない。
  - 理由: 文字色の補間は、既存の `TransitionProp::Paint` が既に含む（`kivact-style/src/sheet.rs` の `TransitionProp::Paint` の doc の「border and text colors」）。G-4a は、これを確かめる E2E だけを足す。
- **C-35**: G-4 から K-2b へ移す。
  - 足す option は、soft の engine（`typeahead.rs`、`selector.rs`、`multi_selector`、`tokenizer`、`calendar.rs`）を消して `use_combobox`・`use_select`・`use_calendar_grid` に置き換えるのに要るものだけにする。名前は soft の今の option の名前にする。
  - 理由: soft-ui-work-order の付録 02 の「Kivact 実装方針」の節は部品ごとに散らばっていて、option の一覧になっていない。soft の engine を消す作業の中でなら、要る option が code から決まる。

### 19.44 日付の入力の表示の仕方を中核へ

（付録 C.2 の date inputs の行の「soft の touch の wheel」の記述を置き換える）

- soft の `DatePresentation::{Adaptive, Desktop, BottomSheet}`（`kivact-ui-soft/src/date_touch.rs`）を `kivact::ui::DatePresentation` にし、`ui::date_input`・`date_range_input`・`time_input`・`date_time_input` の構造の option `.presentation(DatePresentation)` にする。`Adaptive` は `Capabilities.touch` で sheet を選ぶ。
- sheet の wheel の panel は `kivact-ui` へ移し、`use_scalar` の上に作り直す。
- 理由: wheel は soft の date input の `.presentation(..)`（`date_input.rs` の `presentation`）で選ぶ部品の構造で、recipe（style の関数）では選べない。中核の部品は kit の版の機能の和集合を持つ（11.3 節）。

### 19.45 creative の picker は creative に残す

（付録 C.2 の combobox 系の「元の kit の版」から creative の `picker.rs` を外す）

- creative の `picker.rs` は creative に残す。中は既に `use_combobox` を使っている。
- 理由: 付録 C.5 の creative の「残す」に `search_picker` があり、C.2 と食い違っていた。picker は列の追従（SM-K905）などの creative 固有の振る舞いを持つ。

- I-0 の overlay metric: `Theme::overlay_margin()` の既定は `0.0`。viewport への fitting は常に行い、値は inset を表す。旧い未設定の no-fit は廃止し、負数・NaN sentinel や設定有無の mode は作らない。
