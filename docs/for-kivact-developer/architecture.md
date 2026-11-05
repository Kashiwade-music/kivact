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

### 公開 module と利用者の入口

root は App、Result、UiError、ErrorKind。state、view、geometry、style、draw、motion、task、i18n、platform、diagnostics、interaction、ui、custom を意味ごとの正準 module とし、test/embed/gpu_interop は feature の入口とする。prelude は日常の利用に必要な名前を明示再公開し、同じ型を別の業務 module からも出さない。custom は正式な独自部品境界で、内部 engine の型を直接使わせない。

定数 prop は literal または所有 crate の constant_value で扱う。保存用 Reactive は独自 data 型の作者のための境界であり、通常利用者に Prop や内部 ID を見せない。binding は caller の値の書込み先、Selection は single/multiple の選択、CalendarSelection は日付の Single/Range descriptor であり、異なる情報を一つの conversion trait に押し込めない。

### handler の語彙と実行

HandlerResult は `()`、Handled、Result<(), UiError>、Result<Handled, UiError> の四つを受ける。Handled は Yes/No/PreventDefault を区別し、合成でも PreventDefault を失わない。`noop()` は伝播する。bubble しない callback は Handled を捨てるため、その戻り値で calendar の選択を拒否できない。拒否には実遷移前の activation_guard のような policy を使う。

内部 handler は runtime の一つの Handler<E> に揃える。payload なしの Callback は setup に限らず現在の owner の下で構築できるが、owner なしでは misuse。payload ありは Fn を受け、必要なら Callback.to_fn を使う。user handler、Part、既定動作の順序で伝播と既定動作の防止を区別する。

state の on_mount/on_cleanup は FnOnce の lifecycle 登録であり、node の実装用 on_attach(PartMount) と区別する。イベントは on_press、on_edit(EditPhase)、on_submit、on_activate(key)、drag の on_start/on_update/on_end、on_files、on_dismiss(reason)、on_complete/on_exit_complete、on_error(UiError)、on_composition_change、on_active_change、on_add/on_remove/on_reorder とする。selection は binding が正本で、通知のための on_select を足さない。支援技術 action は a11y_action に登録する。

### 構造 option と slot

構造 option は mount 時の node 構成、owner、アルゴリズムの選択を決める。一方、表示値、disabled、caption、calendar の月数・週の始まりなどは reactive にできる。変更できるという理由だけで全 option を IntoValue にしない。

構造の例は DialogMode、keep_mounted、single/multiple selection、ComboboxMode、CarouselMode、RowHeight、variant、keep_focus、modal、region_events、located_at_caller、release_offscreen_focus、TextEditing の multiline/report_text、table の expand_all、overlay の keep_anchor_clear/at_least_anchor_width、Span の italic、Transition の essential/alternate、carousel の looping、drop zone の multiple/pick_on_press、combobox の free_text/chips、disclosure の always_open、empty の announce、drawer/sidebar の side と collapsible、OTP groups/pattern、hover card/menu の delay、step indicator の indicator、skeleton の index、date の presentation である。

ListItem、MenuEntry、Command、TreeNode、DataColumn、ToastOptions、StepperItem は passive な値として構築し、collection の reactive 性は外側に持つ。RenderOptions と AppOptions は private field と setter にまとめる。ThemeOptions と InteractionPolicy のような FRU の設定値は公開 field を持ち、構築前の通常値として渡す。variant の Default では owner を要する reactive 値を作らず、必要な reactive 値は明示 setter に渡す。

`view::component(FnOnce → IntoView)` は mount 時に一度だけ実行する setup の境界。新しい child owner と透明な fragment の下で hook/cleanup を保つ。eager に factory を呼ぶことや、unit の context を provide して owner を偽造することでは代用しない。再度開く content は必要な factory を保ち、捨てた editor owner を再利用しない。

API-11 の診断は blanket impl の内側の bound を rustc が先に報告する場合もある。診断属性に加えて rustdoc の冒頭とガイドにも同じ修正 hint を記し、属性の文言が常に表示されることまでは前提にしない。

API-8 は診断と標準 Rust Error の実際の境界にも限定例外を持つ。UiError/ErrorKind と diagnostics の Trace/NodeInfo/Snapshot の source location、および UiError.from_boxed の Error interop は元の意味を保ち、通常の部品へ拡大しない。理由は定義の直前に記す。__private や cross-crate bridge の到達性を止めるのは doc(hidden) と理由付き internal-contract の両方がある実装 edge だけである。IntoValue/IntoText の内部変換、Value/Binding の runtime/location、TextParagraph の backend adapter が該当する。外部部品の契約である NodeBuilder.node_mut は単に hidden であることだけでは検査対象から外さない。

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

## 検証と保守の単位

一つの関数は80 code行、fileは1000 code行を上限とする。長い処理は意味のある helper に分け、line packing、無意味な型/module、新しいstate、抑制属性で件数だけを減らさない。依存、状態、effect書込み、公開署名、正準path、名前衝突、kit色、OS分岐、構造optionを全workspaceで検査する。検査の例外は実際の境界と理由に限る。

behavior の保存は既存 E2E の契約で確認する。trace は失敗診断用で、golden やf64の丸め一致を作らない。windowを開く試験は全worktreeで同じlockを使う。通常は既定並列、実際に同時doctest compile/linkでlink.exe1180とrustc ICEが起きたためkivact-docsだけ別実行でtest-threads=1とする。GPU試験は別の逐次実行。Cargo job数は制限しない。これはCPU buildの並列性とwindow/focus/実際のlink障害を混同しないための区別である。

変更後の意味情報も実 node から検証する。AsyncViewの既定pendingはStatus Loading、pickerエラーはAlert、Contentsはlayoutの透明性とa11y Group/論理階層を両立する。Desktop操作を検証するdate fixtureは明示Desktop、一般のAdaptive fixtureはtouchのSaveを保つ。iconは実install済みglyphで確かめ、timeoutやdescription relation欠落を期待値の緩和で隠さない。

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

### 最初に読む実装の場所

以下は追加の入口となる現行fileである。新しい性質に必要なleafを選び、すべてを無条件に変更しない。

| 変更 | 主なfile |
|---|---|
| semantics/Role | types の `src/semantics/node.rs`・`semantics/values.rs`、core の `src/view/node_modifiers.rs`・`view/mount/semantics.rs`、native の `src/a11y/conversion.rs` |
| text style | types の `src/text/properties.rs`・`text/computed.rs`・`text/primitives.rs`、style の `src/text.rs`・`sheet.rs`、arvane-text の `src/build.rs` |
| OS service | types の `src/platform.rs`・`env.rs`、native の `src/services.rs` と `port/<os>/services.rs`、core の `src/platform/services.rs` |
| layout/paint | types の `src/layout.rs`・`paint.rs`、style の `src/sheet.rs`・`paint/`、layout の `src/style/` または engine の `src/lower/` |
| 標準部品 | interaction の該当behavior、ui の該当部品と `tokens.rs`・`recipes.rs`、公開API試験の `tests/e2e/`、英日guideとkitの実recipe一覧 |

APG は公式の keyboard interaction と roles/states/properties を基準にする。作者の platform context menu、閉じた Select の矢印/Alt+Down、Tooltip の遅延の決定は優先する。規定されていない時間は InteractionPolicy に残す。検証の手順とコマンドは [CONTRIBUTING](../../CONTRIBUTING.md)、背景は[設計判断](decisions.md)を参照する。


### 具体的な追加経路の机上確認

以下は機能を実装せず、現行ソースから必要な変更箇所を列挙した確認である。新規と記した file は追加先の案であり、まだ存在しない。件数は同じ file を重複せず数え、runtime 実装、再公開だけの変更、試験、ガイドを区別する。既存の共通 transport を通るだけの crate は変更件数に含めない。これらの機能が実装済みであることは意味しない。

#### key_shortcuts: 意味情報の性質

| file | 変更する理由 |
|---|---|
| `crates/kivact-types/src/semantics/node.rs` | Semanticsに省略可能なshortcut文字列とDefaultを追加し、map_relationsで非relation値としてそのまま移す。relationのgenericな写像と順序は変更しない |
| `crates/kivact-core/src/view/node_modifiers.rs` | Nodeの固有setterから既存Common.semanticへreactive text patchを登録する。builderはwith_nodeを使え、NodeBuilderへ同じsetterを複製しない |
| `crates/arvane-native/src/a11y/conversion.rs` | 確定したSemanticsの値をAccessKitのshortcut属性へ写す |

**3 runtime crate、3 file。** engineとshellは同じSemanticsを運ぶため変更不要。新しいSemanticInput/source getterはこの性質をbehaviorが読む要求がない限り足さない。Common.semanticとmountのpatch foldも既存のまま使う。

#### text_wrap_style: 継承するtext styleの性質

新しいTextWrapStyleの値を定義し、通常paragraphとinline paragraphに同じ改行方針を適用する場合を数える。wrapするかどうかを表す既存TextWrapとは別の意味であり、boolへの読み替えや既存enumへの無関係なvariant追加で件数を減らさない。

| file | 変更する理由 |
|---|---|
| `crates/kivact-types/src/text/primitives.rs` | 新しい純粋値TextWrapStyleと既定を定義する |
| `crates/kivact-types/src/text/properties.rs` | 宣言TextStyleに省略可能な値を持たせる |
| `crates/kivact-types/src/text/computed.rs` | computed値、既定、propertyごとの継承を一か所で定義する |
| `crates/kivact-types/src/text.rs` | 値をtext語彙として再公開する |
| `crates/kivact-types/src/lib.rs` | 他crateが使う正準の共有値を再公開する |
| `crates/kivact-style/src/text.rs` | TextExtrasの入力、merge、resolve_extrasによる共有TextStyleへの変換、および値の再公開。relative length等の既存意味は保つ |
| `crates/kivact-style/src/sheet.rs` | text_wrap_style setterを既存text_setterに加える |
| `crates/kivact-style/src/lib.rs` | authored styleの公開値として再公開する |
| `crates/arvane-text/src/build.rs` | backend側の改行方針の適用を共通helperへ集め、paragraph/inlineが異なる規則を持たないようにする |
| `crates/arvane-text/src/text_engine.rs` | 通常paragraphのbreak処理から共通方針を使う |
| `crates/arvane-text/src/inline.rs` | inline objectを含むparagraphのbreak処理にも同じ方針を使う |
| `crates/arvane-text/src/cache.rs` | 計測・描画結果のTextKeyへ新しい入力を反映し、違う方針の結果を再利用しない |
| `crates/arvane-text/src/layouts.rs` | 再利用するParley layoutのKeyを確認し、新方針を保持する構築状態もkeyへ反映する。幅だけの再改行とshape入力の区別は保つ |
| `crates/kivact-core/src/style.rs` | facadeのstyleは明示再公開なので、新しい値を利用者が名前で書けるように再公開する。新しい変換やadapterは不要 |

**機能の実装は3 crate・13 file、再公開だけのcoreを含む実際の変更先は4 crate・14 file。** engineは既存ComputedTextStyle.inheritと共有値をそのまま使うため、追加のstyle modelやadapterは不要。既存resolve.rsはTextExtras全体をresolve_extrasへ渡すので、この方針では変更しない。coreのexportを無視して「全変更が3 crate」とは数えない。新しい値を公開せず内部だけに限定する、という別のscopeとも混同しない。

#### beep: windowが答えるOS service

PlatformRequestのBeepとCapabilitiesの対応可否を追加し、WindowsとX11では実service、Wayland等の未対応環境ではUnsupportedを返す場合を数える。成功は既存PlatformResult::Done、実OS失敗はFailedであり、requestごとの新しいresult型やshell methodは不要である。

| file | 変更する理由 |
|---|---|
| `crates/kivact-types/src/platform.rs` | PlatformRequestにBeepを追加する。resultは既存の値を再利用する |
| `crates/kivact-types/src/env.rs` | 実windowのCapabilitiesにbeep可否を加え、未設定の既定をfalseとする |
| `crates/arvane-native/src/port/beep.rs`（新規） | UI thread上で実native handleの種類に従いWindows/X11のbellを実行する共通leaf。対応判定も同じ実装から得て、Wayland等では未対応を明示する |
| `crates/arvane-native/src/port.rs` | 新しいnative leafを宣言する |
| `crates/arvane-native/src/port/winit/runtime.rs` | 完成したwindowのperformの分岐とcapabilitiesを更新する。Linuxというcompile-time条件だけでWaylandを対応扱いしない |
| `crates/arvane-native/src/port/win32/window.rs` | embed windowの同じperformとcapabilitiesを共通leafへつなぐ |
| `crates/arvane-native/src/port/x11/window.rs` | X11 embed windowの同じperformとcapabilitiesを共通leafへつなぐ |
| `crates/kivact-core/src/platform/services.rs` | 既存ask/HostRequest::Platformを使う利用者向けbeep関数と結果の変換を加える |
| `crates/kivact-core/src/platform.rs` | 同じplatform moduleから関数を再公開する |
| `crates/arvane-virtual/src/window.rs` | 既存performの網羅的matchとCapabilitiesの全fieldを明記したliteralを追従させる。respondが先に応答でき、既定は実音を鳴らさずDoneとする |

**本番の機能実装は3 crate・9 file、virtual backendの追従を含む実際の変更先は4 crate・10 file。** shellの既存window.perform、arvaneの転送とcapabilityのstruct-update、engineのHostRequest::Platform配送、coreのrequest waiterは変更不要。virtualのrespond登録はgenericだが、現在の網羅的matchと全field literalは型追加だけではコンパイルできないため、追従1 fileを省略して数えない。virtualの動作を試験することと、service adapterをもう一層追加することは別である。

#### background_blend_mode: 背景だけのpaint性質

ここでは既存BlendModeを一つ指定し、背景のlayer間だけで合成する性質を仮定する。box全体と外部backdropのmix-blend-modeとは区別する。

| file | 変更する理由 |
|---|---|
| `crates/kivact-types/src/paint.rs` | PaintPropsと必要なpaint差分にbackgroundのblend値を追加し、既定Normalを定める |
| `crates/kivact-style/src/sheet.rs` | authoredな背景blend入力、setter、style mergeを追加する |
| `crates/kivact-style/src/resolve.rs` | state styleを含む背景groupの解決と差分判定でこの値を運ぶ |
| `crates/kivact-engine/src/paint/frame_scene.rs` | paint_boxが背景を描く箇所で確定値を背景専用lower helperへ渡す |
| `crates/kivact-engine/src/lower/fill.rs` | 背景を隔離したgroup内で下から上へ合成し、既存RenderCmdのPushLayer/PopLayerとLayer.blendへ下ろす。Normalの既存fast pathを保つ |

**3 runtime crate、5 file。** BlendModeは既にkivact::drawから名前を書け、render命令とCPU/GPU backendは既にlayer blendingを扱う。arvaneやarvane-renderにproperty名のadapterを足さない。一般のfillを呼ぶtextやcanvasへ背景のpolicyを流さないため、既存fillの署名を一律に変更しない。

#### Rating: previewを持つ標準部品

pointer/focusによるpreviewとcaller bindingの確定値を区別する小さいratingを仮定する。既存role、keyboard、pointer、a11y actionを使い、engineへrating固有の意味を追加しない。

| file | 変更する理由 |
|---|---|
| `crates/kivact-interaction/src/rating.rs`（新規） | use_rating、Api、previewのState/Event/遷移とPartを定義。callerの確定bindingと実previewの単一ownerを持ち、選択値のmirrorを作らない |
| `crates/kivact-interaction/src/lib.rs` | 新しいbehaviorのmodule宣言と正準再公開 |
| `crates/kivact-ui/src/rating.rs`（新規） | label/bindingを取る具体Rating builder、NodeBuilder/IntoView、slotと既存behaviorの配線 |
| `crates/kivact-ui/src/rating_values.rs`（新規） | 実behaviorからのreadonly recipe stateと部品固有Tokensを定義 |
| `crates/kivact-ui/src/look_rating.rs`（新規） | root/itemの既定recipeをthemeと実stateから作る。viewや選択controllerは作らない |
| `crates/kivact-ui/src/recipes.rs` | RatingRecipesの表、既定、override/read getterとUiRecipesへの登録 |
| `crates/kivact-ui/src/tokens.rs` | UiTokens.ratingを登録。themeに依存する寸法等だけを扱い、behavior policyを入れない |
| `crates/kivact-ui/src/lib.rs` | 新しいleafの宣言とconstructor/builder/state/recipes/tokensの正準再公開 |
| `dev/tests/kivact-public-api/tests/e2e/ui_rating.rs`（新規） | keyboard/pointer/a11y、preview取消と確定binding、disabled、focusを一つの公開E2Eの契約として確認 |
| `dev/tests/kivact-public-api/tests/e2e/main.rs` | 同じE2E binaryへ新topicを登録 |
| `dev/tests/kivact-public-api/tests/e2e/theme_conformance.rs` | 新しいrecipeの実getterとdefault/overrideの適用を照合。未overrideのkitは中核既定を使う |
| `docs/for-kivact-user/en/how-to-guide/rating.md`（新規） | 公開APIの利用とpreview/確定の説明 |
| `docs/for-kivact-user/jp/how-to-guide/rating.md`（新規） | 同じ利用契約の日本語ガイド |

**runtimeは2 crate・8 file。公開E2Eを含め3 package・11 Rust file、英日ガイドを含め13 file。** facadeはinteraction/uiのcrateをそのまま公開しているためfacade fileの変更は不要。例は正準ui moduleからimportし、新しいprelude入口や同義wrapperは追加しない。kitごとの別behavior、新しいengine role、backend adapterも不要である。

#### 件数のまとめ

| 追加例 | 機能実装のcrate / file | 再公開・検証も含む変更先 | 既存境界で不要なもの |
|---|---|---|---|
| key_shortcuts | 3 / 3 | 3 crate / 3 file | engine/shellの意味情報の別コピー、NodeBuilderの同義setter |
| text_wrap_style | 3 / 13 | 4 crate / 14 file（coreの再公開1 file） | engineの継承model、styleからbackendへの別adapter |
| beep | 3 / 9 | 4 crate / 10 file（virtualの追従1 file） | shell/arvaneのservice別method、engineのservice別転送 |
| background_blend_mode | 3 / 5 | 3 crate / 5 file | render命令のvariant追加、CPU/GPU adapter |
| Rating | 2 / 8 | E2E込み3 package / 11 Rust file、英日guide込み13 file | facade wrapper、engine変更、kit別behavior |

この確認はcodeを書かない列挙だけで完了する。変更先の正確な総数と、機能の意味を実装する3層という境界の根拠を別に記した。再公開やvirtual backendの網羅性に必要な追従を隠して、総数まで3であるとは主張しない。
