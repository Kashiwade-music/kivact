# 設計判断の記録

この文書は 2026 年 11 月 4〜5 日の基盤再設計で採用した判断を保存する ADR である。現行の規則は[アーキテクチャ](architecture.md)、製品の範囲は[製品戦略](strategy.md)、現在の署名は rustdoc とソースを参照する。

各記録は日付、問い、決定、理由を持つ。過去の unit 名と checkpoint は識別用の履歴であり、現在の作業指示ではない。同じ事項は後の決定が先の決定を置き換える。特に途中の trace 同一性を求めた記録は ADR-19.0 で置換済みであり、過去の未完了 checkpoint を現在の未完了事項として読まない。実装時の改訂（19 系列）は、それ以前の案と台帳の例より優先する。

## 作者の決定

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-04 | Q-01 | Machine の純粋遷移だけ単体試験を例外にするか | 例外にせず App 全体の E2E で確かめる | 利用者の契約を本番経路で検証する方針を揃えるため |
| 2026-11-04 | Q-02 | custom の Experimental 表記を外すか | 外す | 独自 control/editor を公開 API で作れる正式な境界にするため |
| 2026-11-04 | Q-05 | interaction handle の接尾辞は何か | Api とする | 一つの語彙に統一するため |
| 2026-11-04 | Q-07 | ICU4X の全 locale data を含めるか | 数 MB の増加を受け入れ compiled data を含める | locale ごとの独自 table を作らず標準書式を提供するため |
| 2026-11-04 | Q-08 | macOS/mobile/Web backend も実装するか | 今回は実装しない | 実機と環境がなく検証できないため |
| 2026-11-04 | Q-06 | mirror=off の E2E を App 全体の E2E と認めるか | 認める。戦略の文言は変更しない | 元から入力は注入であり、mirror の有無によらず本番 presenter/driver を通るため |

### APG と platform の慣習

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-10-09（2026-11-04再確認） | APG-01 | 右クリック menu をいつ開くか | Capabilities.context_menu_on に従い Windows/Linux は release、macOS は press。keyboard 起動は trigger の位置 | platform の慣習を優先するため。macOS backend が実装済みという意味ではない |
| 2026-10-09（2026-11-04再確認） | APG-02 | 閉じた Select の矢印をどう扱うか | 値を変更し、Alt+Down で開く | 作者が採用した desktop 操作を維持するため |
| 2026-10-09（2026-11-04再確認） | APG-03 | Tooltip の時間と並行表示は何か | 約500ms、skip-delay あり、一度に一つ | 遅延を共有する既存の操作 model を保つため |
| 2026-11-04 | APG-04 | behavior の外部基準を何にするか | 公式 WAI-ARIA APG の Keyboard Interaction と Roles, States, Properties。ただし上記作者決定を優先。時間など未規定の細部は InteractionPolicy | 独自の慣習を無自覚に増やさず、支援技術を含む一貫した操作にするため |

対象 pattern は Accordion、Alert/Alert Dialog、Breadcrumb、Button、Carousel、Checkbox、Combobox、Modal Dialog、Disclosure、Grid/Table、Listbox、Menu/Menubar/Menu Button、Radio Group、Slider/Multi-Thumb Slider、Spinbutton、Switch、Tabs、Toolbar、Tooltip、Tree View、Window Splitter。公式一次資料は [APG patterns](https://www.w3.org/WAI/ARIA/apg/patterns/) を参照する。対象ごとの意図した変更は実際の behavior unit と E2E に含める。

### 非目標の確認

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-04 | SCOPE-01 | OS の SPI があれば各 backend も作るか | Windows/Linux と virtual の境界を整備し、macOS/Android/iOS/Web は追加しない | 実機・検証環境・保守者のない対応を表明しないため |
| 2026-11-04 | SCOPE-02 | Path boolean 演算は Canvas/Path2D 対応の不足か | 不足ではなく追加しない。addPath 相当は既存 Path.append | Canvas/Path2D 自体に boolean 演算はなく、必要とする要求もないため |
| 2026-11-04 | SCOPE-03 | touch wheel のため rotateX を足すか | 追加しない。既存 2D affine による円筒近似を使う | 差は台形の遠近表現であり、操作・支援技術・behavior の欠落ではないため |

## 実装で確定した決定

全 47 節（19.0〜19.46）の判断を以下に収録する。単なる改名手順と恒久的な契約を区別する。

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | ADR-19.0 | 移設の同一性と補足 API をどう定めるか | 厳密な trace 比較を廃止し既存 E2E 全通を保存の基準にする。trace は失敗診断だけに使う。calendar は Into<CalendarSelection>、theme は IntoValue<Theme>、disclosure.animated を維持、軸は Orientation、embed は CloseSignal | f32 から f64 への統一で 58,176 件の trace 差が残り、厳密一致を人工的に作ることはできない。API は実際の情報と既存 behavior を保つ必要がある |
| 2026-11-05 | ADR-19.1 | Backend を新しい巨大な抽象にするか | 既存 ShellEventLoop を Backend と改名し native が実装。virtual は SteppedBackend。embed は free function。各 consuming backend が session shutdown を所有し、Send MakeHandler と UI thread の ChildHooks を分ける | 既存の実行 model と借用 handler を保ち、非 Send backend 全体を Send factory に捕捉しないため |
| 2026-11-05 | ADR-19.2 | virtual の入力と観測をどう分けるか | Script が書込み、Probe が owned read を担う。Step は Input/Wake/Redraw、StepOutcome は idle 情報/exited/failed。frame 後の公開、GPU 非同期初期化、presenter は本番と同じ。mirror は native の UI thread に adapter を接続する | 二つ目の試験専用実行 model と raw 観測 facade を増やさず、実際の App 契約を検証するため |
| 2026-11-05 | ADR-19.3 | 入力正規化の持ち主は誰か | native port が Winit/VK/Keysym を写し、shell/pipeline が key、IME、DnD を正規化する。repeat が未指定なら導出。IME snapshot は UTF-8 byte、native が UTF-16 を変換し engine が excerpt origin を適用。virtual の raw 入力も同じ道 | OS の mapping と共通の入力規則を分離し、native と試験の挙動を一致させるため |
| 2026-11-05 | ADR-19.4 | OS service を window 前に取得するか | 完成した ShellWindow に UI thread 同期の perform を一つ置く。RegisterFont/ReducedMotion は先に engine HostRequest へ分離し、完了 effect 配送前に実 Environment を同期。virtual は応答を上書き可能 | parent dialog と複数 embed resize には window identity が必要であり、late binding を避けるため |
| 2026-11-05 | ADR-19.5 | embed の thread をまたぐ寿命を一つの enum にするか | 共有 ChildStatus と UI thread の ChildLifecycle/HandlerSlot を分ける。再入/drain guard と connection lifetime は別の事実。shutdown/module 保持は実 handler の破棄に従い、handle の早期 close では行わない | 異なる thread の所有権と再入を偽の単一状態にせず、panic/leak 時の安全を保つため |
| 2026-11-05 | ADR-19.6 | surface の失敗をどう数えるか | FrameHealth で skip と loss を分ける。GPU の連続 loss 二回で CPU fallback、任意 failure 三回で失敗。shown と受理された skip で数を reset | 正常な skip を故障と数えず、既存の復旧条件を保つため |
| 2026-11-05 | ADR-19.7 | gpu feature 無効時の API を変えるか | GpuContext/GpuSurface は空 enum、GpuEnv は空値、GPU_BUILT=false として名前を保つ。NoAdapter/External と FailureKind::Unavailable で説明 | 中間層の cfg と利用者側の型分岐を不要にするため |
| 2026-11-05 | ADR-19.8 | embed close を exit turn に変換するか | CloseSignal を入口、client 後、effect 後と core の各 event で確認し追加の exit turn を作らない。HostPhase と presence/surface は別の事実で drawable は投影。SurfaceLost は診断。Launch は Pending/Starting/Running と実 waker を持つ | host 破棄後に App を動かさず、停止と描画 surface の喪失を混同しないため |
| 2026-11-05 | ADR-19.9 | lower を独立 crate に保つか | engine の lower module に統合し、Context.finish が消費可能な LowerReport を返す。latch getter を並べない | 計測・model・paint が内部情報を共有しており、独立した小さい契約が成立しないため |
| 2026-11-05 | ADR-19.10 | layout の pass を型状態に分割するか | 一つの Layout が columns/state を所有し、compute 内で changed を取り、settle は一つの関数。reason は bit flags。pass artifact を保存しない | 所有権を明確にしながら、一回の計算を表すだけの型と中間状態を増やさないため |
| 2026-11-05 | ADR-19.11 | display と hidden をどう表すか | Display は Flex/Grid/Contents。hidden は独立。公開 style に display を置き prelude の別義名と衝突させない | layout mode と表示可否は異なる事実だから |
| 2026-11-05 | ADR-19.12 | 出現・退場の layout をどう動かすか | layout hook が parent item の大きさを scale し、子の自然サイズと cache を保つ。EnteringPending/Animating は同じ phase index。ExitLayout::Pop は exit_size を使わない | 第二の layout model とキャッシュ破壊を避けるため |
| 2026-11-05 | ADR-19.13 | pointer の経路と modal の受入をどこに保存するか | capture と press は独立で四組合せを許す。touch Pending は payload。input 時の Track と dispatch 時の modal admission を分け、identity のない KeyUp は現在 focus に配送。primary hover chain と secondary over を保つ | 一つの phase にまとめると有効な経路情報を失うため |
| 2026-11-05 | ADR-19.14 | 状態台帳を形式的に満たすべきか | 異なる事実は別に持ち、AcceptedTree は a11y 初期化/action 検証のため残す。focus や diff の複製には使わない | enum 化の件数より実際の情報と一つの正本を優先するため |
| 2026-11-05 | ADR-19.15 | runtime module の向きはどうするか | reactive を基底に task/motion、machine→reactive/task、host→reactive/task/motion。passive host_types と Runtime を借りる TurnGuard を実装から分離 | host 実装への逆依存をなくしながら正準 root path を保つため |
| 2026-11-05 | ADR-19.16 | runtime と host を後付け接続するか | constructor が clock、registry、root owner と task/motion extension を完成させる。内部 OnceCell はこの原子的構築に限る。UI wake と thread-safe host wake を分離し、Driver の late attach は作らない | 半完成 object と遅延初期化の公開 phase をなくすため |
| 2026-11-05 | ADR-19.17 | turn の順序はどこが持つか | core の AppClient が一つの turn 関数で clock、reports、入力、async、commit、mount、user effect、output を順序付ける。runtime は Stage/guard/部品を提供。async 1000、commit 100 round の上限を保つ。再入/setup、Commit 外 stamp、構築中 extension 参照を misuse とする | focus/error handler の正当な書込みを妨げず、既存の配送順序を保つため |
| 2026-11-05 | ADR-19.18 | frame loop を future に置き換えるか | FrameTick の callback は同期。clock は一つ、MotionEnv は readonly input。FrameClock と AnimPhase を一つずつ持ち、InternalWatcher が自然完了を安定化中に通知。Sleep の cancel queue は期限確認前に処理 | future の poll に移すと同じ batch 内の入力と frame の順序が変わるため |
| 2026-11-05 | ADR-19.19 | reactive frame と Watcher をどう統一するか | Handler/Setup/Computation の kind と pure/tracking/owner/writer を分離し、callback/untrack/pure でも kind を保持。Watcher の初回は依存と queue だけを作り、以後 untracked callback。PartialEq 比較を加えない | 初回通知と比較を変えると既存 reactive semantics が変わるため |
| 2026-11-05 | ADR-19.20 | 非同期の最新値と phase を統合するか | AsyncCell の phase と latest は異なる事実。pending/failed 中も成功値を保持し T に PartialEq を要求しない。待機 queue があれば失敗 callback 後も Pending。ActionPhase は close 後も queue を drain。PoolShutdown は root cleanup 後、TaskCell の OnceCell は初 poll 前に実 key を設定 | 成功値保持、待機処理、取消と shutdown の既存契約を失わないため |
| 2026-11-05 | ADR-19.21 | App 全体の状態をどこへ置くか | constructor で固定する Extensions と root owner に属する遅延 App state を分ける。unsafe な増設 map AppStates を作らない | 寿命と構築時点の異なる情報を一つの可変 registry にしないため |
| 2026-11-05 | ADR-19.22 | 内部 handler を層ごとに定義するか | runtime に一つの Handler<E> を置き、frame/task/core で共有。unit payload も同じ model | 同義の callback slot を繰り返さないため |
| 2026-11-05 | ADR-19.23 | Machine の実装をどう小さくするか | Self を input snapshot、State/Event/Effect を関連型にし、純粋 transition が Step を返す。send は再入 FIFO、timer は event variant 単位、feed の初回は無通知、破棄後は Disposed。MachineCx は send の口だけ | 状態更新・timer・effect を一つの owner に保ち、小さい reducer として説明するため |
| 2026-11-05 | ADR-19.24 | 起動 API と option 束をどう統一するか | private AppOptions に launch 設定を集め App が委譲、RenderOptions はそれを包む。font の登録元 location は private、blocking は None、theme は IntoValue。戻り値が異なる render_with_handle は残す。内部 engine feature は testing のまま | 同義設定と入口を減らし、実際に異なる返却契約は消さないため |
| 2026-11-05 | ADR-19.25 | Inspect が何を読むか | core 所有の errors/pending/outstanding_blocking/motion_ticks、engine Reader の stats を公開投影にする。既存 backend E2E の private harness は残すが raw 公開観測 API は増やさない | 第二の tree/model を持たず、残る内部契約を試験するため |
| 2026-11-05 | ADR-19.26 | style の内部口と themed 色をどう扱うか | compile/install/record は理由付き hidden free function bridge。Style.merge は recipes 用に公開。display も公開。THEMED の process intern cache は残す | Color を Copy のまま theme 非依存にする実際の役割があり、cache を lifecycle と誤認しないため |
| 2026-11-05 | ADR-19.27 | 地域書式を中間 table に変換するか | ICU4X compiled data を全 locale 分含め、Locale が一度だけ formatter を作る。CalendarDate の from/to days を共通計算にする | kit 固有の en-US table、二重変換、手書き calendar 計算を避けるため |
| 2026-11-05 | ADR-19.28 | 削除結果をいつ handler に渡すか | Removed を borrow 終了後に消費する。論理削除した node の handler は停止し、退場 paint は保つ。node でない ID は HashMap、on_exit は Handler の Option | borrow と callback の再入を分離し、退場中の二重入力を避けるため |
| 2026-11-05 | ADR-19.29 | fragment に専用 NodeKind を作るか | Display::Contents の実 node とし、すべての走査で共通 flat_children 判定を使う。Reader の論理親子は leaving を除外、最終退場まで透明 ancestor を保持、detached subtree も実 node | layout の透過性と論理 ownership を同時に保つため |
| 2026-11-05 | ADR-19.30 | Part が独自 hover を保存するか | engine の FocusTarget(node,region) をそのまま読み、正確に一致する region だけ hovered | node だけの比較や第二の hover state は region の意味を失うため |
| 2026-11-05 | ADR-19.31 | handler の返り値をどう揃えるか | HandlerResult 一つに ()、Handled、Result<(),UiError>、Result<Handled,UiError> の四実装。Handled の PreventDefault を合成でも保つ | 処理済みと既定動作の禁止を区別し、error も同じ入口で扱うため |
| 2026-11-05 | ADR-19.32 | Callback の構築は setup に限るか | payload なし Callback は現在の owner があれば handler/effect 下でも構築可能。owner がなければ misuse | 正当な owner-bound callback の既存利用を不必要に制限しないため |
| 2026-11-05 | ADR-19.33 | 外部 NodeBuilder 実装を閉じるか | node_mut は公開だが doc(hidden)、trait は seal しない。with_node は普通の公開 API | 外部 builder が標準 setter を継承できる実装用の口が必要だから |
| 2026-11-05 | ADR-19.34 | 公開 API 改名をどう進めるか | 層をまたぐ署名を family 単位で一度に移行し、長さ・状態の監査は rename と behavior の変更を区別して行う | 同じ利用箇所を六回書き直すことと、改名に紛れた振る舞い変更を避けるための実装時の判断 |
| 2026-11-05 | ADR-19.35 | InteractionPolicy の所属と形はどうするか | interaction に inputs/menu/tooltip/button/slider/switch/toast/select/hover_card/drawer の nested policy を置く。ui が依存し、逆向きにしない。tooltip に第二の context config を残さない | behavior の設定の正本を一つにするため |
| 2026-11-05 | ADR-19.36 | UiTokens にすべての recipe 値を移すか | 部品ごとの nested tokens に、旧 kit override と theme 依存値だけを置く。形は recipes。data table の sticky/status paint 値も Tokens | 8,800 行の recipe の無差別な token 化では設定面が増え、形と値の境界も失われるため |
| 2026-11-05 | ADR-19.37 | Disclosure の既定 motion は何か | motion は animate=false、duration=200ms。部品ごとの animated reactive override は残す | 標準の既定と旧 kit の明示指定を両立するため |
| 2026-11-05 | ADR-19.38 | overlay_margin の None を何に使うか | Theme.overlay_margin は常に数値で既定 0。必ず window 内に fit し、no-fit sentinel を作らない | 余白と fitting の有無を一つの Option に混ぜないため |
| 2026-11-05 | ADR-19.39 | すべての touch gesture を Drag に統一するか | DragApi に slop（既定0）、axis lock、100ms velocity sampling、公開 DragVelocity を追加。実 drag 部品は共用。touch scalar/switch/toast/select/month の Machine は既存の閾値と activation timing のまま | Drag のために三つの新概念や異なる gesture phase を強制しないため |
| 2026-11-05 | ADR-19.40 | APG への対応を後段に送るか | behavior の移設 commit を分けたうえで同じ unit 内で公式 APG の keyboard/roles/states に合わせ、意図した作者 override と差分を記録し E2E で確認 | 後段だけに送ると状態機械と behavior を二度作り直すため |
| 2026-11-05 | ADR-19.41 | toast/tree 行が独自の表示 phase を持つか | engine presence で出現・退場を描く。visibility 配列や animated height Machine を複製しない | 一つの表示 lifecycle に揃えるため |
| 2026-11-05 | ADR-19.42 | editor の syntax と装飾 API はどうするか | line_styles は content T と cross-line state S を純粋 callback へ渡す。StyledRange は byte range と色/装飾、decorations は全文対象。CodeBlock の highlight は既存 parser と palette を保つ | 行ごとの解析状態と編集内容を二重保存せず、既存 tokenization を失わないため |
| 2026-11-05 | ADR-19.43 | text 色の Transition と未確認の option を増やすか | 既存 Paint が text color を担うため新 Color TransitionProp は不要。Combobox/Select の C35 option は実際の旧 kit の和集合だけを実装 | 同義の表現と要求のない knob を増やさないため |
| 2026-11-05 | ADR-19.44 | touch の日付入力は kit 固有か | DatePresentation Adaptive/Desktop/BottomSheet を構造 option とし、Adaptive は capabilities.touch。calendar/date/time/wheel は標準 UI と scalar に統合し全旧 option を維持 | 同じ日付入力の behavior と OS 判断を kit に重複させないため |
| 2026-11-05 | ADR-19.45 | Creative picker を全て標準 combobox に置換するか | 共通 body は標準 API を使うが、Creative 固有の column-follow picker は kit に残す | column に追従する domain の意味が一般の選択 field と異なるため |
| 2026-11-05 | ADR-19.46 | 一度だけ mount する setup closure をどう渡すか | view::component(FnOnce→IntoView) と prelude を公開。mount 時に child owner/transparent fragment の下で一度実行し、context を登録しない。Variant Default は通常値、reactive 値は明示 setter | eager 評価、provide(())、recipe 評価中の hook で owner と cleanup の寿命を変えないため |

### 実装決定の契約上の補足

次の細部も上表と同じ 2026-11-05 の決定に含む。問いと理由は対応する ADR を参照する。

- ADR-19.1: Backend は run だけで、事前 capabilities/open_child/shutdown は持たない。ShellOptions は native constructor の引数で寸法検証もそこで行う。embed 内部の同名 Backend は ChildPort とした。native への通常依存は外し、既存 native/embed E2E のための arvane の dev-dependency は許す。embed Session は handler を drop した直後、DetachedThreads があれば module を保持する。
- ADR-19.2: SteppedBackend.start は window を作り handler.start を呼んでから返す。Idle は next_deadline/redraw_requested/woken を持つ。試験 host は handler を loop と共有し、engine の既存 inject は試験専用で残す。node query は同じ handler から読む。a11y の公開を frame 後へ揃え、turn 後に redraw 要求があれば必ず描き、見たときだけ描く経路と KIVACT_TEST_LIVE は廃止する。
- ADR-19.2: GPU の renderer 通知を待つ wake は最大5秒。frame の readback は keeps_frames の window だけで行い keep_frame へ渡す。test_font/gpu_unavailable は presenter/text を知る arvane root に所属し、arvane に test feature は置かない。既存 arvane E2E は handler 内で実行する Harness.with、Script、Probe で契約を保つ。virtual に winit を持ち込まない。
- ADR-19.3: raw は Key/Ime/Drop。PlatformKey は Vk（physical code付き）/Keysym/Code で winit 型を含めない。IME は Preedit/Commit/End/Reconvert、DropSession は Idle/Hovering/Dropped。Win32/winit は既存 repeat を明示し、X11 の未指定だけ押下集合から導出する。Screen.raw_input から Script.raw を通す公開試験口を持つ。
- ADR-19.4: CPU の native 提示形式は Xrgb8888。clipboard/dialog 等の個別 ShellWindow service method を並べず perform に統合する。現行対象はすべて同期に回答するため、返信から Host/window に戻る別経路は作らない。CommandKey の既定は Ctrl、ContextMenuTrigger は Release。virtual は memory clipboard、dialog queue、記録を保ち、respond の request→optional result callback で上書きする。別の PlatformRequestKind は不要。
- ADR-19.5: ChildStatus は Running/Stopped(cause,window)、ChildLifecycle は Opening/Live/Disposed/Leaked、HandlerSlot は Unstarted/Running(handler,control)。Win32 の drain 中にも event callback が走るため ReentryGuard と DrainGuard は同時に有効となり得る。X11 の atomic guard は ConnectionAlive と名付ける。
- ADR-19.6: skip が GPU 連続 loss を reset する点は旧実装との差として受け入れた。Healthy/Skipping(frames)/Losing(losses) は相互排他で、Recover/FallbackToCpu/Fail を返す。
- ADR-19.7: GPU import 未対応の公開分類は既存 Unavailable と具体的理由を使い、新しい FailureKind::Unsupported は足さない。core 単独 build の GPU 試験は gpu_unavailable で skip できる。anchor-deps の専用検査は作らない。
- ADR-19.8: HostPhase は Starting/Running/Exiting/Closed、AppPresence は Foreground/Suspended、Surface は Drawable(generation)/Unavailable(NoSurface/ZeroSize)、LastPresent は Settled/Skipped(retry_at_ns)。どの phase でも presence/surface は独立した事実。SurfaceLost は再作成後の一回の HostFailure であり Unavailable::Lost を作らない。
- ADR-19.8: Launch の Pending/Running payload は Box 所有。Starting は引数を移して実 Host/Model を構築中の状態で event は invariant 違反。waker が得られる start で本物の asset store を作り、inline dummy store の差替えを廃止する。前回 Environment は Sent の借用で読み theme/motion override を保つ。
- ADR-19.9: lower は resolved style/DrawList/text run→scene の変換で tree/input を知らない。BoxShape、asset adapter/event、shape は lower 内に留め types へ広げない。一回の Context 内の Cell/RefCell は残し、finish が dropped 件数と failures を一度だけ返す。
- ADR-19.10: 変化がなくても overlay の再配置と geometry drain は可能でなければならない。LayoutReasons は PASS/STYLE/VIEWPORT、settle は sync→layout→lint→sync の唯一の順序を持つ。hit test/file drop/wheel/redraw の前に必要な settle を呼ぶ。旧 columns/state は private、実施結果の Kept/Updated も private とする。
- ADR-19.11: authored display は optional、computed display は確定値。Contents は共通の flatten 処理で消費し Taffy へ渡さない。hidden は display mode と別で保持する。
- ADR-19.12: TransitionProp::Size を使い Orientation 方向に presence scale を適用する。公開 Presence は Entering/Present/Leaving、内部 Pending/Animating は両方 Entering。engine の通知と Reader から NodeRef.presence が初回 query で signal を作る。parent が直接読む item size/min/max/basis にも scale を適用し、child の natural style/cache は保つ。
- ADR-19.13: MouseGesture は Route/Scrollbar/SelectText、TouchGesture は Route/Pending/Pan。press/capture は listener の有無や副buttonでも異なるため struct Route のまま。KeyDownTargets は passive focus 宛先表で routing 実装へ逆依存しない。
- ADR-19.14: soft_keyboard の owner は専用の text-input/IME 処理であり一般 HostPhase へ複製しない。OS service の状態統一は window perform の判断で済ませる。
- ADR-19.15: handler/turn/time を reactive より上の独立 module にすると既存 binding/callback が HandlerResult を使うことで循環する。Runtime は一つで、host module の inherent impl が上位の構築/settle/finish を提供する。motion は task に依存しない。
- ADR-19.16: Runtime の内部 extension OnceCell は constructor が返る前に埋まり外から空を観測できない。build の panic は registry から外す。build 中に signal/trigger/selector は作れるが ext は読めない。Driver、late waker 設定、optional root、turn_requested の保存は置かない。blocking threads の既定は min(4, CPU数)。
- ADR-19.17: TurnReport は timer/drain の最短 deadline と ExitState。exit は Running/Draining(deadline)/Exited、request_exit の再呼出しで deadline を延ばさない。eventなしでも mount boundary がなければ report_now で終わる。before revision と done revision は所定の位置で一回ずつ読む。
- ADR-19.17: mount 待ちは unstamped と revision付き awaiting の型付き queue。deliver_reports は開始 snapshot だけを配送し後の報告は次回へ残す。frame request は時刻と writer revision の write であり bool にしない。finish が user effect/ready task/later report/client pending から一度だけ wake を決める。commit 上限時は user effect を abandon して Cycle を報告する。
- ADR-19.18: MotionEnv は reduced_motion/surface_active/frame_period_ns の readonly Value を constructor の Extensions に置く。attach_env、optional env、reduced-motion setter の mirror は作らない。motion 有効化時の既存 reduce_motion は保つ。Sleep は Unregistered(deadline)/Registered(id)/Fired、Drop は cancel queue に積む。
- ADR-19.19: effect_on の Watcher は User queue、Animated は Internal queue で最初に評価する。生成時へ評価を早めると setup 値になり元の最初の user round の値を失うため行わない。WriteInPureContext の既存 misuse は変更しない。
- ADR-19.20: AsyncCell を書くのは start/finish/reset のみで、state/latest getter を memo にしない。latest の読み手を無関係な state change で起こさない。Drain は Queue/All runner だけから作り Weak registry で保持。shutdown 後 blocking は新 UiError を返さず invariant panic とする。root破棄と最後のpoll後なので正当なApp codeからは届かない。
- ADR-19.21: use_app_state の初回 init は root setup で実行して root context に provide する。Extensions は固定 storage の借用を返すため、unsafe な後付け増設 map とは統合しない。
- ADR-19.22: task 完了 handler は Handler<Result<T,UiError>> で、各 handler へ結果を clone して渡す。既存 on_done が持つ T:Clone の契約内で行う。
- ADR-19.23: dispatch 自体も Idle/Running(queue) 一つの enum。after は同 variant の前 task を cancel し、その場で Sleep を作ってから spawn する。effect は machine owner の handler frame で順に実行する。
- ADR-19.24: AppOptions は内部 launch module の設定で facade から再公開しない。RenderOptions だけの host_locale/dark/scale/mirror/gpu/frame period/from_env などはその builder に置く。font の登録元を保持し、拒否時の報告を失わない。
- ADR-19.25: LaunchHandle.inspect は起動済みのときだけ存在。core だけが答える root/app_view/位置/owner/why/runtime stats/turn record と engine Reader を分ける。semantics snapshot は Reader の host tree を辿る。旧 core tree の一致検査は木の統合までの一時検証で恒久 API ではない。
- ADR-19.26: theme install/query、style/span compile、record、scroll-axis adapter は hidden internal free function とし旧 inherent method を残さない。THEMED は縮まない ThemeColor interning cache であり theme ごとの可変状態ではない。
- ADR-19.28: Removed の ids は論理削除した subtree の前順。handler records は tree borrow が終わってから drop。退場 paint が残っていても core handler、asset failure と NodeRef の detach は論理削除で処理する。
- ADR-19.29: flow の show/dynamic/for_each/for_index/virtual-list行/error-boundary/provide は Contents の実箱を作る。箱自体にsemantics/hit/focusを付けず、未挿入の部分木も親なしの実箱として構築可能。a11y の論理 hierarchy を保つ後の修正は追加判断を参照する。
- ADR-19.31〜33: HandlerResult の変換 method と NodeBuilder.node_mut は hidden な実装口。外部 builder の実装可能性は維持する。payloadなしcallbackをownerの種類で不当に制限しない。
- ADR-19.35: policy は Clone/Debug/PartialEq/Default と公開 field を持つ FRU 値。tooltip delay/skip は500/300ms、hover card open/close は600/300msで部品別上書きも残す。toast slop8px、select slop10px、menu drift context10/other13px。drawer close fraction0.3、flick speed0.6、distance4。現在の追加 field と正確な署名はrustdocを参照する。
- ADR-19.36: 初期 tokens は button busy、progress motion、toast row motion、toggle label reservation、dialog center、disclosure motion、data table sticky/row-status。各部品が所有する nested struct を追加して拡張する。値だけが違う既存 recipe を後から全て作り替える作業はしない。
- ADR-19.38: overlay margin は inset の数値で既定0。負数/NaN、未設定modeを no-fit sentinel として使わない。
- ADR-19.39: DragUpdate.velocity は px/ms、track_velocity 未指定は ZERO。axis_lock は slop を越えた最初の軸に固定し交差方向deltaを0にする。raw touch gesture は primary touch/pen filter、slop前の見た目、activation抑止時点を自分のMachineで持つ。switch10px/month6pxは名前付きprivate定数、既存thresholdを変えない。
- ADR-19.40: 対応の履歴は I-1=Disclosure/Dialog/AlertDialog/Tooltip、I-2a=Combobox/Select Listbox、I-2b=Menu、I-3=Button/Checkbox/Switch、I-4a=List/Tree/Tabs/Radio、I-4b=Grid/Table、I-5a=Slider、I-5b=Splitter、U-2=Spinbutton、K-1b=Carousel、K-2a=Accordion、K-3a=Toolbar、K-3b=Breadcrumb、K-4=Tree semantics、K-6b=Alert。独立した後段I-8は廃止。試験は apg_<pattern> と操作を表す名前で契約を確かめる。
- ADR-19.41: toast は行の exit_size、tree は子行の enter_size/exit_size。二重slots/live-slots、generation、高さAnimated、previous_open/last_heightの補修を除き、RowReveal/ToastRowPhaseという代替Machineも作らない。
- ADR-19.42: content T は Clone+PartialEq、cross-line S は PartialEq+Default。line_styles は (&T,line,&S)→(ranges,next S)、全文decorations は &T→decorations。StyledRange はoptional foreground/background/decoration、DecorationKind はUnderline(style)/Outline/Background。古い行番号だけの案ではcontentとblock-comment等の状態を失うため採用しなかった。
- ADR-19.43: C35 の option 名は実際の kit 利用から決め、未確認の仕様一覧を先に作らない。実装の分担ではcalendarの和集合は日付側に含める。Paint に既存 text color が含まれることはE2Eで確認する。

## 追加で確定した境界

| 日付 | ID | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | BOUNDARY-01 | Theme の値と source を二重に持つか | 固定 root theme は必要な Signal へ一度正規化し、動的 theme は元の Value を保つ | query のために別の source を同期しないため |
| 2026-11-05 | BOUNDARY-02 | Drag/Select に不足した口を alias で埋めるか | Drag.source_with は press 時 factory、active_payload は readonly、Generation.as_u64 は明示 query。Select.trigger_with_sheet/open_sheet/is_sheet_open は既存 owner の実 port | 新しい payload copy や open mirror を作らず実際の要求を表すため |
| 2026-11-05 | BOUNDARY-03 | editor drag と domain selection を混ぜるか | editor は zero slop。pointer/origin/payload は Drag、選択意図は domain、numeric payload は一度だけ保持 | 入力経路と選択の意味が異なるため |
| 2026-11-05 | BOUNDARY-04 | authored text decoration に backend 型を出すか | Color/TextLength を入口とし Rgba/TextLen は内部解決値。Draw と呼び Canvas alias は置かない | API の値と backend 表現を分け一つの名前にするため |
| 2026-11-05 | API8-01 | immutable bytes と shaping の共有を利用者へ露出するか | AssetBytes、ShapedTextSnapshot、GlyphRunSnapshot、SemanticsTextRuns で private Arc を隠す。owned Vec/iterator と参照 query を普通の入口にし、実 backend の shared allocation は理由付き hidden bridge で再利用 | allocation identity/caching/cheap clone を維持しながら ordinary API の Arc をなくすため。Vec 変換の allocation 不要は保証しない |
| 2026-11-05 | API8-03 | 診断とRust Errorの情報まで同義wrapperで隠すか | UiError/ErrorKindとdiagnosticsのsource Location、UiError.from_boxedの標準Error interopは定義上の理由付き例外にする。普通の部品には広げない | 診断の出所と標準Errorの相互運用は実際の境界であり、同義wrapperを増やすと意味と使いやすさを失うため |
| 2026-11-05 | API8-02 | Draw の measurer をアプリが install するか | set_text_measurer は doc(hidden) と precise internal-contract を付す自動 mount/offscreen bridge | 普通の Draw 利用に手動 install の順序を要求するのは不適切であり、実際の呼出者は内部二経路だけのため |
| 2026-11-05 | BOUNDARY-05 | Contents と a11y の論理構造をどう両立するか | layout/paint/input の透過走査を共有しつつ、意味のある論理 hierarchy を a11y で保ち、bounds は実 children の合併にする | 透明 layout を理由に既存の親子意味や正確な bounds を失わないため |
| 2026-11-05 | STATE-01 | Creative jobs の一つの running を Machine にするか | 独立したrun/stop入力を保ちprogress/amountを測定値とする。effectは実FrameLoopのstart/stopのみ | 他Signalの補修がなく、phaseのためのboolではないため |
| 2026-11-05 | STATE-X901 | catalog chat の text と typing をどう表すか | ChatContent::Message(text)/Typing。id/mine は passive identity/著者情報 | 空textとtypingの無効組合せを除き、既存時間と通知/選択ownerを保つため |
| 2026-11-05 | STATE-X902 | integrated meter の latch をどこが持つか | 単一測定ownerがUnmeasured/Measuredと実transportのSample/Resetを処理 | playing mirror、時刻、epochを追加せず測定の事実を保つため |
| 2026-11-05 | STATE-X903 | scroll spy とnavigationが別のactive sectionを持つか | 一つのMachineへObserved/Requestedを送り、geometryがないとき最後の事実を保つ | navigation直後の選択を保ち、scroll/geometry/selectionのmirrorを除くため |

## 基盤再設計の判断台帳

以下は当時の判断台帳の全126行を、問いと決定・理由を省略せず収録したものである。同じunitの複数判断も別行で保持する。旧名、古い件数、当時の検証結果は履歴であり、現行APIの別名や追加のgateを定義しない。欄内の決定と理由は原記録どおり一体として記す。

| 日付 | unit | 問い | 決定・理由 |
|---|---|---|---|
| 2026-11-05 | fix-G1-3 | aggregate が keys 実装の KeyDownTargets に依存し循環する | 受動な focus 宛先表を既存 focus/state へ所属させ、routing 実装への逆依存を除去。 |
| 2026-11-05 | fix-G3-3 | data が host/turn 実装を参照し runtime に循環する | Host/turn の契約値を host_types、Runtime を借りる TurnGuard を data へ所属。公開 root path と動作は維持。 |
| 2026-11-05 | C-1 | 指定 Inspect が既存 Screen の idle/error/blocking/tick を読めない | Inspect に errors/pending/outstanding_blocking/motion_ticks、Reader に stats を追加。AppOptions の title/host_options/window_title に RenderOptions を委譲し、設定の正本を一つに保つ。 |
| 2026-11-05 | C-1 | App API だけでは既存 backend 観測・rootless/worker 契約を保全できない | App 試験は public-api、純粋内部不変条件は debug 検査、残る既存 backend E2E だけを kivact-test 内 private に保持。新しい raw 観測 API や新 backend 試験は足さない。 |
| 2026-11-05 | CP2 | 初期 PresenceChanged が診断 turn を一つ進める | 必須の Entering/Present 通知18件を trace で確認し、why 期待 turn を3→4。依存 chain/位置/内容の全文 assertion は維持。 |
| 2026-11-05 | fix-G1-2 | 非同期 wake が最後の step 後に来ると snapshot から見えない | Probe が同じ live mailbox を読み worker 完了を観測。empty root の安定 identity は allocator が予約し、復帰 redraw は idle 前に確認。 |
| 2026-11-05 | CP2 | 次 wave と並行の WSL 検証も checkpoint の source に固定する必要がある | Linux Clippy も gate worktree の固定 commit を使う。検査中に integration の source が変わる競合を避ける。 |
| 2026-11-05 | fix-G1-1 | LaunchArgs が約2.5KBで Launch 全体を大きくする | private Pending/Running payload を Box 所有にし、Starting 遷移と単一所有を保ちながら enum サイズ差を解消。 |
| 2026-11-05 | E-9a | Taffy は親の item size/basis を hook の出力と別に読む | 親向け private item style に倍率を反映し子の natural style/cache は保持。倍率変更で依存 cache を無効化し、第二 layout 無しで親 auto 箱も縮める。 |
| 2026-11-05 | E-8 | workorder の Fragment variant 指定が設計19.29と矛盾する | 設計優先で Contents 箱だけを実装し、透明判定を一つにする。C-5と同じ形へ指示書を訂正。 |
| 2026-11-05 | E-9b | 旧 Arriving は最初の commit で Mounted になり動作中 Entering を示せない | private ArrivalPhase Pending/Animating を Arriving payload に持ち、arriving は索引一つだけ。動作完了で Mounted に遷移。 |
| 2026-11-05 | E-2 | Pending の所有値を Handler::start の mutable 借用から移す必要がある | Launch に実際の構築中状態 Starting を加える。dummy Client/Engine を作らず、再入は invariant 違反。Sent::get で前回の環境を読み挙動を維持。 |
| 2026-11-05 | E-3 | secondary pointer も over で Enter/Leave/Move を配送する | Hover は primary の chain だけ、各 Track.over が hit target を一所有し secondary の既存配送を保つ。 |
| 2026-11-05 | E-4 | paint/a11y/measure の共有読取りで damage を消費していた | 編集完了時に mutable editor から既存 EditLines cache へ damage を移譲。新 journal や二重保持なしに読み取り整形を維持。 |
| 2026-11-05 | E-5a | 既存の geometry/query/diagnostic が旧 layout 型を利用する | 必要な既存メソッドだけ Layout に移し、旧型は crate private。Settled は private の Kept/Updated。 |
| 2026-11-05 | X-1 | render の import は RenderError、FailureKind に Unsupported はない | 既存署名を維持し未対応は External、arvane の公開境界で Unavailable と具体的理由を返す。新しい error variant は足さない。 |
| 2026-11-05 | A-3 | 遅延 activation と snapshot が最新 focus を要する | AcceptedTree の focus は削除し Shared の最新 update 値だけを保持。PlatformUpdate::focus() で渡し木の差分計算を除く。 |
| 2026-11-05 | fix-G3-1 | core test と virtual backend が同じ MirrorMode を定義する | engine 経由で virtual の正準 enum を再公開し、環境読取りは core の private 関数へ。公開 path と設定挙動は維持。 |
| 2026-11-05 | A-1b | 新しい描画経路で診断 turn と queue state の既存期待が変わる | why の turn を 3、待機中 queue を Pending に更新。残る順序・latest・回復 assertion は維持し公開 API E2E 285 件成功。 |
| 2026-11-05 | fix-G6-1 | catalog が旧 WindowMode と専用 GPU 環境変数を固定する | RenderOptions::from_env に mirror/GPU を統一し、通常は Auto、off gate では Off。catalog の表示・操作 assertion は変えない。 |
| 2026-11-05 | A-1b | MIRROR off の public API binary でも native embed parent が実窓を開く | embed/smoke を含む binary・全 workspace は window-lock で実行。mirror off と GPU の理由付き skip は維持。 |
| 2026-11-05 | A-5 | GPU loss の二重 counter を FrameHealth に集約 | 設計 19.6 どおり skip は連続 loss を戻す。最初の device-loss 診断は private 同期 wrapper で保持。 |
| 2026-11-05 | A-1a | 次 Wake の test helper に借用 closure を保存できない | Harness::with の closure/result に static 制約を付け、move/clone で移行。製品 API を変えず unsafe bridge を避ける。 |
| 2026-11-05 | A-1a | policy は dev 依存と未作成 consumer も検査する | native E2E 用の arvane dev consumer は残し、virtual の kivact-test consumer は C-1 の crate 作成時に足す。通常の native 依存は削除。 |
| 2026-11-05 | A-4 | child frame と CpuSurface の提示方式がまだ異なる | frame Option の解消は既定の softbuffer 提示統合 A-6a で行う。lifecycle の所有は A-4 で一元化。 |
| 2026-11-05 | R-5 | Closed 後も worker と待機列が drain を続ける | private Closed に queue data を保持し、新版 Pending と busy の一致を優先。待機中の完了通知でも Pending とする。 |
| 2026-11-05 | W2 shared target | 別 worktree の改名済み Arvane metadata が混在して単体 check に外部型の不在が出る | 同じ単体 gate を反復せず、初回成功を保持して合流し専用 integration target で確認。R-1/A-0/E-1 統合 check は 0。 |
| 2026-11-05 | W2 resources | 旧 trace/log 削除後も空きは約 74 GB で開始目安 100 GB 未満 | 作業外のデータは削除せず、共有 target と 40 GB の clean 境界で進める。workspace 内の全 cache を消しても目安に届かず、再 build を増やさない最小の選択。 |
| 2026-11-04 | P-0 | Design documents exist only on the starting docs branch | Created integration from main and fast-forwarded the docs branch before unit worktrees. |
| 2026-11-04 | All | Work order requires author approval for design gaps | User explicitly delegates all judgments; orchestrator decides using API consistency, React/Solid accessibility, simple internals, and maintainability. |
| 2026-11-04 | P-0 | First baseline compilation exhausted disk space | Remove only this run's generated target/debug cache and disable incremental/debug symbols for checks; keep runtime debug assertions and optimize-level policy unchanged. |
| 2026-11-04 | P-0 | Baseline GPU read_back aborted with heap corruption after 5/6 tests | Preserve baseline failure; rerun the unchanged full suite with RUST_TEST_THREADS=1 to check whether concurrent GPU tests cause the failure. No source/test suppression. |
| 2026-11-04 | CP0 onward | Unchanged GPU read_back succeeds 6/6 with serial test execution after default parallel heap corruption | Use RUST_TEST_THREADS=1 for checkpoint E2E. Preserve the full test selection, keep window-lock, and retain the baseline concurrency failure as a known existing issue. |
| 2026-11-04 | P-0 | Baseline xtask waited after all architecture outputs were written | Volta node shim left an ELK grandchild holding stderr open. Verified and stopped only that worker; xtask returned 0. Set XTASK_NODE to the bundled node executable for later gates; do not modify vendor. |
| 2026-11-05 | 改訂 | checkpoint の E2E の thread 数（CP0 の行を置き換える） | 全 workspace は既定の並列で回し、`gpu-interop-producer` だけ `--test-threads=1`（作業手順書 0.6 節）。trace の比較は廃止（設計書 19.0 節）。 |
| 2026-11-04 | V-1a | Shared accessibility node/run numeric identity space | Define TextRunId in types::ids and reexport it from types/wire; retain node/run collision rejection with explicit SemanticsId(run.id.0) conversions at validation/error boundaries. |
| 2026-11-04 | V-1b | BoxData.measure contains LeafMeasure, an Rc closure | Put BoxData in engine::contract, not types. This follows the closure boundary and avoids types-to-engine dependency cycles; correct the design/work order. |
| 2026-11-04 | V-1b | Grid repetition and animation repetition are distinct Repeat enums | Rename grid repetition to layout::GridRepeat; retain motion::Repeat and update all consumers without compatibility aliases. |
| 2026-11-04 | CP0 | Trace calls would introduce client-to-diagnostics module dependencies | Keep debug tracing implementation under client::trace; diagnostics documents the public diagnostic workflow without adding a dependency exception. |
| 2026-11-04 | CP0 / F-2 | Configuring disallowed_types enables default warnings before per-crate adoption | Use workspace allow during migration, overridden by each converted crate's explicit deny; F-2 replaces it with workspace deny. Keep P-1 report warnings explicit with -W. |
| 2026-11-04 | V-2 | SemanticsText is both declared and computed, contains float geometry, and cannot support Eq without changing existing equality | Derive PartialEq/Hash, not Eq, for Semantics<R>. Add value Hash to existing text snapshot types with canonical signed-zero hashes; preserve their PartialEq and NaN behavior. Finite::new canonicalizes signed zero. No second text property/transport. |
| 2026-11-04 | V-1b | Style recorder also consumes TextShaper, so engine-only placement would add a forbidden style-to-engine dependency | Keep one backend-neutral pure-query trait in types::text_shaping, depending only on shared values/std; LeafTextMeasurer and closure-owning LeafMeasure remain engine contracts. No duplicate trait or compatibility reexport. |
| 2026-11-04 | CP0 trace | Buffered tracing preserves 781/782 completed reference files byte-for-byte; remaining DX12 file has one extra empty GPU poll | Empty poll counts 4→5; after excluding only fully empty turns and normalizing their turn-number offset, all remaining bytes/order match. Keep raw logs and apply this narrowly documented comparison rule. |
| 2026-11-05 | C-2 | THEMED の lint expect | 既存 Mutex interning を維持し、存在しない expect を追加しない。 |
| 2026-11-05 | C-3a | handler と文字編集 guard | Handled::PreventDefault/and を保持。TextEditsAhead は単一 counter store の所有 guard として dispatch 中の Tree 再借用を可能にする。Runtime::turn は診断 accessor。 |
| 2026-11-05 | C-3b | focus completion と取消 | private FocusCompletion は owner/location、visible は独立入力。取消は内部 Canceled で既存どおり無通知、host 拒否のみ報告。 |
| 2026-11-05 | R-6 / KL | Machine の公開経路 | KL の前提として state::{Machine,Step,MachineHandle,MachineCx,machine} を先に公開。N-D1 の正準経路と同じで alias を作らない。 |
| 2026-11-05 | KL-creative | W3 表と unit の依存 | unit の明示依存 I-3 を優先し W5 へ配置。 |
| 2026-11-05 | L-1 | plural query と ICU hour cycle | Messages::get_plural を純粋 query として公開し境界が利用。ICU 2.4 の private pattern 観測だけ unstable を使用、公開型には漏らさない。 |
| 2026-11-05 | L-1 | Counter release の測定 | 前 64.3068073s / 21,171,200 bytes、後 13.8211922s / 20,431,872 bytes。warm shared target の参考値、formatter 未使用の Counter は dead strip され全 data の容量比較ではない。 |
| 2026-11-05 | C-4 | hover と activation envelope | node hover は子孫を含む既存挙動、region は厳密一致。Fallback は button/target を保持。Writer が各入力直前に準備し Activate は Routed pointer を保持、既存 admit bookkeeping を保つ。 |
| 2026-11-05 | KL-soft | private 名と外部 editor port | AppShellMobileState/MobileNavState で公開型と衝突回避。PowerSearch 編集は original baseline を保持。rich-text toolbar の既存共有 target/composition port は Machine effect から投影し両 mode の連携を保つ。Timer/Timestamp は完成済 machine と lazy memo に構築設定を渡し既存 Text root modifier を保つ。 |
| 2026-11-05 | W3 resource | disk が 40 GB 未満、unit target 使用中 | CP2 終了済みの非稼働 target-gate だけ cargo clean。target-wt の実行を妨げず空き 42.9 GB を確保。 |
| 2026-11-05 | C-5 | side table の lifecycle 所有と再配置 | CoreNodes は metadata のみ、ViewNodes が writer/root/counter/通知を所有。参照・presence source を保ち、Writer::insert は detach 前の継承値と旧新親の layout/a11y を更新して再配置を一元化する。 |
| 2026-11-05 | N-D1 / N-F2 | InlineEdit::field が ui::TextInput を返す | interaction へ移す hook の field は TextEditApi を返し control を描画 node に付ける。draft の実装を interaction に移し ui と共有、label と commit/cancel/blur/cleanup を保って逆向き依存を消す。 |
| 2026-11-05 | CP3 / fix-G1-4 | Contents の削除で layout dirty が消える | 削除前の flat parent を無効化し、共有 cache traversal は空 cache の Contents を通過する。実 box の early stop は保持。共有 target の取り違えは固定 gate で検証する。 |
| 2026-11-05 | CP3 / fix-G6-6 | Contents 箱が Element.children に現れる | query 時だけ透明箱を射影し既存の直接の子の契約を保つ。role query の順序は公開 semantics の child 順で DFS し、allocation 順には依存しない。可変の shadow topology は追加しない。 |
| 2026-11-05 | N-F1 / N-F3 | Recipe の定義が ui にある | 契約は変更せず N-F3 が定義を所有、N-F1 は G3 caller のみ移行し並列の file 所有を保つ。 |
| 2026-11-05 | N-F1 | Transition の定義が types にある | 他 unit の所有と重ならない motion.rs の obsolete decorative 削除を N-F1 に含め、既存 essential と default を保つ。 |
| 2026-11-05 | N fan-out | opaque Component が固有 option を隠す | 設計の「独自部品」に従い合成部品を impl Component、固有 option を持つ標準・kit constructor は具体 builder とし、fluent API を維持。 |
| 2026-11-05 | N-F1 | ActionOutcome の variant が未定義 | Clone の Succeeded(T)/Failed(UiError) とし既存の完了だけ通知、キャンセル時の挙動は増やさない。 |
| 2026-11-05 | N-F2〜N-F6 | Part の参照と InlineEdit の focus 復帰 | 明示的 node_ref を実 node に束縛し Part getter は残さない。共有 TextEditApi の return_focus は既存の commit/cancel 復帰を保つ。 |
| 2026-11-05 | N-F3 | tree_rows の builder 名が未定義 | 複数 root の TreeRows<K>: IntoView を公開し reveal(gap, transition) を持たせる。 |
| 2026-11-05 | N-F2 | 正準 Selection を合成で渡す | sealed IntoSelection の四つの Signal/Binding 入力に Selection 自身の恒等変換を加え、cardinality の分岐と別経路を増やさない。 |
| 2026-11-05 | N-F1 | custom text/semantics に公開 Rc が残る | TextEditing::new は IntoText、accessible_* bool は Option<Value<bool>> に統一。Rc は private 境界だけで使う。 |
| 2026-11-05 | N-F4 | Menu::behavior 削除後の menubar | basic menubar は既存 headless parts と basic recipes で描画し、menu の状態/key 処理を複製しない。getter や別の注入 API は足さない。 |
| 2026-11-05 | N-F4 | 構造を選ぶ kit option と Calendar | Drawer side/Sidebar collapse mode/OTP format を 10.6 に明記。Calendar 表示入力は reactive、navigation/selection は再構築の外側で保持する。 |
| 2026-11-05 | N-F4 / N-F6 | IntoChildren が typed root を消去する | typed child/recipe は private 構築文脈を読み、任意 View は wrapper で配置・clip。公開 View mutation や独自 child trait は作らない。 |
| 2026-11-05 | N-F6 / N-F1 | slot config と factory equality | StepIndicator は plain slot enum。既存 Toast の public field 比較には Callback app/key と ViewFn shared-factory identity を使い、Value 化だけの偽 equality や識別子は足さない。 |
| 2026-11-05 | N-F1-props / N-F4 | core の表示入力が固定値のまま残る | Scroll axes/Overlay 配置は core が reactive に更新し root を保つ。Sidebar side と HoverCard delay は明示の構築 geometry/timing policy として 10.6 に追加。 |
| 2026-11-05 | fix-G3-8 | Prop theme は Rc/owner-bound Value を含み Send でない | 固定 theme と canonical Send startup data を UI thread に渡す。embedded reactive theme は root factory の provide_theme、caller-side reactive App theme は明示 Unsupported。Clone と run/test reactive 評価を保ち unsafe/snapshot を避ける。 |
| 2026-11-05 | fix-G4-3 | key_step を get_untracked で固定すると prop が追従しない | Scalar/RangeScalar は Value を保持し動作時に読む。初期は既存 ScalarRangeError、後から不正なら設定 owner に報告して write しない。 |
| 2026-11-05 | fix-G5-4 | scoped children のサイズ継承が失われる | input_group_with の FnOnce 初期化を group 文脈内で一度実行し eager 入口と mount 処理を共有。C は IntoChildren のまま、既存の継承/明示 size の assertion を保つ。 |
| 2026-11-05 | N-G4 | Part の値が caller 制約へ逆流する | primary を含め semantics を patch に統一し、Part focus は required、caller focus は外部入力として分離。default handler 分類を保持して availability cycle を消す。 |
| 2026-11-05 | fix-G4-5 | 合成 activation と登録解除の不一致 | Popup/List の private anatomy を共有し合成先が activation を一度所有。登録/cleanup は同じ mount.node identity を使い disposed Value を残さない。 |
| 2026-11-05 | N-G5 / G-4c | theme_hint の試験が後の定義に依存 | hook の公開と eager field 登録だけ N-G5 へ前倒し、native pointer/fling/snap は G-4c のまま。 |
| 2026-11-05 | N-8 | API8 が診断と実装 bridge から全 runtime を到達扱いにする | 診断/std Error interop を狭く明記。bridge は doc(hidden)+internal-contract 両方でのみ除外し、普通の hidden API は検査。利用者の text action は Into<String> に直す。 |
| 2026-11-05 | CP4 | canonical facade の逆方向 reexport | state の boundary/owner と style の boundary への再公開だけを個別許可。実装依存と循環は source で除く。AppOptions は必須 title を持つ起動引数なので FRU Default の検査対象外。 |
| 2026-11-05 | I-0 | OverlayViewport の None と f64 metric | 既定 0.0 inset で常時 fitting する。意味が一つの公開数値 metric を優先し、未設定の no-fit を廃止。既存 overlay assertions は維持。 |
| 2026-11-05 | fix-G3-14 | W4 の意図した診断追加と既存期待 | N-G4 の semantics 宣言位置2行と N-F1 の eager pointer write を exact expected に追加。順序・型・発生源・no-op を検証し、既存診断の内容は省略しない。 |
| 2026-11-05 | I-1/I-3 | 指示書の private/public 名の衝突 | 公開 EditSession handle は保持し private enum は EditSessionState。公開 DialogMode は appearance のまま、anchor を持つ private DialogOptionsMode で不正な組合せを除く。 |
| 2026-11-05 | I-3/KL | press 時の snapshot と numeric identity | source_with factory/active_payload 投影と custom::Generation/as_u64 を追加。snapshot Var・Clone side effect・別 allocator counter をなくす。editor zero slop は現状維持。 |
| 2026-11-05 | U-1 | Sheet と native default activation | chooser を trigger の単一 default 内で実行する trigger_with_sheet と state action/projection を追加。on_press の Handled は消えるため抑止に使わない。switch mouse/pen は既存仕様を保持。 |
| 2026-11-05 | I-4 | 存在しない facade Owner.is_alive | 既存の実体 Owned<State>/order の try_get で生存確認し、余分な alive token と新 API を作らない。 |
| 2026-11-05 | T-1 | decoration の公開入力 | style の Color/TextLength record を一度 backend に resolve。StyledRange は style、Decoration/TextLayout は custom、描画引数は既存 Draw。 |
| 2026-11-05 | G-4c | ScrollSnap の形が未定義 | None/Start/Center/End とし既存 ScrollAxes で子を整列。可変サイズを扱い、raw pixel stop の別設定を増やさない。nearest_snap/DragVelocity は types の単一純粋実装を interaction が再公開。 |
| 2026-11-05 | U-2 | inline field は open 後に条件付き mount される | Requested{key} を実際の要求として持ち field mount で Editing へ遷移。dummy session・事前登録の順序要求・後付け slot は作らない。 |
| 2026-11-05 | G-5 | async_view が PartialEq を追加要求する | canonical AsyncCell の Rc publication identity を dynamic が観測。単一 state のまま T:Clone を保ち、Idle は空表示、Pending だけ loading にする。 |
| 2026-11-05 | fix-G5-10 | 既存Basic menuのskip試験とAPG既定Retain | fixtureへ明示Skipを渡し全assertを保持。tooltip fallbackはdeferred Partの既存descriptionを見て決め、空説明でshortcutを消さない。 |
| 2026-11-05 | I-7/G-4b | duration切替で以前のtimingが新設定を上書き | retargetは新target/configured Transitionを基準にする。duration0のdrag即時追従とProportionalの累積短縮を正しく扱い、motion/swipe12件で確認。 |
| 2026-11-05 | G-3 | restored_stateの同期読み取り経路 | Environmentにlaunch時readonly snapshotを一箇所で持つ。virtual scriptだけが永続mapを所有し次launch snapshotを作る。desktop capability false、coreのshadow cacheは作らない。 |
| 2026-11-05 | G-2/fix-G5-11 | queued inputのcapture最終値を現在eventと誤認 | 実releaseを既存requestsの一時mailboxで渡しhandler/default後にcleanup。reader状態の推測・weak registry cache・shadow capture stateは持たない。 |
| 2026-11-05 | K-0 | runtime FrameLoopはcore NodeRefへ依存できない | 公開Core motion handleを正準境界adapterとし、runtimeの単一loop ownerへpresence gateを渡す。互換aliasやcore側rate/stateの写しは作らない。 |
| 2026-11-05 | K-1a | calendar と集合選択の cardinality | CalendarSelection binding descriptor を標準 From で受ける。RangePick anchor/preview のみをstageして取消時はbindingを未変更に保つ、TimeStyle は時刻専用。 |
| 2026-11-05 | K-1b | file metadata と snap | FileInfo は path と純粋 name のみ。検証は caller。Snap None/Mandatory。外部 index binding を操作の正準入口にする。 |
| 2026-11-05 | K-2a | controlled hover と copy command | binding setter を open の唯一の authority とし copy は owner-scoped () command。animated は reactive、custom trigger は attach。 |
| 2026-11-05 | K-2b | 公開 mode と query の所有 | 公開 mode は unit enum、binding は private descriptor。Local/Shared query は排他的 owner、SelectPresentation に統一。 |
| 2026-11-05 | K-3a | joined control の layout state | ButtonGroupMember solid reactive/keep_size structural、toggle recipe の位置は列挙で派生。 |
| 2026-11-05 | K-6a | Key に modifier がない | kbd_combo に canonical Modifiers 入力を加え command_key capability から記号化。 |
| 2026-11-05 | K-8 | Core viewport と snapshot 境界 | ViewTransform types 正準、hidden port で単一 ZoomPanApi 接続。context-local render purpose と immutable CPU fallback。 |
| 2026-11-05 | K-6a | Creative KeyGlyph は keyboard ではない | keyframe の Diamond 等を描く domain enum のため Creative に残し、kbd への誤った移行指定を C.2 から削除。 |
| 2026-11-05 | K-5 | Lightbox pan の正準操作 | use_zoom_pan の middle/Space-primary/touch を採用し、lightbox 独自 primary-drag model を廃止。 |
| 2026-11-05 | K-2b | custom chooser の選択 owner | selection_field を追加し popup/sheet と caller search binding を使う。fake key/selection state は作らない。 |
| 2026-11-05 | K-1b/K-6a | Kit recipe で表せない内容と方向 | CarouselSurfaceState と icon controls、Dropzone.icon、Avatar.fallback_content、badge_label を追加し visual wrapper を廃止。 |
| 2026-11-05 | K-3b | responsive と count の authority | PageCount enum で pages/items を排他にし、Stepper collapsed は geometry の派生値。 |
| 2026-11-05 | K-4 | static colspan の公開意味 | Semantics/Node/native/Element まで同じ column_span を接続。 |
| 2026-11-05 | K-9a | author、history、共通 API | author IntoText、sender enum、line_numbers reactive に統一。外部 dataset revision だけ follow hook に渡す。 |
| 2026-11-05 | K-9b | stat の animation と chart | caller Animated と独立 sparkline を slot に合成し、stat に数値状態を重複させない。 |
| 2026-11-05 | K-8 | viewport の内部依存 | viewport の environment read と GPU adapter read のみを精密な module edge 例外とする。新たな service state を複製しない。 |
| 2026-11-05 | K-1a | 実際の起動元へのfocus復帰とsheet操作 | DialogApi.return_focus とpicker_labelを正準入力とし、Popup既存ownerで閉じた後の復帰を判断。範囲は2点目のみcommit、Cancelは外部bindingを変更しない。 |
| 2026-11-05 | K-4/K-8 | Creativeのviewportとdescription意味 | Time/Arrangementは単一ViewTransform、Graphは独立time/value区間のdomain座標を保持。property_rowは標準descriptionのListItemに統一し既存の数値geometry検証を維持。 |
| 2026-11-05 | K-3b/K-6a/K-6b | 実際の入力とslotをrecipeへ投影 | BreadcrumbItemState、CardState.input、avatar_image/status_content/member/overflowを実ノードへ接続。履歴状態を複製せず装飾とhoverを表現する。 |
| 2026-11-05 | H-G5 | Game haloのreduced motion | effectから時刻を書かず、描画がreduced時に静止値を選ぶ。動作中のphaseは既存frame clockのみが所有する。 |
| 2026-11-05 | H-G5 | InspectorPanel.collapsedの分類 | InspectorPanelは外部データセットの展開値を持つpassive recordであり、mounted builderの構造optionではない。structural_optionsのrecord_typesへ正確に登録する。 |
| 2026-11-05 | K-1a | swipeのactivation抑止 | Callbackの戻り値でpressを止めず、CalendarOptions.activation_guardを単一calendar ownerの遷移前に評価する。Releasedは同一gestureの後続選択を拒否し、新gesture/keyboard/a11yで解除する。 |
| 2026-11-05 | K-4/K-9b | Contentsと意味情報の責務 | Displayはlayout boxのみを制御する。a11yはlogical hierarchyを保ち、boxなしgroupのboundsは実際の子のunionから導出する。layout/paint/inputのflattenは変えない。 |
| 2026-11-05 | K-3b | ページ窓とnavigationのidentity | Buffered/Adjacentはimmutable構築policy。panel_keyは実panelのidentityのみを割当てる。controlled Menuは同じaccepted bindingを共有しHover→Menu effect mirrorを廃止する。 |
| 2026-11-05 | K-4 | Tree/List/Table和集合 | Treeのancestry/marker、ItemRowの実slotとText行制限、列挙済datasetからのreadonly位置を標準部品へ追加。tableのselection toolbar/statusは付録指定の固有合成として保持する。 |
| 2026-11-05 | K-9a | CodeBlockの開閉と履歴選択 | 同じDisclosure ownerのcontent/controlsとshow/presenceを用い、再表示時はsource/highlighter factoryから実editorを構築。履歴は実editorで全選択し、別のhistory/visibility ownerを置かない。 |
| 2026-11-05 | K-7e | unit contextとpure visual defaults | `view::component`を一度だけのmount/setup境界として公開し、偽の`provide(())`を除去。VariantのDefaultはValueを作らず、明示的なsetup setterだけがreactive入力を構築する。metadata全ページ試験で確認した失敗を根本修正。 |
| 2026-11-05 | H-G5 | Soft FocusTrack の分類 | 唯一の可変事実は roving cursor。focused_within は実 NodeRef、selected/disabled は既存 owner の読み取り専用投影であるため、独立した複数状態ではない。型ごとの理由付き目印で分類し、第二の Tabs 状態機械を作らない。 |
| 2026-11-05 | H-G6 | 検査の適用先 | KIT規則の対象は公式5kitの正確なpackage名とし、同prefixのcatalogアプリを誤認しない。Generation/Rgba/Insetsは状態機械/色/幾何の値操作で構造optionではない。Transition::alternateは構築時の再生方向として明示登録する。全skipは空にする。 |
| 2026-11-05 | H-G6 | Creative jobs の running | 単独のrun/stop要求は18.14の独立bool事実。progress/amountは数値測定であり、effectは実FrameLoopのstart/stopのみを行う。別Signalの補正をしないため、bool置換だけのMachineは追加せず既存のapp所有値を保持する。 |
| 2026-11-05 | H-G6 | Draw text measurer の境界 | 実使用はmountとOffscreenの2つの自動install経路だけ。ユーザーにInstall-before-drawを要求するAPIを残さず、doc(hidden)+理由付きinternal-contractのcross-crate bridgeへ分類する。通常の描画・段落測定は自動install済みhostを使う。 |

| 2026-11-05 | CP6/F1 | docsの試験スケジュール | 並列doctestの同時compile/linkでlink.exe1180とrustc ICEが実際に発生した。0.6節の失敗crate例外としてkivact-docsだけ--test-threads=1で別実行する。workspace集約はdocsとGPUproducerを除いて既定並列、GPU6件も従来通り別の逐次実行。Cargoのjob数は制限しない。 |
| 2026-11-05 | fix-CP6 | 変更後の契約に合わせる試験 | AsyncView既定pendingはStatus Loading、pickerエラーはAlert、Display::Contentsはlayoutが透明でもa11y Groupと論理階層を保つ。Desktop専用date fixtureは明示Desktop、汎用adaptiveはtouchのSaveを保つ。iconは実インストール済みglyphで検査する。未知のnative timeoutとdescriptionの関係欠落は期待を弱めず原因を切り分ける。 |

## 調査・停止・定義の追記

台帳の表以外に記録された34項目も以下に保存する。日付は各節に示す。問いは節の見出し、決定と理由は各項目に記す。これらも後の19系列と追加判断で置換された部分は履歴として読む。

### 2026-11-04: 状態の全数調査は何を含めるか

理由はSの規則の対象を明確にし、cacheや独立した事実を形式的なMachineへ変換しないためである。決定・記録:

- J-01: 調査で34件（SM-R901、SM-A901、SM-I901〜904、SM-U901〜905、SM-K901〜923）を追加し計203件とした。SM-C27を現契約へ訂正した時点で検索/checker 521行/Clippy 43か所の未処理は0だった。これは当時の調査結果で、以後の追加を制限しない。
- J-02: PieStateは公開enum Closed / Open { mode: Click または Held, center, hot }（SM-K901）とする。
- J-03: S-9のcache/counter/mailbox/統計、process単位static OnceLock、試験harnessのmailbox、値/topologyのdebug_assertは状態機械にしない。
- J-04: 調査台帳の当時の全文はcommit 6758cb4に保存した。2026-11-05に要約へ置き換えたもので、現行規則の理解に全文の再調査は要求しない。

### 2026-11-04: 明示的な停止要求と未完了checkpointをどう保存するか

理由はユーザーの停止指示を守りながら未証明事項を完了扱いしないためである。決定・記録:

- J-05: ユーザーが適切な境界で停止し結果と記録をcommit/pushするよう明示した。実行中検査を終え、実装/reviewの拡大を止め、CP1のtrace同一性が未証明でも中間integration branchを保存する。この一回の停止指示だけが通常のcheckpoint PASS後pushの順序より優先した。
- J-06: その停止時点でCP1/全体目標を完了にせず、W2未着手と記録した。最終sourceの試験/staticは通ったがtrace厳密比較はREVIEW_REQUIREDだった。誤epoch対応、未解決の数値forward replay、通知/作業順の差を丸め誤差に変更せず保存した。後のADR-19.0が厳密比較自体を置換した。
- J-07: 一時的な診断/再開bundleを保存し、raw trace、大きなJSON、build logはignored targetに置いた。scriptは診断道具でgoldenではない。長期理由をこの記録へ保存した後、一時bundleは削除対象とした。

### 2026-11-04: 次の実装前にbackend/runtime境界の何を確定するか

理由はthread、window identity、実shutdown時点と正準型を失わないためである。決定・記録:

- J-08: A-0: SendのMakeHandlerとembed UI thread構築用ChildHooksを分ける。Rcを持つhandlerをthread間に直接渡さず、borrowed blocking handlerのBox<dyn ShellHandler + '_>はbackend内部の実際の借用契約として保持する。
- J-09: A-0: consuming backendがsession shutdownを所有。native guardは正常/起動失敗/unwindいずれもhandler/presenter cleanup後、embed guardは返却handleの早期close要求ではなく実UI handler破棄後に動く。意図したleak/panic時のmodule保持を保ち、軽量native factoryが非Send backend全体をSend factoryへ捕捉しない。
- J-10: A-0: window作成前capabilityは対応可能な上限。任意native setup後の実ShellWindowのcapabilityが正本。
- J-11: A-2: serviceは完成した各ShellWindowに属しUI threadで構築する。window identityのないBackend/SteppedBackendの事前service getterはparent dialogや複数embed resize交渉をlate bindingなしで扱えないため削除し、OS要求/UI thread返信のperform一つを保つ。
- J-12: A-2: service転送前にRegisterFont/ReducedMotionをengine HostRequestへ分離。Writer時点のfont登録とHostのmotion overrideを保ち、effect完了配送前に実Environmentを同期する。E-2の前提をA-2で実施済みとした。
- J-13: R-1: 既に統一したtypesのTransitionをそのまま移す。as_wireを残す旧指示を訂正し、V-6/VOC-5で除いたwrapperを復活させない。
- J-14: R-1: runtime実装moduleはprivate、既存公開和集合だけrootで明示再公開。macro_exportがconstant_valueをrootへ置き、既存の$crate::IntoValue/Propはroot再公開で成立する。macroのためにprivate namespaceを公開しない。

### 2026-11-04〜05: 共有値とf64移行で何を保全するか

理由は統一後も意味、入力順、精度境界と既存assertionを維持するためである。決定・記録:

- J-15: V-2: Semantics<R>は旧NodeSemantics、二つのproperty record、protocol nodeに分散していた意味情報を統合する。transport nodeはtopology/geometryと一つのsemanticsを持つ。既存wireのoptional boolを優先しprotocol固有protectedはbool。
- J-16: V-2: engineはdraw regionを指すrelationを失わないSemantics<FocusTarget>を持つ。説明用Semantics<NodeId>は不十分で、a11y公開時にnode/regionを組でSemanticsIdへ写す。
- J-17: V-3: EmbedOptionsはsize/scale/prefer_gpu/gpu_preferenceの純粋値。resize callbackとpanic slotはshellのChildHooksとして入口とBackend.open_childで明示的に並べ、callback/panic転送を保つ。typesからshellへの依存は作らない。
- J-18: V-3: FailureKindは既存Arvane ErrorKindのContent/Resource/Backend/Surface/Unavailableを採用し、任意の新分類を足さない。
- J-19: V-3: 旧Arvane Error { kind, message }はFailureと同義なので削除して直接再公開する。新typeで包んでも境界は増えない。render/text adapter errorはFailureへ分類し、core user diagnosticはsource locationと診断の別の意味を保つ。
- J-20: V-3/E-2: W1のfont-generation field追加ではeffect/event順を変えない。E-1のcollection owner移設後、成功登録時にgenerationを進めEnvironmentを公開する。後のnative environment refreshで戻さない。
- J-21: V-3: LayoutViolationは一つのLayoutFindingにcreation location/text/role/name/test IDを付す診断情報。rule/amount/geometryを平行fieldに複写しない。LayoutRule/Limit/Checkを再公開し、共有checkのignore-test-id builderを保つ。
- J-22: V-4: helper定義を先にmergeし、同じ型への重複編集を避けるためmodule横断f64変換をその後、利用側family移行の前に行った。最終scopeは縮小しない。
- J-23: V-4: f32暗黙縮小を除いても1e300の巨大window拒否を保つ。App/resizeはenvironment語彙のpositive/finite/u32 extent判定を共有し、nativeはscale後のphysical sizeも検証する。f64でfiniteになったことを理由に既存invalid fixtureを消さない。
- J-24: CP1: f64::MIN_POSITIVEのtiny fixtureも旧f32縮小で0になっていた。window寸法は少なくとも1 logical pixel、finite、unsigned native extent内と明示しApp/resizeで共有する。rustdocとfixtureを保ち、f32 underflowへ共有APIを依存させない。
- J-25: CP1: 六catalogの二つの偽1px overflow（1.00000190734863281px、1.00000333786010742px）はParley offset/advanceの別々の拡張によるnative f32残差だった。342a2d0でuncut左右endpointをnative精度で確定してから拡張しf64幅を導出した。custom ellipsis、glyph位置、checker許容値、assertionを変えず六試験が通り、一時測定出力はcommitしなかった。
- J-26: CP1: paragraph decorationのemはparagraph fontで、明示span decorationだけspan fontで解決していた。fcd40aeでこの差をcanonical computed styleでも保ち、native metric fallbackはsegmentに従う。別styled node間の相対spacingのchild font解決とは区別する。
- J-27: CP1: pointer sampleを完全に移行しDown/Upで同じbutton/press metadataを持たせる。host-sync二fixtureのNoneを22aa80bで修正しevent/assertionは変えなかった。不正fixtureのためにtrace整合性を緩めない。
- J-28: CP1: deferred inputは旧key pressだけが対象でKeyUpは対象外だった。全KeyInputをdeferするとreleaseも遅れOTPの中間値123対12a3が変わった。KeyPhase::Downだけに限定しreleaseはpendingをpumpする。最終assertionが通ってもbatch/callback/write/commit差を正規化で消さない。
- J-29: CP1: cubic solverの30回二分法は時刻0に1.3969838612726965e-9の残差を残しmenu幅75.05000000551809の余分なLayoutChangedを起こした。38641d4でendpointを厳密な0/1にし内部補間とStepsStartの非0開始を保った。epsilon追加やf32への丸めで実処理差を隠さない。
- J-30: V-4: Generationはcopyable opaque u64、0開始、nextはoverflow確認、is_currentはtoken比較。Sent.updateはOption<&T>契約。detached-thread trackingは既存四sticky atomicと同じprocess恒久事実として保つ。
- J-31: V-5: pressure不明はOption<f64>、is_primaryも保つ。Routed<T>はinput、optional target、optional region、local、captured、activationを一度だけ保存。targetなしはwindow-directed keyboard、pointer/wheelはtargetあり。Activationは当時既存adopted boolを包みC-4へ引き継いだ。
- J-32: V-5: IME selectionは直前surrounding-text snapshotに対するUTF-8 byte。nativeは既存snapshotでUTF-16を変換。engineは既存excerpt originを全文editorへ加え、custom sessionは全文を供給する。native snapshotに重複した全文offsetを足さない。
- J-33: V-6: computed textは継承を解決してもrelative length/current colorを保持。TextLen/LineHeight/TextDecorationを使いpixel化は利用時、無制限行/未指定caretはOptionでよい。親10pxの0.1em spacingは子20pxで2pxとなる。第二のraw style snapshotは作らない。
- J-34: V-6: DrawTextStyleはtoken色とbaseline anchorを持つdrawing optionで、純粋宣言text styleとは異なる。canonical runへ一度変換し、当時のfrontdoor名はN-1まで保持した。RgbaImage/著者用Pixelsは同じcanonical shared pixel値を再公開し別structを削除した。後の共有入力の形はAPI8決定に従う。

## ADR: 検証で確認した term の参照と gauge の範囲

- 日付: 2026-11-05
- 問い: カスタムtermのラベル参照と、旧kit gaugeの試験をどの公開契約に揃えるか。
- 決定: カスタムdescription termの実wrapperはGroupとして公開し、detailのlabelled_byが同じnamed termを指す。term/detailに同名Groupがあっても、detail内の操作と参照先を検証する。Game入門のgaugeも標準ProgressBarの0..1範囲を使い、4/10は0.4と検査する。
- 理由: unroledなGenericはnativeラベル参照先にならず、見えるtermと支援技術の関係が切れる。進捗のvariant間で役割と数値契約を統一し、旧Meterの別スケールを公開APIへ戻さない。

## 拡張境界の最終確認

| 日付 | unit | 問い | 決定 | 理由 |
|---|---|---|---|---|
| 2026-11-05 | F3 | 拡張時の変更箇所 | text styleの公開語彙は所有するstyle crateの明示facade一覧で一元化し、Coreはその限定した一覧だけを転送する。virtualは既知serviceを処理し未知requestはUnsupported、capabilityは構築時の値と実GPU/native事実から投影する。 | 新しい型やserviceごとにCoreの重複一覧やvirtualの網羅matchを修正する第四のcrateを不要にし、型付きAPIと正直な未対応の契約を維持するため |
| 2026-11-05 | F3 Windows mirror | 並列native試験の異常終了 | required mirrorのpublic API e2e binaryでSTATUS_ACCESS_VIOLATIONが発生したため、そのbinaryはtest-threads=1で再実行する | 実窓を伴う失敗binaryだけを直列化し、他の試験とCargo jobsは既定並列を保つため |
