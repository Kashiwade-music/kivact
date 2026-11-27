# Kivact のアーキテクチャ

この文書は、実装を変更するときの境界、所有権、公開 API の規則を定める。製品として何を引き受けるかは[製品戦略](strategy.md)、採用した理由と過去の判断は[設計判断](decisions.md)、現在の型と署名は rustdoc とソースを参照する。規則を変更する場合は、実装と同じ変更でこの文書、依存 policy、利用者向け文書を更新する。古い呼び方を残す互換 wrapper や alias は作らない。

## crate の境界

workspace は 54 package からなる。アプリと公式 kit は `kivact` facade を利用する。内部 crate を直接使わせることで公開 API の不足を埋めない。依存の許可と module の向きは `dev/xtask/policy/workspace.toml` が機械的に検査する。

| package | 担当と境界の理由 |
|---|---|
| `kivact` | 公開 facade。正準 module と prelude を再公開し、任意機能の入口をまとめる。同じ item を二つの module から出さない |
| `kivact-macros` | `#[component]` と `#[derive(Prop)]`。関数の署名から props の struct、constructor、`Option` と `#[prop(default)]` の setter、`IntoView`（戻り値が `impl Element` なら `Element` も）を生成する。自分を使う crate の path を `proc-macro-crate` で解決する（依存に `kivact` があれば `::kivact`、無く `kivact-core` があれば `::kivact_core`）。proc-macro は独立の crate でなければならない |
| `kivact-types` | workspace 共通の純粋な値、識別子、幾何、入力、意味情報と、std だけの小さな同期 helper（`cell::Recent`、`cell::Deferred`、`cell::SharedDeferred`、`process`）。ほかの workspace crate に依存しない |
| `arvane-protocol` | engine と renderer の契約（`RenderScene`、`Composition`、`CpuFrame`、`SemanticsTree`、描画用の paint・gradient・span）。作るのは engine だけ、使うのは arvane-* だけ |
| `kivact-runtime` | reactive、task、motion、Machine、turn の状態と wake。これらは同じ owner と実行順に依存するため一つに置く |
| `kivact-i18n` | Locale、messages、ICU4X による地域書式・日付計算。reactive context の所有は core |
| `kivact-style` | authored Style（CSS の名前と値の型）、Theme（`Arc` で共有する値）、theme 変数（`ThemeVar`）、Draw、Path と値の解決。外部描画 library の型を公開契約に出さない |
| `kivact-shader` | ユーザーの WGSL の検証と shader module の組み立て。naga だけに依存し、GPU の build だけが使う |
| `kivact-svg` | 記録した `DrawList` を SVG 1.1 の文書に書く。kivact-types と png だけに依存し、文字の baseline と画像の画素は `kivact-offscreen` が `Resources` で答える |
| `kivact-offscreen` | window なしの描画。入力は Theme・font・描画の closure、出力は `Pixels` か SVG の文字列。view と runtime を読まない。facade が `kivact::draw::Offscreen` として再公開する |
| `kivact-editing` | buffer、journal、editing session。window、描画、OS を知らない |
| `kivact-layout` | layout の計算と一つの Layout owner。Taffy は内部実装 |
| `kivact-core` | App、view、mount、context、入力配送、overlay、platform、diagnostics、embed。turn 全体の順序を所有する。view・client・platform・devtools は node 表の内部を共有するので分割しない |
| `kivact-interaction` | 描画しない behavior、Part と状態機械。view を返さない（`surface()` を除く）。UI の recipe を知らない |
| `kivact-ui` | 標準部品（`#[component]`）、部品の variant の enum、`kivact::ui::theme`（部品ごとの `<Name>Theme`・`<Name>State`・部品の theme 変数、`Look`、`Variant`）。kit なしで実用画面を作れる部品と、二つ以上の kit が共有する部品を持つ。入力で動く `impl Machine` と signal を書く `effect` を持たない |
| `kivact-test` | App 全体の E2E、Screen、Inspect。公開 view API と engine の試験境界を使う葉であり、view tree の内部を直接読まない |
| `kivact-devtools-protocol` | 3 者（agent・DevTools アプリ・MCP）が共有する通信の語彙と JSON Schema。I/O を持たない純粋な値と純粋関数。feature `client` が discovery・接続・型付きの呼び出し・通知の受信を足し、アプリと MCP が共有する |
| `kivact-devtools-agent` | socket・thread・file・process を扱う server。Kivact の内部を知らず、core は `take_incoming` で受け取った `Call` を処理して `Responder::send` で答える |
| `kivact-devtools` | DevTools の GUI。公開 API と標準部品だけで作り（P-07 の dogfooding）、私有の theme を持つ。kit に依存しない。パネルは module にする |
| `kivact-devtools-mcp` | MCP の仕様への追従と stdio。GUI を link しない |
| `kivact-engine` | window ごとの tree/model（`Model`）と App 全体の `AppModel`、計測、入力と配送の受理（admission）、paint と per-node の draw cache、a11y、host pump。lower と text_input は内部 module。多数の内部情報を共有する処理を偽の crate 境界で分けない |
| `arvane` | window driver と presenter。GPU の device は Driver（一つの App の event loop）に一つで、window ごとには frame の状態だけを持つ。他の crate を再公開しない |
| `arvane-shell` | OS 非依存の window SPI、入力語彙と正規化 pipeline。winit、softbuffer、AccessKit を持ち込まない |
| `arvane-native` | Windows/Linux の実 window、OS port、入力 mapping、present、支援技術、services、embed、mirror |
| `arvane-virtual` | 台本、仮想 clock、window、surface と SteppedBackend。native mirror を任意に接続する |
| `arvane-text` | text shaping と計測。Parley を内部に閉じる |
| `arvane-render` | RenderScene のラスタライズ（Vello）。vello は内に閉じる。wgpu は GPU の device を持つ arvane-compose と共有する |
| `arvane-compose` | Composition の plan と、その CPU・GPU の合成。GPU の device を Driver ごとに一つ持つ。3D、cache、shader、外部 texture を Vello の外で一つの合成として扱う（Chrome の Skia と cc の分担と同じ） |
| `arvane-asset` | image、SVG、font の読み込みと decode（premultiplied）、resvg の SVG（font は arvane-text の `FontLookup`。system font を二度走査しない）、遅延して起こす worker |
| `kivact-gpu-share` | UI に依存しない GPU texture の import/export 契約 |
| `kivact-ui-basic` / `kivact-ui-pop` / `kivact-ui-game` / `kivact-ui-creative` / `kivact-ui-soft` | 五つの公式 kit。共通の骨格（`theme(mode)`、kit の theme 変数、`recipes_<group>.rs`、`variants.rs`、分野の部品は `src/<domain>/`）。共通 behavior を複製しない |
| `kivact-plot` | 数値の XY plot（`PlotModel` を受けて view を返す）。色は `kivact_plot::color` の theme 変数で受け、どの kit とも組み合わせられる |
| `kivact-template-basic` | アプリの雛形 |
| `kivact-example-counter` / `kivact-example-form` / `kivact-example-list` / `kivact-example-async` / `kivact-example-component` | 公開 API の小さな利用例 |
| `kivact-ui-basic-catalog` / `kivact-ui-pop-catalog` / `kivact-ui-game-catalog` / `kivact-ui-creative-catalog` / `kivact-ui-soft-catalog` | 五つの kit の利用例と recipe の見本 |
| `catalog-shell` | 五つの catalog の shell・page 一覧・検索・scroll-spy（`examples/catalog-shell`、`publish = false`）。入力は page の一覧と theme の関数 |
| `kivact-public-api-tests` | facade の公開契約の試験。test binary は `e2e_core`・`e2e_ui`・`e2e_native` の 3 つ |
| `kivact-kit-tests` | kit に依存する公開 API の E2E（`e2e_kits`。theme_conformance と kit の crate が要る試験）。`dev/tests/kivact-kit-tests` |
| `kivact-docs` | 英語の文書と README の Rust のコンパイル。日本語は xtask の `docs-pairs` が英日のファイルの集合と Rust のブロックの token の列（文字列の中身を除く）を確かめる |
| `gpu-interop-producer` | GPU 共有を検証する別の描画 producer |
| `kivact-bench` | 性能の指標（K1〜K6）を手動で測る probe と `run.sh`。facade の公開 API だけを使い、CI では実行しない |
| `xtask` | 依存、API、状態、style、文書の対の検査。図の生成は gate の外の `cargo xtask diagrams` |

library は 34、アプリは 2、雛形・小例・catalog と catalog の shell は 12、試験・道具は 6 package。検査用 package も workspace の数に含める。旧 wire、reactive、task、motion、text、lower の独立 crate と、DevTools の client だけの crate は復活させない。

次は境界が明確でないので分割しない（B-3）：kivact-core（view・client・platform・devtools は node 表の内部を共有する）、kivact-engine（host が model の内部を直接読む）、engine の `lower`・`text_input`（狭い API ができるまで module のまま）、arvane-native の embed（a11y・services・入力・present を共有する）、arvane-render と arvane-compose、kivact-ui の分野の部品（calendar・chat など）、kivact-types の `cell`。

### 境界を作る規則（B）

- B-1: 入出力を少数の型で説明でき、相手の内部状態を調べず、一つの変更理由を持つ単位にする。
- B-2: 明確な境界は crate にして依存、可視性、feature、最適化を制御する。
- B-3: 明確でない境界は統合する。将来の DAG や最適化だけを理由に crate を作らない。
- B-4: crate/module の先頭に英語の `//! Role:` と役割を書く。crate の境界の根拠も説明する。例・雛形・docs は対象外。
- B-5: `visible_to` にない利用者は追加しない。dev-dependency は `visible_to` の対象にしない（全体の既定）。新しい依存が必要なら境界の判断と policy を先に変更する。

### module の向き

矢印は「右側を参照する」を表す。正確な許可リストと理由のある例外は policy を正とし、再公開だけの facade を実装の循環と混同しない。

| crate | 依存の向き |
|---|---|
| runtime | `reactive` が基底。`task → reactive`、`motion → reactive`、`machine → reactive/task`、`host → reactive/task/motion`。passive host_types と Runtime を借りる TurnGuard を実行実装から分離 |
| core | `view → state/geometry/task/motion/style`、`platform/diagnostics/gpu_interop → view`、`client → platform/diagnostics/gpu_interop/view`、`devtools → task/motion/style/state/geometry/sync/view/platform/diagnostics/client`、`launch → client/devtools`、`app → launch`、`embed → app/launch/view`。`custom → view/draw`。`client` は `devtools` を参照せず、起動時に一度設定する `DevtoolsHooks`（trait object）だけを呼ぶ。`launch` は `kivact-test` だけが使う hidden な契約 |
| style | asset/length/color などの値 → geometry/path/theme/effect/paint/text → draw/sheet → resolve の順に組み立てる。上位から下位だけを参照 |
| interaction | policy、axis、press、edit、disclosure、focus_return、roving などの小さい behavior を下に置き、list/drag、popup/scalar、dialog/select/menu、table/palette を組む。UI への逆依存は禁止。view を返さない（`surface()` を除く） |
| ui | 部品ごとの module（view と `<Name>Theme`）。interaction の behavior を使い、入力で動く Machine と signal を書く effect を持たない。部品の module どうしは参照しない |
| engine | `Model` は window ごとの tree/model、`AppModel` は App 全体の event の列・motion の時計・frame の要求を持つ。contract/lower/text_input が基底。tree/mutation → measure → model → text_measure/paint/a11y/check → host/api/embed/testing。model が読む paint/a11y/check の passive state leaf は明示した例外だけ。hit test（`model/interact/hit`）は描画の上界を答える `HitInk` を model で定義し、paint が実装する。paint は描き直した node だけを lower し直す draw cache を持ち、restack は重なりの事実が変わったときだけ行う（PERF-6）。model から paint の内部を参照しない |
| shell | `events → types`、`pipeline → events/types/ime`、`input → pipeline`、`window_input → events/types/ime/pipeline/mouse/clicks/contacts/input`、`window → events/ime/types/services/a11y/target/input`。正規化は backend から独立 |
| native | port が present/a11y/services/display を使い、mirror が port/present/a11y を使う。OS 固有の入口から engine へ逆依存しない |
| virtual | clock を script が使い、window が clock/script/surface を使い、backend と mirror がそれを組み立てる |
| compose | cache が基底。plan は cache を使い、cpu と gpu は plan・cache・tables を使う。cpu と gpu は互いを参照しない |
| arvane | driver が process の GPU device を持ち、window ごとの presenter が frame の状態を持つ。engine は arvane-text・arvane-compose・arvane-asset・arvane-shell を直接使い、arvane の再公開を通さない |

## 一つの語彙（VOC）

VOC-1 は一つの意味に一つの型、VOC-2 は値を types、closure・動的 dispatch を含む契約を利用側の crate、VOC-3 は同名の異義を作らない、VOC-4 は既定値を `kivact-types::defaults`、VOC-5 は依存のない純粋値とその builder を types に置き facade でそのまま再公開する、という規則である。reactive 値を保持する Style、kurbo の幾何が必要な Path のように、異なる情報・依存を持つ場合だけ包む。記録値は PathData と呼ぶ。

## 状態と事実の所有（S）

| 規則 | 内容 |
|---|---|
| S-1 | 三つ以上の状態、または phase により field の有効性が変わるものは enum にする |
| S-2 | 二つ以上の状態の bool/Option/counter、または Option と State/Phase enum の併置を検査する。独立した一つの bool を機械的に Machine にしない |
| S-3 | 同時に有効な値を variant の payload に入れる。依存する Some/None を並べない |
| S-4 | 同じ事実を二度保存しない。表示・readonly getter は正本の投影にする |
| S-5 | 遷移だけが状態を書く。effect による状態の補修、同期用 flag、epoch/counter を作らない |
| S-6 | phase の順序は型、状態、runtime guard で表す |
| S-7 | 必要な依存を constructor に渡す。未完成の object を作って後から slot を埋めない |
| S-8 | 通知には `Trigger` を使う。counter は件数・ID・統計の意味がある場合だけ |
| S-9 | ReportOnce、Sent、Generation、Mailbox、純粋 cache、統計を区別する。異なる事実を無理に一つの phase にしない |

**検査の単位は型（xtask の `state`）。** 次の (a) と (b) を満たす struct と enum variant が対象になる。

- (a) `bool`、`Option<_>`、整数の counter、`Cell`/`RefCell`/`Signal`/`Var`/`Atomic*` の field を二つ以上持つ。
- (b) `&mut self` の method を持つか、内部可変の field を constructor の外で書く。
- 名前の接尾辞や derive による除外は置かない（名前を変えるだけで状態が見えなくなるため）。

対象の型は、型の直前の一行の印 `// xtask-allow(state): <組合せの閉じ方>` が無ければ error になる。閉じ方は「すべての組合せが有効な理由」か「どの組合せをどの遷移が排除しているか」を一行で書く。constructor、`Default`、値渡しの builder（`fn x(mut self, ..) -> Self`）だけで書く field は状態ではなく、印も検査も要らない。関数の中の状態の local（`signal(false)` と `signal(None)` の組など）の検査は残る。例・雛形・docs は検査の対象外。

既存の印から機械的に作った印は `// xtask-allow(state): legacy: ...` と書き、閉じ方をまだ確かめていないことを示す。xtask は `legacy:` の数を報告する。`legacy:` の印は台帳で一件ずつ閉じ方（enum にする、一つの持ち主にする、正当な独立の組合せとして理由を書き直す）を決めて消し、新しく足さない。

検査は意味の誤り（真に排他な組合せを独立と書いた等）までは見つけられない。新しい状態の型を足す変更では、印の一行が型の実際の遷移と合っていることを review で確かめる。

Creative jobs の run/stop は独立した一つの入力であり、progress/amount は数値の測定事実である。effect が実 FrameLoop の start/stop だけを行う場合、bool を置換するだけの第二の Machine は不要である。

### 事実の持ち主

| 事実 | 唯一の持ち主 / 利用側 |
|---|---|
| window ごとの keyboard focus | window ごとの engine Focus。NodeRef の focused は投影。支援技術の focus も publish 時に導出 |
| 開いている window | core の WindowRecord の表。`open` が true で mount した window だけが入り、root と owner は ViewNodes が持つ |
| DevTools が target に掛けた状態（highlight、overlay、forced states、style override、emulation、時間の倍率、記録） | target に一つ。core の DevTools の session が持ち、全 client が共有する。表示用の写しを正本にしない |
| node pointer capture | engine。region/Part capture は core PartCapture、engine は node の経路を保持 |
| pointer の press、capture、現在の over | engine の pointer ごとの Track。press と capture は別の事実で四つの組合せを許す |
| region hover | engine の node/region を組にした target。primary hover chain と secondary pointer の over を混同しない |
| modifiers / mouse | shell の ModifierState / Mouse。engine に同じ入力状態を複写しない |
| activation の配送 | 一回の routed envelope。入力時の route と dispatch 時の modal admission を分ける |
| editable text と selection | 実 editor/session。別の signal に selection を複写しない |
| caller の Signal | caller。部品の draft と確定値を区別し、effect で逆同期しない |
| range の選択途中 | RangePick の anchor/preview。最初の click は stage だけ、二つ目で確定。取消は Signal を書き戻さない |
| popup の focus 復帰先 | 既存の focus-return owner が opening ごとに実 invoker を snapshot。外側に移った実 focus は維持 |
| host の描画可否 | engine HostPhase と presence/surface。frame clock の gate は投影 |
| 時刻・timer | runtime clock と scheduler。virtual の script clock が同じ経路へ時刻を渡す |
| font generation | engine の font collection。native Environment の更新で巻き戻さない |
| 起動時の Environment | 復元した readonly snapshot。virtual の永続応答 map は台本 owner |
| a11y の前回 accepted tree | 初期化・action 検証用の受理事実。focus や差分用 model の複製ではない |
| App の外からの終了要求 | `AppHandle` の `AppLink`（thread をまたぐ唯一の正本）。drain は runtime の `ExitState` |
| 埋め込み App の panic のメッセージ | 子の PanicSlot。最初の payload の文字列だけを保持し、`Embedded::status` が `ErrorKind::Panicked` として投影する |
| scrub の起点 | Creative の Numeric の mode Machine の Anchor。直前に適用した値とポインタ位置を持ち、修飾キーの倍率が変わると付け替える |
| foreign window の子 | native の `WinitWindow.foreign`。engine は `WindowContent::Foreign` に handle と最後に送った寸法を持ち、core は `ForeignPhase` で `on_destroy` の負債だけを持つ |

catalog でも同じ規則を使う。chat entry は Message(text)/Typing、integrated meter は Unmeasured/Measured と実 Sample/Reset、active section は単一 owner が Observed/Requested を受ける。最後の観測を保持し、geometry、scroll、playing、selection の mirror を足さない。

### Machine の書き方

Machine は独自 control のための、小さい reducer に effect と timer を加えたもの。通常の部品利用に必要な prelude の概念にはしない。

1. `Self` を readonly input snapshot、State を Clone + PartialEq の状態 enum、Event と Effect を意味のある値で定義する。
2. `transition(&self, state, event)` は純粋に Step を返す。Step の stay/to/with/after/cancel を使い、field を直接構築しない。
3. `machine(initial, input, run)` は owner に属する。send 時に input を untracked で読み、純粋遷移、状態更新、timer 操作、handler frame の effect 実行の順に処理する。再入した send は FIFO に積む。
4. timer は event の variant ごとに一つ。複数項目で同時に必要なら項目ごとに Machine を持つ。期限は future の最初の poll より前に確定する。
5. feed は指定 source の変化を送る。最初の依存収集では callback を呼ばない。MachineCx は send の口であり、timer や状態の補修用の第二 owner にしない。
6. owner の破棄後は Disposed。timer、task、frame の停止も同じ owner の cleanup に従う。

標準部品（kivact-ui）は入力で動く Machine を持たない。入力の状態機械は interaction の behavior に置き、ui はその Api と Part を使う。

## 公開 API の 17 規則

| ID | 規則 |
|---|---|
| API-1 | 正準 module path と prelude だけを入口にする。constructor と型を同じ module に置く。behavior/data は interaction、部品と variant は ui、部品の theme（`<Name>Theme`、`<Name>State`、部品の theme 変数、`Look`、`Variant`）は `kivact::ui::theme`。同じ item を二つの path から出さない |
| API-2 | 動的な部品の option は `Value<T>` の props（`impl IntoValue<T>` を受ける）。構造を決める flag/mode、構築前の option 束、passive record は通常の値 |
| API-3 | `on_` は event だけ。payload の無い handler（`on_press` など、`#[component]` の `Callback<()>` props）は `impl IntoHandler<(), M>`（closure と `Callback<()>`）、payload のある handler（`on_key`、`on_activate`、`on_commit`、`on_edit`、`Callback<E>` props）は `impl Fn(E) -> R + 'static`（共有の `Callback<E>` は `move \|e\| cb.call(e)`）を受ける。戻り値は `HandlerResult`。lifecycle の登録は FnOnce。純粋 query は名詞と借用値。部品の必須の handler は位置引数、任意の handler は `Option<Callback<E>>` の setter |
| API-4 | 論理座標、寸法、連続値は入力も読み返しも f64。件数/index/level/font weight は整数。整数 literal の変換と異なり reactive closure は f64 を返す |
| API-5 | 見える text は `Value<String>`（`IntoText` を受ける）、ID/name は `Into<String>`、message key/default は static str。label は control の名前、title は見える container 見出し、Node.name は生の意味情報 |
| API-6 | 一度 mount する slot は `View`、作り直す content は `ViewFn`（または `Fn(Arg) -> V`）、項目は item ごとの factory。`provide` と `scope` は FnOnce。slot 名の語彙を揃える |
| API-7 | `use_` は owner に結び付いた behavior、context、App 全体の service の読み出し（`use_toaster`、`use_announcer`、`use_theme`、`use_context`）。signal/memo/machine/resource/action/animated/frame_loop/node_ref と app-wide environment query に `use_` を付けない。provide_x/use_x は対 |
| API-8 | Rc/Arc/Box dyn/Location/明示 lifetime を普通の公開署名に出さない。lifetime の例外は Constraints と Element の借用。実在する内部 cross-crate bridge だけ doc(hidden) にし、`__private`/`__macro` の module と policy の `[macro_support]` の crate は検査の側で例外にする |
| API-9 | backend の型、公開 path で名前を書けない型を漏らさない |
| API-10 | kit の名前は facade prelude、ui、標準 prelude と衝突させない。kit 名の接頭辞を型名に付けない |
| API-11 | conversion trait の診断を具体的にする。独自の型を props の定数にするのは `#[derive(Prop)]`（内部の hidden の仕組みは runtime と types の基本型だけ） |
| API-12 | setup/context の misuse は track_caller 付きで、修正方法の分かる panic にする |
| API-13 | 必須位置引数は見える label/title、data、選択、開閉、children。見えない名前を持つ入力部品（text_input、text_area、select、combobox、slider、number_input、日付・時刻の入力）は名前を位置引数で取らず、`.label(..)` か `field(..)` で受ける。focus できる control に名前が無いと mount で位置付きの診断（debug build と `kivact::test` では error）。項目の集まりは items。値を編集する部品は `Signal<T>` を位置引数で受ける（0 個以上を選ぶ部品は `IntoSelection`、単一は `Signal<Option<K>>`、必ず一つは `Signal<K>`）。閉じていると意味の無い部品（dialog、window、drawer、sheet）は `open: Signal<bool>` を位置引数、開閉が任意の部品は setter `.open(signal)`（呼ばなければ部品が局所の signal を作る）。一つの root は `impl Element` |
| API-14 | bool query は is_/has_、それ以外は名詞。get_/_value は使わず、更新は set/動詞、失敗可能は try_。略語は a11y/ime/id 以外を増やさない |
| API-15 | Api、State、Variant、Theme（部品の recipe 表）、Options、Policy、Mode、Kind、Event、Phase、Reason を役割どおり使う。InteractionPolicy の入れ子も Policy |
| API-16 | 開閉の behavior は `use_x(.., open: Signal<bool>)` で open を必須の位置引数に取り、命令 `show`/`hide`/`toggle`（引数なし、冪等）と `is_open() -> Value<bool>`、`trigger()`・`content()` の Part、`surface(placement, content) -> View` を持つ。開いている間だけ content を mount するのは `surface` か `show(api.is_open(), f)` |
| API-17 | 公開の recipe state は、排他な事実を pub の enum の field（型名は owner の接頭辞を除いた名前。部分を表す enum は State で終わらせない。例 CardAction、CheckControl）、常にある事実を pub の field、一部の種類にだけある事実と導出する事実を accessor で表す。bool の accessor は is_/has_ で始める。part の状態を読む唯一の方法は `Part::state()` |

### 公開 module と利用者の入口

root は App、Result、UiError、ErrorKind と macro の `component`・`Prop`・`bind`。state、view、geometry、style、draw、motion、task、i18n、platform、diagnostics、interaction、ui、custom を意味ごとの正準 module とし、test/embed/gpu_interop は feature の入口とする。GPU の起動の希望（GpuPreference、GpuPower、GpuBackend、GpuStart、AdapterMatch）は feature なしで `platform` に置き、gpu feature の無い build では無視する。CSS の値の型（Shadow、Filter、Transform、Paint、gradient、Shape、CornerRadius、BlendMode、Mask*、BorderImage*、PaintShader、FilterShader、FilterGraph）は `style` だけ、描画者の型（Draw、Path、Stroke、Layer、Mix、Text*、Pixels、Offscreen）は `draw` だけに置く。prelude は設計で決めた一覧（日常の利用に必要な名前）を明示再公開し、変換 trait（IntoValue、IntoText、IntoHandler）を入れない。custom は kit 作者の正式な独自部品境界（Owner、Part、primitive、scope、raise、IME、文字編集）で、内部 engine の型を直接使わせない。

定数 prop は literal または `#[derive(Prop)]` を付けた型の値で扱う。reactive な値は `Value<T>`、書ける値は `Signal<T>`（lens も同じ型）、selection は single/multiple の選択、CalendarSelection は日付の Single/Range descriptor であり、異なる情報を一つの conversion trait に押し込めない。

### handler の語彙と実行

HandlerResult は `()`、Handled、Result<(), UiError>、Result<Handled, UiError> の四つを受ける。Handled は Yes/No/PreventDefault を区別し、合成でも PreventDefault を失わない。`noop()` は伝播する。bubble しない callback は Handled を捨てるため、その戻り値で calendar の選択を拒否できない。拒否には実遷移前の activation_guard のような policy を使う。

handler は runtime の一つの `Handler<E>`（`Rc<dyn Fn(E) -> Result<Handled, UiError>>`、arena の object を作らない）に揃える。受け方は二つだけ（DEC-16）：payload の無い handler は sealed な `IntoHandler<(), M>`（closure `Fn() -> R`（M = Nullary）と `Callback<()>`（M = Handle））、payload のある handler は `impl Fn(E) -> R + 'static`。rustc は closure の引数の型を `Fn` の bound からしか推論しないので、payload のある handler を trait で包まない。共有したい handler は `callback(f)` で作り、payload のある handler には `move |e| cb.call(e)` で渡す。`#[component]` の `Callback<E>` props は内部で `Handler<E>` を持ち、本体には Copy の `Callback<E>` を渡す。owner への登録は本体が子に渡すときだけ行う。user handler、Part、既定動作の順序で伝播と既定動作の防止を区別する。

state の on_mount/on_cleanup は FnOnce の lifecycle 登録であり、node の実装用 on_attach(PartMount) と区別する。イベントは on_press（payload のない押下）、on_activate(key)（key のある項目の実行）、on_commit(value)（確定）、on_edit(EditPhase)（編集の段階）、on_submit、drag の on_start/on_update/on_end、on_files、on_dismiss(reason)、on_complete(Result)/on_exit_complete、on_error(UiError)、on_composition_change、on_active_change、on_reorder とする。項目の活性化は on_activate とし、on_select・on_pick・on_item_activate を使わない。値は Signal が正本で、通知のための専用の handler（on_select・on_add・on_remove に当たるもの）を足さない。支援技術 action は a11y_action に登録する。

### 構造 option と slot

構造 option は mount 時の node 構成、owner、アルゴリズムの選択を決める。一方、表示値、disabled、caption、calendar の月数・週の始まりなどは reactive にできる。変更できるという理由だけで全 option を IntoValue にしない。

構造の例は DialogMode、keep_mounted、single/multiple selection、ComboboxMode、CarouselMode、RowHeight、variant、keep_focus、modal、region_events、located_at_caller、release_offscreen_focus、TextEditing の multiline/report_text、table の expand_all、overlay の keep_anchor_clear/at_least_anchor_width、Span の italic、Transition の essential/alternate、carousel の looping、drop zone の multiple/pick_on_press、combobox の free_text/chips、disclosure の always_open、empty の announce、drawer/sidebar の side と collapsible、OTP groups/pattern、hover card/menu の delay、step indicator の indicator、skeleton の index、date の presentation である。

ListItem、MenuEntry、Command、TreeNode、DataColumn、ToastOptions、StepperItem は passive な値として構築し、collection の reactive 性は外側に持つ。App と RenderOptions は private field と setter にまとめる（AppOptions は App の非公開の field の型）。ThemeOptions と InteractionPolicy のような FRU の設定値は公開 field を持ち、構築前の通常値として渡す。variant の Default では owner を要する reactive 値を作らず、必要な reactive 値は明示 setter に渡す。

部品は `#[component]` の関数だけで書く（アプリ、kivact-ui、五つの kit、kivact-plot、catalog、例）。必須の props は宣言順の位置引数、`Option<..>` と `#[prop(default = ..)]` は名前付きの setter。props の種類は型の字面の名前で決まる（`Value<T>`、`Signal<T>`、`Callback<E>`、`View`、`ViewFn`、`Vec<View>`。別名はそれ以外の props）。戻り値が `impl Element` のとき、生成する struct は隠した `__root: Node` を持ち、`into_view` は本体を `scope` の中で一度だけ実行して `Node::absorb` で呼出し側の modifier を本体の root に移す（呼出し側の style の層は本体の後ろ、handler は本体の前、name・description・disabled・test_id・node_ref・variant は呼出し側の値）。`impl Element for` / `impl IntoView for` を手で書けるのは kivact-core の view module の基本要素（node、text、column、row、stack、grid、scroll、spacer、image、icon、canvas、window、overlay など）だけで、xtask が確かめる。`Element` の method と同じ名前の固有 method を `Element` の実装型に置かない。

`kivact::custom::scope(FnOnce → IntoView)` は mount 時に一度だけ実行する setup の境界で、macro を使わずに部品を組む kit 作者の逃げ道である。新しい child owner と透明な fragment の下で hook/cleanup を保つ。eager に factory を呼ぶことや、`provide(())` で空の context を置いて owner を偽造することでは代用しない。再度開く content は必要な factory を保ち、捨てた editor owner を再利用しない。

API-11 の診断は blanket impl の内側の bound を rustc が先に報告する場合もある。診断属性に加えて rustdoc の冒頭とガイドにも同じ修正 hint を記し、属性の文言が常に表示されることまでは前提にしない。

API-8 は診断と標準 Rust Error の実際の境界にも限定例外を持つ。UiError/ErrorKind と diagnostics の Trace/NodeInfo/Snapshot の source location、および UiError.from_boxed の Error interop は元の意味を保ち、通常の部品へ拡大しない。理由は定義の直前に記す。__private や cross-crate bridge の到達性を止めるのは doc(hidden) と理由付き internal-contract の両方がある実装 edge だけである。IntoValue/IntoText の内部変換、Value/Signal の runtime/location、TextParagraph の backend adapter が該当する。外部部品の契約である Element.node_mut は単に hidden であることだけでは検査対象から外さない。

### 共有データの境界

AssetBytes は private な Arc を持つ不変バイト列で、Asset::Shared、Pixels.rgba、GlyphRun.font が共用する。普通の利用者は所有する Vec などから構築する。Vec から Arc への標準変換に allocation が不要とは保証しない。Pixels::new は Into<AssetBytes> を受ける。

ShapedTextSnapshot と GlyphRunSnapshot は不変描画データを安価に clone する handle で、参照と allocation identity を維持する。custom の TextLayout とは別概念である。SemanticsTextRuns は既存 run の共有 list を隠し、利用者は owned run の Vec/iterator で構築し、iter/get/index は参照を返す。backend の既存共有 allocation は、理由を明記した hidden from_shared/as_shared 等の内部 bridge で再利用する。普通の From<Arc> を追加したり、境界のために Vec を深く clone したりしない。Draw の text measurer 設定は mount/offscreen が自動で行う hidden bridge で、アプリに手動 install を要求しない。

## kit の規約（KIT）

| ID | 規約 |
|---|---|
| KIT-1 | `theme(mode) -> Theme`。実際の option があるときだけ `ThemeOptions` と `theme_with(mode, options)`。Theme の extension trait を作らない。kit は theme を `thread_local!` の static で一度だけ作る（fn pointer を key にする cache は作らない） |
| KIT-2 | kit の色・大きさは kit の theme 変数（`<kit>::PRIMARY` などの `const` と `PRIMARY_VAR` などの `static`）。`theme()` が中核の変数（`color::ACCENT_VAR` など）を kit の変数に向ける。色の名前の旧型と Tokens の値の field を持たない。色 literal は theme 変数の定義と `theme()` だけに置く |
| KIT-3 | 形の差は `recipes_<group>.rs` の `<Name>Theme` の上書き、値の差は theme 変数。全 recipe を無差別に上書きせず、上書きしない部品は中核の既定を使う |
| KIT-4 | 公開名は API-10 に従う。kit 名の接頭辞を型名に付けない（`Game` で始まる型名にしない） |
| KIT-5 | slot は API-6、children は `Vec<View>`。kit 私有の同義 slot/children trait を作らない |
| KIT-6 | 見た目の選択は `variants.rs` の `<Component>Variant` 一つにまとめ、`Element::variant` で渡す。`*Appearance`・`*Look`・`*Hue`・`*Tone` と呼ばない。標準 Element の disabled を隠さない |
| KIT-7 | kit の use_ は固有 domain に限る。共通 hook は core/interaction、状態は Machine |
| KIT-8 | 標準 message は component.key、kit 固有は kit.component.name。共通意味に複数 key を作らない |
| KIT-9 | kit の E2E は `crates/kivact-ui-<kit>/tests/consumer` の一つの binary（kit 固有の部品の契約）と、全 kit を通す `kivact-kit-tests` の `e2e_kits` の theme_conformance（共通部品の配線）。interaction の behavior の試験を kit に写さない |
| KIT-10 | 共通 font asset は一か所に置き、kit の theme が `Theme::with_fonts` で持つ。license も同じ asset と管理する |

共通部品へ移すときは移す元の kit の option と behavior の和集合を残す。単なる wrapper や再公開で kit に二つ目の入口を残さない。固有 domain は `src/<domain>/` に残す。例は Creative の keyframe glyph と column-follow picker、Pop の conversation-preview row、Soft の dictation/streaming/tool-call/token。catalog は削除せず、`kivact-catalog-shell` の shell の上で、その kit の recipe で標準部品を使う見本にする。catalog の文言は英語にする。

### 日付・複合部品の共通 behavior

calendar は Single/Range の CalendarSelection Signal descriptor を標準 From で受ける。集合の IntoSelection を変更しない。Locale の parse_date/month_name/weekday_name と既存 calendar grid を使い、kit に日付計算をコピーしない。date/time field は TextBox と別の picker button。keyboard で開いた input、button で開いた button を区別し、開いたら実 grid に focus、確定・取消で実 invoker に戻す。外側の click が既に focus を移したときはその先を維持する。

DatePresentation は Adaptive/Desktop/BottomSheet。Adaptive は capabilities.touch を読む。sheet の day cell は theme の大きい値を縮めず最低 44px。day 選択では閉じず Save、月見出しから Month/Year wheel と Done、date-time は Date の Save で Time へ進み次の Save で閉じる。wheel は共通 scalar と Trigger を使い、Up/Increment と PageUp が増加、Down/Decrement と PageDown が減少。物理 gesture の向きは維持する。swipe の Released phase と遷移前 guard で同じ gesture の click を抑え、timer/bool を足さない。range の Escape は stage を取消して dialog の dismiss に伝播する。確定した選択だけを localized Status で通知する。

chat、code block、stat、collections でも同じ原則を適用する。CodeBlock は既存 disclosure の owner と controls 関係、show/presence の高さ motion を使い、第二の disclosure を作らない。history は実 editor の selection を操作する。stat の animation と chart は caller の値と slot を合成する。Contents は layout/paint/input の走査では透過し、a11y の論理 hierarchy と子の bounds の合併は保つ。

## 性能とメモリ（PERF、MEM）

| 規則 | 内容 |
|---|---|
| PERF-1 | frame・turn・layout pass・a11y の公開は、変わったノードとその祖先（と、その祖先の子の一覧の安価な比較）だけを処理する。全ノードの走査は、構造の初期化、font の登録、環境の変化、明示的な全体の再構築（`Full`）、a11y の構造・role・名前の変化（Metadata の経路）だけに使い、その関数の doc comment に理由を書く。新しい O(N) の処理を足すときも同じ |
| PERF-2 | 一度しか要らない重い初期化（system font の DB、GPU の device、pipeline、ICU の formatter）は使う時に作る。起動の critical path（main から最初の present まで）に同期の処理を足さない |
| PERF-3 | CPU 合成は、平行移動だけ・不透明度 1・Normal と SrcOver・mask と cover なしの描画に整数の高速経路を使う。汎用の loop はそれ以外にだけ使う |
| PERF-4 | frame ごとの処理が読む特別な prop を持つノードの集合（shader の clock、再生中の画像、layout motion など）は、prop の書き手が更新する索引（engine の `MotionSources`）で持つ。毎 frame の全走査で探さない |
| PERF-5 | hit test は paint の ink（描画の上界）で子を枝刈りする。ink は model が定義する `HitInk` で読む。3D の投影や drawable を含み ink が上界にならない subtree は、ink の `beyond` の印で枝刈りと culling から外す |
| PERF-6 | frame の描画は、描き直した node だけを lower し直す（per-node の draw cache、描画中の node だけが持つ）。paint list の並べ直し（restack）は、重なりの事実（stacking context、z、layer の事実、子の組の変化）が変わったときだけ行う |
| PERF-7 | Arvane は engine が渡す layer ごとの `content_version` で変化を知る。command の hash は変化した chunk だけ。debug build では hash による照合で版の上げ忘れを検出する |
| PERF-8 | GPU の device は Driver（一つの App の event loop）に一つ。window ごとに持つのは frame の状態だけ。同じ process の二つの埋め込み editor は、それぞれの Driver の device を持つ |
| PERF-9 | 診断の記録（turn の trace、原因の文字列、node の provenance）は、DevTools・`kivact::test` の trace・debug の trace sink が有効にしたときだけ行う。release で常に持つのは counter だけ |
| PERF-10 | 起動の critical path（main から最初の present まで）で、DevTools agent の起動・asset worker の起動・使わない GPU pipeline の compile をしない。system font の走査は process の開始時に別 thread で始めて window と GPU の準備に重ね、最初の文字の計測は残りの時間だけ待つ |
| PERF-11 | 項目の多い collection（list、combobox、menu、select、tabs、tree、table、palette）は、collection ごとに一つの key の索引（`Index<K>`）と `selector` を使い、項目あたりの memo は最大 2 つ。例外：menu の項目は最大 5 つ（press-drag の間の pointer の移動ごとに全行の effect と submenu の timer を動かさないため） |
| PERF-12 | CPU の合成・コピー・present は傷んだ画素だけを回り、target 全体を画素ごとの判定で回らない。raster は、到達範囲が自分の領域に届く command だけを encode する。raster の scene の hash は 1 frame に高々 1 回で、layer が content_version を保つ間は 0 回（PERF-7） |
| MEM-1 | ノードごとの値は、既定値なら保持しない（None）。読む側は None を既定値として扱う |
| MEM-2 | 64 byte を超える値を、多数のノードが持つ `Option<T>`・`Rc<RefCell<Option<T>>>`・`Rc<Cell<Option<T>>>` の field にしない。`Option<Box<T>>` にする |
| MEM-3 | 多数のノードが同じ値（継承した computed text style、prop、style）を持つときは `Rc` を共有する。親と同じなら親の `Rc`、そうでなければ `kivact_types::cell::Recent` で直近の等しい値の `Rc` を使う |
| MEM-4 | authored `Style` は field の群ごとの疎な struct で持ち、使った群だけを確保する。操作の履歴を保持しない |
| MEM-5 | 部品 1 つの reactive node は、その部品で実際に独立して変わる値の数までにする。同じ source から導く複数の `memo<bool>` は、Copy な struct を返す一つの memo にする。使わない機能の clock・Machine・node は作らない |
| MEM-6 | ノードごとの主要な型（`HostNode`、`NodeProperties`、`NodeStyle`、`Style`、`StateStyles`、`Look`）に `const` の大きさの上限を置き、超える変更をコンパイルできないようにする。上限を上げるときは同じ変更に理由を書く |
| MEM-7 | 空の view・tuple・Vec と部品の境界（`#[component]`・`scope`・`provide`）は host node を作らない |
| MEM-8 | idle の window は最後の present から 1 秒後に CPU compositor の frame state を手放す。状態は `LastPresent` の variant で表す |

PERF-1 の Metadata の経路の例外は残す。ただし Metadata の経路で semantic order を二度数えず、clip の計算は attached な node だけを回る。

遅延初期化の道具は一つ（DEC-13）：`clippy.toml` は `OnceCell`・`OnceLock`・`LazyCell`・`LazyLock` を禁止する。使う時に作る重い資源（PERF-2、PERF-10：ICU の formatter、GPU の pipeline、system font、既定の theme など）は、`kivact_types::cell::Deferred<T>`（UI thread の中、`!Sync`）と `kivact_types::cell::SharedDeferred<T>`（`static` と thread をまたぐもの、`Sync`）だけで持つ。どちらも `const fn new()`、`get_or_init(&self, f) -> &T`、`get(&self) -> Option<&T>` を持つ。std の型を使うのはこの二つの実装の中だけ。状態の補修や「未完成の object を後で埋める」（S-7）には使わない。唯一の明記した例外は runtime の `Runtime.app`：App の構築の中（利用者のコードが動く前）で一度だけ設定し、設定前の読出しは debug の assert で落とす。

## feature と OS（DEP）

DEP-1 は同じ意味の feature 名を揃えること、DEP-2 は feature が自 crate の振る舞いだけを変えること、DEP-3 は OS の判断を port と事実に閉じること、DEP-4 は optional な機能の入口を facade/core と末端の実装に閉じ、中間層を常にコンパイルすることである。別 crate が match する enum variant や構築する struct field を feature で増減しない。DEP-5 は、重い任意機能（ICU の分かち書きの辞書など）を facade の feature にして末端の crate が実装し（DEP-4 の形）、既定では有効にすることである。

| package | feature と意味 |
|---|---|
| kivact | default は gpu と complex-scripts。gpu は arvane-compose/gpu（arvane-render/gpu を含む）、complex-scripts は arvane-text/complex-scripts、gpu-interop は gpu と compose/core/test の共有入口、embed は native/core/test、test は kivact-test、devtools は core と test の devtools（有効な release build は起動時に警告する）。embed は全 OS で compile でき、未対応の環境では `App::embed` が `Unsupported` を返す、tokio は core/tokio |
| kivact-core | tokio、embed、gpu-interop、devtools。gpu/test feature は持たない |
| kivact-runtime | tokio（task の実行 adapter） |
| kivact-test | embed、gpu-interop、devtools を core に転送 |
| kivact-devtools-protocol | client（discovery・接続・型付きの呼び出し・通知。DevTools アプリと MCP が使う） |
| kivact-engine | testing（virtual/mirror、png、native/test）。これは内部試験の意味なので test に改名しない |
| arvane-native | default は空、embed、test（embed を含む） |
| arvane-virtual | default は空、mirror（native/test） |
| arvane-render | gpu |
| arvane-compose | gpu（render の gpu、wgpu と GPU 共有の契約）、gpu-interop（gpu を含む） |
| arvane-text | complex-scripts（Parley の ICU の分かち書きの辞書） |
| kivact-gpu-share | default は空、export、import、wgpu（export と wgpu adapter） |
| gpu-interop-producer | wgpu |
| public-api-tests | default は gpu と complex-scripts。gpu、complex-scripts、gpu-interop、devtools、embed、tokio を facade に転送 |
| docs | default は gpu と complex-scripts を facade に転送。英語の文書だけを compile する。gpu-interop は GPU 共有のガイドの例 |
| template、五つの catalog、kivact-devtools | default は gpu と complex-scripts。どちらも facade に転送 |
| kivact-bench | default は gpu と complex-scripts。latency は facade の test（`kv_latency`） |
| その他 | feature なし。五つの kit に gpu-interop 等の中継 feature を置かない。kivact-plot も同じ |

GPU 無効時も GpuContext/GpuSurface は空 enum、GpuEnv は空の値として同じ名前がある。GPU_BUILT は false、利用不能は NoAdapter/External や FailureKind::Unavailable で説明し、feature のための API 分岐を利用者へ移さない。embed 無効の native open は Unsupported を返す。

OS 分岐の主な持ち主は arvane-native の Windows/Linux port。共有 GPU texture の handle/import/export に必要な分岐だけは kivact-types の gpu、kivact-gpu-share、arvane-render の GPU context に置く。core、interaction、UI、kit は command key、context-menu trigger、pointer class、touch 等を Environment/Capabilities から読む。macOS、Android、iOS、Web backend は未実装で、SPI があることと対応済みであることを混同しない。

GPU の device は Driver（一つの App の event loop）に一つで、必要時に起動する（既定は `GpuStart::OnDemand`）。起動の入口は presenter の `Gpu` の二つだけで、`begin`（`AtLaunch` のとき window の作成時）と `demand`（OnDemand で待っているとき）である。需要は、composition の user shader（CPU で代わりの見た目を描いた報告）、core の `gpu_interop::adapter()` の読取り（`HostRequest::DemandGpu`）、CPU の frame が表示周期の半分を 8 frame 続けて超えたこと、の三つ。`GpuStart::Never`、gpu feature の無い build、環境変数 `ARVANE_RENDERER=cpu` の window は需要があっても起動しない。`ARVANE_RENDERER` の `cpu`・`gpu`・`auto` は `Never`・`AtLaunch`・`OnDemand` に対応する。試験は `RenderOptions::gpu_on_demand()` で OnDemand を選び、`Screen::renderer()` で何が描いているかを確かめる。

## turn、Stage と時間

runtime の TurnState は Quiet / WakeSent / Running(Stage)。wake は Quiet から WakeSent に一度だけ移し、turn 中の pending は finish で計算する。RunningRequested や wants_frame の mirror を持たない。thread の ThreadWaker は UI 状態を触らず host を起こせる。

core の turn は開始時（idle の近道より前）に `AppHandle` の close 要求を取り、受理すれば `Runtime::request_exit` で drain を始める。embed は要求を拒否し、`ErrorKind::Unsupported` を root owner に報告して `Running` に戻す。

App の終了要求の turn では、core が開いている foreign window の `open` を false にし、同じ turn の安定化で owner を破棄する（`on_destroy` は子 window が生きている間に走る）。engine は turn の後に OS window を閉じる。`AppHandle::close` も同じ経路を通る。

| Stage | 担当 |
|---|---|
| Setup | root owner と初期 view、extension の構築 |
| Begin | turn の開始、clock の更新と pending の受理 |
| Reports | report 配送、依存の安定化、mount の処理 |
| Dispatch | 入力ごとの handler、activation、安定化と boundary mount |
| Async | task と完了通知を settle |
| Commit | frame plan、focus、asset failure、writer commit と mount stamp |
| Mounts | commit 済み mount callback |
| UserEffects | user effect とその後の安定化 |
| Output | output を取り出して turn を終える |

実行順は core の AppClient が一か所で所有し、runtime に TurnClient を作らない。clock は batch の前、途中の FrameTick でも進み、単調非減少。setup で registry、root owner、extensions を完成させてから返す。構築中の extension 参照、turn の再入、Commit 外の mount stamp は misuse。focus/error handler まで禁止するような一律の commit 書込み禁止は設けない。

mount と report の安定化、入力ごとの dispatch、async settle（最大 1000 round）、commit loop（最大 100 round）を順に行う。commit は frame plan、focus、asset failure の除去、writer commit、stamp、mount、user effect、再安定化・async を必要なだけ反復する。

時間の正本は runtime clock 一つ。Sleep は deadline heap と cancel queue を使い、due/next deadline の確認前に取消を処理し、Drop で取り消す。FrameTick は同期に frame callback を呼ぶ。next_frame future の task に置換すると入力 batch の順序が変わるため行わない。FrameClock の request と last tick は一つの phase に保つ。Animated は一つの AnimPhase を持ち is_running は投影、自然完了の通知は安定化中の InternalWatcher が handler を呼ぶ。同じ安定化内の完了・再開では誤った完了通知をしない。

Timeline は Playing の間だけ MotionState の Anim として登録し、毎 tick で track の値を publish する。drive された Timeline は progress を時間として扱い、clock には従わない。

reactive の frame kind（Handler/Setup/Computation）と pure/tracking/owner/writer は別の事実であり、untrack/callback/pure が元の kind を消さない。Watcher の最初の評価は依存と queue を初期化するだけで通知しない。以後の callback は untracked で呼び、値の PartialEq 比較を勝手に加えない。

host の資源を使うことの宣言（`gpu_interop::adapter()` が `None` の読取りで送る `HostRequest::DemandGpu`）は、答えを返さず、App が読む状態を書かず、何度送っても一度と同じで、閉じた window の分は host が捨てる。render effect が engine の木を書くのと同じく、memo・render effect・draw・text の closure の中で送ってよい。答えや UI の状態の変化を伴う platform request、signal の書込み、focus の移動は従来どおり `require_effectful` の対象とする。window ごとの「需要を待つか」は presenter の `Gpu::State` だけが持ち、core は記録しない。

## 検証と保守の単位

一つの関数は 80 code 行、file は 1000 code 行を上限とする。長い処理は意味のある helper に分け、line packing、無意味な型/module、新しい state、抑制属性で件数だけを減らさない。依存、状態（`state`）、effect の書込み、公開署名、正準 path、名前衝突、kit の色、OS 分岐、module の規則（interaction は view を返さない、ui は入力で動く Machine と signal を書く effect を持たない、`impl Element for` は core の view だけ、`machine(()` と `type State = ()` の禁止、`Element` の method と同名の固有 method の禁止）、英日の文書の対（`docs-pairs`）を全 workspace で検査する。検査の例外は実際の境界と理由に限り、同じ理由を繰り返す印は規則の側の例外にする。

性能は `dev/bench` の K1（起動）、K2（メモリ）、K3（CPU）、K4（フレーム時間）、K5（更新の遅延）、K6（バイナリの大きさ）で測る。比べる二つの commit を同じ機械で `run.sh` にかけ、比で判定する（機械ごとに絶対値が違うため）。K6 は Kivact 自身の大きさを GPU 一式を除く build（`--no-default-features --features complex-scripts`）で比べ、既定の build（`gpu` 込み）は大きくしないことを確かめる。結果は commit しない。E2E の `Stats` の a11y の走査の件数（`a11y_enumerated`）と資源の回収（mount の前の `live_reactive`・`live_owners`・`tree_nodes` に戻ること）で PERF-1 と回収の後退を検出する。

試験は App 全体の E2E（実 window と `kivact::test`）である。過去の不具合の再現だけの試験、画素や時間の厳密な比較、golden は足さない。ただし最適化の正しさを守る最小の不変条件の試験（cache の無効化、資源の寿命、状態の遷移）は、その変更が所有する試験の file に足してよい。kivact-test の自己試験（host の非同期の資源・font・cache の契約）は `crates/kivact-test/tests/` に置く。CPU だけの build は、package ごとの `cargo check -p <crate> --no-default-features --lib`（`kivact`、`kivact-offscreen`、`arvane-render`、`arvane-compose`、`kivact-style`）と、`cargo tree -e normal -p kivact --no-default-features` に wgpu が無いことで確かめる（workspace の `--no-default-features` は dev-dependency の feature で GPU が混ざる）。公開 API の E2E は依存の軸で 4 つの test binary に分ける：`kivact-public-api-tests` の `e2e_core`（`kivact` だけ）、`e2e_ui`（ui・apg・interaction。`kivact` だけ）、`e2e_native`（embed、GPU、foreign window、DevTools、GPU 共有）と、`kivact-kit-tests` の `e2e_kits`（theme_conformance と kit の crate が要るもの）。docs の doctest は英語の木と `README.md` だけを compile し、日本語の Rust の code block は `docs-pairs` が token の列で英語と比べる（文字列の中身は除く）。gate は 3 段（tier a：変更ごと、tier b：まとまりごと、tier c：merge 前）で、正確なコマンドと window-lock・GPU の試験の直列化は [CONTRIBUTING](../../CONTRIBUTING.md) の「テスト」と「merge 前の検査」に書く。

変更後の意味情報も実 node から検証する。AsyncView の既定 pending は Status Loading、picker エラーは Alert、Contents は layout の透明性と a11y Group/論理階層を両立する。Desktop 操作を検証する date fixture は明示 Desktop、一般の Adaptive fixture は touch の Save を保つ。icon は実 install 済み glyph で確かめ、timeout や description relation 欠落を期待値の緩和で隠さない。

NodeRef::snapshot は poll 時の node を Host の request/reply へ送り、layout/paint と geometry report が settle してから CPU 描画を取得する。owner の寿命を応答時にも確認する。古い scene の同期 read や、別の geometry モデル、追加の FrameTick 待ちで取得順序を代用しない。

## 追加・変更するときのチェックリスト

共通して、実際の要求を確認し、一つの事実の持ち主を決め、型・builder・変換の重複を作らず、公開契約の E2E と必要な英日ガイドを更新する。境界の増設だけで配管を増やさない。

| 足すもの | 変更する場所と確認 |
|---|---|
| 支援技術の性質（例 key_shortcuts） | types の Semantics、core の Node setter、native の変換の 3 crate。relation は node/region identity も検証 |
| Role | types の Role、native の role mapping、protocol の `Role` と core devtools の `convert`、protocol の JSON Schema の再生成（`KIVACT_WRITE_SCHEMA=1 cargo test -p kivact-devtools-protocol --test schema_generated`） |
| text style（例 text_wrap_style） | types の `TextStyle`（宣言・computed 値・継承を一か所）、style の setter、arvane-text の 3 crate。relative unit/current color と native 精度を保つ |
| layout/paint の性質 | types の一つの値、style の入口、layout または engine::lower の計算・描画の 3 crate。削除済み lower crate を再設置しない。画素が要る効果は compositor の LayerEffects |
| OS service（例 beep） | types の request/result/capability、native の window.perform、core/platform の 3 crate。virtual は台本の応答を登録するだけ |
| window に向けた service（例 focus_window） | types の request/result/capability、shell の window SPI、native の window.perform、core/platform の 4 crate。呼んだ owner の window を対象にし、main window へ暗黙に落とさない |
| Environment field | types、native の事実、core の readonly query の 3 crate。別の environment model を作らない |
| OS backend | native/port に入口、shell の共有 pipeline を利用。core に外部 backend 注入 API を増やさない。virtual が常時第二の SPI 実装 |
| foreign window の子 window | types の `WindowContent` と `PlatformRequest::ForeignChild`、native の `ForeignOs`、engine の `HostWindow.content`、core の `foreign_window`（4 crate） |
| 入力の正規化 | native port で raw code を写し、shell/pipeline で key/IME/DnD を正規化。window ごとの `WindowInput`（native の winit・virtual・注入が共有。embed の port は別） |
| 地域書式 | i18n の Locale と ICU4X。全 locale の compiled data を利用し、kit に en-US table や日付計算を作らない |
| behavior | interaction の State/Event/Effect と Machine、Part の配線、一つの公開 E2E。APG の当該 pattern の keyboard/roles/states を同じ変更で確認 |
| 標準部品 | interaction の behavior、ui の部品の module（`#[component]`、`<Name>Theme`、部品の theme 変数、variant）、移す元の kit の全 option の和集合、`e2e_ui` の公開 E2E、`kivact-kit-tests` の `e2e_kits` の theme_conformance、英日ガイド |
| DevTools の method | protocol の leaf の型と `lib.rs` の再公開、`protocol_methods!` の 1 行、JSON Schema の再生成、core の `devtools/` の振り分けと leaf、MCP の除外の一覧、ガイドの tool の一覧。版の規則（wire の型の field の削除・改名・型の変更、method・通知の削除・改名は `PROTOCOL_VERSION` を上げる。追加は上げない） |
| 独自 editor | custom の文字編集、IME session、line_styles と decorations。内容と cross-line state を純粋な callback で解析し、byte range を使う。子や独自描画を失わせない |
| アプリの描画表示 | 標準 viewport と interaction の `ZoomPan`。domain 固有の time/value 座標は独立した意味を持つ場合だけ保持 |
| E2E | App 全体を virtual backend と本番 presenter/driver で動かす。依存に合う binary（`kivact-public-api-tests` の `e2e_core`・`e2e_ui`・`e2e_native`、`kivact-kit-tests` の `e2e_kits`）に置く。mirror は任意。Screen の共通 helper と既存の数値・focus・Signal の assertion を保つ |

### 最初に読む実装の場所

以下は追加の入口となる現行fileである。新しい性質に必要なleafを選び、すべてを無条件に変更しない。

| 変更 | 主なfile |
|---|---|
| semantics/Role | types の `src/semantics/node.rs`・`semantics/values.rs`、core の `src/view/node_modifiers.rs`・`view/mount/semantics.rs`、native の `src/a11y/conversion.rs` |
| text style | types の `src/text/properties.rs`・`text/computed.rs`・`text/primitives.rs`、style の `src/text.rs`・`sheet.rs`、arvane-text の `src/build.rs` |
| OS service | types の `src/platform.rs`・`env.rs`、native の `src/services.rs` と `port/<os>/services.rs`、core の `src/platform/services.rs` |
| layout/paint | types の `src/layout.rs`・`paint.rs`、style の `src/sheet.rs`・`paint/`、layout の `src/style/` または engine の `src/lower/` |
| 標準部品 | interaction の該当 behavior、ui の該当部品の module、`dev/tests/kivact-public-api/tests/e2e_ui/`、英日 guide |

APG は公式の keyboard interaction と roles/states/properties を基準にする。作者の platform context menu、閉じた Select の矢印/Alt+Down、Tooltip の遅延の決定は優先する。規定されていない時間は InteractionPolicy に残す。検証の手順とコマンドは [CONTRIBUTING](../../CONTRIBUTING.md)、背景は[設計判断](decisions.md)を参照する。
