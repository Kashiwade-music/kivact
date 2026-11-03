# foundation/motion — Astryx のモーション全抽出（kivact-ui-soft 再現用）

この文書は Astryx（Meta、React + StyleX、`D:\w\meta\astryx`）のアニメーションを、Kivact 上の `kivact-ui-soft` で**数値どおりに**再現するための作業指示書である。
モーションの調整を目分量で行ってはならない。この文書の数値・曲線・順序をそのまま実装すること。

## 0. 読み方と前提

- 特に断りがなければ、パスは `D:\w\meta\astryx\packages\core\src` からの相対パスで示す。`file:line` は調査時点の行番号である。
- 単位: 時間は ms、スプリングの時間は s、速度は px/s（例外は明記する）。距離は CSS px で、Astryx は UIKit の pt を 1:1 で px として扱う（`motion/uikit/gestures.ts:14-16`）。
- **対象テーマは Neutral**（`packages/themes/neutral/src/neutralTheme.ts:85` の `motion: {fast:125, medium:300, slow:700, ratio:0.75}`）。core の既定値も「既定 / Neutral」の形で併記する。kivact-ui-soft の値は Neutral 側を使う。
- `UNRESOLVED` はソースから確定できなかった値である。推測で埋めないこと。
- 「[推論]」はコードを読んで導いた挙動で、ブラウザでは確認していない。

### 0.1 二層構造（最重要）

Astryx のモーションには、性格の異なる 2 つの層がある。

1. **CSS トークン層（ほぼ全コンポーネント）**
   - 中身は StyleX の `transition` / `@keyframes` / `@starting-style`。値は `--duration-*` と `--ease-*` のトークンを読む。
   - 適用範囲: デスクトップ（fine pointer）の全挙動と、タッチでも大半のコンポーネント。
2. **タッチ・プレゼンテーション層（UIKit 実測値）**
   - `motion/` 配下にある。解析解スプリング、decay、ラバーバンド、ページング、FLIP、Reduce Motion 置換を備えたコンポジタ・アニメーター。
   - 数値はすべて iOS 26.2 シミュレータでの実測値か Apple 文書の値で、各値に `@source` タグが付いている。
   - **実際に `createAnimator` を使うコンポーネントは Switch（`Switch/useSwitchTouch.ts`）と Slider（`Slider/sliderTouchController.ts`）の 2 つだけ**（grep で確認済み）。
   - その他の利用者:
     - `animatedLayout.ts`（FLIP）は Collapsible / TreeList / Table 行展開 / SideNav / AppShell が使う。
     - ProgressBar のタッチ表示は、テーブル値を CSS で使う。
     - BottomSheet / Toast / Item swipe は、独自の CSS ベース実装で、UIKit スプリングを使っていない。
   - `springs.ts` にある sheet / menu / alert / banner などのスプリングは、**定義されているが未使用**である。

kivact-ui-soft は「Astryx が実際に再生しているもの」を再現する。したがって未使用の UIKit スプリングを勝手に適用してはならない。§1.3 の表の「使用箇所」列で、実際に使われているかを確認すること。

---

## 1. モーションモデル

### 1.1 CSS 時間トークン

既定値の出典は `theme/tokens.stylex.ts:300-310`。Neutral 値は `theme/expandMotionScale.ts` で生成される。計算式は min = base×ratio、max = base÷ratio で、5ms 単位に丸める。

| トークン | 既定 | **Neutral** | 用途（コメント由来） |
|---|---|---|---|
| `--duration-fast-min` | 130ms | **95ms** | 最小のマイクロインタラクション |
| `--duration-fast` | 175ms | **125ms** | hover / toggle / checkbox |
| `--duration-fast-max` | 230ms | **165ms** | やや長いマイクロ（レイヤー登場） |
| `--duration-medium-min` | 310ms | **225ms** | 速い登場・退場 |
| `--duration-medium` | 410ms | **300ms** | 標準の登場・退場（drawer, sheet） |
| `--duration-medium-max` | 550ms | **400ms** | 劇的な登場（Dialog） |
| `--duration-slow-min` | 730ms | **525ms** | 速い連続（Spinner 1 周） |
| `--duration-slow` | 975ms | **700ms** | 連続 |
| `--duration-slow-max` | 1300ms | **935ms**（933.3 を丸め） | ゆっくりした連続 |

- Neutral の slow 帯はテーマのコメントに書かれていない。式から計算した値である。
- `tempo` ノブ（`theme/motionKnobs.ts`）は 0.75〜1.5 にクランプされ、全 `--duration-*` に掛かる。**Neutral は tempo を設定しないので 1** である。
- ビジーループ（Spinner, Skeleton）は `--_astryx-pre-tempo-duration-*` を読み、tempo の影響を受けない（`theme/themeMotion.ts` の `preTempoName`）。Neutral では pre-tempo 変数が書かれないため、通常トークンへフォールバックする。
- press の release フェードだけは `--_astryx-motion-tempo` を読む。式は `200ms × min(tempo, 1.5)`。

### 1.2 CSS イージングトークン

出典は `theme/tokens.stylex.ts:324-337`。Neutral は easing も character も設定しないので、既定値のままである。

| トークン | 値 | 性格 |
|---|---|---|
| `--ease-standard` | `cubic-bezier(0.24, 1, 0.4, 1)` | 強い減速（「到着と状態変化」）。**全コンポーネントの基本カーブ** |
| `--ease-entrance` | 同上（character=standard 時） | 登場。コンポーネントからはほぼ参照されない |
| `--ease-move` | 同上 | 移動。コンポーネントからはほぼ参照されない |
| `--ease-exit` | `cubic-bezier(0.3, 0, 0.6, 0.6)` | 加速して去る。終端の傾きは 1（減速しない）。使うのは BottomSheet の close だけ |

character ノブ（`themeMotion.ts:59-63`）は `--ease-entrance` と `--ease-move` だけを変える。値は crisp = `cubic-bezier(0,0,0.2,1)`、expressive = `cubic-bezier(0.3,1.2,0.5,1)`。Neutral では無関係である。

各カーブの進捗表を下に示す。数値は実装の確認用で、Kivact の `Easing::CubicBezier` に同じ制御点を渡せば一致する。

| カーブ | t=0.05 | 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.6 | 0.8 | 0.9 |
|---|---|---|---|---|---|---|---|---|---|
| standard (0.24,1,0.4,1) | 0.198 | 0.372 | 0.639 | 0.807 | 0.902 | 0.954 | 0.980 | 0.998 | 1.000 |
| exit (0.3,0,0.6,0.6) | 0.005 | 0.021 | 0.079 | 0.167 | 0.275 | 0.397 | 0.525 | 0.778 | 0.894 |
| move/easeInOut (0.42,0,0.58,1) | 0.005 | 0.020 | 0.082 | 0.187 | 0.332 | 0.500 | 0.668 | 0.918 | 0.980 |
| release (0,0,0.2,1) | 0.175 | 0.304 | 0.500 | 0.645 | 0.755 | 0.839 | 0.902 | 0.978 | 0.995 |
| CSS `ease` (0.25,0.1,0.25,1) | 0.033 | 0.095 | 0.295 | 0.513 | 0.683 | 0.802 | 0.885 | 0.976 | 0.994 |
| CSS `ease-out` (0,0,0.58,1) | 0.082 | 0.161 | 0.308 | 0.445 | 0.571 | 0.685 | 0.785 | 0.938 | 0.983 |

**重要な観察**: `--ease-standard` は時間の 20% で 64%、50% で 95% まで進む。体感上の長さは公称 duration の半分以下になる。Astryx の「キビキビ感」はこのカーブから来ている。CSS `ease` や Kivact の `Easing::EaseOut`（三次多項式）で代用してはならない。

### 1.3 モーションロール（全 33 個）

ロールとは、登録されたモーションテーブルの値（spring / `…Ms` duration / 4 点 curve）のうち、テーブルが `roles` に列挙したものである（`motion/roles.ts:67-159`）。生成表は `docs/specs/AST-094-touch-presentations/motion-roles.md` にある。

表中の物理値は `motion/spring.ts` の変換で計算した。

- settle は、単位移動（0→1、初速 0）が ε=0.001 以内に収まるまでの時間（`settlingDuration`）である。
- overshoot は ζ<1 のときのピーク超過率で、式は exp(−πζ/√(1−ζ²)) である。

**springs テーブル**（`motion/uikit/springs.ts`、ロール登録は `SPRING_ROLES` :510-529）

| ロール | 記述（ソースの語彙） | m | k | c | ζ | response | settle | overshoot | 実使用 |
|---|---|---|---|---|---|---|---|---|---|
| `springs.settle` | duration 0.5, bounce 0（SwiftUI `.smooth`） | 1 | 157.914 | 25.133 | 1.00 | 0.500s | **734.8ms** | 0 | **animatedLayout（FLIP・エッジ reveal・シェブロン）**、scroll.bounce と同値 |
| `springs.quickChange` | 0.5, bounce 0.15（`.snappy`） | 1 | 157.914 | 21.363 | 0.85 | 0.500 | 736.7 | 0.63% | 未使用（README の例のみ） |
| `springs.emphasis` | 0.5, bounce 0.3（`.bouncy`） | 1 | 157.914 | 17.593 | 0.70 | 0.500 | 863.0 | 4.60% | 未使用 |
| `springs.fingerTracking` | response 0.15, ζ 0.86 | 1 | 1754.596 | 72.047 | 0.86 | 0.150 | 219.2 | 0.50% | 未使用 |
| `springs.stateChange` | response 0.55, ζ 1 | 1 | 130.507 | 22.848 | 1.00 | 0.550 | 808.2 | 0 | 未使用 |
| `springs.directChange` | response 0.5, ζ 0.825 | 1 | 157.914 | 20.735 | 0.825 | 0.500 | 753.1 | 1.02% | 未使用 |
| `springs.timedChange` | duration 0.5, bounce 0 | 1 | 157.914 | 25.133 | 1.00 | 0.500 | 734.8 | 0 | 未使用 |
| `springs.viewChange` | m3 k1000 c500 → 臨界にクランプ（c=109.545） | 3 | 1000 | 109.545 | 1.00 | 0.344 | 505.7 | 0 | 未使用 |
| `springs.menuOpen` | 0.32 / 0.8 | 1 | 385.531 | 31.416 | 0.80 | 0.320 | 493.7 | 1.52% | 未使用 |
| `springs.menuClose` | 0.35 / 0.85 | 1 | 322.273 | 30.518 | 0.85 | 0.350 | 515.7 | 0.63% | 未使用 |
| `springs.subMenuOpen` | 0.4 / 0.85 | 1 | 246.740 | 26.704 | 0.85 | 0.400 | 589.3 | 0.63% | 未使用 |
| `springs.subMenuBack` | 0.4 / 1 | 1 | 246.740 | 31.416 | 1.00 | 0.400 | 587.8 | 0 | 未使用 |
| `springs.contextMenuLift` | 0.32 / 0.8 | 1 | 385.531 | 31.416 | 0.80 | 0.320 | 493.7 | 1.52% | 未使用 |
| `springs.contextMenuDismiss` | 0.35 / 0.85 | 1 | 322.273 | 30.518 | 0.85 | 0.350 | 515.7 | 0.63% | 未使用 |
| `springs.contextMenuMorph` | 0.3 / 0.8 | 1 | 438.649 | 33.510 | 0.80 | 0.300 | 462.8 | 1.52% | 未使用 |
| `springs.sliderRelease` | 0.4 / 1 | 1 | 246.740 | 31.416 | 1.00 | 0.400 | 587.8 | 0 | **Slider** |
| `springs.sliderLift` | 0.215 / 0.6 | 1 | 854.049 | 35.069 | 0.60 | 0.215 | 425.9 | 9.48% | **Slider** |
| `springs.sliderDrop` | 0.5 / 0.7 | 1 | 157.914 | 17.593 | 0.70 | 0.500 | 863.0 | 4.60% | **Slider** |

**switchControl テーブル**（`motion/uikit/switch.ts`。すべて **Switch のタッチ表示**で使用）

| ロール | 値 | k | c | ζ | 備考 |
|---|---|---|---|---|---|
| `switchControl.lift` | 0.23 / 0.55 | 746.3 | 30.05 | 0.55 | 指が触れたときのレンズ拡大 |
| `switchControl.settle` | 0.5 / 0.7 | 157.9 | 17.59 | 0.70 | 指を離した後の縮小 |
| `switchControl.travel` | 0.3 / 1（= `SWITCH_THUMB_MOVE_SPRING`） | 438.649 | 41.888 | 1.00 | つまみの移動（settle 440.9ms） |
| `switchControl.tracking` | 0.36 / 1 | 304.6 | 34.91 | 1.00 | ドラッグ中の指追従 |
| `switchControl.wellOn` | m1 k85.3772 c18.48 | 85.377 | 18.480 | 1.00 | トラック色 ON（response 0.68s） |
| `switchControl.wellOff` | m1 k246.7401 c31.4159 | 246.740 | 31.416 | 1.00 | トラック色 OFF（response 0.40s） |
| `switchControl.minimumLiftMs` | 235ms | | | | lift 開始から settle 開始までの最短時間 |

**timing テーブル**（`motion/uikit/timing.ts`）

| ロール | 値 | 実使用 |
|---|---|---|
| `timing.stateChangeMs` | 250ms | 未使用（CATransaction 既定） |
| `timing.curves.viewChange` | (0.25, 0.1, 0.25, 1) | 未使用 |
| `timing.curves.move` | (0.42, 0, 0.58, 1) | **Reduce Motion のクロスフェード、ProgressBar の減速パルス** |
| `timing.curves.exit` | (0.42, 0, 1, 1) | 未使用 |
| `timing.curves.entrance` | (0, 0, 0.58, 1) | 未使用 |

**progress テーブル**（`motion/uikit/progressBar.ts`。ProgressBar のタッチ表示と減速パルスで使用）

| ロール | 値 |
|---|---|
| `progress.fillTrackMs` | 1000ms（トラック全長 0→1 の時間。変化量に比例） |
| `progress.curves.fill` | (0, 0, 1, 1)、linear |
| `progress.reducedMotionPulseMs` | 2000ms（片道） |

**ロールではない定数スプリング**（`springs.ts`）

- 実測値だが未使用、または参照値: navigationPush/Pop 0.3/1、navigationInteractivePopFinish 0.3/0.85、navigationBarElements duration 0.6 bounce 0、sheetPresent/Dismiss m3 k1000 c500、sheetDetent 0.3441442326/1、sheetReleaseFast/DismissSwipe 0.3441442326/0.8、alert/actionSheet m1 k522.35 c45.71（settle 404ms）、menuReducedMotion 0.4/1（**再生しない参照値**）、keyboard m1 k555.0265 c47.118、segmentedThumb 0.3/0.75（GUESS）、banner m3 k1000 c500（GUESS）。
- `scroll.bounce` = {duration 0.5, bounce 0}（GUESS M-S3）は、coast がエッジに達したときのバネとして使われる。

### 1.4 ロールの解決

- `registerMotionTable(id, table, {roles, exclude, rename})`（`roles.ts:79-159`）
  - テーブルを歩き、`roles` に列挙された leaf だけを `id.path` のロールとして登録する。
  - 2 つのキーが同じ leaf オブジェクトを共有している場合、各ロールが固有のオブジェクトを持つように複製する（:138-145）。
  - 同じテーブルの再登録は no-op である。
- 各ロールには CSS 変数 `--motion-<kebab>` が割り当てられる（例 `--motion-springs-settle`、`roleValue.ts`）。
- **`resolveMotion(element, leafOrGroup, key?)` は常にテーブル値をそのまま返す**（`resolveMotion.ts:60-68`）。
  - CSS 変数は読まない（spec AST-094 DEC-15）。テーマにもページにもロールは上書きできない。
- 結論: Kivact では各ロールを `const` として実装すればよい。テーマ化は不要。
- AST-108 FR3 には「スプリングは tempo・character で変形する」とあるが、**未実装**である。`themeMotion.ts` のコメントは "No spring reads the knobs yet" と明言している。
- CSS 形式（`roleValue.ts`）:
  - spring は `"<duration>s <bounce>"`（例 `0.5s 0`）
  - duration は `"250ms"`
  - curve は `cubic-bezier(...)`
  - パース条件: spring は duration>0 かつ −1<bounce<1。curve は x1, x2 ∈ [0,1]。

### 1.5 スプリングの数学（`motion/spring.ts`）

**モデル**: m·x″ + c·x′ + k·x = 0 を閉形式で解く。数値積分ではない。単位は時間 s、速度は値の単位/s。

**変換式**（`spring.ts:26-37, 111-162`）

- (duration d, bounce b) → m=1、k=(2π/d)²
  - b≥0 のとき c = (1−b)·4π/d
  - b<0 のとき c = 4π/(d·(1+b))
  - b は [−0.999999, 1] にクランプする
- (response r, dampingRatio ζ) → ω₀=2π/r、m=1、k=ω₀²、c=2ζω₀。過減衰（ζ>1）は保持する
- (mass, stiffness, damping) → `springFromPhysics`。`allowOverdamping=false`（既定）では c を臨界値 2√(km) にクランプする
- 逆変換 `durationBounceOf`: duration=2π/ω₀。bounce は ζ≤1 なら 1−ζ、それ以外は 1/ζ−1

**閉形式解**（`solveSpring`、:281-387）。x0 = from − to、v0 = 初速とする。

- 不足減衰（ζ < 1−1e-6）: λ=ζω₀、ωd=ω₀√(1−ζ²)
  - x(t) = e^(−λt)·(x0·cos ωd t + B·sin ωd t)、B = (v0 + λx0)/ωd
  - v(t) = e^(−λt)·(v0·cos ωd t − ((λv0 + ω₀²x0)/ωd)·sin ωd t)
- 臨界（|ζ−1| ≤ 1e-6）:
  - x(t) = (x0 + (v0 + ω₀x0)t)·e^(−ω₀t)
  - v(t) = (v0 − ω₀(v0+ω₀x0)t)·e^(−ω₀t)
- 過減衰: r1,2 = −ζω₀ ± ω₀√(ζ²−1)
  - c1 = (v0 − r2·x0)/(r1−r2)、c2 = x0 − c1
  - x = c1 e^(r1 t) + c2 e^(r2 t)

**収束判定**（settleTime）: 包絡線が threshold を下回る時刻である。

- 不足減衰は閉形式で求める: ln(|A|+|B| / threshold)/(ζω₀)。
- 臨界・過減衰は二分法で求める（倍々で上限を探索し、最大 100 回）。
- `SETTLING_EPSILON = 0.001`（単位移動に対して）。
- アニメーターでは、プロパティごとの絶対 threshold `SETTLE_THRESHOLD` を使う（`animator.ts:627-634`）:
  - x, y = 0.05 px
  - rotate = 0.01°
  - scaleX, scaleY = 0.0001
  - opacity = 0.001
- 実際の再生時間は **移動量に依存する**。例えば settle スプリングで 300px 動かすと、0.05px までかかる。
- animatedLayout だけは単位移動 ε=0.001 固定の 734.8ms を使う（§1.9）。

**ランプ追従**（`solveSpringTowardRamp`、:399-436）: 目標が target + slope·t で動く場合の解である。

- lag = c·slope/k だけ遅れて追従し、残りは自由振動になる。
- README の例: press で scale の目標を毎秒 −0.16 で下げ続け、quickChange スプリングで追う。

**UIKit レガシー**（`springFromUIKitDampingDuration`）: ω₀·duration を ζ ごとの実測表で補間する。ζ=1 で 9.2334 である。Astryx のコンポーネントでは使われない。

### 1.6 アニメーターの実行モデル（`motion/animator.ts`、Switch と Slider のみが使用）

- 駆動できるプロパティは `x`, `y`, `rotate`, `scaleX`, `scaleY`（`scale` は両方）, `opacity` である。
  - CSS の個別プロパティ `translate`, `rotate`, `scale`, `opacity` に書く。
  - 要素の `transform` には触らない（コンポーネント側のもの）。
- 曲線は Web Animations のキーフレームに焼き込む。
  - 30Hz のグリッド（`SAMPLE_GRID_MS = 1000/30`）で始め、線形補間の誤差が `SAMPLE_TOLERANCE` 以内になるまで分割する。tolerance は x, y 0.25px、rotate 0.05°、scale 0.0005、opacity 0.004。
  - キーフレーム数の上限は `MAX_KEYFRAMES = 1200`。再生は `easing: linear`。
  - 終了後 `END_HOLD_MS = 60000` は終値を保持する（ブラウザ固有の対策）。
  - `MAX_DURATION_MS = 30000` で収束しない曲線はスナップする。
- 位置と速度は DOM を読まず、解析曲線を時刻で評価する（`get(property, time)` / `getVelocity`）。
- Kivact で再現するときに意味のある要点は 3 つだけである。
  - (1) 曲線は解析解である。
  - (2) 値と速度をいつでも読める。
  - (3) 複数の曲線が 1 つのクロックを共有する（`{at}`）。
- コンポジタへの昇格、WebKit 対策、`presentation.ts` のヒットテスト補正はブラウザ固有なので、再現は不要である。

### 1.7 中断・リターゲット（速度引き継ぎ）

- `animate(to, {spring, velocity?})`
  - **常に現在の表示値と表示速度から開始する**。明示の velocity（指を離したときの速度、px/s）を渡すと、それが優先される。
  - したがってリターゲットは位置も速度も連続する（spec AST-094 FR6）。
- `handle.retarget(to)` は同じスプリングのまま目標だけを替える。
- `{additive: true}` は UIKit 方式の加算リターゲットである。旧曲線を走らせ続けたまま、「新旧目標の差」のスプリングを 0 から上乗せする。
- `stop(props)` は表示値で凍結する（指でつかんだとき）。
- `rebase({y: −shift})` はレイアウトずれ分を即座に移す。スプリングで動いている途中なら、スプリング・速度・目標を保持する。
- `set(values)` はドラッグ中の毎フレーム直書きである。
- `play(curves)` は任意の閉形式曲線（value(t), velocity(t)）を同じ仕組みで再生する。
- `decay(curves)` は x と y の慣性専用である（§1.8）。
- Reduce Motion のフェード中に割り込まれた場合は、凍結せずに完了させる（`isFade`、:480 付近）。

### 1.8 decay / coast / ラバーバンド / 投影 / ページング / スナップ

**減衰（decay、`motion/decay.ts`）**: UIScrollView の慣性。

- `scroll.decelerationRate.normal = 0.998`（/ms）、`fast = 0.99`。
- 連続形: λ = −1000·ln(rate) [/s]（normal は λ=2.002/s）。
  - v(t) = v0·e^(−λt)、x(t) = x0 + v0(1−e^(−λt))/λ
  - 静止点 = x0 + v0/λ。例: 2000px/s なら約 999px 進む。
- settleTime(threshold) = ln((|v0|/λ)/threshold)/λ。coast では threshold×2 を使う。

**coast（`motion/coast.ts`）**: プロパティごとに次のように場合分けする。

- (a) 範囲内で放した場合: decay で静止点まで進む。
- (b) decay がエッジに達する場合: 到達時刻の速度を `scroll.bounce`（settle と同じ臨界スプリング 0.5s）に渡し、エッジに収める。
- (c) エッジの外で放した場合: 放した速度のままスプリングでエッジへ戻る。`isTransition: true` なので、Reduce Motion ではフェードになる。
- (d) 速度が 0 の場合: その場で静止する。

**ラバーバンド（`motion/rubberBand.ts`）**

- b = (1 − 1/(x·c/d + 1))·d。c = 0.55、d = 軸方向のビューサイズ、x = エッジを越えた指の量（pan の 10pt ヒステリシス後）。
- エッジでの傾きは c で、指の 55% 速度で動く。b は d に漸近する。
- 逆関数は x = (d/c)·b/(d−b)。エッジの外で跳ね返り中の内容を指でつかんだとき、この逆関数で指位置を復元する。
- 傾き: c/u²、u = x·c/d+1。
- `rubberBandClamp(value, min, max, d)` は範囲内なら素通しし、外なら帯域処理する。
- Switch のドラッグでは c=0.55、d=12（28pt トラック基準。トラック高に比例）を使う。

**投影（`motion/projection.ts`、WWDC18）**

- projectDistance(v) = (v/1000)·r/(1−r)。r=0.998 では v×0.499 になり、1000px/s なら 499px 進む。
- `selectSnapPoint(points, {position, velocity, fromIndex?, maxSteps?})` は投影位置に最も近い点を選ぶ。同点の場合は進行方向側を選ぶ。

**スナップ方針（`motion/resolveSnapPoint.ts`）**: 実コンポーネントからは未使用。AST-107 の公開フック用である。

- slow release: 投影位置（`projection.seconds` 秒分。`minSpeed` 未満なら投影しない）に最も近い点。
- flick（|v| ≥ `flickThreshold`）: 位置より先にある次の点まで必ず進む（`tolerance` 内の点は通過済み扱い）。投影の方が遠ければ投影先。
- `start` 指定時: 開始点へ戻る向きに動いていれば、開始点に最も近い点へキャンセルする。
- `fromIndex` / `maxSteps` で移動数を制限する。
- AST-107 FR2: `useDragToSnapPoints` の flickThreshold 既定値は **1008 px/s**（UIKit sheet が settle→flick スプリングに切り替わる実測区間 (996.7, 1028.3]）。

**ページング（`motion/paging.ts`、`motion/uikit/paging.ts`）**: 実コンポーネントからは未使用。Carousel はネイティブの scroll-snap を使う。

- 1ms ステップの積分: v = (v + (target − x)·springPull)·friction; x += v
  - springPull = 0.000284858、friction = 0.970815、stepsPerSecond = 960（60Hz の 1 フレーム = 16 ステップ）
  - 終了条件は |to−x| < 0.5 かつ |v| < 0.05（pt/step）。終値は to にスナップする。静止状態から約 0.35s で収まる。
  - 初速は velocity/1000（pt/step）。
- release:
  - commitFraction 0.5
  - advanceSpeed 300 pt/s 以上ならページ送り（閾値で判定し、投影ではない）
  - velocityFloor 225 pt/s 未満は初速 0
  - maxPagesFromStart 1
- catchStopMs 85: 移動中に指が触れても、約 85ms 走ってから停止する。
- panAxisRatio 2: 水平から atan(1/2) = 26.6° 以内でのみ pan を開始する。
- `setContentOffset(animated)` は距離によらず 0.3s で、x = from + Δ·(1−cos πu)/2。

### 1.9 レイアウト（FLIP）アニメーション（`motion/animatedLayout.ts`、フック `hooks/useAnimatedLayout.ts`）

利用者は Collapsible、TreeList、Table 行展開、AppShell、SideNav（同じクロックを使う）である。**ハンドル（`animatedLayout` prop）がなければ何も動かさず、即時に切り替える**（AST-105 FR2）。

- **運動**: ロール `springs.settle`（臨界、ω₀=12.566 rad/s）を単位移動として解き、残り割合 remaining(t) = (1 + ω₀t)e^(−ω₀t) を使う。
  - duration = settleTime(0.001) = **734.77ms**。
  - 60 サンプル/秒（46 キーフレーム）、`easing: linear`。
  - テーマ（Neutral 含む）に関係なく固定。
- **測定（First）**
  - 変化の直前に `measureBefore()` を呼ぶ。React では `LayoutChangeSnapshot`（クラスの `getSnapshotBeforeUpdate`、`motion/LayoutChangeSnapshot.tsx`）で、DOM 変更前の矩形を読む。
  - 測る範囲は、変化した要素からコンテナまで各階層をさかのぼり、後続の兄弟（中央寄せ・末尾寄せ・`order` の行では先行の兄弟も）の矩形。
  - 可視領域の上下に 1 ビューポート分（または要素サイズ分）の帯を越えたら打ち切る。
- **Last / Invert / Play**
  - commit 後に、1 フレーム内の変化をまとめてマイクロタスクで 1 回だけ再測定する。
  - 移動量 dx, dy は inline-start 端で測る（RTL では right 端）。
  - 0.5px 未満（`MIN_SHIFT_PX`）は無視する。前後どちらでも可視部と交差しない要素は無視する。
  - absolute / fixed / top-layer / hidden / サイズ 0 / 固着した sticky / アプリ自身が動かしている要素はスキップする。
  - 親が動いている場合、子は自分の差分だけ動く（`dx -= moving ancestor's rawX`）。
  - 再生は `translate: Xpx Ypx` を offset → 0 へ remaining(t) で動かす（replace 合成）。**高さは決してアニメーションしない。スクロールもしない。**
- **スケール補正**
  - 祖先の scale を補正する。scaleOf = 描画幅 ÷ offsetWidth ÷ 自身の scale で求め、±0.01 以内は 1 とみなす。移動量 ÷ 祖先 scale にする（:897-917, :1110-1113）。
  - 要素自身の scale（press の 0.98 など）は translate に影響しないので補正しない。
  - 子要素の歪み補正（Framer Motion のような scale correction）は**行わない**。scale アニメーションを使わないからである。
- **中断**: 既にグライド中の要素は、残りの曲線 `carried(elapsed + t)` に新しい移動 `move·remaining(t)` を足して 1 本の新しいグライドにする。速度も連続する。duration = max(734.77, 旧の残り時間)（:1003-1016, :1110-1130）。
- **エッジ reveal（変化した要素自身の登場、`playLayoutChange` :1570-1616）**
  - キーフレームは {opacity: 1 − hidden, clipPath: inset(0 0 hidden% 0)}、hidden = (1 − from)·remaining(t)。
  - 既定のエッジは block-end（下端から現れる）。block-start と inline 端も指定でき、RTL では反転する。
  - 後続の要素がグライドを始めたら、クリップ量をそのグライドに毎フレーム追従させる（`rideEdge`）。端と後続が離れない。
- **退場（`holdLayoutExit` :1825-1925）**
  1. 要素は即座に空間を手放す（負の `margin-block-end` = −高さ。flex 行なら `margin-inline-end`）。
  2. inert かつ pointer-events none にする。
  3. {opacity: shown, clipPath: inset(0 0 (1−shown)% 0)} で 1→0。上へ畳むように隠れ、734.77ms、fill forwards。
  - 完了後に unmount する。
  - 測定されていない変化、画面外、Reduce Motion かつ fine pointer、margin で空けられない場合（grid track など）は、保持せず即時に消す。
  - `releaseLayoutExit` は退場を取り消し、表示割合を記録する。再オープンは途中から逆再生になる。
- **シェブロン**（`applyLayoutChangeTransition` :2784-2810）
  - CSS transition にインラインで `transition-duration: 735ms` と、同じスプリングから作った 25 点の `linear()` を入れる:
    `linear(0 0%, 0.058 4.167%, 0.18 8.333%, 0.321 12.5%, 0.455 16.667%, 0.573 20.833%, 0.671 25%, 0.75 29.167%, 0.812 33.333%, 0.86 37.5%, 0.897 41.667%, 0.924 45.833%, 0.944 50%, 0.96 54.167%, 0.971 58.333%, 0.979 62.5%, 0.985 66.667%, 0.989 70.833%, 0.992 75%, 0.994 79.167%, 0.996 83.333%, 0.997 87.5%, 0.998 91.667%, 0.999 95.833%, 1 100%)`
  - Reduce Motion では 0s。ハンドルがなければインラインを消し、コンポーネント自身の 175/125ms 遷移に戻す。
- **スワップ（`holdLayoutSwap` / `playLayoutSwap`）**: 旧内容を 1 つの transient copy として保持し、新内容を同じ運動で入れる（AppShell）。
- **2 状態エッジ（`holdLayoutEdge` / `playLayoutEdge`）**: 大きい側の状態を px の `clip-path inset()` で切り、グライドに乗せる（SideNav レール）。
- **Reduce Motion**（`layoutMotion` :153-170）
  - reduce かつ fine: 即時（グライドなし、reveal なし、退場保持なし）。
  - reduce かつ coarse: 動いた要素は opacity 0→1、退場は 1→0。200ms、`cubic-bezier(0.42,0,0.58,1)`（`CROSS_FADE_CAP_MS = 200`）。

### 1.10 presentation と transient copy

- **`presentation.ts`**: iOS WebKit がコンポジタで移動中の要素を開始位置でヒットテストしてしまう問題の補正である。**Kivact では不要**。Kivact は補間後の transform でヒットテストする（Kivact 調査）。
- **`transientCopy.ts`**: 退場・スワップ用に旧内容の「不活性なコピー」を作る。
  - コピーは inert かつ aria-hidden で、id / test id を持たず、フォームや radio group に参加せず、メディアを読み込まない。
  - 祖先の外に置く場合は、継承プロパティ（color, font-*, line-height, direction, writing-mode など 26 個）を明示的に書き込む。
  - 要素数が 1500（`MAX_COPIED_ELEMENTS`）を超える場合や、読み戻せない canvas がある場合はコピーしない（新内容だけが入る）。
  - 使用箇所は AppShell のメイン内容スワップと SideNav レールの折りたたみ。
- **アンカーレイヤー（Tooltip, Popover など）は transient copy を使わない**。退場アニメ自体がない（§4.3）。

### 1.11 Bézier ソルバ（`motion/bezier.ts`）

標準的な CSS 実装である。x を Newton 法（8 回、誤差 1e-7）で解き、失敗したら二分法に切り替える。Kivact の `Easing::CubicBezier`（30 回の二分法）でも実用上同等である。

---

## 2. Reduced motion

### 2.1 方針（`docs/design/reduced-motion.md`、承認済み 2026-10-24）

- **DR1**: `prefers-reduced-motion: reduce` のとき、登場・退場・表示切替・状態変化で、移動・拡縮・リサイズ・スプリングをしてはならない。
- **DR2**: 主ポインタが coarse の場合、またはタッチドラッグの release が起点の場合、**opacity のみのクロスフェードで 200ms 以下**にする。
  - 登場は最終位置に現れてフェードイン、退場はその場でフェードアウト。
  - 投げの慣性（decay、エッジでの収束）は直接操作扱いで、**継続する**（DEC-2）。
  - release 後のスナップ・dismiss・ページ送りはフェードにする。
  - その場で変わる値（progress fill など）は、どのポインタでも即時に適用する（DEC-3）。
- **DR3**: fine pointer では即時に完了する（アニメーションするプロパティを一切持たない）。
- **DR4**: コンポーネントの prop やテーマで逃げ道を作ってはならない。フェードを延ばす手段もない。
- 指によるドラッグ追従は直接操作なので、Reduce Motion 下でも追従する。
- 定数（`motion/uikit/timing.ts:92-117`、`motion/reducedMotion.stylex.ts`）は **200ms、`cubic-bezier(0.42, 0, 0.58, 1)`**。UIKit の実測は 0.5s だが、owner 判断で 200 に上限を設けた。
- メディアキー（`motion/reducedMotion.ts`）は次の 2 つである。素の `(prefers-reduced-motion: reduce)` も許容され、その場合は全ポインタで即時になる。
  - fade = `(prefers-reduced-motion: reduce) and (pointer: coarse)`
  - instant = `(prefers-reduced-motion: reduce) and (not (pointer: coarse))`

### 2.2 アニメーターの置換計画（`motion/reducedMotionPlan.ts`）

`animator.reducedMotion()` は `'off' | 'crossfade' | 'instant'` を返す。

- crossfade（coarse、または `touchRelease`）:
  - 登場（同じ呼び出しで opacity が 0 から上がる、または `crossfadeAs:'entrance'`）: 移動を即座に着地させ、0→200ms でフェードイン。
  - 退場（opacity → 0、または `crossfadeAs:'exit'`）: その場で 200ms フェードアウトし、透明になってから移動を着地させる。
  - opacity だけの変化: 200ms でその値へフェード。
  - その他の移動は **dip**: 0→100ms で opacity → 0、100ms で着地、100→200ms で元の opacity に戻す。スタイルシートの opacity が 0 なら即時。
- instant: すべて即時。
- 押下の lift のように、フェード手段を持たないフィードバックは `'off'` 以外ではスキップする。

### 2.3 コンポーネント別の Reduce Motion 挙動（実装どおり。方針からの逸脱も含む）

| コンポーネント / 機構 | Reduce Motion 時の実挙動 |
|---|---|
| Button 状態遷移・press scale 0.98 | duration 0s（scale は即時に適用される） |
| Button ローディング | 遅延 0（即時入れ替え）。spinner の 175/125ms フェードは**残る** |
| press オーバーレイ release（全 pressable） | **残る**。min(200ms, 200ms×tempo)、Neutral では 200ms |
| layerAnimations（Tooltip/Popover/HoverCard/Menu/Selector 系） | fine: なし（即時）。coarse: opacity だけ min(fast-max, 200ms) = **165ms**、カーブは `--ease-standard` のまま（方針の easeInOut ではない） |
| Dialog / AlertDialog / CommandPalette | fine: なし。coarse: opacity のみ 200ms easeInOut（方針どおり） |
| Drawer（パネル・スクリム） | 0.01s（coarse でもフェードしない。逸脱） |
| BottomSheet 登場 | fine: 0.01s。coarse: transform 0.01s + opacity 200ms（`--ease-standard`） |
| BottomSheet 退場 | 移動なし。coarse: opacity 200ms（`--ease-exit`）。タッチスワイプ dismiss はどのポインタでも opacity 200ms |
| BottomSheet ドラッグ | 指追従は継続。settle は 0.01s。haptic 停止 |
| BottomSheet スタック後退・揃え | 0.01s、揃えのオフセットは 0（その場でフェード） |
| MobileNav | 0.01s。close 遅延 0 |
| Toast（カード・行） | 0.01ms。スワイプ中の追従は継続 |
| Item swipe | settle 0（即時）。ドラッグ追従は継続 |
| Switch（fine） | トラック・つまみの遷移 0s |
| Switch（touch） | lift なし。travel と色はアニメーターのクロスフェード 200ms dip。ドラッグ追従は継続 |
| Slider | lift なし。トラックのタップは即時。指を離した coast は**継続**（慣性扱い） |
| ProgressBar 確定 | 0s（即時。DEC-3） |
| ProgressBar 不確定 | 幅 100% で opacity 1→0.84 を各 2000ms、easeInOut、alternate、無限ループ |
| Spinner | **止めずに 1 周 3s に減速** |
| Skeleton | アニメーションなし。opacity 0.25 で静止 |
| StatusDot パルス / Chat "sending" パルス | なし（静止） |
| useEntryAnimation（FieldStatus） | なし（coarse でもフェードしない。逸脱） |
| animatedLayout（Collapsible/TreeList/Table 行展開/AppShell/SideNav レール） | fine: 即時。coarse: 200ms 不透明度フェード |
| シェブロン（ハンドルあり） | 0s |
| シェブロン（ハンドルなし Collapsible/TreeList/Table、TabMenu、TopNav、Selector、MultiSelector） | **回転する（逸脱）** |
| シェブロン（ComplexSelector, SideNavCollapseButton, SideNavItem, Banner, ChatToolCalls） | 0s（即時） |
| シェブロン（CodeBlock） | **回転する（逸脱）** |
| TopNavMegaMenu | fine: 0s。coarse: opacity のみ 200ms easeInOut |
| SideNavItem グループ展開 | 0s |
| TopNav モバイル展開 | **アニメーションする（逸脱）** |
| Stepper 接続線 | 0s |
| Carousel スクロール | `auto`（即時） |
| useStreamingText / Markdown フェード | 全文を即時に表示 / 0s |
| useChatStreamScroll | 1 フレームでジャンプ |
| Wheel / MonthScroller の自前スクロール | coarse: opacity 1→0→1 の 200ms dip（100ms で着地）。fine: 即時 |
| TouchDateField パネル切替 | 0.01s（フェードしない。逸脱） |
| Overlay | 0s |
| Lightbox ズーム | 0ms |
| Field 入力枠・FileInput・Checkbox・Radio・Slider つまみ色・Calendar セル | 0s |
| SegmentedControl, Tab, Token, Citation, ClickableCard, SelectableCard, Thumbnail, Link, Item ホバー, Table ホバー, Resizable, Pagination, Outline, ChatComposer(Drawer), Carousel ナビピル, CodeBlock 折りたたみ | **Reduce Motion の扱いなし（そのままアニメーションする）** |

kivact-ui-soft は**この実挙動を再現する**こと。逸脱も含めて Astryx と一致させる。方針どおりに「修正」したい場合は別の決定事項として扱い、勝手に変えてはならない。

---

## 3. ジェスチャー

### 3.1 UIKit 定数（`motion/uikit/gestures.ts`、`motion/uikit/longPress.ts`）

| グループ | 値 |
|---|---|
| TAP | allowableMovement 45px（ユークリッド距離）、maximumIntervalBetweenTaps 350ms、allowableSeparation 45px、maximumTapDuration 1500ms |
| LONG_PRESS | duration **500ms**、allowableMovement **10px**（距離） |
| PAN | hysteresis **10px（軸ごと）**: \|dx\| または \|dy\| が 10 以上で開始 |
| SWIPE | minimumDistance 50px、maximumDuration 500ms、maximumSecondaryMovement 50px |
| PINCH | hysteresis 8px（スパン変化） |
| ROTATION | 5°（GUESS） |
| SCREEN_EDGE_PAN | edgeRegion 40.2px（402pt 画面の 10%）、hysteresis 15px（エッジゾーン外では 10px） |
| CONTROL | trackingExpansion 70px（押したまま外に出ても 70px までは inside） |
| PRESS | contentTouchDelay 150ms、quickTapFlash 100ms、slop 10px、scrollBrake 150ms、momentumMemory 3000ms（GUESS） |
| VELOCITY | currentWeight 0.2、scrollReleaseWeight 0.25（GUESS）、stillness 40ms（GUESS） |
| HAPTIC（`navigator.vibrate` 用） | selection [5]、impactLight [8]、impactMedium [12]、impactHeavy [18]、impactSoft [10]、impactRigid [6]、notificationSuccess [8,60,12]、notificationWarning [12,80,12]、notificationError [12,50,12,50,18] |

### 3.2 速度推定（`gestures/VelocityTracker.ts`）

- 最小二乗法でも時間窓でもない。**UIKit 実測の 2 区間 IIR ブレンド**である。
  - 「動いた」サンプルを最大 3 個保持する。touch-down が最初のサンプルになる。
  - 区間速度 = Δpos/Δt × 1000（px/s）。Δt ≤ 0 なら 0。
  - サンプルが 2 個なら区間速度そのもの。3 個なら **0.2·v(最新区間) + 0.8·v(前区間)**。
  - scroll 用（ページング、ホイール）は 0.25·pan最新 + 0.75·pan前。
  - 同位置のサンプルは無視する（pinch を除く）。過去の時刻は無視する。同時刻は置き換える。
- release 規則: 速度は 0 とする、ただし次の 2 条件を両方満たす場合を除く。
  - 移動区間が 2 以上ある。
  - release 時刻 − 最終移動時刻 ≤ **40ms**。
  - これは UIKit からの唯一の逸脱である。UIKit は静止後も flick する。
- lift 位置はサンプルしない。pinch は同じトラッカーを scale 値に使う（scale/s）。
- **実コンポーネントは独自の推定を使う**（§3.6）。BottomSheet は 100ms 窓、Toast はジェスチャー全体の平均、Item は最後の 2 サンプル。

### 3.3 認識器（`gestures/*Recognizer.ts`、`GestureArena.ts`）

- Tap: タップ開始時に maximumTapDuration タイマー。移動 > 45px、指の追加、2 打目が 45px 超離れた場合は失敗。複数タップでは打間 350ms タイマー。lift 時に判定する。
- LongPress: touch-down + 500ms（イベントクロック）で開始。開始前に距離 10px を超えるか、lift したら失敗。開始後の移動は無制限。duration 0 なら touch-down 時に開始（メニュー選択トラッカー用）。
- Pan:
  - max(\|dx\|,\|dy\|) ≥ 10 で分類する。\|along\| > axisRatio·\|across\| なら開始、逆なら失敗、どちらでもなければ待機。待機はヒステリシスに達した時点で失敗する（ネイティブスクロールとの競合を 10px 以内に決着させる）。
  - `startFromHere` はしきい値分の移動を飲み込む（例: (10.67,10.67) を越えたとき (0.67,0.67) と読む）。
  - 指の増減では重心を再基準化し、ジャンプさせない。
- Swipe: 移動中に、許可方向へ 50px、500ms 以内、横ずれ 50px 以下で認識する（離散）。
- Pinch: 最初の 2 指。\|span − initial\| ≥ 8 で開始し、initialSpan += sign·8 を基準に scale = span/initialSpan。
- ScreenEdgePan: タッチ専用で 1 指。開始 x ≤ 40.2（または ≥ width − 40.2）。
- アリーナの優先順位:
  - 最も内側の認識器を優先し、同じイベントで認識したら内側が勝つ。
  - 最初に認識した認識器が、タッチを共有する他の possible 状態の認識器を失敗させる。
  - `requireToFail` は依存先が失敗するまで待つ。
  - UIControl 規則: 祖先の 1 本指シングルタップは、コントロール自身のアクションに勝たない。

### 3.4 押下フィードバック（`hooks/usePressFeedback.ts` → `utils/pressFeedback.ts`、`utils/pressGesture.ts`、`utils/interactionOverlay.stylex.ts`）

- **fine pointer**:
  - `:hover`（`@media (hover:hover)` 下のみ）は `--color-overlay-hover`、`:active` は `--color-overlay-pressed` を塗る。
  - フェードするかどうかは各コンポーネントの transition 次第である。グラデーション背景（`background-image`）は補間されないので、実際には即時切替になる。
- **coarse pointer**（文書全体のコントローラが `data-astryx-press` を書く。`:active` は無効化される）:
  - onset `PRESS_ONSET_MS = 150`: スクローラ内のタッチは 150ms 後に塗る。スクローラ外は同フレーム。
  - slop `PRESS_SLOP_PX = 10`: onset 前に 10px 動いたら押下を破棄し、一度も塗らない。
  - quick tap: onset 前に指を離すと、全強度で `PRESS_FLASH_MS = 100` 表示してから release する。
  - scroll brake 150ms: 動いたスクローラへのタッチは、押下ではなくブレーキ扱い。
  - `"on"`: 強度（登録済み `@property --astryx-press-alpha`、`<number>`）を最初のフレームで即 1 にする。**フェードインしない**。
  - `"fading"`: キーフレーム `pressRelease` で強度 1→0。duration `calc(200ms × min(tempo,1.5))`（Neutral では 200ms）、easing **`cubic-bezier(0, 0, 0.2, 1)`**（`RELEASE_EASE`。中間点で強度約 0.16〜0.2）、fill both（:139-208）。
  - キャンセル時は transition なしで消す（内容が動く 1 フレーム前）。
  - 新しい押下は強度 1 から再開する。
- **press scale**（フックではなく各コンポーネントが持つ）:
  - 対象: Button（ButtonGroup 内を除く）、Selector / MultiSelector / ComplexSelector のトリガー。
  - fine `:active` または `[data-astryx-press="on"]` で `scale(0.98)`。coarse の `:active` では `scale(1)`。
  - 遷移は `--duration-fast`（125ms）、`--ease-standard`。押し込みも戻りも同じで、Reduce Motion では 0s（`Button/Button.tsx:121-149`）。
- 利用者:
  - Button
  - SegmentedControlItem、Tab
  - Switch、CheckboxInput、RadioListItem
  - Token、AvatarGroupOverflow
  - NumberInput、TouchDateField
  - Thumbnail、ClickableCard、SelectableCard
  - Link、Item、TreeListItem
  - Collapsible、Outline、Stepper/Step、Calendar
  - CommandPaletteItem、DropdownMenuItem
  - ComplexSelector、MultiSelector、Selector
  - SideNavHeading、SideNavItem
  - TopNavItem、TopNavMegaMenu、TopNavMegaMenuItem、TopNavMenu
- PressTracker のプロファイル（`gestures/PressTracker.ts:149-170`）:
  - control = {highlightDelay 'inScrollers', quickTapFlash 100, releaseMs 200}
  - cell = {…, cancelTravel 10, quickTapFlash 0, releaseMs 200}

### 3.5 長押し・メニュー押下

- `hooks/useLongPress.ts`: setTimeout 方式、`delayMs` 500、`moveCancelPx` 10。判定は**軸ごと**（\|dx\|>10 \|\| \|dy\|>10）で、認識器のユークリッド距離とは異なる。指が 1 本以外になったら解除する。ContextMenu が使う。
- `hooks/useMenuPress.ts` / `menuPressGesture.ts`:
  - `MENU_PRESS_SETTLE_MS = 300`: 開いた押下の release は、メニュー内に入ったか 300ms 経過後にだけ作用する。
  - 長押し 500ms、トリガーのドリフト許容 13px（ContextMenu は 10）、迷子クリックのガード 400ms。
  - 端の自動スクロール: ゾーン 24px、16ms ごとに 6px（約 375px/s）。
  - マウスは押した瞬間に開く。指は 500ms 長押しが必要。

### 3.6 コンポーネント固有のドラッグ / スワイプ / flick 規則

**BottomSheet**（`BottomSheet/useSheetGestures.ts`、`snapOffsets.ts`）

- オフセットは 0 が全開で、大きいほど下（閉じる方向）。
- detent:
  - snapPoints（viewport 比、`n%`、`npx`）から offsets = [0, sheetHeight − h…] を作る。48px 以内（`DETENT_DEDUP_PX`）は重複として除く。
  - peek は見える高さが全体の 25% 以下の最短停止点。
- dismiss ライン = maxOffset + 0.4 × 最短 detent の可視高（`DISMISS_OVERSHOOT_RATIO`）。
- 開始:
  - ハンドルは即開始（slop なし）。
  - 本体は 8px（`DRAG_PROMOTION_SLOP`）で昇格する。下向きは下のスクローラがすべて先頭にあるとき、上向きはすべて末尾かつ折りたたみ中のとき。
  - 2 本目の指で終了（pinch はブラウザに任せる）。
- 移動中:
  - raw < 0 なら max(−48, raw×0.35)（ラバーバンド的抵抗）。
  - 最深 detent より下ならそのまま。
  - detent 間は、40px 以内で value += (target−value)·(1 − t²)、t = 距離/40（磁石）。
  - transform を直書きし、transition は none。
- 速度: 100ms の窓で (y_now − y_oldest)/min(dt,100)（px/ms）。最後の移動から 100ms 以上たって離したら 0。
- release:
  - projected = offset + v × 499（= 0.998/(1−0.998)）。移動量が 48px 以下なら投影しない。
  - 下向きで projected > dismiss ラインなら dismiss（purpose 'info' のみ）。それ以外は最深 detent へ。
  - それ以外は、ドラッグ方向側の detent の中から clamp(projected) に最も近いものを選ぶ。
- settle は **CSS transition `--duration-medium`（300ms）、`--ease-standard`**。スプリングも速度引き継ぎもない。
- 移動中に指が触れると、settle 目標へジャンプする（表示位置でつかまない）[推論]。
- haptic: detent 変化で `navigator.vibrate(8)`。Reduce Motion ではスキップ。

**Toast**（`Toast/useToastGesture.ts:16-23`、タッチ / ペンのみ）

- 縦の意図が 8px を超えたら追従を始める（inline の transition-duration を 0s にする）。縦の意図比は 1.2。
- 進捗 p = 移動 / 高さに対して、opacity 1 − 0.4p、scale 1 − 0.02p。
- dismiss 条件: 移動 ≥ max(0.4×高さ, 48px)、または「移動 ≥ 48 かつ平均速度（総移動 / 開始からの経過 ms）> 1.2 px/ms」。
- dismiss 時は translateY(±120%) かつ opacity 0 を `--duration-fast` で。それ以外は inline を消し、`--duration-fast` / `--ease-standard` で戻す。
- auto hide の既定は 5000ms。

**Item swipe**（`Item/useSwipeAction.ts:70-92`）

- 軸ロック 10px。
- commit: 移動 ≥ max(パネル幅 + 48, 0.5×行幅)、または fling（最後の 2 サンプルの速度 ≥ 0.6 px/ms、かつ移動 ≥ 44px）。
- reveal モードはパネル幅の 0.5 以上で開いたまま止まる。commit 点を越えた移動には抵抗 0.35。
- スライドアウトは行幅 + 24px まで。
- settle は `SWIPE_SETTLE_MS = 200`、`ease-out`（CSS キーワード = (0,0,0.58,1)）。クリック抑止 400ms。

**Drawer**: ドラッグ / スワイプなし。

**Carousel**: 独自物理なし。ネイティブの `scroll-snap-type: x mandatory`（`hasSnap` 時）と `scrollBy({behavior:'smooth'})`。

**Switch ドラッグ**（タッチ）:

- つまみは指を tracking スプリング（0.36/1）で追う。両端はラバーバンド（c=0.55、d=12×トラック高/28）。
- 指が着地点から flipDistance（トラック幅 − つまみ幅。iOS では 26）進むと反転する。

**Slider**（タッチ / ペン）:

- 値 = (max−min)×移動 / トラック長（しきい値 10pt 以降から数える）。
- release coast: 目標 = lift 値 + 0.4×投影距離（rate 0.996）。sliderRelease スプリングで、指を離した速度から動かす。

**MonthScroller**（`DateInput/useOwnScrollGesture.ts`）: SWIPE_DISTANCE 32px、AXIS_LOCK_SLOP 6px、VERTICAL_DOMINANCE 2。

**Wheel マウスドラッグ**: DRAG_SLOP 4px、settle フォールバック 260ms。

**Lightbox pan**: 拡大中のみ。境界・ラバーバンド・慣性はなし。キーボードでは 50px ずつ。

### 3.7 haptics（`gestures/haptics.ts`）

- `playHaptic(kind)` は `navigator.vibrate` を呼ぶ（Android のみ）。
- iOS 用は `attachTapHaptic`（隠し switch input）で、本物のタップのときだけ鳴る。
- Reduce Motion とは無関係（BottomSheet だけが例外的に止める）。
- 実際に鳴らすのは Slider の両端（impactMedium、入るたびに 1 回）と BottomSheet の detent（vibrate(8)）。Switch は鳴らさない。

---

## 4. コンポーネント別アニメーションカタログ

この章の約束事:

- 時間は「**Neutral**（既定）」の順に書く。例: `--duration-fast` = **125ms**（175ms）。
- 「std」は `--ease-standard` = `cubic-bezier(0.24, 1, 0.4, 1)` を指す。
- CSS transition の中断は、ブラウザ標準どおり**現在値から新目標へ再開**する。正確な逆転では、CSS の reversing shortening で残り時間分に短縮される。
- 「RM」は Reduce Motion の略。

### 4.1 Button（`Button/Button.tsx`）

- **状態遷移**（:121-127）
  - `transition-property: background-image, background-color, color, opacity, transform`、`--duration-fast` 125ms（175）、std。RM では 0s。
  - 対象は、disabled 時の opacity 1→0.5（:152）、variant の色、press の scale。
  - hover / press のオーバーレイは `background-image` のグラデーションなので、補間されず即時に切り替わる [推論]。
  - 立ち上げも戻りも同じ時間・カーブ。
- **press scale**（:129-149。ButtonGroup 内では無効、:750）
  - fine `:active` で `scale(1)→scale(0.98)`。coarse では `:active` は scale(1) のまま。
  - `[data-astryx-press="on"]` で 0.98。`"fading"` になると 1 に戻る。
  - 125ms、std、RM では即時。disabled では transform none。
- **touch press オーバーレイ**: §3.4（onset 150ms / flash 100ms / release 200ms (0,0,0.2,1)）。
- **ローディング**（:406-457, :786-805）
  - 明示的な `isLoading`: 内容色を transparent にし、spinner を即時に表示する。アニメーションなし。
  - 遅延経路（`clickAction` の pending、または `isInterruptible`）:
    - spinner オーバーレイはキーフレーム `spinnerReveal` で opacity 0→1。`--duration-fast` 125ms、timing function 未指定なので CSS `ease`、fill backwards、**delay `--duration-medium-min` = 225ms（310）**。
    - 内容は `contentHide`（color → transparent、1ms、fill forwards、同じ delay）。
    - RM では delay 0。フェード自体は残る。
  - 終了は即時に unmount する（退場アニメなし）。
- tooltip は Tooltip（§4.29）。

### 4.2 IconButton（`IconButton/IconButton.tsx:52`）

`<Button isIconOnly>` そのもの。固有のモーションはない。

### 4.3 ToggleButton / ToggleButtonGroup（`ToggleButton/ToggleButton.tsx:44-66`）

- Button の `isInterruptible` 版なので、ローディング spinner は常に遅延経路を通る。
- pressed 状態の背景 `--color-overlay-pressed` は、Button の background-color 遷移（125ms、std、RM 0s）でフェードする。
- ラベルの semibold 化は即時（隠しサイザーで幅を確保している）。
- Group 固有のモーションはない。

### 4.4 ButtonGroup

固有のモーションはない。グループ内の Button は press scale を失い、色の遷移だけが残る。

### 4.5 SegmentedControl / SegmentedControlItem（`SegmentedControl/SegmentedControlItem.tsx:98-122`）

- **スライドする選択インジケーターはない**。各セグメントが自分の色を変えるだけである。
- `transition-property: color, background-color, box-shadow`、`--duration-fast` 125ms、std。**RM の扱いなし**。
- 選択されたセグメントは color → `--color-text-primary`、bg → `--color-background-surface`、box-shadow → `--shadow-low` へフェードする。旧選択は逆向きにフェードする。font-weight は即時。
- 非選択セグメントの hover / `:active` は `interactionOverlayStyles.backgroundColor` を使い、125ms でフェードする。タッチは §3.4。
- `springs.segmentedThumb`（0.3/0.75、GUESS）は**使われていない**。

### 4.6 TabList / Tab / TabMenu（`TabList/*`）

- Tab ラベル: `color` を 125ms、std で遷移（secondary → primary）。semibold は即時（Tab.tsx:133-135）。
- Tab の hover 層: background-color transparent → `--color-overlay-hover`（`:active` 時は pressed）、125ms、std（:137-180）。
- **インジケーター**: タブごとに 2px のバーを持ち、**スライドしない**。
  - `opacity, background-color` を 125ms、std で遷移（:184-209）。
  - 選択されたタブは opacity 1・`--color-accent`、非選択は opacity 0・transparent。新旧がその場でクロスフェードする。
- RM の扱いなし。
- TabList のスクロール帯:
  - `scroll-behavior: smooth`（UA 依存。RM では `auto`、:274-277）。
  - 端のフェードは `mask-image` を `--duration-medium` 300ms で遷移する（RM 0ms）。ただし none ↔ gradient は離散的なので、実質即時 [推論]（:288-294）。
  - 選択タブを可視範囲に入れるときは `scrollBy` を `'instant'` で行う（:496-499）。
- TabMenu:
  - トリガー色は 125ms。
  - シェブロンは open で `rotate(180deg)`、125ms、std、**RM の扱いなし**（:167-176）。
  - インジケーターは選択時にだけ mount されるので、遷移なしで出る。
  - ポップオーバーは layerAnimations を持たない（登場アニメなし）。

### 4.7 Switch（`Switch/Switch.tsx`、`useSwitchTouch.ts`、`switchTouch.ts`、`motion/uikit/switch.ts`）

**fine pointer（CSS）**

- トラック: `background-color` を `--duration-fast` 125ms、std で遷移。RM とタッチでは 0s（:334-346）。
  - off は `--color-background-gray`、on は `--color-accent`。
  - hover は `--color-tint-hover` を color-mix（off 5%、on 15%、:476-505）。
- つまみ: `transform, width, height` を 125ms、std で遷移。RM 0s（:542-554）。**動きながら大きくなる**。
  - md: off 16×16・translateX(0) → on 20×20・translateX(14px)。RTL は −14px。
  - sm: off 14×14 → on 16×16・translateX(12px)。
- `:active` はグラデーションで即時。disabled は opacity 0.5。

**coarse pointer（アニメーター駆動。CSS 遷移は off）**

- 静止時のつまみはインセットで配置する（md off: 16px・inset 4、md on: 20px・inset 2）。
- **値の変化**: つまみを `switchControl.travel`（0.3/1、臨界）で、**現在の速度から**動かす（`showSide`、useSwitchTouch.ts:890-911）。
- **トラック色**: トラック `::after` のフェード層（will-change opacity）の opacity を動かす（`animateFade` :808-824）。
  - ON は `wellOn`（臨界、response 0.68s）、OFF は `wellOff`（臨界、0.40s）。**ON はゆっくり、OFF は速い**。
  - 値の確定時に `swapFade` で opacity を 1 − 2·opacity に rebase し、ジャンプしないようにする（:831-843）。
- **lift**（touch-down、`lift()` :928-948）
  - つまみの scaleX/Y を liftScale へ、`switchControl.lift`（0.23/0.55、アンダーシュートあり）で動かす。
  - liftScale = min(58/37, 38.333/24, 1 + inset/thumbW, 1 + inset/thumbH)。md off は **1.25**、md on は **1.10**、sm off は 1.214、sm on は 1.125（switchTouch.ts:67-75）。
  - 影の層（つまみ `::after`、`--shadow-low`）の opacity 0→1 も同じスプリングで動かす。
- **release**（`settle()` :952-991）
  - 1 本の合成曲線で動かす。hold = max(release 時刻, lift 開始 + 235ms) までは lift スプリングを続ける。
  - その後 `switchControl.settle`（0.5/0.7）で scale を 1 に、影を 0 に戻す。収束の閾値は scale 1e-4、opacity 1e-3。
  - **素早いタップでも、最低 235ms は持ち上がったレンズが見える**。
- **ドラッグ**
  - つまみは指を `tracking`（0.36/1）で追う（:680）。
  - 両端のラバーバンドは c 0.55、d = 12×trackH/28 で、上限は inset − liftGrowth。
  - flipDistance = trackW − thumbW。
  - ドラッグ中に React state は描画しない。
- **press の塗り**: トラック・つまみの `::before` は `"on"` で opacity 1。`"fading"` では `pressLayerRelease` で 1→0、200ms、(0,0,0.2,1)、fill both（:209-213, :383-450, :559-622）。
- **RM（タッチ）**
  - lift なし。
  - travel と色はアニメーターの 200ms クロスフェード。つまみは旧位置で dip し、静止位置に再び現れる。
  - ドラッグ中の直接追従は残る。
- busy 時はつまみ内に Spinner を出す（:1056）。
- 中断: スプリングは現在の値と速度からリターゲットし、rebase も速度を保つ。フェード中に値が変わったら、着地まで基底の塗りをインラインで保持する（`holdBase`）。

### 4.8 CheckboxInput / CheckboxList / CheckboxIndicator

- 箱（`Indicator/CheckboxIndicator.tsx:35-74`）: `background-color, border-color` を 125ms、std で遷移。RM 0s。
  - 未選択 → 選択: bg `--color-background-surface` → `--color-accent`、border `--color-border-emphasized` → `--color-accent`。
  - hover は color-mix（未選択 border 20%・bg 5%、選択 15%）。
- **チェックマーク・不確定バーは `display none ↔ block` で即時。描画アニメーション（stroke draw）も scale もない。**
- press 層（ラッパーの `::after`、CheckboxInput.tsx:95-125）: マウスは即時、タッチは 200ms release。
- disabled は opacity 0.5。busy は Spinner（:534）。
- CheckboxList 固有のモーションはない。

### 4.9 RadioList / RadioListItem / RadioIndicator

- 円（`Indicator/RadioIndicator.tsx:39-44`）: `background-color, border-color` を 125ms、std。RM 0s。色の組み合わせは Checkbox と同じ。
- 中心の点（sm 8px）は条件付き描画で即時。
- 行（Item）の hover 背景は抑止される。press は円の `::after` に出る（マウスは即時、タッチは 200ms、RadioListItem.tsx:67-90, :140-169）。
- CheckIndicator にアニメーションはない。

### 4.10 Slider（`Slider/Slider.tsx`、`sliderTouchController.ts`、`motion/uikit/slider.ts`）

- つまみ（20px）: `background-color, box-shadow` を 125ms、std。RM 0s（Slider.tsx:412-417）。hover は accent に tint-hover 15%（:462-468）。
- ドラッグ中の pressed はグラデーションで即時（:476-478）。
- **マウス、キーボード、controlled prop による値変化では、つまみもフィルも遷移せず、即座に動く。**
- タッチ / ペン:
  - **lift**: touch highlight 時に scale 1 → **1.55**（20→31px）を `springs.sliderLift`（0.215/0.6、約 9.5% オーバーシュート）で（controller :1119-1135）。マウスでは lift しない（:513）。
  - **drop**: scale → 1 を `springs.sliderDrop`（0.5/0.7）で（:1138-1150）。
  - **release coast**（:180-217, :994-998）
    - 目標 = lift 値 + 0.4 × projectDistance(v, 0.996) を範囲内にクランプした値。
    - `springs.sliderRelease`（0.4/1）を、指を離した速度から動かす。
    - duration = clamp(settleTime(0.05), 1/120s, 3s)。
    - `animator.decay(…, {touchRelease})` で、つまみとフィルを同じクロックで動かす。
    - onChange は 3 フレーム後（`HAND_OFF_FRAMES`）。0.5px 未満の coast は省略。
  - トラックをタップした場合: 静止状態から 0.4/1 スプリングで `play()`（:796-833）。
  - 中断: coast 中に触れたら、その場で停止する（`begin()` :597-602）。
  - 両端の haptic は impactMedium。
- RM: lift なし。タップは即時（クロスフェードなし）。指を離した coast は慣性扱いで継続する。

### 4.11 ProgressBar（`ProgressBar/ProgressBar.tsx`、`ProgressBarTouch.ts`）

- **確定値（determinate）**
  - 全幅バーを `translateX(calc(fill% − 100%))` で位置決めする（RTL は反転、:300-321）。will-change transform。
  - fine: `transform` を `--duration-medium` **300ms**（410）、std で遷移（:326-353）。
  - coarse（UIProgressView モデル、ProgressBarTouch.ts:55-106）
    - カーブは linear（`cubic-bezier(0,0,1,1)`）。duration = \|Δ%\|/100 × 1000ms × tempo（0→100 で 1000ms、20→80 で 600ms）。
    - 最小描画幅 16px。0% では fill の opacity 0（即時）。
    - CSS の逆転短縮を防ぐため、`--_progress-bar-flip` で 0/1/2 × 0.01px ずらす。
  - RM: どちらのポインタでも 0s。
  - forced-colors では width を遷移させる（coarse では 0s）。
- **不確定値（indeterminate）**（:156-187, :369-403）
  - 幅 40% のバー。キーフレーム `indeterminateSlide` で translateX(−100%) → translateX(250%)（RTL は 100% → −250%）。
  - **1.5s、ease-in-out**（CSS キーワード (0.42,0,0.58,1)）、infinite、normal（往復しない）。
  - RM: 幅 100%、`indeterminatePulse` で opacity 1 → 0.84、各 2000ms、(0.42,0,0.58,1)、alternate、infinite。tempo は無関係。
- マークの tooltip は Tooltip。

### 4.12 Spinner（`Spinner/Spinner.tsx`）

- 静止したリング（−90° 回転済み）上の弧を動かす。弧長は円周の 0.375（`--spinner-arc-fraction`）。
- キーフレーム `dashMotion`: `stroke-dashoffset` 0 → −(直径×π)、1 周で円周 1 つ分（:228-231）。**見た目は弧の等速回転と同じ**。
- duration は `var(--_astryx-pre-tempo-duration-slow-min, --duration-slow-min)` = **525ms**（730）。linear、infinite（:342-353）。tempo は無関係。
- **RM: 1 周 3s に減速する（止めない）**。
- **位相同期**: バッチ化した rAF で、全リングのアニメーションの `startTime = 0`（document timeline の原点）にする。全スピナーが同位相で回り、新しく mount したスピナーは周期の途中から始まる（:155-211）。
- サイズ sm / md / lg / xl: 直径 10 / 14 / 18 / 28px、線幅 2 / 3 / 3 / 4px。
- 自身の登場・退場フェードはない（Button がフェードを付ける）。

### 4.13 Skeleton（`Skeleton/Skeleton.tsx`）

- 静止時の opacity は 0.25（forced-colors では 1）。
- キーフレーム `skeletonFade`（:51-72）: 10 段の**保持付きステップ**。値は 0.25, 0.325, 0.40, 0.475, 0.55, 0.625, 0.70, 0.775, 0.85, 0.925。各段を n% から (n+9.99)% まで保持し、0.01% で次へ移る。linear、**alternate**、infinite。
- duration は `var(--_astryx-pre-tempo-duration-medium-max, --duration-medium-max)` = **400ms**（550）。tempo は無関係（:102）。
- **delay = 1000ms + 100ms × index**（`DELAY_TIME`、`STAGGER_TIME`、:30, :36, :141-143）。fill-mode なし（静止値 = 0 フレーム目の値なのでジャンプしない）。
- **shimmer（グラデーションの移動）ではない**。段階的な明滅である。
- RM: アニメーションなし。0.25 で静止。

### 4.14 StatusDot（`StatusDot/StatusDot.tsx:32-64`）

- `isPulsing` のとき、キーフレーム 0% opacity 1、50% 0.5、100% 1。**2s、ease-in-out**、infinite。
- RM: なし。それ以外は静止。

### 4.15 Badge / Card / Avatar

アニメーションはない。Avatar のステータス点の translate は静的配置（Avatar.tsx:262-276）。Avatar の tooltip は Tooltip。

### 4.16 Banner（`Banner/Banner.tsx`）

- 展開シェブロン: `transform` を 125ms、std。RM 0s。展開で rotate(180deg)（:358-368）。
- 内容の表示切替は条件付き描画で即時（:639）。**高さアニメーションはない**。
- dismiss は即時に除去。登場アニメもない。
- `springs.banner`（GUESS）は未使用。

### 4.17 Toast（`Toast/Toast.tsx`、`ToastViewport.tsx`、`useToastGesture.ts`）

- **カード**（Toast.tsx:74-124）
  - `opacity: var(--_toast-swipe-opacity, 1)`、`transform: translateY(var(--_toast-swipe-y, 0)) scale(var(--_toast-swipe-scale, 1))`。
  - `transition: opacity, transform` を `--duration-fast` **125ms**（175）、std。RM は 0.01ms（transitionend は発火する）。
  - **登場**（`@starting-style`）: opacity 0 → 1、translateY(slide-y) → 0。slide-y は下スタックで +0.5rem（8px）、上スタックで −0.5rem（ToastViewport.tsx:60, :143-148）。
  - **退場**（:379）: opacity → 0、translateY(swipe-exit-y ?? swipe-y ?? slide-y)。カーブは登場と同じ std（`--ease-exit` ではない）。
- **行ラッパー**（ToastViewport.tsx:128-170）
  - `grid-template-rows` と padding を 125ms、std で遷移。RM 0.01ms。
  - 登場は `@starting-style` 0fr・padding-block-end 0 から、1fr・0.5rem の gap へ。
  - 退場は 0fr・padding 0 へ。**カードのフェードと同時にスタックが詰まる**。
  - 行自身の grid-template-rows の transitionend で、内側の clip（overflow hidden）を visible に戻し、影を描けるようにする（:265-281）。
  - 除去は退場時の grid-template-rows の transitionend だけを待つ（**フォールバックタイマーなし**）。
- **スタック展開（ホバーで重なりが広がるなど）はない**。通常の縦並びである。
- **スワイプ**: §3.6。dismiss 時は translateY(±120%) と opacity 0 を 125ms で。それ以外は 125ms、std で元に戻る。
- auto hide 5000ms。

### 4.18 Token / Tokenizer

- Token（`Token/Token.tsx:147-155`）
  - interactive では `background-image` を 125ms、std で遷移。グラデーションなので実質即時。RM の扱いなし。
  - タッチ release は 200ms。disabled は 0.5。
- **Tokenizer: トークンの追加・削除にアニメーションはない**。固有の transition / keyframe はない。
  - ポップオーバーは layerAnimations なしで、登場もしない（:478）。
  - アイドル時の spinner は `visibility:hidden` と `content-visibility:hidden` で止める（:426-433）。
  - シート表示は BottomSheet、確認は AlertDialog。
  - `internal/tokenizer-presentation.ts` は再 export のみ。

### 4.19 AvatarGroup / AvatarGroupOverflow

overflow チップはマウスでは即時、タッチでは 200ms release。AvatarGroup 本体は動かない。

### 4.20 Thumbnail（`Thumbnail/Thumbnail.tsx`）

- オーバーレイ `::after`: `background-color` を transparent から hover / pressed 色へ、125ms、std で遷移。press の塗り中（on / fading）は transition none（:186-245）。RM の扱いなし。
- 削除ボタン（`showRemoveOn='hover'`）は `useContainerReveal` の `isLayoutPreserved` 版（opacity のみ）。
  - コンテナの `:hover`（hover:hover）または `:focus-within` で opacity 0→1。`(any-pointer: coarse)` では常に 1。
  - 125ms、std、delay 0。RM 0s（`hooks/containerReveal.stylex.ts:216-224`）。
- **画像読み込みのフェードインはない**。src がない読み込み中は Skeleton、src がある読み込み中は upload オーバーレイ + Spinner。
- disabled は 0.5。

### 4.21 Citation（`Citation/Citation.tsx:93-164`）

- ラベルのピル: `background-color, border-color, color` を **`--duration-fast-max` 165ms**（230）、std で遷移。hover で bg → overlay-hover、文字色 secondary → primary。
- 番号バッジ: bg を 165ms で遷移（accent-muted → overlay-hover）。
- RM の扱いなし。

### 4.22 ClickableCard / SelectableCard

- ClickableCard（:87-180）
  - `::after` のオーバーレイは Thumbnail と同じ（125ms、std、press の塗り中は none）。
  - bordered では border-color → `--color-border-emphasized` を 125ms で。
  - RM の扱いなし。
- SelectableCard（:80-219）
  - `box-shadow, border-color` を 125ms、std で遷移。
  - 選択時は border → accent、`--_card-ring` に `inset 0 0 0 2px <色>` を入れる（Card の box-shadow に合成）。**2px の内側リングがフェードインする**。
  - RM の扱いなし。

### 4.23 Link（`Link/Link.tsx:81-155`）

- `color, text-decoration` を 125ms、std で遷移。hover で下線が出る（離散的なので即時）。
- 色は variant ごとに tint-hover を 15% 混ぜる。
- `:active` はマウスでは即時、タッチでは 200ms。RM の扱いなし。

### 4.24 Item / List（`Item/Item.tsx`、`ItemSwipeLayer.tsx`、`useSwipeAction.ts`）

- 行の背景: `background-color` を **`--duration-fast-min` 95ms**（130）、std で遷移。hover → overlay-hover、`:active` → overlay-pressed、highlighted → overlay-hover、selected → `--color-accent-muted`（:406-414）。RM の扱いなし。
- スワイプ（タッチ）
  - 行に `translate3d(var(--_item-swipe-travel),0,0)`。transition は `background-color` が 95ms・std、`transform` が `var(--_item-swipe-duration, 0s)`・`ease-out`（:534-541）。
  - パネルは逆向きに translate しつつ幅を伸ばす。同じクロックで ease-out（ItemSwipeLayer.tsx:96-109）。
  - ドラッグ中は 0s で、毎フレーム 1 回 rAF で書き込む。
  - settle（戻り、開いたまま停止、行幅 + 24px までのスライドアウト）は **200ms ease-out**。同じ長さのタイマーで次の段階へ移る。
  - スライドアウト → 発火 → 保持（hasRemoval）または 200ms で戻る。
  - RM: settle 0。
- List は ListItem（Item + CheckboxInput）を継承し、固有のモーションはない。

### 4.25 Field / TextInput / TextArea / NumberInput / FileInput / InputGroup / RichTextEditor

- 入力枠（`Field/inputStyles.stylex.ts:48-71`）: `border-color, box-shadow` を 125ms、std で遷移。RM 0s。
  - 静止時は border `--color-border-emphasized`、shadow なし。
  - hover（非フォーカス、hover:hover）: `inset 0 0 0 2px color-mix(--color-border-emphasized 30%, transparent)`。
  - `:focus-within`: border `--color-accent` と `inset 0 0 0 2px --color-accent-muted`。
  - status の variant は border と hover の影（`--shadow-inset-*`）を切り替える。disabled は 0.5（遷移しない）。
- PanelSearchInput: box-shadow を 125ms（RM 0s）。
- TextInput / TextArea: 枠の遷移と busy Spinner だけ。
- NumberInput: ステッパーボタンのマウス操作は即時、タッチは 200ms。increment アイコンの rotate(180deg) は静的。
- FileInput（:162-226）
  - `border-color, box-shadow, background-color` を 125ms、std、RM 0s。
  - hover は `inset 0 0 0 2px color-mix(accent 20%)`。
  - drag-over は border → accent、bg → accent-muted。
- InputGroup: 静的な disabled 0.5 のみ。
- **packages/richtext**: 固有のモーションはない。エディタの枠は Field の `inputWrapperStyles.base` と同じ（RichTextEditor.tsx:656-664）。キャレットはネイティブのまま（点滅のカスタマイズなし）。

### 4.26 FieldStatus（`FieldStatus/FieldStatus.tsx:215` + `hooks/useEntryAnimation.ts`）

- **mount 時のみ**: `useEntryAnimation('slideDown')` で opacity 0→1、translateY(−0.5rem) → 0。`--duration-fast-max` **165ms**（230）、std、fill backwards。
- 初回 paint 後に挿入された要素だけが再生する。hydration 中と、モジュール読み込み直後の最初の rAF 内では再生しない（useEntryAnimation.ts:16-30, :116-120, :149-155）。
- 文言や type が変わっても再生しない。退場アニメはない。
- RM: なし（coarse でもフェードしない）。
- 他のプリセット（現在はどこも使っていない）:
  - slideUp: translateY(+0.5rem) → 0
  - fadeIn: opacity のみ
  - scaleIn: scale 0.95→1 + opacity

### 4.27 CodeBlock（`CodeBlock/CodeBlock.tsx`）

- 折りたたみ: grid の `grid-template-rows: 1fr ↔ 0fr` を `--duration-medium` **300ms**（410）、std で遷移（:202-215）。領域は mount されたまま。初期状態は展開。**RM の扱いなし**。
- シェブロンの出現（`chevronReveal` キーフレーム、:113-124）
  - from width 0・margin-inline-end 0・opacity 0 → to width 14px・`--spacing-1`・opacity 1。300ms、std。
  - 折りたたみ可能になって mount されたときに再生する（初回描画を含み、hydration ガードなし）。RM ではなし。
- シェブロン: `transform` を 125ms、std で遷移。展開で rotate(90deg)。**RM の扱いなし**。
- コピー: アイコンを即座に check へ替え、2000ms 後に戻す（`hooks/useClipboard.ts:29`）。

### 4.28 アンカーレイヤー共通の登場（`Layer/layerAnimations.stylex.ts`）

使用者は Tooltip、HoverCard、Popover、DropdownMenu（root とサブメニューのフライアウト）、ContextMenu、Selector、ComplexSelector、PowerSearch、Breadcrumbs のメニュー。

- キーフレーム（すべて opacity 0→1、transform は `translate(0) scale(1)` で終わる、:32-90）:

| バリアント | from transform |
|---|---|
| below（アンカーの下に出る） | `translateY(calc(-1 * 0.5rem)) scale(0.95)`（8px 上から） |
| above | `translateY(0.5rem) scale(0.95)` |
| end | `translateX(-0.5rem) scale(0.95)`（RTL では +） |
| start | `translateX(0.5rem) scale(0.95)`（RTL では −） |
| fadeIn | opacity のみ |

- duration は `--duration-fast-max` **165ms**（230）。std、fill backwards、delay なし（:92-101）。
- **transform-origin は未指定（要素の中心）**。0.95 からの拡大はアンカー側ではなく中心基準になる。
- RM: fine ではアニメーションなし。coarse では fadeIn を min(fast-max, 200ms) = 165ms で（カーブは std のまま）。
- **退場アニメなし**: `hidePopover()` で 1 フレームで消える。AST-090 の `--duration-exit-*` は spec では accepted だが、**ソースに存在しない**（grep 0 件）。
- 中断: 閉じれば CSS アニメーションは消え、再オープンでは 0% から再開する（リターゲットしない）。
- 配置フォールバック（flip）で上下が反転しても、登場方向は再計算されない（要求された placement の方向のまま）。
- `placeAfresh` / `preserveAnimations`: 再配置しても登場アニメを最初からやり直さない。

### 4.29 Tooltip（`Tooltip/useTooltip.tsx`）

- 既定: placement 'above'、**表示遅延 200ms**（:302）、hideDelay 0（:303）、オフセット `--spacing-1`。
- mouseleave / focus-out では `HOVER_BRIDGE_DELAY = 100ms` 後に隠す（:50, :433-436）。tooltip の上に乗れば取り消す。
- キーボードフォーカス（`:focus-visible`）では即時に表示する。タッチでも即時（toggle では固定表示）。pointerdown と Escape で即時に隠す。
- warm-up（連続ホバー時の遅延スキップ）は**ない**。
- 登場は §4.28（above なら 8px 下から）。退場なし。

### 4.30 HoverCard（`HoverCard/useHoverCard.tsx:281-399`）

- 既定: placement 'above'、**表示 300ms、非表示 200ms**。カード上にいる間は隠さない。
- focus-in では即時（タッチ由来を除く）。タップで開く。
- 登場は §4.28。退場なし。Timestamp のホバーカードも同じ。

### 4.31 Popover（`Popover/Popover.tsx:507-1006`）

- 既定: placement 'below'、alignment 'start'。登場は `layerAnimations[placement]`。退場なし。
- `presentation='bottom-sheet'`、またはコンパクトタッチでの `'adaptive'` では BottomSheet（§4.37）になる。
- コンパクトタッチの条件: `(width < 768px) and (pointer: coarse)` または `(height < 500px) and (pointer: coarse)`（`hooks/useAdaptivePresentation.ts`、`theme/themeAdaptations.ts:83-94`）。
- `usePopover` 単体はアニメーションを持たない。呼び出し側が xstyle を渡したときだけ動く。

### 4.32 DropdownMenu / MoreMenu / サブメニュー

- 登場は `layerAnimations[placement]`（既定 below、:1738）。退場なし。MoreMenu も同じ。
- `dropdown` スタイルとフライアウトには `opacity` 125ms の transition が宣言されているが、opacity が変わらないので**死んだ定義**である（:179-182、DropdownMenuSubMenu.tsx:209-212, :309-312）。
- 項目に highlight の遷移はない。disabled は 0.5。
- **サブメニューの hover intent**（`hooks/useMenuHover.ts`）
  - showDelay 150ms、hideDelay 200ms。`(hover: hover)` のときだけ働く。
  - 閉じてから 300ms 以内の再 enter は無視する。hover で開いてから 500ms 以内のクリックは固定扱い。
  - safe triangle: 離脱点からフライアウトの近い辺まで、両端を 8px 広げた三角形。その中での pointermove は 200ms の hide タイマーを再始動する。
  - フライアウトの登場は `layerAnimations.end`（placement end、alignment start）。RTL では反転。
- コンパクトタッチでのサブメニューは drill-in で、ビューの入れ替えにアニメーションはない（`useMenuDrillIn.tsx`）。
- `springs.subMenuOpen` / `subMenuBack` / `menuOpen` / `menuClose` は未使用。
- シート表示は BottomSheet。

### 4.33 ContextMenu（`ContextMenu/ContextMenu.tsx`）

- 右クリックまたは 500ms の長押しで、カーソル位置（1px のアンカー）に開く。
- 登場は**常に `layerAnimations.below`**（:907）。退場なし。
- 持ち上げプレビューやモーフはない（`contextMenuLift` / `Morph` は未使用）。:29 の「exit animation」というコメントは古い。

### 4.34 Selector / MultiSelector / ComplexSelector / Typeahead / PowerSearch

- **Selector**
  - ポップオーバーの登場は `layerAnimations[placement ?? 'below']`（:1311, :1943）。
  - リスト（:354-404）は opacity 1、`transition: opacity` を `--duration-fast` 125ms、**CSS `ease`**（トークンではない）。RM では coarse 200ms easeInOut、fine 0s。
  - overlay モード（placement も検索もないとき）では、選択行の中心をトリガーの中心に重ねる位置決め（`useSelectedItemOffset`）が終わるまで opacity 0（transition none）にし、その後 0→1 で 125ms フェードする [推論]。キーフレームの登場と重なる。
  - シェブロン: open で rotate(180deg)、125ms、std、**RM の扱いなし**（:246-254）。
  - トリガーの press: scale 1 → 0.98（fine `:active` / `"on"`）。125ms、std、RM 0s（:271-314）。
- **MultiSelector**: ポップオーバーに**登場アニメなし**（:2460-2465）。シェブロンの 180°（RM の扱いなし）と press 0.98 だけ。
- **ComplexSelector**: 登場は `layerAnimations[placement]`（既定 below）。シェブロン 180° は 125ms で、RM 0s（selector 系で唯一の対応）。press 0.98。
- **Typeahead**: アニメーションなし。
- **PowerSearch**: 登場は `layerAnimations.below`（:1157）。退場なし。
- シート表示（SelectorBottomSheet）は BottomSheet。

### 4.35 Dialog / AlertDialog / CommandPalette（`Dialog/Dialog.tsx`）

- ネイティブの `<dialog>` で、`showModal()` / `close()`。
- **登場**（キーフレーム `enterDirectional`、:145-152）
  - from opacity 0、`translate(var(--dialog-dir-x, 0px), var(--dialog-dir-y, 16px)) scale(0.95)` → to opacity 1、`translate(0,0) scale(1)`。
- **方向ベクトル**（`getDialogDirection` :92-108）
  - 開いた要素（トリガー）の中心 − ビューポート中心を正規化し、16px 倍して丸める。**ダイアログはトリガー側に 16px ずれた位置から中心へ収まる**。
  - トリガーがない、または body の場合は (0, 16px)（下から上がる）。
- duration は **`--duration-medium-max` 400ms**（550）、std、fill backwards（:204-213）。transform-origin は中心。
- フルスクリーン変種は opacity のみ（`enterFullscreen`）。
- RM: coarse では opacity のみ 200ms `cubic-bezier(0.42,0,0.58,1)`。fine ではなし。
- SSR で開いていたダイアログは、アニメーションなしで開く（`useSettledOpen`）。
- **backdrop**（`::backdrop`、`--color-overlay` + `backdrop-filter: blur(2px)`）は**アニメーションしない**。
- **退場なし**: `close()` で即座に消える。
- 中断 [推論]: 登場中に閉じれば即時に消える。再オープンではキーフレームを再開し、方向も再計算する。
- AlertDialog は standard 変種で、同じ登場。
- CommandPalette は追加で、検索 spinner を遅らせて出す（CommandPaletteInput.tsx:78-86）。opacity 1、`@starting-style` 0、transition 1ms、**delay 150ms**。150ms 以上 pending のときだけ見える（ちらつき防止）。

### 4.36 Drawer（`Drawer/Drawer.tsx`、`useDrawerDialogPresence.ts`）

- hasScrim なら `<dialog>` の `showModal()`、なしなら manual popover。表示は React の `rendered` で管理する。
- **パネル**: `transform` だけを `--duration-medium` **300ms**（410）、std で遷移（:110-150）。
  - side 'end'（既定）: 閉 translateX(100%)（RTL −100%）。開 translateX(0)、`@starting-style` translateX(100%)。
  - side 'start' はその鏡像。
- **退場**: open スタイルを外し、同じ 300ms、**std**（ease-exit ではない）で ±100% に戻る。
  - 退場方向は開いたときの side に固定する（`exitSideRef`）。子は退場中も mount したまま。
- 完了待ち: `transform` の transitionend / transitioncancel を待つ。バックストップ = 計算上の duration + delay + 50ms（読めなければ 250 + 50ms）。その後 close、unmount、フォーカス復帰を 1 タスクで行う。
- **スクリム**: `::backdrop`（`--color-overlay` + blur(2px)）。opacity 0→1 を `@starting-style` で、300ms、std。退場時はパネルと並行して 0 へ。
- RM: パネル・スクリムとも 0.01s（coarse でもフェードしない）。
- 中断 [推論]: 退場中の再オープンでは CSS transition が現在値から逆転する。
- ドラッグ / スワイプはない。モバイル幅は min(width, 100dvw − 56px)。

### 4.37 BottomSheet / BottomSheetSwitcher（`BottomSheet/*`）

- **登場**（BottomSheetPanel.tsx:132-191）
  - transform translateY(0)、`@starting-style` translateY(100%)。100% はパネル自身の高さで、safe-area + 48px の下部パディングを含む（−48px margin で相殺）。
  - `transform, opacity` を `--duration-medium` **300ms**（410）、std。will-change transform, opacity。
  - RM fine: 0.01s。RM coarse: transform 0.01s、opacity 200ms、std（opacity の `@starting-style` 0、transform は移動なし）。
  - SSR で開いていた場合は 0.01s。最初の transitionend で解除する。
- **退場**（`sheetClosing` :206-223）
  - translateY(100%)。**カーブは `--ease-exit` (0.3,0,0.6,0.6)**（coarse ではリテラル値でテーマ不可）。duration は同じ 300ms。
  - **登場は減速、退場は加速**。ここが Astryx で唯一の非対称カーブの退場である。
  - 開始位置は現在位置（detent、またはスワイプを離した位置）。**離した速度は引き継がない**（静止から始まる固定長の曲線）。
  - RM: 移動なし。coarse は opacity 0 を 200ms。タッチスワイプによる dismiss は、どのポインタでも opacity 200ms（`sheetClosingCrossFade`）。
  - 完了待ちは transitionend / transitioncancel + バックストップ（duration + delay + 50ms）。
- **スクリム**（BottomSheet.tsx:173-209）
  - `::backdrop`（`--color-overlay`、blur なし）。opacity `var(--_sheet-scrim-opacity, 1)`、`@starting-style` 0。
  - `transition: opacity, display` を `var(--_sheet-scrim-duration, --duration-medium)`、std、allow-discrete。
  - **退場時は easing を `linear` に切り替え**、300ms 全体でフェードする（std だと 410ms 中 163ms で 90% 消えるため）。
  - ドラッグ中はシートに追従する（`scrimOpacityForOffset`）。最後の作業 detent までは 1 で、そこから peek で 0.3、peek がなければ dismiss ラインで 0 へ線形に下げる。
  - スタック間の受け渡しは、2 回の rAF の間 duration 0s に固定して、フェードさせない。
- **スタック**（BottomSheet.tsx:224-253）: 覆われたシートは後退する。
  - transform-origin 50% 0、300ms、std。
  - 深さ 1: translateY(−8px) scale(0.96)。深さ 2 以上: translateY(−16px) scale(0.92)。
  - pop で元に戻る。backdrop はスタックで 1 枚だけ見せる。
- **置換**（bottomSheetStack.ts:1424-1511）
  - 旧シートは `covered` → (`aligning`：上端が 1px を超えてずれていれば、新しいシートの上端まで translateY で 300ms スライド) → `fading`（opacity 0、300ms、std）の順に進む。
  - RM ではオフセット 0（その場でフェード）。
- **ドラッグ物理**: §3.6。settle は CSS 300ms、std。スプリングも速度引き継ぎもない。
- `springs.sheet*` は未使用。

### 4.38 Lightbox（`Lightbox/Lightbox.tsx`）

- 開閉は `showModal()` / `close()` で、**アニメーションなし**。backdrop（blur 2px）も動かない。
- ズーム（`hasZoom`、画像のみ）
  - ダブルクリック、Enter / Space、+ / − キーで 1 ↔ 2 を切り替える。
  - transform = `scale(zoom) translate(pan.x/zoom px, pan.y/zoom px)`。origin は中心。
  - `transition: transform 200ms ease-out`（(0,0,0.58,1)）。RM 0ms（:198-212, :640-643）。
- pan: 拡大中のドラッグで、transition なしの直接追従。境界・慣性はない。キーボードでは 50px ずつ（200ms で遷移）。
- ギャラリーの送りは即時の入れ替え。pinch なし。サムネイルからの FLIP / ズームもない。

### 4.39 Overlay（`Overlay/OverlayScrim.tsx:51-183`）

- `opacity, visibility, transform` を `--duration-fast` **125ms**、std。RM 0s。
- fill 配置: opacity 0・hidden ↔ 1・visible。mount 時は `@starting-style` opacity 0。
- top / bottom ストリップは、加えて translateY(∓100%) → 0 のスライド。
- トリガー: controlled `isOpen`、`showOn` が 'always'（mount 時に登場）、'hover'、'hover-or-focus'、'focus'。
- 退場は同じ遷移の逆再生。visibility は最後に切り替わる。

### 4.40 Calendar / DateInput / DateRangeInput / DateTimeInput / TimeInput

- Calendar の日セル: `background-color, color` を 125ms、std。RM では transition-property none（Calendar/styles.ts:214-219）。**月の切り替えは即時（スライドなし）**。
- DateInput / DateRangeInput / DateTimeInput のデスクトップ用ポップオーバーは、**登場アニメなし**・退場なし。
- TouchDateField（BottomSheet 内）
  - カレンダーと月年ホイールの切り替えで、`SWAP_DURATION` = `--duration-fast` 125ms を使う。
  - ホイールのパネルは opacity・visibility を **linear** で遷移する。下のカレンダーは visibility だけを遷移する。
  - ヘッダー矢印、Reset、曜日は linear でフェードする。タイトルのシェブロンは 180°、std。
  - RM: 0.01s。
- TouchDateTimeField のパネル切り替えには transition がない（即時）。
- **MonthScroller**
  - `scroll-snap-type: x mandatory`、ネイティブの慣性。矢印は `scrollTo` smooth。
  - 落ち着いた後（scrollend、または 220ms 静止）に、最寄りのペインへ smooth スクロールで補正する。
  - RM coarse: `animate([{opacity:1},{opacity:0},{opacity:1}], 200ms, easeInOut)` で、100ms で着地。fine: 即時。
- **Wheel**（日付・時刻ピッカーの回転ホイール）
  - 行 28px、ペイン 264px（6×44）、端パディング 118px、perspective 520px。
  - scroll-driven animation（`animation-timeline: view(y)`、range cover 0%→100%）。
    - 0%: opacity 0.3、`rotateX(52deg) scale(0.86)`
    - 50%: opacity 1、`rotateX(0) scale(1)`
    - 100%: opacity 0.3、`rotateX(-52deg) scale(0.86)`
  - RM では opacity だけ。
  - アクティブ行の color / font-weight は 125ms `ease`（RM の扱いなし）。
  - 自前スクロールは smooth。RM coarse では 200ms の dip（100ms で着地）。
  - マウスドラッグ: slop 4px、release 時に smooth で最寄り行へ。

### 4.41 Collapsible / CollapsibleGroup（`Collapsible/Collapsible.tsx`）

- **シェブロン**（:128-175）: `transform` を 125ms、std で遷移。
  - 末尾シェブロン（chevronDown）は rotate 0→180deg。先頭シェブロン（chevronRight）は 0→90deg（RTL では scaleX(−1) を静的に付ける）。
  - ハンドルなしの場合、**RM の扱いなし**。
  - ハンドルありの場合、735ms の `linear()` スプリング（§1.9）をインラインで付け、RM では 0s（:417-436）。
- **内容**: `animatedLayout` prop があるときだけ動く。ないときは `display:none` の即時切替（高さアニメなし）。
  - 開く: `releaseLayoutExit` の後に `playLayoutChange`。下端からのクリップと opacity の reveal を 734.77ms で行い、後続は translate でグライドする。
  - 閉じる: `isExiting` で mount を保ったまま `holdLayoutExit`。即座に空間を譲り、inert にしてから、上へ畳むように 734.77ms で隠す。
  - 非対称性: 退場は即座に空間を手放し、登場は確保済みの空間に現れる。カーブと時間は同じ。
- トリガーの press は §3.4。hover 面はない。
- CollapsibleGroup 固有のモーションはない（各子に animatedLayout が必要）。

### 4.42 TreeList（`TreeList/TreeList.tsx`、`TreeListItem.tsx`）

- シェブロンは rotate 0→90deg（RTL では反転）、125ms、std。ハンドルありなら 735ms の linear() スプリング。
- 子の `<ul role="group">`（ハンドルありのみ）
  - 展開は `playLayoutChange`。折りたたみは `holdLayoutExit`（`data-tree-exiting` を付けて保持し、終了時に削除）。
  - 下の行は 734.77ms でグライドする。入れ子で変化した行は外側にまとめ、1 回だけ動かす。
  - 退場中の行はキーボードナビゲーションから外す。
  - ハンドルがなければ即時。
- 行の hover / press: `transition: background-image` を 125ms。グラデーションなので実質即時。selected（accent-muted）は遷移なし。

### 4.43 Table（`Table/*`）

- 行の hover（`hasHover` のときのみ）: background-color を overlay-hover へ、125ms、std。RM の扱いなし。
- 選択: 行の bg を accent-muted に。`hasHover` があれば 125ms で、なければ即時。
- **行展開**（`plugins/rowExpansion`）
  - シェブロンは rotate 0→90deg、125ms。ハンドルありかつ列罫線なしなら 735ms の linear() スプリング。
  - ハンドルあり:
    - 開く: 後続の `<tr>` を 734.77ms でグライドし、新しい panel 行は clip と opacity で reveal。
    - 閉じる: panel 行は**即時に unmount**し、下の行だけがグライドで上がる。
    - グライド中は、スクロールラッパーの高さを inline の padding と負の margin で保持する。
  - 罫線: グライド中の行は ::after の罫線が一緒に動く（`glidingDivider.ts`）。
  - 列罫線（dividers 'columns' / 'grid'）があるテーブルでは、アニメーションを一切しない。
  - RM fine は即時、coarse は 200ms フェード。
- ツリーデータ / グループ行: 子行は即時に出入りする。展開ボタンとシェブロンは **150ms `ease`**（リテラル）。RM の扱いなし。
- 固定列のスクロール影: 6px の `::after` グラデーションの opacity 0 ↔ 1 を **150ms `ease`**。スクロールのたびに更新する。
- 列リサイズのインジケーター: `background-color 150ms ease, width 150ms ease`。ドラッグ中は 1→2px、accent。スクロール中は hint を消す（120ms 静止で戻す）。列幅自体は即時。
- **ソートの並べ替え FLIP はない**（その場で再描画）。ソート / フィルタのアイコンの opacity 0.35→1 は即時。

### 4.44 Carousel（`Carousel/Carousel.tsx`）

- 独自のページング物理はない。ネイティブの overflow-x と `scroll-snap`（hasSnap 時は mandatory、start）。
- ボタンと `scrollTo` は `scrollBy({behavior: smooth | auto(RM)})`。UA 依存の時間・カーブ。
- 1 ステップ = max(clientWidth − 先頭アイテム幅×0.5, 先頭アイテム幅)。
- 端のフェード（mask-image）は**即時切替**。
- ナビのピル: opacity 1 ↔ 0 を 125ms、std。RM の扱いなし。

### 4.45 ScrollableArea

**アニメーションなし**。スクロール影もフェードもない。ネイティブのスクロールバー。

### 4.46 Resizable（`Resizable/ResizeHandle.tsx`）

- パネルサイズは指・キーボードに直接追従し、アニメーションしない。
- 区切り線: background-color を 125ms、std。ドラッグ中は border-emphasized。
- ピル型グリップ: `opacity, background-color, transform` を 125ms（isAlwaysVisible=false なら hover で 0→1）。
- RM の扱いなし。

### 4.47 SideNav（`SideNav/*`、`railMotion.ts`）

- **レールの折りたたみ・展開**（SideNav.tsx:524-595）
  - レイアウトは最終幅へ即座に変わる。見える端は inline-end 側の `clip-path: inset()`（px）で、旧幅から新幅へ springs.settle（734.77ms）で動かす。
  - 折りたたみ: 旧レールの inert コピーをクリップする。実レールは上（z-index 2）に置く。
    - コピーのラベル opacity = 1 − clamp(p/0.4)（前半 40% でフェード）
    - コピー全体の opacity = 1 − clamp((p−0.7)/0.3)（最後の 30%）
    - 実レールの opacity は (p−0.7)/0.3 で上げる
  - 展開: 実レールを折りたたみ幅からクリップして開く。
    - コピーの opacity = 1 − clamp(p/0.6)
    - 実ラベルの opacity = clamp((p−0.4)/0.6)
  - AppShell 内では、端がコンテンツ列のグライドに毎フレーム乗る。
  - 中断: 現在の幅と opacity を読んで、そこから再開する。
  - RM: coarse は 200ms フェード（クリップの移動なし）、fine は即時。
- 折りたたみボタンのシェブロン: 0 ↔ 180deg、125ms、std、RM 0s。
- 折りたためるグループ行（SideNavItem.tsx:130-191）
  - `grid-template-rows 1fr ↔ 0fr` を 300ms（410）、std。`overflow` は allow-discrete。
  - 開くときは overflow を 300ms 遅らせて visible にし、閉じるときは即座に clip する。
  - シェブロンは 180deg、125ms。RM はすべて 0s。
- SideNavHeading のシェブロンは静的。ポップオーバーは即時。

### 4.48 TopNav（`TopNav/*`）

- **MegaMenu パネル**（TopNavMegaMenu.tsx:124-159）
  - Popover API。opacity 0→1、translateY(−4px)→0（`@starting-style`）。閉じるときは逆再生。
  - `opacity, transform, overlay, display` を allow-discrete で、`--duration-medium-min` **225ms**（310）、std。
  - RM coarse は opacity のみ 200ms easeInOut、fine は 0s。
  - hover intent: 表示 150ms、非表示 250ms。
- TopNavMenu のドロップダウン: **パネルのアニメーションなし**。hover 表示 150ms、非表示 200ms。
- トリガーのシェブロン: 180deg、125ms（RM の扱いなし）。各色の遷移は 125ms（RM の扱いなし）。
- モバイルのドロワー内セクション: grid-template-rows 0fr→1fr、300ms（overflow は遅延）、シェブロン 180deg。**RM の扱いなし**。

### 4.49 MobileNav（`MobileNav/MobileNav.tsx`）

- `<dialog>` の `showModal()`。
- ドロワーのスライド: start 側は translateX(−100%) → 0（RTL では +100%）、end 側は +100% → 0。`@starting-style`、`transform` を `--duration-medium` **300ms**、std。閉じるときは逆再生。
- backdrop: `--color-overlay` + blur(2px)。opacity 0→1 を 300ms、std。
- display の保持（allow-discrete、300ms）により、スライドアウト中も描画を続ける。
- `close()` の遅延 = min(250ms, 最短 transition × 0.6) = **180ms**（246）。RM では 0。
  - [推論] backdrop は close 時点で消えるので、パネルより先に消える。
- RM: 0.01s。

### 4.50 AppShell（`AppShell/AppShell.tsx`）

- レールの変化に合わせて、コンテンツ列が springs.settle（734.77ms）でグライドする（:940-1080）。
- コンテンツのスワップ
  - 旧メイン内容をコピーとして保持し、列に乗せる。
  - コピーの opacity = held×(1−p)、新しい子の opacity = entering + (1−entering)×p。
- 展開時の隙間埋め: サーフェスに `box-shadow: ${−travel·(1−p)}px 0 0 0 <サーフェス色>` を付けて動かす。
- RM: グライドなし。coarse は 200ms のクロスフェード、fine は即時。
- MobileNav を閉じるとき、Activity を 250 + 50 = 300ms 保持する（RM では 0）。

### 4.51 Stepper（`Stepper/Step.tsx`）

- 接続線のフィル: `::before` を軸方向に scaleY / scaleX 0 ↔ 1。origin は top（縦）、left（横、RTL では right）。**timing function は linear**（意図的）。
- 基準時間 `FILL_SPAN_TIME` = `--duration-medium` **300ms**（410）。
- **1 段だけ進む場合（activeStep = previous + 1）にだけアニメーションする**。戻る・飛ぶ・mount は即時。
- 区間の分配:

| レイアウト | 配分 | Neutral（duration / delay） |
|---|---|---|
| 分離バー | 到着バー 1.0 | 300 / 0 |
| トラック上・横 | 離脱 0.5、到着 0.5 | 150 / 0、次に 150 / 150 |
| トラック上・縦（内容スロットなし） | 0.7 / 0.3 | 210 / 0、90 / 210 |
| トラック上・縦（内容スロットあり） | レール 0.175、内容 0.525、到着 0.3 | 52.5 / 0、157.5 / 52.5、90 / 210 |

- RM: 0s。
- インタラクティブ行: background-color を 95ms（130）、std、RM 0s。

### 4.52 Pagination（`Pagination/Pagination.tsx:225-247`）

ドット: background-color neutral → accent、125ms、std。RM の扱いなし。ページボタンは Button。

### 4.53 Breadcrumbs（`Breadcrumbs/BreadcrumbItem.tsx:718`）

overflow メニューは `layerAnimations.below`（165ms）。退場なし。

### 4.54 Outline（`Outline/Outline.tsx`、`useScrollSpy.ts`）

- **アクティブインジケーター**（:164-181）
  - 2px のバー。CSS アンカーポジショニングで、アクティブなリンクの top と height に追従する。
  - `transition: top, height` を **`--duration-fast-min` 95ms**（130）、std。**旧リンクから新リンクへ滑りながら伸縮する**。中断時はリターゲット。
  - RM の扱いなし。
- リンク: `background-color, color` を 125ms。アクティブ時の semibold は即時。
- クリックによる移動: `scrollIntoView` / `scrollBy` を smooth（ネイティブ）。移動中はスクロールスパイを止めてバーを凍結し、scrollend（またはフォールバック 1200ms）で 1 回だけ移動する。

### 4.55 Chat（`Chat/*`）

- **メッセージ登場アニメはない**。タイピングインジケーター、思考表示、点滅カーソルもない。
- **ChatComposerDrawer**: RM の扱いなし。
  - 高さ: grid-template-rows 1fr ↔ 0fr、`--duration-medium` 300ms、std。
  - 内容の opacity:
    - 展開: 0→1 を 300ms、**delay `--duration-fast` 125ms**（完了は 425ms 後）。
    - 折りたたみ: 1→0 を 125ms、delay なし。
  - 折りたたみ時のサマリー: opacity を 125ms で、ハンドルとクロスフェード。
  - グラブバー（20×2px）: opacity と色を 125ms。
- **ChatComposer**: box-shadow と border-color を 125ms。low では `--shadow-low` → `--shadow-med`（hover / focus-within）。RM の扱いなし。
- **ChatDictationButton**（音量バー）
  - 5 本のバー。`scaleY(0.08 + level×0.92)`、level = min((band/0.2)^0.5, 1)。
  - 色は accent。音量 ≥ 0.2 で error 色へ mix する。
  - 毎フレーム rAF で更新する（AnalyserNode: fftSize 256、smoothing 0.5）。
  - `transition: transform, background-color 60ms ease-out`。RM 0s。
- **ChatLayoutScrollButton**（最下部へ戻るボタン）
  - 下端から 100px 以上離れたとき、または新着があるときに表示する。
  - `opacity, transform, max-width, visibility` を `--duration-fast-max` 165ms、std。transform は宣言だけで未使用。
  - 表示: opacity 0 ↔ 1。ラベルの有無で max-width 32 ↔ 200px。RM 0s。
- **useChatStreamScroll**（自動追従スプリング）
  - フレームごとの自前積分: v = (0.7·v + 0.05·diff)/1.25、position += v × (経過ms / 16.67)。
  - |diff| < 0.5px かつ |v| < 0.1 でスナップする。
  - 過減衰で、100px からの収束は約 0.78s。
  - RM: 1 フレームでジャンプ。
- **ChatMessageMetadata "sending"**: opacity 1 → 0.5 → 1、**1.5s ease-in-out**、infinite。RM ではなし。
- **ChatToolCalls**
  - シェブロン: none ↔ rotate(180deg)、125ms。RM none。
  - グループ展開: grid-template-rows 0fr ↔ 1fr、300ms、std（opacity なし）。RM none。
  - 実行中は Spinner sm。"pile" スタイルは死んだ定義。
- ChatMessageList: 上方向の読み込み中は Spinner md。

### 4.56 Markdown（ストリーミング、`hooks/useStreamingText.ts`、`Markdown/streaming.ts`）

- **useStreamingText**（speed 'natural'）
  - 1 tick あたりの文字数: natural 10、fast 4（UTF-16 単位を grapheme 境界にスナップ）。
  - tick 間隔 = max(4, round(`--duration-fast-min` / 10))。Neutral では **10ms**（既定 13ms）。fast は /20 で 5ms。
  - rAF でフレーム量子化されるので、60Hz では毎フレーム 10 文字（約 600 文字/s）。
  - 新しい対象が来たら 0 から始める。isStreaming が false になったら全文。
  - RM: 即時に全文。
- **フェードイン**（Markdown.tsx:659-676）
  - 新しく挿入された span は opacity 0→1（`@starting-style`）。`--duration-medium` **300ms**（410）、std。RM 0s。
  - tick ごとに 1 チャンクの span になる（実質的に 1 tick ずつのスタッガー）。
  - 同時に存在する span は最大 12（古いものは確定テキストに統合されるので、フェード末尾で 1 にスナップする）。
  - inline code、コードブロック、citation、改行はフェードしない。
- HeadingLinks の "#" ボタン: `useContainerReveal` の layout preserved 版。opacity を 125ms、coarse では常に 1、RM 0s。コピー時は 1500ms check を表示。

### 4.57 Timer / Timestamp

CSS アニメーションはない。Timer は 1000ms ごとにテキストを書き換える。Timestamp は相対時刻を interval で更新し、ホバーカードは HoverCard。

### 4.58 useContainerReveal（`hooks/useContainerReveal.ts`、`containerReveal.stylex.ts`）

- hover / focus-within で opacity 0→1 にし、さらに `position` を absolute → static に離散切り替えする（allow-discrete）。
- `--duration-fast` 125ms、std。
- delay（opacity, position の順）:
  - rest: `0s, --duration-fast`（フェードが終わってから流れの外に出る）
  - hover: `hoverDelay`（既定 0）
  - focus と coarse: 0s
- `isLayoutPreserved` では opacity だけ。
- RM: 0s。delay は reduced 用の変数を使う。

### 4.59 アニメーションが一切ないコンポーネント

- Badge、Card（静的なリングの合成のみ）、Kbd、Code、Icon、Heading、Text、Blockquote、Divider、EmptyState、AspectRatio、Toolbar
- Timer、Avatar、AvatarGroup、InputGroup、CheckIndicator、CheckboxList、ButtonGroup、IconButton（Button の挙動のみ）
- Typeahead、NavMenu、NavItem、NavIcon、OverflowList、ScrollableArea、MetadataList、Section、Grid、Stack / HStack / VStack / Center、Layout*、FormLayout
- MobileNavToggle、SideNavSection、SideNavHeading、TopNav 本体、TopNavHeading、TopNavMenu のパネル
- Table の Cell / Header / Body / Footer、TableSelectionToolbar
- Chat の Message / Bubble / SystemMessage / SendButton / ComposerInput / PastedTextToken / TokenizedText
- DateRangeInput と DateTimeInput のデスクトップ用ポップオーバー、TimeInput（デスクトップ）
- `packages/richtext`（Field の枠を除く）
- `reset.css`（グローバルなモーション規則も reduced-motion リセットもない）
- `tailwind-theme.css`（`--ease-out: var(--ease-standard)` のみ）

存在しない機能（どこにも実装がない）:

- View Transitions（`startViewTransition`）
- `interpolate-size`、`::details-content`
- ソートの FLIP
- shimmer
- タイピングインジケーター
- サムネイルからの Lightbox ズーム
- UIKit 風のメニューやシートのスプリング再生

---

## 5. サマリー表（Neutral 値）

- std = `cubic-bezier(0.24,1,0.4,1)`、exit = `cubic-bezier(0.3,0,0.6,0.6)`、io = `cubic-bezier(0.42,0,0.58,1)`。
- 「RM」列の値: 「0」は即時、「fade200」は coarse で 200ms io の opacity フェード（fine は即時）、「なし」は Reduce Motion の扱いがなく、そのまま動く。

| コンポーネント | アニメーション | プロパティ | 時間 | カーブ | RM |
|---|---|---|---|---|---|
| Button | 状態色・disabled | bg, color, opacity | 125ms | std | 0 |
| Button | press | scale 1→0.98 | 125ms | std | 0（scale は即時に適用） |
| Button | ローディング spinner 出現 | opacity 0→1（delay 225ms） | 125ms | CSS ease | delay 0、フェードは残る |
| 全 pressable（touch） | press release | オーバーレイ強度 1→0 | 200ms | (0,0,0.2,1) | 残る |
| ToggleButton | pressed 背景 | bg | 125ms | std | 0 |
| SegmentedControl | 選択 | color, bg, box-shadow | 125ms | std | なし |
| Tab | ラベル・hover・インジケーター | color / bg / opacity | 125ms | std | なし |
| TabMenu | シェブロン | rotate 0→180 | 125ms | std | なし |
| Switch（fine） | トラック色 | bg | 125ms | std | 0 |
| Switch（fine） | つまみ | translateX 0→14, size 16→20 | 125ms | std | 0 |
| Switch（touch） | つまみ移動 | x | spring 0.3/1（約 441ms） | spring | fade200 dip |
| Switch（touch） | 色 ON / OFF | 層の opacity | spring k85.38/c18.48 / k246.74/c31.42 | spring | fade200 |
| Switch（touch） | lift → settle | scale 1→1.25(md off)→1、影 0→1→0 | lift 0.23/0.55、最短 235ms、settle 0.5/0.7 | spring | lift なし |
| Checkbox / Radio | 箱・円 | bg, border | 125ms | std | 0 |
| Checkbox | チェックマーク | display | 即時 | — | — |
| Slider（fine） | つまみ色 | bg, shadow | 125ms | std | 0 |
| Slider（touch） | lift / drop | scale 1→1.55→1 | 0.215/0.6 / 0.5/0.7 | spring | なし（lift しない） |
| Slider（touch） | release coast | 位置 | 0.4/1、投影 0.4×(v·0.996/0.004/1000) | spring | 継続 |
| ProgressBar 確定値（fine） | fill | translateX | 300ms | std | 0 |
| ProgressBar 確定値（touch） | fill | translateX | \|Δ\|×1000ms | linear | 0 |
| ProgressBar 不確定 | スライド | translateX −100%→250%（幅 40%） | 1.5s ∞ | ease-in-out | opacity 1↔0.84、2000ms io alternate |
| Spinner | 弧の回転 | stroke-dashoffset 1 周 | 525ms ∞ | linear | 1 周 3s |
| Skeleton | 段階的明滅 | opacity 0.25→0.925（10 段、保持） | 400ms alternate ∞、delay 1000+100i | linear | 静止 0.25 |
| StatusDot | パルス | opacity 1→0.5→1 | 2s ∞ | ease-in-out | 静止 |
| Banner | シェブロン | rotate 180 | 125ms | std | 0 |
| Toast | 登場 | opacity 0→1, translateY ±8px→0 | 125ms | std | 0.01ms |
| Toast | 退場 | opacity →0, translateY | 125ms | std | 0.01ms |
| Toast | 行の開閉 | grid-template-rows 0fr↔1fr + gap | 125ms | std | 0.01ms |
| Toast | スワイプ dismiss | translateY ±120%, opacity 0 | 125ms | std | 0.01ms |
| Token / TreeList 行 | hover | background-image | 125ms（実質即時） | std | なし |
| Thumbnail / ClickableCard | オーバーレイ | bg | 125ms | std | なし |
| SelectableCard | 選択リング | box-shadow inset 2px, border | 125ms | std | なし |
| Citation | hover | bg, border, color | 165ms | std | なし |
| Link | hover | color, text-decoration | 125ms | std | なし |
| Item | 行背景 | bg | 95ms | std | なし |
| Item | スワイプ settle | translate3d, width | 200ms | ease-out | 0 |
| Field 系 / RichText | フォーカス・hover | border, box-shadow | 125ms | std | 0 |
| FileInput | drag-over | border, shadow, bg | 125ms | std | 0 |
| FieldStatus | 登場 | opacity, translateY −8px→0 | 165ms | std | なし（即時） |
| CodeBlock | 折りたたみ | grid-template-rows | 300ms | std | なし |
| CodeBlock | シェブロン出現 | width 0→14, opacity | 300ms | std | なし |
| CodeBlock | シェブロン | rotate 90 | 125ms | std | なし |
| アンカーレイヤー共通 | 登場 | opacity, translate ±8px, scale 0.95→1 | 165ms | std | fine 0 / coarse opacity 165ms std |
| 同上 | 退場 | — | 即時 | — | — |
| Tooltip | 表示遅延 / hide ブリッジ | — | 200ms / 100ms | — | 遅延は残る |
| HoverCard | 表示 / 非表示遅延 | — | 300ms / 200ms | — | 遅延は残る |
| サブメニュー | hover intent | — | 150ms / 200ms | — | — |
| Selector | リストのフェード | opacity | 125ms | CSS ease | fade200 |
| Selector / MultiSelector | シェブロン | rotate 180 | 125ms | std | なし |
| ComplexSelector | シェブロン | rotate 180 | 125ms | std | 0 |
| Selector 系 | トリガーの press | scale 0.98 | 125ms | std | 0 |
| Dialog / AlertDialog / CommandPalette | 登場 | opacity, translate(トリガー方向 16px), scale 0.95→1 | 400ms | std | fade200 |
| Dialog | backdrop・退場 | — | 即時 | — | — |
| CommandPalette | 検索 spinner | opacity（delay 150ms） | 1ms | — | なし |
| Drawer | パネル開閉 | translateX ±100%↔0 | 300ms | std（両方向） | 0.01s |
| Drawer | スクリム | opacity | 300ms | std | 0.01s |
| BottomSheet | 登場 | translateY 100%→0 | 300ms | std | fine 0.01s / coarse opacity 200ms std |
| BottomSheet | 退場 | translateY →100% | 300ms | **exit** | opacity 200ms |
| BottomSheet | スクリム登場 / 退場 | opacity | 300ms | std / **linear** | 0.01s |
| BottomSheet | detent への settle | translateY | 300ms | std | 0.01s |
| BottomSheet | スタックの後退 | translateY −8/−16, scale 0.96/0.92 | 300ms | std | 0.01s |
| BottomSheet | 置換 | aligning translateY → fading opacity | 300ms + 300ms | std | オフセット 0 |
| Lightbox | ズーム / pan | transform scale 1↔2 | 200ms | ease-out | 0 |
| Overlay | 表示 | opacity, visibility, translateY ±100%（ストリップ） | 125ms | std | 0 |
| Calendar | 日セル | bg, color | 125ms | std | 0 |
| TouchDateField | パネル切替 | opacity, visibility | 125ms | linear | 0.01s |
| Wheel | 3D フォールオフ | opacity 0.3↔1, rotateX ±52°, scale 0.86 | スクロール駆動 | linear | opacity のみ |
| MonthScroller / Wheel | 自前スクロール | scroll | UA smooth | UA | fade200 dip（100ms で着地） |
| Collapsible / TreeList / Table 行展開（ハンドルあり） | 内容 reveal / 退場 | clip-path inset + opacity | 734.77ms | spring settle | fade200 |
| 同上 | 後続のグライド | translate | 734.77ms | spring settle | fade200（opacity 0→1） |
| 同上 | シェブロン | rotate 90/180 | 735ms | linear()（spring） | 0 |
| 同上（ハンドルなし） | シェブロン | rotate | 125ms | std | なし |
| Table ツリー / グループ | シェブロン | rotate 90 | 150ms | CSS ease | なし |
| Table | スクロール影 / リサイズ線 | opacity / bg, width | 150ms | CSS ease | なし |
| Carousel | スクロール | scroll | UA smooth | UA | auto |
| Carousel | ナビのピル | opacity | 125ms | std | なし |
| Resizable | 区切り線・グリップ | bg, opacity | 125ms | std | なし |
| SideNav | レールの開閉 | clip-path inset px + opacity（区間） | 734.77ms | spring settle | fade200 |
| SideNav | グループ行 | grid-template-rows | 300ms | std | 0 |
| SideNav / Collapse ボタン | シェブロン | rotate 180 | 125ms | std | 0 |
| TopNavMegaMenu | 開閉 | opacity, translateY −4px | 225ms | std | fade200 |
| TopNav | シェブロン / 色 / モバイルの展開 | rotate / color / grid-rows | 125 / 125 / 300ms | std | なし |
| MobileNav | パネル / backdrop | translateX ±100% / opacity | 300ms | std | 0.01s |
| AppShell | 列のグライド / スワップ | translate / opacity | 734.77ms | spring settle | fade200 |
| Stepper | 接続線フィル | scaleX/Y 0→1（区間分配） | 300ms | linear | 0 |
| Stepper | 行 | bg | 95ms | std | 0 |
| Pagination | ドット | bg | 125ms | std | なし |
| Outline | インジケーター | top, height | 95ms | std | なし |
| ChatComposerDrawer | 高さ / 内容 | grid-rows / opacity（展開 delay 125） | 300 / 300 / 125ms | std | なし |
| ChatLayoutScrollButton | 表示 / 幅 | opacity / max-width 32↔200 | 165ms | std | 0 |
| useChatStreamScroll | 追従 | scrollTop | 自前スプリング（約 0.78s/100px） | — | ジャンプ |
| Chat sending | パルス | opacity 1→0.5→1 | 1.5s ∞ | ease-in-out | 静止 |
| ChatToolCalls | シェブロン / 展開 | rotate 180 / grid-rows | 125 / 300ms | std | 0 |
| ChatDictation | 音量バー | scaleY, bg | 60ms | ease-out | 0 |
| Markdown | ストリーミング | 10 文字/10ms tick | — | — | 即時に全文 |
| Markdown | チャンクのフェード | opacity 0→1 | 300ms | std | 0 |
| useContainerReveal | 表示 | opacity（+ position 離散） | 125ms | std | 0 |

---

## 6. Kivact 実装ノート

調査対象は `C:\w\kashiwade\kivact\crates`。公開窓口は `kivact::motion` で、**prelude には含まれない**ので `use kivact::motion::…` と書く（`kivact-core/src/motion.rs:10-13`）。

### 6.1 Kivact が提供するもの（引用。存在しない API を作らないこと）

- **`Transition`**（`kivact-motion/src/transition.rs`）
  - `Transition::tween(ms, Easing)` :127
  - `Transition::spring(Spring)` :135
  - `Transition::keyframes(frames: [(offset f32, T)], ms)` :141。区間内の補間は**線形のみ**（`kivact-wire/src/motion.rs:204-224`）。
  - `.delay(ms)` :161（最初の 1 回だけ）
  - `.repeat(Repeat::Times(n) | Repeat::Forever)` :169
  - `.alternate()` :176
  - `.decorative(bool)` :186（既定 true。RM で即時に終わる）
- **`Easing`**（`kivact-types/src/motion.rs:22-95`）
  - バリアントは `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, `CubicBezier(x1,y1,x2,y2)`, `Steps{…}`。
  - **`EaseIn` / `EaseOut` / `EaseInOut` は三次多項式で、CSS の同名カーブとは別物**。Astryx のカーブは必ず `CubicBezier` で指定すること。
  - CubicBezier の y は 0..1 の外に出てよい（オーバーシュートも可）。
- **`Spring { stiffness: f32, damping: f32, mass: f32 }`**（`kivact-types/src/motion.rs:99-129`）
  - プリセットは GENTLE / WOBBLY / STIFF / DEFAULT。Astryx の値ではないので**使わない**。
  - 積分は半陰的 Euler、1ms サブステップ（`kivact-wire/src/motion.rs:227-239`）。
  - 収束条件は全成分で |x−to| < 1e-3 **かつ** |v| < 1e-2（絶対値。単位系に依存しない固定値）。
- **`Animated<T>`**（`kivact-motion/src/animated.rs`）
  - `animated(target, t)` :220（初回はアニメーションしない）
  - `Animated::from(start, target, t)` :227
  - `get` / `get_untracked` / `jump_to`（速度を 0 にする）/ `pause` / `resume` / `on_complete` / `is_running`
  - リターゲットは現在値から始まる。**spring は速度を保持し、tween は全 duration をやり直す**（`animated.rs:116-141`、`kivact-wire/src/motion.rs:99-104`）。
  - **速度の読み出し・注入 API はない**。
- **Animatable**: f32、f64、2〜4 要素のタプル、Point、Size、Rect、Color（sRGB の単純 lerp）、Transform。
- **スタイル遷移**: `Style::transition(TransitionProp::Paint, t)`（`kivact-style/src/sheet.rs:150, :895`）。
  - 補間するもの: opacity、単色背景、border 色、transform の各成分と origin、先頭 2 つの shadow、filter、文字色。
  - **補間しないもの**: グラデーション、radius、clip_path、mask、backdrop-filter、レイアウトプロパティ全般。
  - hover / pressed / focus / selected / group の状態変化はすべてこの遷移に乗る（`kivact-engine/src/model/state_motion.rs`）。
- **登場・退場**: `Style::enter(Style)` :869（CSS `@starting-style` 相当）、`Style::exit(Style)` :889。
  - 退場中のノードは、ヒットテスト・フォーカス・a11y からは外れるが、**レイアウト上の位置は保持する**。保持させないなら `Position::Absolute` にする。
  - 退場の完了コールバックはない。
- **フレームループ**: `frame_loop(|FrameInfo| …)` / `FrameLoop::start(FrameRate)`（`kivact-motion/src/frame_loop.rs:346-417`）。
  - RM では自動停止しない。
  - クロックは `kivact::primitive::clock_now()`（ns）。
- **Reduce Motion**: `kivact::platform::reduced_motion()`（tracked、`kivact-core/src/platform/env.rs:54`）。
  - **OS 検出は Windows のみ**（SPI_GETCLIENTAREAANIMATION）。他の OS では常に false。
  - アプリ側の上書き手段もない。
- **ジェスチャー**
  - `use_drag`（`kivact-interaction/src/drag.rs:293`）: slop なし、速度なし、軸ロックなし。
  - `PointerEvent` にタイムスタンプはない。
  - `PointerKind::{Mouse, Touch, Pen}` はイベント単位で取れる（`kivact-types/src/platform.rs:30`）。
- **スムーズスクロール**: `NodeRef::scroll_to/scroll_by(.., ScrollBehavior::Smooth)` は固定 240ms（`node_ref.rs:229-234`）。
- **クリップ**: `clip_path(Shape)`（`Shape::inset` あり、`kivact-style/src/effect.rs`）。遷移では補間されない。

### 6.2 Astryx のプリミティブと Kivact の対応

| Astryx プリミティブ | Kivact での実装 | 状態 |
|---|---|---|
| CSS transition（bezier）: hover 色、枠、shadow、opacity、transform | `style.transition(TransitionProp::Paint, Transition::tween(125, Easing::CubicBezier(0.24,1.0,0.4,1.0)))`。状態スタイルは `.hover()` / `.pressed()` / `.focus()` / `.selected()` | **対応**。ただしグラデーション背景、radius、clip は補間されない（Astryx でもグラデーションは実質即時なので一致する） |
| 状態ごとに異なる遷移（press の立ち上げは即時、release だけ 200ms で戻すなど） | ノードの遷移は 1 つで、方向に関係なく共通（`state_motion.rs` 冒頭）。状態スタイル内に別の transition を書けるかは **UNRESOLVED** | **GAP の可能性**。回避策: press の強度を `Animated<f32>` で持ち、pointer down で `jump_to(1.0)`、up で目標 0 + tween(200, CubicBezier(0,0,0.2,1)) にして、style クロージャで色を混ぜる |
| CSS transition の中断（現在値から再開。逆転時は残り時間に短縮） | tween は現在値から**全 duration をやり直す** | **差異**。体感差は小さいが、短縮規則を再現するなら自前で duration を縮める |
| `@starting-style` による登場 | `Style::enter(...)` + Paint 遷移 | **対応** |
| 退場（Drawer, BottomSheet, MobileNav, Toast, MegaMenu, Overlay） | `Style::exit(...)` + Paint 遷移 | **一部対応**: paint のみ。退場中もレイアウト位置を保持する。完了コールバックなし |
| Toast 行の grid-template-rows 0fr↔1fr、CodeBlock / SideNavItem / ChatToolCalls / ChatComposerDrawer の高さ開閉 | レイアウトプロパティの遷移はない。`Animated<f32>` の高さを style クロージャで毎フレーム与えればよいが、**自然な高さ（auto）を測る同期 API がない** | **GAP**（auto 高さアニメーション） |
| スプリング（response/ζ、duration/bounce、m/k/c） | `Transition::spring(Spring{stiffness, damping, mass})`。§6.3 の変換を使う | **対応**（パラメータの記述形式は自前で変換） |
| スプリングのリターゲットで速度を保持 | spring の `Animated` はリターゲットで速度を保持する | **対応** |
| 指を離したときの速度を渡す（Switch travel、Slider coast、sheet） | `Animated` に速度を注入する手段がない。`jump_to` は速度 0 | **GAP**。回避策: (a) ドラッグ中も「目標 = 指」の spring `Animated` を動かし続け、release で目標だけ替える（Switch の tracking 0.36/1 はまさにこの方式で、Astryx と一致する）。(b) `frame_loop` で §1.5 の閉形式を自前で評価する |
| 速度の読み出し（`getVelocity`） | なし | **GAP** |
| decay / coast（Slider coast は spring で代用。Astryx の実使用は Slider の decay 呼び出しだけ） | なし | **GAP**。`frame_loop` で x(t) = x0 + v0(1−e^(−λt))/λ を評価する |
| ラバーバンド | なし | **GAP**（純関数なので §1.8 の式を移植するだけ） |
| ページング積分 / snap | なし。Kivact の Carousel はインデックスを tween で動かすだけ | **GAP**。ただし Astryx 側も Carousel は**ネイティブの scroll-snap**なので、kivact-ui-soft の Carousel に必要なのは「smooth な scrollBy」と「snap」。Kivact には snap もない → **GAP** |
| scroll-snap（Carousel, MonthScroller, Wheel） | なし | **GAP** |
| 慣性スクロール | なし（タッチパッドもピクセル差分なし） | **GAP** |
| FLIP（animatedLayout: 測定 → translate グライド → 速度連続の加算） | layout 遷移なし、layoutId なし。`NodeRef::rect()` は非同期で、描画後の境界 | **GAP**（最大の欠落）。変化前に同期で測る手段がなく、1 フレームのちらつき（flash）が出る可能性がある（未検証、UNRESOLVED）。Collapsible / TreeList / Table / SideNav / AppShell の「ハンドルあり」モードはこれに依存する。Astryx でも**ハンドルなしが既定**（即時切替）なので、まず即時版で実装し、FLIP は後回しにしてよい |
| エッジ reveal（clip-path inset + opacity を spring で） | `clip_path(Shape::inset(..))` を `Animated<f32>` から style クロージャで毎フレーム組み立てる | **一部対応**（手動駆動。グライドに乗せる部分は FLIP 待ち） |
| transient copy（AppShell スワップ、SideNav レール） | 旧ツリーを `exit` で残し、新ツリーを `enter` で重ねる（両方 absolute） | **一部対応**（スナップショット機構はない） |
| 無限ループ: Spinner | `Animated::from(0.0, 360.0, Transition::tween(525, Easing::Linear).repeat(Repeat::Forever).decorative(false))` + `rotate`（`kivact-ui-basic/src/primitives.rs:176-187` と同じパターン）。弧長 0.375 の円弧を描画 | **対応**。ただし RM では止めずに **3000ms へ切り替える**（`decorative(false)` + `reduced_motion()` で duration を選ぶ）。全スピナーの位相同期は、`clock_now()` からの角度 = (t mod 525ms)/525×360 を `frame_loop` で計算すれば再現できる |
| 無限ループ: Skeleton | `Transition::keyframes([(0.0,0.25),(0.0999,0.25),(0.1,0.325),…,(1.0,0.925)], 400).repeat(Forever).alternate().delay(1000 + 100*i)` | **対応**（keyframes は線形補間なので保持点を 2 点で表す。delay は初回のみで、CSS と一致） |
| 無限ループ: StatusDot / Chat sending | `Animated::from(1.0, 0.5, tween(1000 または 750, CubicBezier(0.42,0,0.58,1)).repeat(Forever).alternate())` | **対応**（CSS のキーフレーム区間ごとの ease-in-out と等価） |
| 無限ループ: ProgressBar indeterminate | `Animated::from(-1.0, 2.5, tween(1500, CubicBezier(0.42,0,0.58,1)).repeat(Forever))` を幅 40% のバーの translate_x（自身の幅に対する %）に使う。RM では opacity 1↔0.84、2000ms io の alternate | **対応** |
| shimmer | Astryx には存在しない | — |
| Reduce Motion: fine では即時 / coarse では 200ms フェード | `reduced_motion()`（Windows のみ）。**「主ポインタが coarse か」を問い合わせる API は見つからない**（UNRESOLVED） | **GAP**。デスクトップ（Kivact の主用途）は fine 扱いにして「即時」で統一するのが妥当。coarse 分岐は検出 API ができるまで保留 |
| decorative = RM で即時終了 | 既定で `decorative(true)` なので、DR3（fine では即時）と一致 | **対応**。Spinner と ProgressBar のパルスは `decorative(false)` |
| hover intent の遅延（Tooltip 200/100、HoverCard 300/200、submenu 150/200） | `use_delayed`（`kivact-interaction/src/delayed.rs:79`） | **対応**（safe triangle は自前） |
| 長押し 500ms / 10px、pan のヒステリシス 10px、swipe、pinch | 認識器なし。`use_drag` には slop もない | **GAP**（§3 の閾値で自前実装） |
| VelocityTracker（0.2/0.8 ブレンド、40ms の静止判定） | なし。PointerEvent にタイムスタンプもない（`clock_now()` で代用） | **GAP** |
| スタッガー | 子ごとに `.delay(i * step)` | **一部対応** |
| シーケンス（press の lift → hold 235ms → settle、Switch の合成曲線） | タイムラインはない。`on_complete` でつなぐか、`frame_loop` で合成曲線を評価する | **GAP**（自前） |
| View Transitions | なし（Astryx も使わない） | 不要 |
| scroll-driven animation（Wheel のフォールオフ） | なし。スクロール位置を signal で読んで style を計算する | **一部対応**（自前） |
| 3D 変換（Wheel の rotateX 52°、perspective 520px） | Transform は 2D（translate、rotate、skew、scale）。rotateX はない | **GAP**（scaleY と opacity で近似するか、見送る） |
| smooth scroll | `ScrollBehavior::Smooth` は固定 240ms | **差異**（ブラウザも UA 依存なので許容） |
| アンカーポジショニングの top / height 遷移（Outline） | レイアウト遷移なし。インジケーターを absolute にし、`Animated<(f32,f32)>`（top, height）を spring または tween(95, std) で動かす | **一部対応**（`NodeRef::rect` が非同期で、遅れる可能性がある） |

### 6.3 実装上の具体指示

1. **スプリングの変換**は kivact-ui-soft 内の純関数として書く（Kivact に API を追加するのではない）。
   - (response r, ζ) → `Spring { stiffness: (TAU/r)², damping: 2ζ·TAU/r, mass: 1.0 }`
   - (duration d, bounce b≥0) → `Spring { stiffness: (TAU/d)², damping: (1−b)·4π/d, mass: 1.0 }`
   - (m, k, c) は、c > 2√(km) のとき c = 2√(km) にクランプする（`springs.viewChange` は c = 109.545）。
   - §1.3 の表の k / c を、そのまま定数として持ってもよい。
2. **収束閾値の差**: Kivact は 1e-3 / 1e-2 固定、Astryx は px 0.05、scale 1e-4、opacity 1e-3。
   - Kivact の方が px では厳しく（尾が少し長い）、scale ではやや緩い。視覚的な差は無視してよい。
   - ただし `on_complete` で後続をつなぐ場合は、時間がずれることを前提にする。
3. **カーブ定数**は `const EASE_STANDARD: Easing = Easing::CubicBezier(0.24, 1.0, 0.4, 1.0)` のように一元化する。
   - EXIT (0.3,0,0.6,0.6)、MOVE/IO (0.42,0,0.58,1)、RELEASE (0,0,0.2,1)、CSS_EASE (0.25,0.1,0.25,1)、CSS_EASE_OUT (0,0,0.58,1)、CSS_EASE_IN_OUT (0.42,0,0.58,1) も同様にする。
4. **時間定数**は Neutral の値を固定で持つ。
   - FAST_MIN 95、FAST 125、FAST_MAX 165、MEDIUM_MIN 225、MEDIUM 300、MEDIUM_MAX 400、SLOW_MIN 525、SLOW 700、SLOW_MAX 935。
   - Spinner と Skeleton は pre-tempo の値（= 同じ数値）を使う。
5. **押下の状態色**は Paint 遷移 125ms std で付ける。タッチ用の release フェード（200ms、release 曲線）と onset 150ms / flash 100ms は、PointerKind::Touch のときだけ `Animated` で自前実装する。
6. **退場がないコンポーネントに退場を付けない**。アンカーレイヤー、Dialog、Lightbox、Banner、FieldStatus、Button の spinner は即時に消す。Astryx と一致させること。
7. **FLIP 系は段階的に進める**。まず「ハンドルなし」（即時の開閉、シェブロンだけ 125ms）で全コンポーネントを揃え、animatedLayout 相当は Kivact 側の FLIP 基盤（変化前の同期測定 + translate グライド + 残り曲線の加算）ができてから着手する。
8. RM の分岐は `kivact::platform::reduced_motion()` を読み、§2.3 の「fine 側」の列（即時 / 0s）に従う。Spinner は 3s にする。ProgressBar の不確定値はパルスにする。

---

## 7. UNRESOLVED と不整合（Codex が判断で埋めてはならない箇所）

1. **AST-090 の退場トークン**（`--duration-exit`, `--duration-exit-dialog`, `--duration-exit-layer`）は、spec では accepted だがコードにない。現状どおり「退場なし」で実装し、spec 側の決定を待つ。
2. **AST-108 FR3**（スプリングが tempo / character で変形）は未実装。Neutral は tempo も character も使わないので影響はない。
3. **Kivact で「主ポインタが coarse か」を調べる手段**が見つからない。また、状態スタイルごとに異なる遷移を持てるかどうかも未確認。
4. **Kivact の FLIP 用の同期測定**ができるか（1 フレームのちらつきが出るか）は未検証。
5. `useStreamingText` の JSDoc（「約 2 文字/フレーム」）はコード（10 文字/tick）と食い違う。**コードの値を採用する**。
6. Markdown の同時 span 上限 12 は fast-max（165ms）基準で決まっているのに、フェード自体は medium（300ms）である。そのため 60Hz では約 200ms 後にフェードが途中で 1 にスナップする [推論]。Astryx の実挙動として、そのまま再現する。
7. MobileNav の backdrop は close（180ms）の時点で消えるため、パネル（300ms）より先に消える [推論]。
8. BottomSheet は、settle 中に指が触れると表示位置ではなく settle 目標へジャンプする [推論]。
9. Button の spinner 出現の CSS `ease`（timing function 未指定）は意図的かどうか不明。そのまま `ease` (0.25,0.1,0.25,1) を使う。
10. TabList の `scroll-behavior: smooth` と Carousel、Outline の smooth スクロールは、時間・カーブとも UA 定義である。Kivact では `ScrollBehavior::Smooth`（240ms）で代用する。
11. layerAnimations は、配置フォールバックで反転しても方向を再計算しない（Astryx のバグの可能性）。そのまま再現する。
12. UIKit の `springs.*` のうち未使用のもの（sheet, menu, alert, banner, segmentedThumb）を**使わない**。Astryx の現在の挙動は CSS である。
