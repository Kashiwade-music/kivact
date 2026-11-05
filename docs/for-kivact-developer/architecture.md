# Kivact のアーキテクチャ

この文書は、実装を変更するときの境界、所有権、公開 API の規則を定める。製品として何を引き受けるかは[製品戦略](strategy.md)、採用した理由と過去の判断は[設計判断](decisions.md)、現在の型と署名は rustdoc とソースを参照する。規則を変更する場合は、実装と同じ変更でこの文書、依存 policy、利用者向け文書を更新する。古い呼び方を残す互換 wrapper や alias は作らない。

## crate の境界

workspace は 40 package からなる。アプリと公式 kit は `kivact` facade を利用する。内部 crate を直接使わせることで公開 API の不足を埋めない。依存の許可と module の向きは `dev/xtask/policy/workspace.toml` が機械的に検査する。

| package | 担当と境界の理由 |
|---|---|
| `kivact` | 公開 facade。正準 module と prelude を再公開し、任意機能の入口をまとめる |
| `kivact-types` | workspace 共通の純粋な値、識別子、幾何、入力、描画・意味情報。ほかの workspace crate に依存しない |
| `kivact-runtime` | reactive、task、motion、Machine、turn の状態と wake。これらは同じ owner と実行順に依存するため一つに置く |
| `kivact-i18n` | Locale、messages、ICU4X による地域書式・日付計算。reactive context の所有は core |
| `kivact-style` | authored Style、Theme、Draw、Path と値の解決。外部描画 library の型を公開契約に出さない |
| `kivact-editing` | buffer、journal、editing session。window、描画、OS を知らない |
| `kivact-layout` | layout の計算と一つの Layout owner。Taffy は内部実装 |
| `kivact-core` | App、view、mount、context、入力配送、overlay、platform、diagnostics、embed、offscreen。turn 全体の順序を所有する |
| `kivact-interaction` | 描画しない behavior、Part、binding と状態機械。UI の recipe を知らない |
| `kivact-ui` | 標準部品、その builder、variant、recipes、tokens。公式 kit にあった共通 behavior の和集合を所有する |
| `kivact-test` | App 全体の E2E、Screen、Inspect。公開 view API と engine の試験境界を使う葉であり、view tree の内部を直接読まない |
| `kivact-engine` | 一つの tree/model、計測、入力、paint、a11y、host pump。lower と text_input は内部 module。多数の内部情報を共有する処理を偽の crate 境界で分けない |
| `arvane` | presenter と text、asset、render の組み立て。backend SPI の利用側 |
| `arvane-shell` | OS 非依存の window SPI、入力語彙と正規化 pipeline。winit、softbuffer、AccessKit を持ち込まない |
| `arvane-native` | Windows/Linux の実 window、OS port、入力 mapping、present、支援技術、services、embed、mirror |
| `arvane-virtual` | 台本、仮想 clock、window、surface と SteppedBackend。native mirror を任意に接続する |
| `arvane-text` | text shaping と計測。Parley を内部に閉じる |
| `arvane-render` | CPU/GPU の render。Vello、wgpu を内部に閉じる |
| `arvane-asset` | image、SVG、font の読み込みと decode |
| `kivact-gpu-share` | UI に依存しない GPU texture の import/export 契約 |
| `kivact-ui-basic` / `kivact-ui-pop` / `kivact-ui-game` / `kivact-ui-creative` / `kivact-ui-soft` | 五つの公式 kit。theme、recipes と固有 domain の部品。共通 behavior を複製しない |
| `kivact-template-basic` | アプリの雛形 |
| `kivact-example-counter` / `kivact-example-form` / `kivact-example-list` / `kivact-example-async` / `kivact-example-component` | 公開 API の小さな利用例 |
| `kivact-ui-basic-catalog` / `kivact-ui-pop-catalog` / `kivact-ui-game-catalog` / `kivact-ui-creative-catalog` / `kivact-ui-soft-catalog` | 五つの kit の利用例と recipe の見本 |
| `kivact-public-api-tests` | facade と公式 kit の公開契約の試験 |
| `kivact-docs` | 文書中の Rust のコンパイル |
| `gpu-interop-producer` | GPU 共有を検証する別の描画 producer |
| `xtask` | 依存、API、状態、style と図の検査 |

library は 25、雛形・小例・catalog は 11、試験・道具は 4 package。検査用 package も workspace の数に含める。旧 wire、reactive、task、motion、text、lower の独立 crate は復活させない。

### 境界を作る規則（B）

- B-1: 入出力を少数の型で説明でき、相手の内部状態を調べず、一つの変更理由を持つ単位にする。
- B-2: 明確な境界は crate にして依存、可視性、feature、最適化を制御する。
- B-3: 明確でない境界は統合する。将来の DAG や最適化だけを理由に crate を作らない。
- B-4: crate/module の先頭に英語の `//! Role:` と役割を書く。crate の境界の根拠も説明する。
- B-5: `visible_to` にない利用者は追加しない。新しい依存が必要なら境界の判断と policy を先に変更する。

### module の向き

矢印は「右側を参照する」を表す。正確な許可リストと理由のある例外は policy を正とし、再公開だけの facade を実装の循環と混同しない。

| crate | 依存の向き |
|---|---|
| runtime | `reactive` が基底。`task → reactive`、`motion → reactive`、`machine → reactive/task`、`host → reactive/task/motion`。passive host_types と Runtime を借りる TurnGuard を実行実装から分離 |
| core | `view → state/geometry/task/motion/style`、`platform/diagnostics/gpu_interop → view`、`client → platform/diagnostics/gpu_interop/view`、`launch → client`、`app → launch`、`embed → app/launch/view`。`custom → view/draw` |
| style | asset/length/color などの値 → geometry/path/theme/effect/paint/text → draw/sheet → resolve の順に組み立てる。上位から下位だけを参照 |
| interaction | policy、axis、press、edit、disclosure、focus_return、roving などの小さい behavior を下に置き、list/drag、popup/scalar、dialog/select/menu、table/palette を組む。UI への逆依存は禁止 |
| engine | contract/lower/text_input が基底。tree/mutation → measure → model → text_measure/paint/a11y/check → host/api/embed/testing。model が読む paint/a11y/check の passive state leaf は明示した例外だけ |
| shell | `events → types`、`pipeline → events/types/ime`、`window → events/ime/types/services/a11y/target`。正規化は backend から独立 |
| native | port が present/a11y/services/display を使い、mirror が port/present/a11y を使う。OS 固有の入口から engine へ逆依存しない |
| virtual | clock を script が使い、window が clock/script/surface を使い、backend と mirror がそれを組み立てる |

## 一つの語彙（VOC）

VOC-1 は一つの意味に一つの型、VOC-2 は値を types、closure・動的 dispatch を含む契約を利用側の crate、VOC-3 は同名の異義を作らない、VOC-4 は既定値を `kivact-types::defaults`、VOC-5 は依存のない純粋値とその builder を types に置き facade でそのまま再公開する、という規則である。reactive 値を保持する Style、kurbo の幾何が必要な Path のように、異なる情報・依存を持つ場合だけ包む。記録値は PathData と呼ぶ。

## 状態と事実の所有（S）

| 規則 | 内容 |
|---|---|
| S-1 | 三つ以上の状態、または phase により field の有効性が変わるものは enum にする |
| S-2 | 二つ以上の state bool/Option/counter、または Option と State/Phase enum の併置を検査する。独立した一つの bool を機械的に Machine にしない |
| S-3 | 同時に有効な値を variant の payload に入れる。依存する Some/None を並べない |
| S-4 | 同じ事実を二度保存しない。表示・readonly getter は正本の投影にする |
| S-5 | 遷移だけが状態を書く。effect による状態の補修、同期用 flag、epoch/counter を作らない |
| S-6 | phase の順序は型、状態、runtime guard で表す |
| S-7 | 必要な依存を constructor に渡す。未完成の object を作って後から slot を埋めない |
| S-8 | 通知には Notifier を使う。counter は件数・ID・統計の意味がある場合だけ |
| S-9 | ReportOnce、Sent、Generation、Mailbox、純粋 cache、統計を区別する。異なる事実を無理に一つの phase にしない |

Options/Event/Phase/Reason/Input/Report/Tokens/Recipes、NodeBuilder の builder、Api、policy、純粋値は、それ自体では lifecycle を表さない。passive な record を検査から除くときは、各 field の直前に `xtask-allow(state-fields)` とその field が状態でない正確な理由を書く。型全体の曖昧なコメント、名前による広い除外は使わない。

Creative jobs の run/stop は独立した一つの入力であり、progress/amount は数値の測定事実である。effect が実 FrameLoop の start/stop だけを行う場合、bool を置換するだけの第二の Machine は不要である。

### 事実の持ち主

| 事実 | 唯一の持ち主 / 利用側 |
|---|---|
| keyboard focus | engine Focus。NodeRef の focused は投影。支援技術の focus も publish 時に導出 |
| node pointer capture | engine。region/Part capture は core PartCapture、engine は node の経路を保持 |
| pointer の press、capture、現在の over | engine の pointer ごとの Track。press と capture は別の事実で四つの組合せを許す |
| region hover | engine の node/region を組にした target。primary hover chain と secondary pointer の over を混同しない |
| modifiers / mouse | shell の ModifierState / Mouse。engine に同じ入力状態を複写しない |
| activation の配送 | 一回の routed envelope。入力時の route と dispatch 時の modal admission を分ける |
| editable text と selection | 実 editor/session。別の signal に selection を複写しない |
| caller binding | caller。部品の draft と確定値を区別し、effect で逆同期しない |
| range の選択途中 | RangePick の anchor/preview。最初の click は stage だけ、二つ目で確定。取消は binding を書き戻さない |
| popup の focus 復帰先 | 既存の focus-return owner が opening ごとに実 invoker を snapshot。外側に移った実 focus は維持 |
| host の描画可否 | engine HostPhase と presence/surface。frame clock の gate は投影 |
| 時刻・timer | runtime clock と scheduler。virtual の script clock が同じ経路へ時刻を渡す |
| font generation | engine の font collection。native Environment の更新で巻き戻さない |
| 起動時の Environment | 復元した readonly snapshot。virtual の永続応答 map は台本 owner |
| a11y の前回 accepted tree | 初期化・action 検証用の受理事実。focus や差分用 model の複製ではない |

catalog でも同じ規則を使う。chat entry は Message(text)/Typing（SM-X901）、integrated meter は Unmeasured/Measured と実 Sample/Reset（SM-X902）、active section は単一 owner が Observed/Requested を受ける（SM-X903）。最後の観測を保持し、geometry、scroll、playing、selection の mirror を足さない。

### Machine の書き方

Machine は独自 control のための、小さい reducer に effect と timer を加えたもの。通常の部品利用に必要な prelude の概念にはしない。

1. `Self` を readonly input snapshot、State を Clone + PartialEq の状態 enum、Event と Effect を意味のある値で定義する。
2. `transition(&self, state, event)` は純粋に Step を返す。Step の stay/to/with/after/cancel を使い、field を直接構築しない。
3. `machine(initial, input, run)` は owner に属する。send 時に input を untracked で読み、純粋遷移、状態更新、timer 操作、handler frame の effect 実行の順に処理する。再入した send は FIFO に積む。
4. timer は event の variant ごとに一つ。複数項目で同時に必要なら項目ごとに Machine を持つ。期限は future の最初の poll より前に確定する。
5. feed は指定 source の変化を送る。最初の依存収集では callback を呼ばない。MachineCx は send の口であり、timer や状態の補修用の第二 owner にしない。
6. owner の破棄後は Disposed。timer、task、frame の停止も同じ owner の cleanup に従う。

## 公開 API の 15 規則

| ID | 規則 |
|---|---|
| API-1 | 正準 module path と prelude だけを入口にする。constructor と型を同じ module に置く。behavior/data は interaction、部品/state/variant/recipes/tokens は ui |
| API-2 | 動的な builder option は IntoValue。構造を決める flag/mode、構築前の option 束、passive record は通常の値 |
| API-3 | on_ は event だけ。payload は Fn(E) → R: HandlerResult、payload なしは Into<Callback<()>>。純粋 query は名詞と借用値。handler を位置引数にしない |
| API-4 | 論理座標、寸法、連続値は入力も読み返しも f64。件数/index/level/font weight は整数。整数 literal の変換と異なり reactive closure は f64 を返す |
| API-5 | 見える text は IntoText、ID/name は Into<String>、message key/default は static str。label は control の名前、title は見える container 見出し、Node.name は生の意味情報 |
| API-6 | 一度 mount する slot は IntoView、作り直す content は Fn/IntoViewFn、項目は item ごとの factory。slot 名の語彙を揃える |
| API-7 | use_ は owner に結び付いた behavior または context の読み出し。signal/memo/machine/resource/action/animated/frame_loop/node_ref/toaster/announcer と app-wide environment query に use_ を付けない。provide_x/use_x は対 |
| API-8 | Rc/Arc/Box dyn/Location/明示 lifetime を普通の公開署名に出さない。lifetime の例外は Constraints と Element の借用。実在する内部 cross-crate bridge だけ doc(hidden) と理由付き internal-contract を付ける |
| API-9 | backend の型、公開 path で名前を書けない型を漏らさない |
| API-10 | kit の名前は facade prelude、ui、標準 prelude と衝突させない |
| API-11 | conversion trait の診断を具体的にする。自 crate の enum は所有 crate で constant_value を実装し、それ以外は closure を案内 |
| API-12 | setup/context の misuse は track_caller 付きで、修正方法の分かる panic にする |
| API-13 | 必須位置引数は label/title、data、binding/selection、children。具体 builder と IntoView を保ち、一つの root は NodeBuilder、attach は合成 view を返す |
| API-14 | bool query は is_/has_、それ以外は名詞。get_/_value は使わず、更新は set/動詞、失敗可能は try_。略語は a11y/ime/id 以外を増やさない |
| API-15 | Api、State、Variant、Recipes、Tokens、Options、Policy、Mode、Kind、Event、Phase、Reason を役割どおり使う。InteractionPolicy の入れ子も Policy |

### handler の語彙と実行

HandlerResult は `()`、Handled、Result<(), UiError>、Result<Handled, UiError> の四つを受ける。Handled は Yes/No/PreventDefault を区別し、合成でも PreventDefault を失わない。`noop()` は伝播する。bubble しない callback は Handled を捨てるため、その戻り値で calendar の選択を拒否できない。拒否には実遷移前の activation_guard のような policy を使う。

内部 handler は runtime の一つの Handler<E> に揃える。payload なしの Callback は setup に限らず現在の owner の下で構築できるが、owner なしでは misuse。payload ありは Fn を受け、必要なら Callback.to_fn を使う。user handler、Part、既定動作の順序で伝播と既定動作の防止を区別する。

イベントは on_press、on_edit(EditPhase)、on_submit、on_activate(key)、drag の on_start/on_update/on_end、on_files、on_dismiss(reason)、on_complete/on_exit_complete、on_error(UiError)、on_composition_change、on_active_change、on_add/on_remove/on_reorder とする。selection は binding が正本で、通知のための on_select を足さない。支援技術 action は a11y_action に登録する。

### 構造 option と slot

構造 option は mount 時の node 構成、owner、アルゴリズムの選択を決める。一方、表示値、disabled、caption、calendar の月数・週の始まりなどは reactive にできる。変更できるという理由だけで全 option を IntoValue にしない。

構造の例は DialogMode、keep_mounted、single/multiple selection、ComboboxMode、CarouselMode、RowHeight、variant、keep_focus、modal、region_events、located_at_caller、release_offscreen_focus、TextEditing の multiline/report_text、table の expand_all、overlay の keep_anchor_clear/at_least_anchor_width、Span の italic、Transition の essential/alternate、carousel の looping、drop zone の multiple/pick_on_press、combobox の free_text/chips、disclosure の always_open、empty の announce、drawer/sidebar の side と collapsible、OTP groups/pattern、hover card/menu の delay、step indicator の indicator、skeleton の index、date の presentation である。

ListItem、MenuEntry、Command、TreeNode、DataColumn、ToastOptions、StepperItem は passive な値として構築し、collection の reactive 性は外側に持つ。RenderOptions、AppOptions、ThemeOptions、InteractionPolicy は構築前の private field と setter にまとめる。variant の Default では owner を要する reactive 値を作らず、必要な reactive 値は明示 setter に渡す。

`view::component(FnOnce → IntoView)` は mount 時に一度だけ実行する setup の境界。新しい child owner と透明な fragment の下で hook/cleanup を保つ。eager に factory を呼ぶことや、unit の context を provide して owner を偽造することでは代用しない。再度開く content は必要な factory を保ち、捨てた editor owner を再利用しない。

### 共有データの境界

AssetBytes は private な Arc を持つ不変バイト列で、Asset::Shared、Pixels.rgba、GlyphRun.font が共用する。普通の利用者は所有する Vec などから構築する。Vec から Arc への標準変換に allocation が不要とは保証しない。Pixels::new は Into<AssetBytes> を受ける。

ShapedTextSnapshot と GlyphRunSnapshot は不変描画データを安価に clone する handle で、参照と allocation identity を維持する。custom の TextLayout とは別概念である。SemanticsTextRuns は既存 run の共有 list を隠し、利用者は owned run の Vec/iterator で構築し、iter/get/index は参照を返す。backend の既存共有 allocation は、理由を明記した hidden from_shared/as_shared 等の内部 bridge で再利用する。普通の From<Arc> を追加したり、境界のために Vec を深く clone したりしない。Draw の text measurer 設定は mount/offscreen が自動で行う hidden bridge で、アプリに手動 install を要求しない。

## kit の規約（KIT）

| ID | 規約 |
|---|---|
| KIT-1 | theme(mode) → Theme、追加設定は theme_with(mode, ThemeOptions)。Theme の extension trait を作らない |
| KIT-2 | Tokens は ThemeExtension、ColorName は ThemeColor。標準の tokens 読み出しを使う。色 literal は tokens/theme/color の定義に限る |
| KIT-3 | UiTokens は元の kit override と theme に依存する値、recipes は形の差。全 recipe の値を無差別に token 化しない。README の実際の一覧と UiRecipes.filled/theme_conformance を一致させ、未実装は中核の既定を使う |
| KIT-4 | 公開名は API-10 に従う |
| KIT-5 | slot は API-6、children は IntoChildren。kit 私有の同義 slot/children trait を作らない |
| KIT-6 | 見た目の選択は一つの Variant にまとめる。標準 NodeBuilder の disabled を隠さず、option 束は private field/setter、同義 alias は置かない |
| KIT-7 | kit の use_ は固有 domain に限る。共通 hook は core/interaction、状態は Machine |
| KIT-8 | 標準 message は component.key、kit 固有は kit.component.name。共通意味に複数 key を作らない |
| KIT-9 | kit の E2E は tests/consumer.rs と topic module の一つの binary。Screen.evidence、RenderOptions.with_theme/reduced_motion、Screen.assert_layout_clean を使い、意図した clip の例外は具体的に保つ |
| KIT-10 | 共通 font asset は一か所に置き、App.fonts で登録する。license も同じ asset と管理する |

共通部品へ移すときは旧 kit 全体の option と behavior の和集合を残す。単なる wrapper や再公開で kit に二つ目の入口を残さない。固有 domain は残す。例は Creative の keyframe glyph と column-follow picker、Pop の conversation-preview row、Soft の dictation/streaming/tool-call/token。catalog は削除せず、その kit の recipe で標準部品を使う見本にする。

### 日付・複合部品の共通 behavior

calendar は Single/Range の CalendarSelection binding descriptor を標準 From で受ける。集合の IntoSelection を変更しない。Locale の parse_date/month_name/weekday_name と既存 calendar grid を使い、kit に日付計算をコピーしない。date/time field は TextBox と別の picker button。keyboard で開いた input、button で開いた button を区別し、開いたら実 grid に focus、確定・取消で実 invoker に戻す。外側の click が既に focus を移したときはその先を維持する。

DatePresentation は Adaptive/Desktop/BottomSheet。Adaptive は capabilities.touch を読む。sheet の day cell は theme の大きい値を縮めず最低 44px。day 選択では閉じず Save、月見出しから Month/Year wheel と Done、date-time は Date の Save で Time へ進み次の Save で閉じる。wheel は共通 scalar と Notifier を使い、Up/Increment と PageUp が増加、Down/Decrement と PageDown が減少。物理 gesture の向きは維持する。swipe の Released phase と遷移前 guard で同じ gesture の click を抑え、timer/bool を足さない。range の Escape は stage を取消して dialog の dismiss に伝播する。確定した選択だけを localized Status で通知する。

chat、code block、stat、collections でも同じ原則を適用する。CodeBlock は既存 disclosure の owner と controls 関係、show/presence の高さ motion を使い、第二の disclosure を作らない。history は実 editor の selection を操作する。stat の animation と chart は caller の値と slot を合成する。Contents は layout/paint/input の走査では透過し、a11y の論理 hierarchy と子の bounds の合併は保つ。

## feature と OS（DEP）

DEP-1 は同じ意味の feature 名を揃えること、DEP-2 は feature が自 crate の振る舞いだけを変えること、DEP-3 は OS の判断を port と事実に閉じること、DEP-4 は optional な機能の入口を facade/core と末端の実装に閉じ、中間層を常にコンパイルすることである。別 crate が match する enum variant や構築する struct field を feature で増減しない。

| package | feature と意味 |
|---|---|
| kivact | default は gpu。gpu は arvane-render/gpu、gpu-interop は gpu と render/core/test の共有入口、embed は native/core/test、test は kivact-test、tokio は core/tokio |
| kivact-core | tokio、embed、gpu-interop。gpu/test feature は持たない |
| kivact-runtime | tokio（task の実行 adapter） |
| kivact-test | embed、gpu-interop を core に転送 |
| kivact-engine | testing（virtual/mirror、png、native/test）。これは内部試験の意味なので test に改名しない |
| arvane-native | default は空、embed、test（embed を含む） |
| arvane-virtual | default は空、mirror（native/test） |
| arvane-render | gpu、gpu-interop（gpu を含む） |
| kivact-gpu-share | default は空、export、import、wgpu（export と wgpu adapter） |
| gpu-interop-producer | wgpu |
| public-api-tests | default gpu、gpu、gpu-interop、embed、tokio を facade に転送 |
| docs、template、五つの catalog | default gpu、gpu を facade に転送 |
| その他 | feature なし。五つの kit に gpu-interop 等の中継 feature を置かない |

GPU 無効時も GpuContext/GpuSurface は空 enum、GpuEnv は空の値として同じ名前がある。GPU_BUILT は false、利用不能は NoAdapter/External や FailureKind::Unavailable で説明し、feature のための API 分岐を利用者へ移さない。embed 無効の native open は Unsupported を返す。

OS 分岐の主な持ち主は arvane-native の Windows/Linux port。共有 GPU texture の handle/import/export に必要な分岐だけは kivact-types の gpu、kivact-gpu-share、arvane-render の GPU context に置く。core、interaction、UI、kit は command key、context-menu trigger、pointer class、touch 等を Environment/Capabilities から読む。macOS、Android、iOS、Web backend は未実装で、SPI があることと対応済みであることを混同しない。

## turn、Stage と時間

runtime の TurnState は Quiet / WakeSent / Running(Stage)。wake は Quiet から WakeSent に一度だけ移し、turn 中の pending は finish で計算する。RunningRequested や wants_frame の mirror を持たない。thread の ThreadWaker は UI 状態を触らず host を起こせる。

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

reactive の frame kind（Handler/Setup/Computation）と pure/tracking/owner/writer は別の事実であり、untrack/callback/pure が元の kind を消さない。Watcher の最初の評価は依存と queue を初期化するだけで通知しない。以後の callback は untracked で呼び、値の PartialEq 比較を勝手に加えない。

## 追加・変更するときのチェックリスト

共通して、実際の要求を確認し、一つの事実の持ち主を決め、型・builder・変換の重複を作らず、公開契約の E2E と必要な英日ガイドを更新する。境界の増設だけで配管を増やさない。

| 足すもの | 変更する場所と確認 |
|---|---|
| 支援技術の性質（例 key_shortcuts） | types の Semantics、core の Node setter、native の変換の 3 crate。relation は node/region identity も検証 |
| Role | types の Role と native の role mapping の 2 crate |
| text style（例 text_wrap_style） | types の宣言・computed 値・継承、style builder、arvane-text の 3 crate。relative unit/current color と native 精度を保つ |
| layout/paint の性質 | types の一つの値、style の入口、layout または engine::lower の計算・描画の 3 crate。削除済み lower crate を再設置しない |
| OS service（例 beep） | types の request/result/capability、native の window.perform、core/platform の 3 crate。virtual は台本の応答を登録するだけ |
| Environment field | types、native の事実、core の readonly query の 3 crate。別の environment model を作らない |
| OS backend | native/port に入口、shell の共有 pipeline を利用。core に外部 backend 注入 API を増やさない。virtual が常時第二の SPI 実装 |
| 入力の正規化 | native port で raw code を写し、shell/pipeline で key/IME/DnD を正規化。virtual も同じ道を通す |
| 地域書式 | i18n の Locale と ICU4X。全 locale の compiled data を利用し、kit に en-US table や日付計算を作らない |
| behavior | interaction の State/Event/Effect と Machine、Part の配線、一つの公開 E2E。APG の当該 pattern の keyboard/roles/states を同じ unit で確認 |
| 標準部品 | interaction の behavior、ui の concrete builder/state/variant/Recipes/Tokens、旧 kit 全 option の和集合、実 recipe/getter と README、公開 E2E と英日ガイド |
| 独自 editor | custom の文字編集、IME session、line_styles と decorations。内容と cross-line state を純粋な callback で解析し、byte range を使う。子や独自描画を失わせない |
| アプリの描画表示 | 標準 viewport と canonical ViewTransform。domain 固有の time/value 座標は独立した意味を持つ場合だけ保持 |
| E2E | App 全体を virtual backend と本番 presenter/driver で動かす。mirror は任意。Screen の共通 helper と既存の数値・focus・binding assertion を保つ |

APG は公式の keyboard interaction と roles/states/properties を基準にする。作者の platform context menu、閉じた Select の矢印/Alt+Down、Tooltip の遅延の決定は優先する。規定されていない時間は InteractionPolicy に残す。検証の手順とコマンドは [CONTRIBUTING](../../CONTRIBUTING.md)、背景は[設計判断](decisions.md)を参照する。
