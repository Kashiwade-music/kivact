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
| 2026-11-05 | ADR-21 | 非同期の最新値と phase を統合するか | AsyncCell の phase と latest は異なる事実。pending/failed 中も成功値を保持し T に PartialEq を要求しない。待機 queue があれば失敗 callback 後も Pending。ActionPhase は close 後も queue を drain。PoolShutdown は root cleanup 後、TaskCell の OnceCell は初 poll 前に実 key を設定 | 成功値保持、待機処理、取消と shutdown の契約を失わないため |
| 2026-11-05 | ADR-22 | App 全体の状態をどこへ置くか | constructor で固定する Extensions と root owner に属する遅延 App state を分ける。unsafe な増設 map AppStates を作らない | 寿命と構築時点の異なる情報を一つの可変 registry にしないため |
| 2026-11-05 | ADR-23 | 内部 handler を層ごとに定義するか | runtime に一つの Handler<E> を置き、frame/task/core で共有。unit payload も同じ model | 同義の callback slot を繰り返さないため |
| 2026-11-05 | ADR-24 | Machine の実装をどう小さくするか | Self を input snapshot、State/Event/Effect を関連型にし、純粋 transition が Step を返す。send は再入 FIFO、timer は event variant 単位、feed の初回は無通知、破棄後は Disposed。MachineCx は send の口だけ | 状態更新・timer・effect を一つの owner に保ち、小さい reducer として説明するため |
| 2026-11-05 | ADR-25 | 起動 API と option 束をどう統一するか | private AppOptions に launch 設定を集め App が委譲、RenderOptions はそれを包む。font の登録元 location は private、blocking は None、theme は IntoValue。戻り値が異なる render_with_handle は残す。内部 engine feature は testing のまま | 同義設定と入口を減らし、実際に異なる返却契約は消さないため |
| 2026-11-05 | ADR-26 | Inspect が何を読むか | core 所有の errors/pending/outstanding_blocking/motion_ticks、engine Reader の stats を公開投影にする。backend の E2E が使う private harness は残すが raw 公開観測 API は増やさない | 第二の tree/model を持たず、残る内部契約を試験するため |
| 2026-11-05 | ADR-27 | style の内部口と themed 色をどう扱うか | compile/install/record は理由付き hidden free function bridge。Style.merge は recipes 用に公開。display も公開。THEMED の process intern cache は残す | Color を Copy のまま theme 非依存にする実際の役割があり、cache を lifecycle と誤認しないため |
| 2026-11-05 | ADR-28 | 地域書式を中間 table に変換するか | ICU4X compiled data を全 locale 分含め、Locale が一度だけ formatter を作る。CalendarDate の from/to days を共通計算にする | kit 固有の en-US table、二重変換、手書き calendar 計算を避けるため |
| 2026-11-05 | ADR-29 | 削除結果をいつ handler に渡すか | Removed を borrow 終了後に消費する。論理削除した node の handler は停止し、退場 paint は保つ。node でない ID は HashMap、on_exit は Handler の Option | borrow と callback の再入を分離し、退場中の二重入力を避けるため |
| 2026-11-05 | ADR-30 | fragment に専用 NodeKind を作るか | Display::Contents の実 node とし、すべての走査で共通 flat_children 判定を使う。Reader の論理親子は leaving を除外、最終退場まで透明 ancestor を保持、detached subtree も実 node | layout の透過性と論理 ownership を同時に保つため |
| 2026-11-05 | ADR-31 | Part が独自 hover を保存するか | engine の FocusTarget(node,region) をそのまま読み、正確に一致する region だけ hovered | node だけの比較や第二の hover state は region の意味を失うため |
| 2026-11-05 | ADR-32 | handler の返り値をどう揃えるか | HandlerResult 一つに ()、Handled、Result<(),UiError>、Result<Handled,UiError> の四実装。Handled の PreventDefault を合成でも保つ | 処理済みと既定動作の禁止を区別し、error も同じ入口で扱うため |
| 2026-11-05 | ADR-33 | Callback の構築は setup に限るか | payload なし Callback は現在の owner があれば handler/effect 下でも構築可能。owner がなければ misuse | 正当な owner-bound callback の利用を不必要に制限しないため |
| 2026-11-05 | ADR-34 | 外部 Element 実装を閉じるか | node_mut は公開だが doc(hidden)、trait は seal しない。with_node は普通の公開 API | 外部 builder が標準 setter を継承できる実装用の口が必要だから |
| 2026-11-05 | ADR-35 | 公開 API の改名をどう進めるか | 層をまたぐ署名は family 単位で一度に変え、改名と振る舞いの変更は別の変更に分ける | 同じ利用箇所を何度も書き直すことと、改名に紛れた振る舞い変更を避けるため |
| 2026-11-05 | ADR-36 | InteractionPolicy の所属と形はどうするか | interaction に inputs/menu/tooltip/button/slider/switch/toast/select/hover_card/drawer の nested policy を置く。ui が依存し、逆向きにしない。tooltip に第二の context config を残さない | behavior の設定の正本を一つにするため |
| 2026-11-05 | ADR-37 | UiTokens にすべての recipe 値を移すか | 部品ごとの nested tokens に、kit が上書きする値と theme 依存値だけを置く。形は recipes。data table の sticky/status paint 値も Tokens | recipe の値を無差別に token 化すると設定面が増え、形と値の境界も失われるため |
| 2026-11-05 | ADR-38 | Disclosure の既定 motion は何か | motion は animate=false、duration=200ms。部品ごとの animated reactive override は残す | 標準の既定と kit の明示指定を両立するため |
| 2026-11-05 | ADR-39 | overlay_margin の None を何に使うか | Theme.overlay_margin は常に数値で既定 0。必ず window 内に fit し、no-fit sentinel を作らない | 余白と fitting の有無を一つの Option に混ぜないため |
| 2026-11-05 | ADR-40 | すべての touch gesture を Drag に統一するか | DragApi に slop（既定0）、axis lock、100ms velocity sampling、公開 DragVelocity を追加。実 drag 部品は共用。touch scalar/switch/toast/select/month の Machine は既存の閾値と activation timing のまま | Drag のために三つの新概念や異なる gesture phase を強制しないため |
| 2026-11-05 | ADR-41 | APG への対応を後から別に行うか | behavior を変えるときは同じ変更で公式 APG の keyboard/roles/states に合わせ、作者の決定による差分を記録し E2E で確認する | 後から別に合わせると状態機械と behavior を二度作り直すため |
| 2026-11-05 | ADR-42 | toast/tree 行が独自の表示 phase を持つか | engine presence で出現・退場を描く。visibility 配列や animated height Machine を複製しない | 一つの表示 lifecycle に揃えるため |
| 2026-11-05 | ADR-43 | editor の syntax と装飾 API はどうするか | line_styles は content T と cross-line state S を純粋 callback へ渡す。StyledRange は byte range と色/装飾、decorations は全文対象。CodeBlock の highlight は既存 parser と palette を保つ | 行ごとの解析状態と編集内容を二重保存せず、tokenization を失わないため |
| 2026-11-05 | ADR-44 | text 色の Transition と未確認の option を増やすか | Paint が text color を担うため新 Color TransitionProp は不要。Combobox/Select の option は kit の利用者が実際に使っていた option の和集合だけを実装 | 同義の表現と要求のない knob を増やさないため |
| 2026-11-05 | ADR-45 | touch の日付入力は kit 固有か | DatePresentation Adaptive/Desktop/BottomSheet を構造 option とし、Adaptive は capabilities.touch。calendar/date/time/wheel は標準 UI と scalar に統合し kit の option をすべて維持 | 同じ日付入力の behavior と OS 判断を kit に重複させないため |
| 2026-11-05 | ADR-46 | Creative picker を全て標準 combobox に置換するか | 共通 body は標準 API を使うが、Creative 固有の column-follow picker は kit に残す | column に追従する domain の意味が一般の選択 field と異なるため |
| 2026-11-05 | ADR-47 | 一度だけ mount する setup closure をどう渡すか | view::scope(FnOnce→IntoView) と prelude を公開。mount 時に child owner/transparent fragment の下で一度実行し、context を登録しない。Variant Default は通常値、reactive 値は明示 setter | eager 評価、provide(())、recipe 評価中の hook で owner と cleanup の寿命を変えないため |
| 2026-11-08 | ADR-48 | 複数の window を扱う shell の境界をどうするか | 一つの handler が loop 全体と複数の window を相手にし、backend の event を `EventTarget::{Loop, Window(WindowId)}` で分ける。handler の口は `ShellLoop`・`ShellWindow` とし、`open_window`・`close_window` を持つ | window ごとに backend と handler の組を増やさず、loop 全体の event（SaveState・Wake など）と window の event を一つの型で区別するため |
| 2026-11-08 | ADR-49 | engine の model を App 全体で一つにするか window ごとに分けるか | 一つの window の `Model` と App 全体の `AppModel` に分ける。`AppModel` が `windows`・`window_changes`・`AppClock`・`client_frames`・共有の `EventSink` と assets と writes を持つ | window ごとの tree・focus・surface を保ちながら、motion の時計と frame の要求を App に一つにし、ある window の surface が止まっても他の window の animated が進むようにするため |
| 2026-11-08 | ADR-50 | OS の window を公開 API でどう表すか | `window(title, open)` を `overlay` と同じ構造の view とし、content は宣言した場所の context を引き継ぐ（React の Portal と同じ）。閉じる操作は `on_dismiss` が取り消さない限り既定で window を閉じる（`dialog` と同じ） | open の binding で寿命を表し、overlay・dialog と同じ組み立てと dismiss の規則を共有するため |
| 2026-11-08 | ADR-51 | DevTools を target の中の UI にするか別プロセスにするか、protocol の定義をどこに置くか | DevTools は別プロセスのアプリとし、loopback TCP の JSON-RPC で target の agent に繋ぐ。protocol の型・method・通知は `kivact-devtools-protocol` を単一の正本とし、JSON Schema・Python・MCP をそこから生成する | DevTools の不具合で target を止めず、同じ語彙を Rust・Python・MCP に複製しないため |
| 2026-11-08 | ADR-52 | 状態の合成を許す印を型の前と field の前に書けるようにするか | 印は field の直前だけに置き、型の前の印は error にする。印の無い stored の field を名前付きで error に並べる | 型の前の印が中の field すべてを黙らせ、どの field が何の事実かを失うため |
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
| 2026-11-12 | ADR-64 | SVG の書き出しを別の型にするか | 別の型を作らず `Offscreen::render_svg` とし、書き出しは `kivact-svg`（DrawList → SVG）に置く。SVG 1.1 で表せない命令・値は Unsupported の error にし、黙って落とさない | theme・font・cache の設定と記録の経路を PNG と一つにし、同じ描画関数で両方を書き出すため。png と SVG の知識を一つの葉に閉じ、engine を知らずに試験できるため |
| 2026-11-12 | ADR-65 | 数値の XY plot をどこに置き、範囲の操作をどう表すか | basic kit の plot とし、範囲は軸の値の座標（PlotRange）で持つ。pan は interaction の Drag、wheel・key・double click は EditSession の 1 回の編集にし、範囲の binding を直接書いて on_edit(EditPhase) で報告する。等方の ViewTransform は使わない | 構造化データは kit が描く。x と y を対数軸で別々に拡大するため等方の変換では表せない。操作の取消しと確定を他の部品と同じ EditPhase に揃え、確定だけの通知を別に作らないため |
| 2026-11-12 | ADR-66 | game と pop の sparkline をどこに置くか | kivact-ui の標準部品 `sparkline(values)` にし、線・塗り・余白は `UiTokens::sparkline`、大きさは recipe で kit が決める。kit の sparkline は削除する | 二つの kit が同じ意味の部品を持っていたため（strategy 6 章の 5）。外観だけが kit ごとに違う |

### 実装の決定の補足

次の細部も上表と同じ日付の決定に含む。問いと理由は対応する ADR を参照する。

- ADR-02: Backend は run だけで、事前 capabilities/open_child/shutdown は持たない。ShellOptions は native constructor の引数で寸法検証もそこで行う。embed 内部の同名 Backend は ChildPort とした。native への通常依存は持たず、native/embed の E2E のための arvane の dev-dependency は許す。embed Session は handler を drop した直後、DetachedThreads があれば module を保持する。window 作成前の capability は対応可能な上限で、native setup 後の実 ShellWindow の capability が正本である。
- ADR-03: SteppedBackend.start は window を作り handler.start を呼んでから返す。Idle は next_deadline/redraw_requested/woken を持つ。試験 host は handler を loop と共有し、engine の inject は試験専用で残す。node query は同じ handler から読む。a11y の公開は frame 後に揃え、turn 後に redraw 要求があれば必ず描き、見たときだけ描く経路は持たない。
- ADR-03: GPU の renderer 通知を待つ wake は最大5秒。frame の readback は keeps_frames の window だけで行い keep_frame へ渡す。test_font/gpu_unavailable は presenter/text を知る arvane root に所属し、arvane に test feature は置かない。arvane の E2E は handler 内で実行する Harness.with、Script、Probe で契約を保つ。virtual に winit を持ち込まない。
- ADR-04: raw は Key/Ime/Drop。PlatformKey は Vk（physical code付き）/Keysym/Code で winit 型を含めない。IME は Preedit/Commit/End/Reconvert、DropSession は Idle/Hovering/Dropped。Win32/winit は repeat を明示し、X11 の未指定だけ押下集合から導出する。Screen.raw_input から Script.raw を通す公開試験口を持つ。IME の詳細は IME-01 を参照する。
- ADR-05: CPU の native 提示形式は Xrgb8888。clipboard/dialog 等の個別 ShellWindow service method を並べず perform に統合する。現行対象はすべて同期に回答するため、返信から Host/window に戻る別経路は作らない。CommandKey の既定は Ctrl、ContextMenuTrigger は Release。virtual は memory clipboard、dialog queue、記録を保ち、respond の request→optional result callback で上書きする。別の PlatformRequestKind は不要。
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
- ADR-27: theme install/query、style/span compile、record、scroll-axis adapter は hidden internal free function とする。THEMED は縮まない ThemeColor interning cache であり theme ごとの可変状態ではない。
- ADR-29: Removed の ids は論理削除した subtree の前順。handler records は tree borrow が終わってから drop。退場 paint が残っていても core handler、asset failure と NodeRef の detach は論理削除で処理する。
- ADR-30: flow の show/dynamic/for_each/for_index/virtual-list行/error-boundary/provide は Contents の実箱を作る。箱自体にsemantics/hit/focusを付けず、未挿入の部分木も親なしの実箱として構築可能。a11y の論理 hierarchy は BOUNDARY-05 を参照する。
- ADR-32〜34: HandlerResult の変換 method と Element.node_mut は hidden な実装口。外部 builder の実装可能性は維持する。payloadなしcallbackをownerの種類で不当に制限しない。
- ADR-36: policy は Clone/Debug/PartialEq/Default と公開 field を持つ FRU 値。tooltip delay/skip は500/300ms、hover card open/close は600/300msで部品別上書きも残す。toast slop8px、select slop10px、menu drift context10/other13px。drawer close fraction0.3、flick speed0.6、distance4。現在の追加 field と正確な署名はrustdocを参照する。
- ADR-37: 初期 tokens は button busy、progress motion、toast row motion、toggle label reservation、dialog center、disclosure motion、data table sticky/row-status。各部品が所有する nested struct を追加して拡張する。値だけが違う recipe を token へ移すことはしない。
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
| 2026-11-05 | BOUNDARY-04 | authored text decoration に backend 型を出すか | Color/TextLength を入口とし Rgba/TextLen は内部解決値。Draw と呼び Canvas alias は置かない | API の値と backend 表現を分け一つの名前にするため |
| 2026-11-05 | BOUNDARY-05 | Contents と a11y の論理構造をどう両立するか | layout/paint/input の透過走査を共有しつつ、意味のある論理 hierarchy を a11y で保ち、bounds は実 children の合併にする | 透明 layout を理由に親子の意味や正確な bounds を失わないため |
| 2026-11-07 | BOUNDARY-COMPOSE | 合成をどこに置くか | `arvane-compose` を新設し、Vello は layer の 2D ラスタライズだけにする | 3D、cache、shader、外部 texture は、Vello の外で一つの合成として扱うほうが単純であるため（Chrome の Skia と cc の分担と同じ） |
| 2026-11-07 | BOUNDARY-SHADER | WGSL の検証をどこに置くか | `kivact-shader`（naga だけに依存） | style が構築時に検証して UiError を返せ、style に wgpu を持ち込まないため |
| 2026-11-05 | API8-01 | immutable bytes と shaping の共有を利用者へ露出するか | AssetBytes、ShapedTextSnapshot、GlyphRunSnapshot、SemanticsTextRuns で private Arc を隠す。owned Vec/iterator と参照 query を普通の入口にし、実 backend の shared allocation は理由付き hidden bridge で再利用 | allocation identity/caching/cheap clone を維持しながら ordinary API の Arc をなくすため。Vec 変換の allocation 不要は保証しない |
| 2026-11-05 | API8-02 | Draw の measurer をアプリが install するか | set_text_measurer は doc(hidden) と precise internal-contract を付す自動 mount/offscreen bridge | 普通の Draw 利用に手動 install の順序を要求するのは不適切であり、実際の呼出者は内部二経路だけのため |
| 2026-11-05 | API8-03 | 診断とRust Errorの情報まで同義wrapperで隠すか | UiError/ErrorKindとdiagnosticsのsource Location、UiError.from_boxedの標準Error interopは定義上の理由付き例外にする。普通の部品には広げない | 診断の出所と標準Errorの相互運用は実際の境界であり、同義wrapperを増やすと意味と使いやすさを失うため |

## 公開 API の契約

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | CONTRACT-01 | 部品の constructor は opaque な Component を返すか、具体 builder を返すか | 固有 option を持たない合成部品は impl Element を返す。固有 option を持つ標準・kit の constructor は具体 builder を返し、fluent API を保つ。アプリの部品は #[component] で props の builder を生成し、impl IntoView を返す（CONTRACT-08） | opaque な型は固有 option を呼び出し側から隠してしまうため |
| 2026-11-05 | CONTRACT-02 | ScrollSnap をどう表すか | ScrollSnap は None/Start/Center/End とし、ScrollAxes の方向で子を整列する。可変サイズの子を扱い、raw pixel の停止位置の別設定は増やさない。nearest_snap と DragVelocity は types の一つの純粋実装を interaction が再公開する | snap の位置を子の配置から導き、同じ計算を二か所に持たないため |
| 2026-11-05 | CONTRACT-03 | window の寸法をどこまで受け入れるか | window の寸法は少なくとも 1 logical pixel、有限、unsigned の native extent 内とし、App の起動と resize が同じ判定を使う。native は scale 後の physical size も検証する | f64 で有限になったことを理由に不正な寸法を通さず、f32 の underflow に共有 API を依存させないため |
| 2026-11-05 | CONTRACT-04 | reactive な theme を thread をまたいで渡すか | 固定の theme と Send な起動用 data だけを UI thread に渡す。embed の reactive theme は root factory の中で provide_theme し、App 側の呼出し元で作った reactive theme は明示的に Unsupported とする | Rc や owner に束縛された Value は Send でなく、unsafe や snapshot で偽装しないため |
| 2026-11-05 | CONTRACT-05 | 非同期 action の結果をどう表すか | ActionOutcome は Clone の Succeeded(T)/Failed(UiError) とし、完了だけを通知する。キャンセル時の通知は増やさない | 完了の結果と取消を混ぜず、既存の完了の契約を保つため |
| 2026-11-05 | CONTRACT-06 | Pagination の総数をどう渡すか | pagination の総数は pages と items のどちらか一方で受け、内部の PageCount で排他にする。Stepper の collapsed は geometry から導く値とする | 二つの総数の不整合と、導ける値の別保存を避けるため |
| 2026-11-05 | CONTRACT-07 | 失敗の分類をどこまで増やすか | FailureKind は Content/Resource/Backend/Surface/Unavailable とし、任意の新分類を足さない。render/text adapter の error は Failure へ分類し、core の利用者向け診断は source location を持つ別の意味として保つ | 分類を増やしても境界は増えず、診断の出所を失わないため |
| 2026-11-09 | CONTRACT-08 | props の builder をどう作るか | `#[component]` が関数の署名から struct、constructor、任意 props の setter と `IntoView` を生成する。必須の props（handler を含む）は位置引数、任意の props は名前付きの setter。生成した型は `Element` を実装しない | 手書きの builder（約 80 行）は Rust に慣れない利用者の障壁であり、React の「component は view を返す関数」と同じ形にするため。外から変えてよいものを props で表すため |
| 2026-11-09 | CONTRACT-09 | 名前の衝突をどう解くか | `NodeBuilder` を `Element` に、trait `Component` を削除して `impl Element` に、setup の境界の関数 `component()` を `scope()` に、試験の `Element` を `Locator` に改名する | 一つの名前に一つの意味を持たせ、component を `#[component]` の意味に一本化するため |
| 2026-11-09 | CONTRACT-10 | handler の型を一つの trait にするか | しない。payload ありは `Fn(E) -> R`、なしは `Into<Callback<()>>` を全部品に守らせ、xtask の api-shape で検査する。builder の handler は setter、`#[component]` の必須の handler は位置引数 | marker 型の trait では closure の引数の型が推論されず、`on_edit(\|phase\| ..)` に型注釈が要るため |
| 2026-11-10 | CONTRACT-11 | GPU の起動の希望をどこから公開するか | feature なしの `kivact::launch` に GpuPreference など 5 型だけを置き、`App::gpu_preference` と `RenderOptions::gpu_preference` も feature なしにする。gpu の無い build では無視する。`gpu_interop` からの再公開はしない | GPU 共有を使わないアプリも起動の時期を選ぶため。core の launch module を丸ごと出すと内部の型が公開されるため |
| 2026-11-10 | CONTRACT-12 | 開閉の命令をどう名付けるか | `show`/`hide` は冪等、反転は `toggle`、閉じる理由を渡すものは `hide(reason)`。状態は `is_open` のまま。menu と command palette の content を開いている間だけ mount する method は `while_open(factory)` | 命令を binding の setter `.open(..)` と別の名前にし、`content()`（content の Part）と衝突させないため。React の `{open && <X/>}`、Solid の `<Show when>` に当たる |
| 2026-11-10 | CONTRACT-13 | 公開の recipe state の形と名前 | 排他な事実は pub の enum の field、常にある事実は pub の field、一部の種類にだけある事実と導出する事実は accessor（bool は is_/has_）。外部の theme は `state.input.pressed` を `state.input.is_pressed()` のように読む | 無効な組合せを型で除き、読み方を部品の間で揃えるため |
| 2026-11-10 | CONTRACT-14 | `provide_theme` を prelude に置くか | 置かない。`kivact::style::provide_theme` を明示して import する | `provide_locale`・`provide_messages` と揃える。App の root で一度書くだけで、明示の import の方が出所が分かる |
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
| 2026-11-09 | STATE-AUDIT | 暗黙の状態機械をどう見つけ、閉じたと言えるか | lint（state-fields）は補助、人の監査が主。閉じ方は型（enum、Machine）、導出して保持しない、唯一の書き手の投影、真に独立の四つ。`--report-audit` の一覧を人が読む | 名前と型の構文を見る lint は関数・crate をまたぐ意味の制約を網羅できず、静的な検査だけでは隠れた状態が無いことを証明できないため |
| 2026-11-10 | STATE-CLAIMS | 印の主張を機械で検査するか | `projection:` は見える範囲で唯一の書き手を検査し、検証できない書込みは error。`independent:` の理由文はファイルをまたいで共有しない。閉じていない組合せは通常の検査で error | 偽の投影と定型文の理由は機械で検出でき、gate と監査の条件を一致させるため。意味の確認は人に残る |

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
