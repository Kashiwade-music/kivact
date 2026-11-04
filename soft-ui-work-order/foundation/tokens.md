# Astryx Neutral テーマ デザイン基盤トークン（kivact-ui-soft 再現用）

対象: Astryx (Meta, React + StyleX) の docsite（https://astryx.atmeta.com/components）がすべてのコンポーネントプレビューに使っている **Neutral テーマ**（`@astryxdesign/theme-neutral` v0.7.7、`packages/themes/neutral`）。ライトモードとダークモードの両方を扱う。

調査元は `D:\w\meta\astryx` のクローン（読み取りのみ。コードは実行していない。`node_modules` は存在しないので、lucide-react のパスデータなど依存パッケージの中身は確認できていない）。

この文書の値の読み方:

- 値は **解決済みの最終値**。Neutral が上書きしたトークンは Neutral の値を、上書きしていないトークンは core のデフォルト値（`packages/core/src/theme/tokens.stylex.ts`）を書いている。
- `[L, D]` は `[ライト, ダーク]` の意味。
- 8 桁 hex の `#RRGGBBAA` は末尾 2 桁がアルファ値。対応する `rgba()` も併記している。
- 「手計算」と書いた値は、ソースの式（color-mix、アルファ合成、スケール展開）を自分で解いたもの。計算は Python の自作スクリプトで行い、astryx のコードは一切実行していない。
- px 値はルートフォントサイズ 16px を前提に rem から換算している。
- 解決できなかった項目には **UNRESOLVED** と書き、理由を添えた。

---

## 1. トークンアーキテクチャの概要

### 1.1 三層構造

1. **パレット（生の色）**
   - 実体は `packages/themes/neutral/src/neutralPalettes.generated.ts`。レシピ `astryx-oklch-v1`、vibrancy 50 で生成されている。
   - 色ファミリーは 10 個: neutral, red, orange, yellow, green, teal, cyan, blue, purple, pink。
   - 各ファミリーに light ランプと dark ランプがあり、それぞれ 21 段階（0, 5, 10, …, 100）。
   - テーマのソースが実際に参照するのは、その中から抜き出した `neutralPaletteRefs.generated.ts` の値だけ。
   - **コンポーネントはパレットを直接参照しない。**
2. **セマンティックトークン（CSS カスタムプロパティ）**
   - `--color-text-primary` や `--radius-element` のような名前。
   - core が `tokens.stylex.ts` で `stylex.defineVars()` によって宣言し、デフォルト値を持たせている。
   - テーマは `defineTheme({tokens: {...}})` でこれを上書きする。`[light, dark]` のタプルは CSS の `light-dark(light, dark)` に変換される。ページが `color-scheme: light` か `dark` かによって、CSS がどちらかを選ぶ仕組み。
3. **コンポーネント上書き**
   - `defineTheme({components: {button: {'variant:destructive': {...}}}})` の形で書く。
   - スコープ付き CSS として生成され、`@layer astryx-theme` に入る。
   - Neutral が使っている上書きは §1.4 にまとめた。

コンポーネント側は、たとえば `backgroundColor: colorVars['--color-accent']`（コンパイル後は `var(--color-accent)`）のように参照する。テーマを差し替えると、カスケードで値がそのまま入れ替わる。CSS レイヤーの順番は `reset → astryx-base → astryx-theme` で、レイヤー外の利用者 CSS が最も強い。

**Kivact への示唆:** パレットを Rust 側に持つ必要はない。セマンティックトークン（この文書の §13）を `Tokens` 構造体として持ち、ライト用とダーク用の 2 インスタンスを用意すれば十分。

### 1.2 生成ステップ（defineTheme / resolveThemeValues）

後の段ほど優先される。

1. `color`（アクセントからのスケール生成）。Neutral は**使っていない**。
2. `typography.scale` → `expandTypeScale`（§3）。Neutral は `{base: 14, ratio: 1.2}`。
3. `radius` → `expandRadiusScale`。Neutral は使わず、明示トークンで指定している。
4. `motion` → `expandThemeMotion`（§11）。Neutral は `{fast: 125, medium: 300, slow: 700, ratio: 0.75}`。
5. フォントファミリートークン（§3）。
6. `syntax` テーマ（§2.10）。
7. 明示 `tokens`（最優先）。

ステータス派生（`statusRoleDerivation.ts`）について:

- テーマが設定しなかった `--color-on-<status>`、`--color-text-<status>`、`--color-on-<status>-muted` を、コントラスト 4.5:1 を満たすように自動計算する仕組みがある。
- Neutral はこれらを**すべて明示している**ので、自動派生は起きない。
- `--color-icon-<status>` と `--color-border-<status>` は `var(--color-text-<status>)` へのエイリアスになる（`generateThemeRules.ts` の `SCOPED_STATUS_ROLE_DEFAULTS`）。

### 1.3 コンポーネントが使う公開 CSS 変数（全リスト）

`tokens.stylex.ts`、`dataTokens.stylex.ts`、`syntax/tokens.ts` が正典。

- **color (core)**
  - 基本: `--color-accent`, `--color-accent-muted`, `--color-on-accent`, `--color-neutral`, `--color-background-surface`, `--color-background-body`, `--color-overlay`, `--color-overlay-hover`, `--color-overlay-pressed`, `--color-background-muted`
  - テキスト: `--color-text-primary`, `--color-text-secondary`, `--color-text-disabled`, `--color-text-accent`, `--color-on-dark`, `--color-on-light`
  - アイコン: `--color-icon-accent`, `--color-icon-primary`, `--color-icon-secondary`, `--color-icon-disabled`
  - 面: `--color-background-card`, `--color-background-popover`, `--color-background-inverted`, `--color-background-error-inverted`
  - ステータス: `--color-{info,success,warning,error}`, `--color-{…}-muted`, `--color-on-{…}`, `--color-on-{…}-muted`, `--color-text-{…}`, `--color-icon-{…}`, `--color-border-{…}`
  - 境界線: `--color-border`, `--color-border-emphasized`
  - エフェクト: `--color-skeleton`, `--color-track`, `--color-shadow`, `--color-tint-hover`
  - カテゴリ色: `--color-{background,border,icon,text}-{blue,cyan,gray,green,orange,pink,purple,red,teal,yellow}`（10 色 × 4 役割）
- **data (domain)**
  - `--color-data-categorical-{blue,orange,purple,green,pink,cyan,red,teal,brown,indigo}`
  - `--color-data-neutral`
  - `--color-data-{blue,shamrock,orange,pink,purple,red,teal,yellow,gray}-{1..5}`
  - 合計 56 トークン
- **syntax (domain)**
  - `--color-syntax-{keyword,string,comment,number,function,type,variable,operator,constant,tag,attribute,property,punctuation,background}`
  - 差分表示用: `--color-syntax-{added,removed,diff-header,added-marker,removed-marker}`
- **spacing:** `--spacing-0`, `-0-5`, `-1`, `-1-5`, `-2`, `-3`, `-4`, `-5`, `-6`, `-7`, `-8`, `-9`, `-10`, `-11`, `-12`
- **size:** `--size-element-sm`, `--size-element-md`, `--size-element-lg`
- **border:** `--border-width`
- **focus:** `--focus-outline-width`, `--focus-outline-style`, `--focus-outline-color`, `--focus-outline-offset`
- **radius:** `--radius-none`, `--radius-inner`, `--radius-element`, `--radius-container`, `--radius-page`, `--radius-chat`, `--radius-full`
- **shadow:** `--shadow-low`, `--shadow-med`, `--shadow-high`, `--shadow-inset-hover`, `--shadow-inset-selected`, `--shadow-inset-success`, `--shadow-inset-warning`, `--shadow-inset-error`
- **duration:** `--duration-{fast,medium,slow}{-min,,-max}`（9 個）
- **ease:** `--ease-standard`, `--ease-entrance`, `--ease-move`, `--ease-exit`
- **font family:** `--font-family-body`, `--font-family-code`, `--font-family-heading`
- **font size:** `--font-size-{4xs,3xs,2xs,xs,sm,base,lg,xl,2xl,3xl,4xl,5xl}`
- **font weight:** `--font-weight-{normal,medium,semibold,bold}`
- **type scale:**
  - `--text-heading-{1..6}-{size,weight,leading}`
  - `--text-{body,large,label,code,supporting,display-1,display-2,display-3}-{size,weight,leading}`

公開トークンとは別に、テーマが上書きするための中間変数や内部変数もある。代表的なものは `--container-padding-{inline,block}-{start,end}`、`--astryx-card-padding*`、`--astryx-section-padding*`、`--astryx-dialog-padding*`、`--astryx-drawer-padding*`、`--astryx-bottom-sheet-padding*`、`--_press-paint`、`--astryx-press-alpha`、`--_icon-width`、`--_icon-height`、`--_field-radius`、`--_button-radius`、`--_card-radius`、`--_popover-radius`。これらは実装の詳細なので、Kivact では各コンポーネントの内部値として扱えばよい。

- 補足 1: `expandTypeScale.ts` のコメントに出てくる `--leading-tight … --leading-relaxed` は**実在しない**（古いコメント）。
- 補足 2: Neutral はテーマローカルトークン `--astryx-theme-neutral-color-status-fill-accent: [#0074e2, #6d9cfe]` を 1 つだけ出力している。これは非推奨で、Neutral 自身はどこからも参照していない。無視してよい。

### 1.4 Neutral のコンポーネント上書き（`neutralTheme.ts` の `components`）

| 対象 | 上書き内容 |
| --- | --- |
| `button` `variant:destructive` | `backgroundColor: var(--color-error-muted)`、`color: var(--color-on-error-muted)`。core の濃い赤塗りではなく、淡い赤の面に濃い赤の文字になる。 |
| `badge` `variant:neutral` / `gray` | `--color-background-gray` 地 + `--color-text-gray` 文字 |
| `badge` `variant:{red,orange,yellow,green,teal,cyan,blue,purple,pink}` | `--color-background-<hue>` 地 + `--color-text-<hue>` 文字（core と同じ対応付けを明示しているだけ） |
| `segmented-control` base | `padding: var(--spacing-1)`（4px） |
| `segmented-control-item` `size:sm/md/lg` | `height: calc(var(--size-element-<size>) - 8px)` → 20 / 24 / 28px |
| `segmented-control-item` `selected` | `boxShadow: none`（選択中のアイテムに影を付けない。トラックに対して平らに見える） |
| `switch` base | `--color-background-gray: var(--color-border-emphasized)`。OFF 状態のトラック色が #d4d4d4 / #525252 になる。 |
| `progress-bar` base | `--color-background-muted: var(--color-border-emphasized)`。トラック色が #d4d4d4 / #525252 になる。 |
| `card` base | `padding: var(--spacing-3)`（12px。core の既定は 16px） |
| `section` base | `padding: var(--spacing-3)`（12px） |
| `heading` / `text` | type scale から自動生成。h3 と h4 は bold（§3） |

---

## 2. COLOR（色）

### 2.1 Neutral パレットの性質（見た目を決める根拠）

- **neutral ランプは完全な無彩色。** R=G=B で、色相は一切ない。値は次のとおり（light と dark で同じ値）:

  | stop | 色 | stop | 色 | stop | 色 |
  | --- | --- | --- | --- | --- | --- |
  | 0 | #000000 | 35 | #525252 | 70 | #ababab |
  | 5 | #111111 | 40 | #5e5e5e | 75 | #b9b9b9 |
  | 10 | #1b1b1b | 45 | #6a6a6a | 80 | #c6c6c6 |
  | 15 | #262626 | 50 | #777777 | 85 | #d4d4d4 |
  | 20 | #303030 | 55 | #848484 | 90 | #e2e2e2 |
  | 25 | #3b3b3b | 60 | #919191 | 95 | #f1f1f1 |
  | 30 | #474747 | 65 | #9e9e9e | 100 | #ffffff |

  package.json の説明文には "Restrained warm grays" とあるが、実際の値に暖色の偏りはない。
- **アクセントも無彩色。** ライトは #1b1b1b（ほぼ黒）、ダークは #f1f1f1（ほぼ白）。青いアクセントは存在しない。
- 有彩色は次の 3 か所にしか出てこない:
  - ステータス色
  - カテゴリ色（Badge、Token、Card の色バリアント）
  - シンタックスハイライト
- ダーク用の有彩色ランプは stop 5〜25 の彩度を 50% に落とし（yellow は 65%）、stop 60 で標準レシピに戻している（`palette.config.json` の `darkChromaTaper`、spec DEC-3）。

### 2.2 背景・面

| トークン | Light | Dark | 由来 / 備考 |
| --- | --- | --- | --- |
| `--color-background-body` | #f1f1f1 | #1b1b1b | neutral.light95 / neutral.dark10。ページの地。 |
| `--color-background-surface` | #ffffff | #262626 | neutral.light100 / neutral.dark15。主要な面（入力欄、ダイアログ、Section）。 |
| `--color-background-card` | #ffffff | #1b1b1b | **ダークでは surface より暗く、body と同じ色。** |
| `--color-background-popover` | #ffffff | #1b1b1b | ダークでは body と同じ。浮いている感じは影の内側リム（§7）で出す。 |
| `--color-background-muted` | #f1f1f1 | #1b1b1b | muted な Section、無効状態の未チェック Checkbox、コードの差分ヘッダー |
| `--color-background-inverted` | #0A1317 | #FFFFFF | **Neutral は上書きしていない。core の既定値**（ライトはわずかに青みがかった黒）。Toast で使う。 |
| `--color-background-error-inverted` | #AA071E | #E3193B | Neutral は上書きしていない。core の既定値。エラー Toast で使う。 |
| `--color-neutral` | #0000000F = rgba(0,0,0,0.0588) | #ffffff1A = rgba(255,255,255,0.1020) | 半透明の中立面。secondary Button、Avatar、Token の neutral 色に使う。 |
| `--color-accent-muted` | #f1f1f1 | #262626 | 選択中の行（Item selected）、入力フォーカス時の内側リング |

不透明に合成した参考値（手計算。`--color-neutral` は surface 上で L #f0f0f0 / D #3c3c3c、body 上で L #e3e3e3 / D #323232）は §2.8 にまとめた。

### 2.3 テキスト・アイコン

| トークン | Light | Dark | 由来 |
| --- | --- | --- | --- |
| `--color-text-primary` | #000000 | #ffffff | neutral.light0 / neutral.dark100。**純黒と純白。** |
| `--color-text-secondary` | #474747 | #9e9e9e | light30 / dark65。placeholder もこの色。 |
| `--color-text-disabled` | #919191 | #525252 | light60 / dark35 |
| `--color-text-accent` | #1b1b1b | #f1f1f1 | アクセント文字。Link の accent 色など。 |
| `--color-on-accent` | #ffffff | #111111 | アクセント塗りの上に載せる文字 |
| `--color-on-dark` | #ffffff | #ffffff | 暗いメディア面の上の文字 |
| `--color-on-light` | #111111 | #111111 | 明るいメディア面の上の文字 |
| `--color-icon-primary` | #000000 | #ffffff | |
| `--color-icon-secondary` | #6a6a6a | #9e9e9e | **ライトでは text-secondary（#474747）より明るい。** |
| `--color-icon-disabled` | #919191 | #525252 | |
| `--color-icon-accent` | #1b1b1b | #f1f1f1 | ただし Icon の `color="accent"` は `--color-accent` を参照する（値は同じ）。 |

Link の色は Text と同じセマンティック色（primary / secondary / accent）を使う。core には「リンク専用の青」は存在しない。

### 2.4 境界線・エフェクト

| トークン | Light | Dark | 備考 |
| --- | --- | --- | --- |
| `--color-border` | #00000014 = rgba(0,0,0,0.0784) | #ffffff1A = rgba(255,255,255,0.1020) | 区切り線、Card の 1px 枠、`hr`。不透明換算: surface 上で L #ebebeb / D #3c3c3c。 |
| `--color-border-emphasized` | #d4d4d4 | #525252 | 入力欄の枠、Checkbox の枠。Switch の OFF トラックと ProgressBar のトラックにも転用される。 |
| `--color-skeleton` | #f1f1f1 | #525252 | Skeleton の地。`prefers-contrast: more` のときは text-primary を 30% 混ぜる → L #a9a9a9 / D #868686（手計算）。 |
| `--color-track` | #CCD3DB | #5A5E66 | **Neutral は上書きしていない。core の既定値**で、わずかに青みがある。Slider のレールと Spinner のトラックに使う。 |
| `--color-shadow` | #0000001A | #0000004D | Neutral の影トークン自体はこれを参照せず、リテラルの黒を使っている（§7）。 |
| `--color-tint-hover` | #000000 | #ffffff | color-mix 方式のホバーで混ぜる色（§9.3） |
| `--color-overlay` | #00000080 = rgba(0,0,0,0.502) | #000000CC = rgba(0,0,0,0.8) | モーダルのスクリム。Dialog の `::backdrop` はさらに `backdrop-filter: blur(2px)` をかける。 |
| `--color-overlay-hover` | #0000000D = rgba(0,0,0,0.051) | #ffffff0D = rgba(255,255,255,0.051) | ホバー時に重ねるオーバーレイ層 |
| `--color-overlay-pressed` | #0000001A = rgba(0,0,0,0.102) | #ffffff1A = rgba(255,255,255,0.102) | 押下時に重ねるオーバーレイ層 |

### 2.5 フォーカスリング色

`--focus-outline-color` の値は `var(--color-accent)` なので、L **#1b1b1b** / D **#f1f1f1**。詳しくは §8。

例外として、destructive Button のフォーカスリングは `--color-error`（L #c9303a / D #ff705d）になる。

### 2.6 ステータス色（info / success / warning / error）

Astryx では danger を `error` と呼ぶ。`neutral` ステータスは `--color-neutral` が担う。

| 役割 | 用途 | info | success | warning | error |
| --- | --- | --- | --- | --- | --- |
| `--color-<s>` | 濃い塗り（Badge、StatusDot） | L #0074e2 / D #6d9cfe | L #198100 / D #64af4c | L #ffce2f / D #ffce2f | L #c9303a / D #ff705d |
| `--color-on-<s>` | 濃い塗りの上の文字 | L #ffffff / D #111111 | L #ffffff / D #111111 | L #111111 / D #111111 | L #ffffff / D #111111 |
| `--color-<s>-muted` | 淡い塗り（Banner、淡い destructive Button） | L #b9d7ff / D #88bcff3D | L #bce0bb / D #90ca903D | L #fae19e / D #e2b6233D | L #ffc4be / D #ff98903D |
| `--color-on-<s>-muted` | 淡い塗りの上の文字 | L #003978 / D #a1caff | L #00490b / D #90ca90 | L #4b3900 / D #eec448 | L #76000c / D #ffaea7 |
| `--color-text-<s>` | ページやカード上のステータス文字 | L #0067ce / D #a1caff | L #2f7d33 / D #90ca90 | L #836700 / D #eec448 | L #c0262c / D #ffaea7 |
| `--color-icon-<s>` | `var(--color-text-<s>)` と同値 | 上と同じ | 上と同じ | 上と同じ | 上と同じ |
| `--color-border-<s>` | `var(--color-text-<s>)` と同値。invalid な入力欄の枠など。 | 上と同じ | 上と同じ | 上と同じ | 上と同じ |

ダークの muted はアルファ 0x3D = 0.2392 の半透明色。不透明換算（手計算）は次のとおり:

| muted 色 | surface #262626 上 | card / body #1b1b1b 上 |
| --- | --- | --- |
| info-muted | #3d4a5a | #354252 |
| success-muted | #3f4d3f | #374537 |
| warning-muted | #534825 | #4b401d |
| error-muted | #5a413f | #523937 |

濃い塗りの色はパレットに一致する stop がなく、明示値になっている（spec DEC-6）。

### 2.7 カテゴリ色（Badge / Token / Card 色バリアント / Icon 色）

Badge の **14 の組み込みバリアント**は次のとおり（`packages/core/src/Badge/Badge.tsx`）:

- 意味を持つ 5 種: `neutral`, `info`, `success`, `warning`, `error`。濃い塗り（§2.6 の `--color-<s>` + `--color-on-<s>`）を使う。
- 色名の 9 種: `blue`, `cyan`, `green`, `orange`, `pink`, `purple`, `red`, `teal`, `yellow`。淡い地 + 濃い文字を使う。

Neutral はさらに `gray` を定義し、`neutral` を `gray` と同じ配色に差し替えている。Token（チップ）は `neutral` に加えて上記 9 色と `gray`。Card の色バリアントは `--color-background-<hue>` のみを使う。Avatar はパレット色を使わず、`--color-neutral` 地 + `--color-text-secondary` 文字。

| 色 | background L / D | border L / D | icon L / D | text L / D |
| --- | --- | --- | --- | --- |
| red | #ffc4be / #5b2b28 | #ffaea7 / #fa6762 | #76000c / #ff9890 | #76000c / #ffaea7 |
| orange | #ffc7a1 / #503424 | #ffc7a1 / #df843f | #622700 / #ff9e55 | #622700 / #ffb37e |
| yellow | #fae19e / #453a1c | #e3c36c / #c29900 | #4b3900 / #e2b623 | #4b3900 / #eec448 |
| green | #bce0bb / #2b422b | #aad4a9 / #6ab26b | #00490b / #90ca90 | #00490b / #90ca90 |
| teal | #a9e2d6 / #28413c | #90d7c8 / #4fb1a0 | #00463d / #81c9bb | #00463d / #90d7c8 |
| cyan | #9ae2f4 / #274046 | #85d5e9 / #49adc4 | #004351 / #71c7dd | #004351 / #85d5e9 |
| blue | #b9d7ff / #253c5a | #a1caff / #5aa0f8 | #003978 / #88bcff | #003978 / #a1caff |
| purple | #f7d5ff / #4a2f51 | #f3bfff / #d885eb | #5c0e6c / #e496f6 | #5c0e6c / #eaacf8 |
| pink | #ffc0d7 / #572b3d | #ffc0d7 / #fc78b1 | #70003f / #fd92bd | #70003f / #ffa9ca |
| gray | #e2e2e2 / #303030 | #d4d4d4 / #262626 | #474747 / #9e9e9e | #1b1b1b / #d4d4d4 |

見た目の特徴:

- ライトはパステル地（stop 85〜90）に非常に濃い文字（stop 25）。
- ダークは彩度を落とした暗い地（dark stop 25）に明るいパステル文字（stop 75〜80）。
- すべて不透明色（core 既定のような 33 アルファの半透明は使っていない）。

Badge の形状: 高さ `--spacing-5` = 20px、左右パディング 8px、要素間 gap 4px、角丸 `--radius-full`（ピル形）、文字は supporting サイズ 12px / 行高 20px / weight 500。

### 2.8 インタラクション用オーバーレイの描き方（要点）

ホバーと押下の描き方には 2 系統ある。どちらも `--color-overlay-*` や `--color-tint-hover` を使う。

1. **オーバーレイ層方式**（Button、Item、メニュー行、Tab、Table 行、Card など大半）
   - 地の色（`background-color`）はそのまま残す。
   - その上に `background-image: linear-gradient(<overlay>, <overlay>)` を重ねる。
   - ホバー: `--color-overlay-hover`（黒または白の 5.1%）。`@media (hover: hover)` の環境でだけ効く。
   - 押下: `--color-overlay-pressed`（黒または白の 10.2%）。マウスでは `:active` のとき。
   - 地が `--color-neutral` の場合（secondary Button）は、オーバーレイの下にさらに neutral の層を重ねる。
2. **color-mix 方式**（Checkbox、Radio のインジケータ、Switch、Slider のつまみ、Link の文字色）
   - `color-mix(in srgb, <基準色>, var(--color-tint-hover) N%)` で色そのものを置き換える（§9.3）。

合成結果の参考値（手計算、不透明換算）:

| 対象 | rest | hover | pressed |
| --- | --- | --- | --- |
| secondary Button（neutral 地、surface 上）L | #f0f0f0 | #e4e4e4 | #d8d8d8 |
| secondary Button D | #3c3c3c | #464646 | #505050 |
| ghost Button / Item（surface 上）L | #ffffff | #f2f2f2 | #e5e5e5 |
| ghost Button / Item D | #262626 | #313131 | #3c3c3c |
| primary Button L（#1b1b1b に黒を重ねる） | #1b1b1b | #1a1a1a | #181818 |
| primary Button D（#f1f1f1 に白を重ねる） | #f1f1f1 | #f2f2f2 | #f2f2f2 |
| destructive Button L（error-muted 地） | #ffc4be | #f2bab4 | #e5b0ab |
| destructive Button D（surface 上） | #5a413f | #624b49 | #6b5453 |

**注意:** primary Button はオーバーレイがアクセントとほぼ同じ色になるので、ホバーしても見た目はほとんど変わらない。目に見える反応は押下時の `scale(0.98)` だけ。これがソースどおりの挙動。

### 2.9 データ可視化色（domain）

**Neutral は data トークンを上書きしていない。** 値は core の既定値で、すべてライトとダークで同じ（gray と neutral だけはダーク用の値が別にある）。

- categorical:

  | 名前 | 値 | 名前 | 値 |
  | --- | --- | --- | --- |
  | blue | #0171E3 | cyan | #0171A4 |
  | orange | #EB6E00 | red | #F5394F |
  | purple | #6B1EFD | teal | #08A3A3 |
  | green | #0B991F | brown | #965E03 |
  | pink | #F351C0 | indigo | #6F8AFF |

- `--color-data-neutral`: L #8494A3 / D #8C939B
- 連続ランプ（5 が最も濃く、1 が最も淡い）:

  | ランプ | 5 | 4 | 3 | 2 | 1 |
  | --- | --- | --- | --- | --- | --- |
  | blue | #02165E | #004CBC | #2694FE | #78BEFF | #DBECFF |
  | shamrock | #0B603D | #138546 | #24BB5E | #8EF7AA | #D6FEE4 |
  | orange | #A13F04 | #D66100 | #FD9537 | #FDB876 | #FFE6CF |
  | pink | #8E1073 | #D123A1 | #F989D3 | #FEADE3 | #FCE3F4 |
  | purple | #3E0697 | #6B1EFD | #9081FF | #B3B0FE | #E8E8FB |
  | red | #9D0519 | #D31130 | #FB7D87 | #FFB2B8 | #FEE4E6 |
  | teal | #08767D | #0C9293 | #0DB7AF | #6CE6D8 | #D7FCF8 |
  | yellow | #8A5001 | #D69804 | #FBCE03 | #FCEC85 | #FDF6BA |
  | gray L | #25363F | #5D6C7B | #AFB9C4 | #CCD3DB | #F1F4F7 |
  | gray D | #333338 | #666A72 | #B2B8BE | #D0D3D6 | #F2F4F6 |

### 2.10 シンタックスハイライト（`neutralSyntax`、`astryx-neutral`）

| スロット | Light | Dark |
| --- | --- | --- |
| keyword | #6b187c | #efa8ff |
| type | #6b187c | #efa8ff |
| string | #0b5615 | #a4d6a3 |
| comment | #6a6a6a | #9e9e9e |
| operator | #6a6a6a | #9e9e9e |
| punctuation | #6a6a6a | #9e9e9e |
| number | #733100 | #ffb37e |
| constant | #733100 | #ffb37e |
| function | #00458c | #a1caff |
| variable | #111111 | #e2e2e2 |
| tag | #8a0011 | #ffaea7 |
| attribute | #584400 | #eec448 |
| property | #005348 | #90d7c8 |
| background | #ffffff | #111111 |
| added（差分の追加行の地） | #bce0bb40（alpha 0.25） | #90ca900F（alpha 0.0598） |
| removed（差分の削除行の地） | #ffc4be40（alpha 0.25） | #ff98900F（alpha 0.0598） |
| diff-header | transparent | transparent |
| added-marker / removed-marker | punctuation と同じ（#6a6a6a） | punctuation と同じ（#9e9e9e） |

- added と removed は既定式 `color-mix(in srgb, var(--color-success-muted) 25%, transparent)`（error も同様）を手計算したもの。transparent と混ぜるので、色はそのままでアルファだけが 0.25 倍になる。
- diff-header とマーカーは `defineSyntaxTheme` の規則に従う。テーマが値を設定しないと、diff-header は transparent、マーカーは `var(--color-syntax-punctuation)` になる。

---

## 3. TYPOGRAPHY（タイポグラフィ）

### 3.1 フォントファミリー

| トークン | 値（解決済み） |
| --- | --- |
| `--font-family-body` | `Figtree, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif` |
| `--font-family-heading` | 上と同じ（Figtree） |
| `--font-family-code` | `ui-monospace, "SF Mono", Monaco, Consolas, "Liberation Mono", "Courier New", monospace` |

- `buildFontFamily` は名前に空白が含まれる場合だけ引用符で囲む。Figtree は空白を含まないので、引用符なしになる。
- `Theme` のラッパーは `color: var(--color-text-primary)` と `font-family: var(--font-family-body)` を設定する。

フォントファイルについて（`packages/themes/neutral/fonts.css`）:

- 中身は `@import url('https://fonts.googleapis.com/css2?family=Figtree:wght@400;500;600;700&display=swap');` の 1 行だけ。
- 使うウェイトは 400 / 500 / 600 / 700。イタリックは読み込んでいない。
- リポジトリにフォントファイルは同梱されていない。README にも「Figtree は別途読み込みが必要。読み込まない場合はシステムフォントにフォールバックする」とある。
- **ライセンスは UNRESOLVED。** リポジトリに記載がない。一般知識としては、Figtree（Erik Kennedy 作）は SIL Open Font License 1.1 で配布されている。Kivact に同梱する場合は Google Fonts / GitHub から入手し、OFL.txt を添付すること。
- コード用フォントはシステムの等幅フォントに任せている。Windows では実際には Consolas が使われる。

### 3.2 ウェイト

| トークン | 値 |
| --- | --- |
| `--font-weight-normal` | 400 |
| `--font-weight-medium` | 500 |
| `--font-weight-semibold` | 600 |
| `--font-weight-bold` | 700 |

### 3.3 サイズの基本スケール（base 14 × ratio 1.2、`round(14 × 1.2^step)`）

| トークン | step | px | rem |
| --- | --- | --- | --- |
| `--font-size-4xs` | -5 | 6 | 0.375 |
| `--font-size-3xs` | -4 | 7 | 0.4375 |
| `--font-size-2xs` | -3 | 8 | 0.5 |
| `--font-size-xs` | -2 | 10 | 0.625 |
| `--font-size-sm` | -1 | 12 | 0.75 |
| `--font-size-base` | 0 | 14 | 0.875 |
| `--font-size-lg` | +1 | 17 | 1.0625 |
| `--font-size-xl` | +2 | 20 | 1.25 |
| `--font-size-2xl` | +3 | 24 | 1.5 |
| `--font-size-3xl` | +4 | 29 | 1.8125 |
| `--font-size-4xl` | +5 | 35 | 2.1875 |
| `--font-size-5xl` | +6 | 42 | 2.625 |

### 3.4 Text の type と Heading の level（Neutral の解決値）

行高は次の規則で決まる（手計算で確認済み）:

- まず目標比率を決める: サイズ 20px 未満なら 1.5、20〜31px なら 1.4、32px 以上なら 1.25。
- 4px グリッドに丸める: `round(size × 目標比率 ÷ 4) × 4`。
- ただし下限は `ceil((size + 4) ÷ 4) × 4`。

| 役割 | サイズ | 行高 (px) | line-height 比 | ウェイト | ファミリー | 既定の文字色 |
| --- | --- | --- | --- | --- | --- | --- |
| `display-1` | 42px | 52px | 1.2381 | 400 | heading | primary |
| `display-2` | 35px | 44px | 1.2571 | 400 | heading | primary |
| `display-3` | 29px | 40px | **1.3793** | 400 | heading | primary |
| Heading level 1 | 24px | 32px | 1.3333 | 600 | heading | primary |
| Heading level 2 | 20px | 28px | 1.4 | 600 | heading | primary |
| Heading level 3 | 17px | 24px | 1.4118 | **700**（Neutral 指定） | heading | primary |
| Heading level 4 | 14px | 20px | 1.4286 | **700**（Neutral 指定） | heading | primary |
| Heading level 5 | 12px | 20px | 1.6667 | 600 | heading | primary |
| Heading level 6 | 10px | 16px | 1.6 | 600 | heading | primary |
| `large` | 17px | 24px | 1.4118 | 600 | body | primary |
| `body` | 14px | 20px | 1.4286 | 400 | body | primary |
| `label` | 14px | 20px | 1.4286 | 500 | body | primary |
| `code` | 14px | 20px | 1.4286 | 400 | code | primary |
| `supporting` | 12px | 20px | 1.6667 | 400 | body | **secondary** |

- Text の既定 type は `body`。Heading の既定色は primary で、display は block。
- **display-3 の行高は計算で求めた値。** core の既定トークンは `1.2414`（36px）になっているが、Neutral は `typography.scale` を指定しているので `expandTypeScale` の結果で上書きされる。29px は 32px 未満なので目標比率 1.4 → 40.6 → 4px 丸めで 40px → 1.3793。docsite の Neutral プレビューでもこの値になるはずだが、実際の描画は確認していない。
- **字間（letter-spacing）は全役割で `normal`**（0）。core のどこにも letter-spacing の指定はない（例外は Layer のテキストリセットで `normal` を指定しているものだけ）。uppercase のラベルもない。
- 数字: 既定はプロポーショナル数字。`hasTabularNumbers` を指定したときだけ `font-variant-numeric: tabular-nums` になる。
- その他の Text オプション: `textWrap: balance / pretty`、`hasCapsize`（`text-box-trim: trim-both`、`text-box-edge: cap alphabetic`）、`maxLines`（行数で切り詰めて省略記号を付ける）。
- フォントのレンダリング: reset.css が body に `-webkit-font-smoothing: antialiased` と `-moz-osx-font-smoothing: grayscale` を指定している。

### 3.5 タッチ + 狭い画面での適応（adaptation）

適用条件は `pointer: coarse` かつ幅 < `lg`（= 1024px）。このときスケールが `{base: 16, ratio: 1.1736}` に切り替わる（手計算）。

| 役割 | サイズ / 行高 |
| --- | --- |
| body / label / code | 16 / 24 |
| supporting | 14 / 20 |
| large | 19 / 28 |
| h1 | 26 / 36 |
| h2 | 22 / 32 |
| h3 | 19 / 28 |
| h4 | 16 / 24 |
| h5 | 14 / 20 |
| h6 | 12 / 20 |
| display-1 | 42 / 52（デスクトップと同じ） |
| display-2 | 36 / 44 |
| display-3 | 30 / 44 |

基本スケールは次のとおり:

| トークン | px |
| --- | --- |
| `--font-size-4xs` | 7 |
| `--font-size-3xs` | 8 |
| `--font-size-2xs` | 10 |
| `--font-size-xs` | 12 |
| `--font-size-sm` | 14 |
| `--font-size-base` | 16 |
| `--font-size-lg` | 19 |
| `--font-size-xl` | 22 |
| `--font-size-2xl` | 26 |
| `--font-size-3xl` | 30 |
| `--font-size-4xl` | 36 |
| `--font-size-5xl` | 42 |

デスクトップ向けの Kivact では、この適応は任意でよい。

### 3.6 入力欄のフォント下限

TextInput の文字サイズは `--text-body-size` だが、`(pointer: coarse)` かつ `@supports (-webkit-touch-callout: none)`（iOS WebKit のみ）の条件では `max(1rem, body)` = 16px 以上になる。iOS のズームを防ぐための処置。Kivact では無視してよい。

---

## 4. SPACING（余白）

`--spacing-N` = N × 0.25rem = **N × 4px**（ルート 16px のとき）。Neutral は上書きしていない。

| トークン | px | トークン | px | トークン | px |
| --- | --- | --- | --- | --- | --- |
| `--spacing-0` | 0 | `--spacing-3` | 12 | `--spacing-8` | 32 |
| `--spacing-0-5` | 2 | `--spacing-4` | 16 | `--spacing-9` | 36 |
| `--spacing-1` | 4 | `--spacing-5` | 20 | `--spacing-10` | 40 |
| `--spacing-1-5` | 6 | `--spacing-6` | 24 | `--spacing-11` | 44 |
| `--spacing-2` | 8 | `--spacing-7` | 28 | `--spacing-12` | 48 |

- 数値の props（`padding={3}`、`gap={3}` など）が受け付ける `SpacingStep` は `0 | 0.5 | 1 | 1.5 | 2 | 3 | 4 | 5 | 6 | 8 | 10`。たとえば `padding={6}` は 24px、`gap={3}` は 12px になる。
- コンテナの padding 変数としては 7、9、11、12 の段階もある（`Layout/container.stylex.ts`）。
- **コンテナの既定パディング:** Card、Section、Dialog、Drawer、BottomSheet は core では `--spacing-4` = 16px。**Neutral では Card と Section だけ 12px**（`--spacing-3`）。Card の 1px 枠はパディングの内側に入るので、枠とパディングを合わせた内側の余白はちょうど 12px になる。
- **レイヤーの画面端ガター:** `--spacing-4`（16px）。safe-area-inset のほうが大きければそちらを使う。Dialog の最大幅は `calc(100dvw - 32px)`。
- **レイアウト幅のトークンはない。** `Layout` の幅は利用者が props で渡す（`--layout-content-width` など）。UNRESOLVED ではなく、そもそも存在しない。
- よく使われる余白の組み合わせ:

  | 部品 | 値 |
  | --- | --- |
  | 入力欄 | 上下 4px / 左右 8px / gap 8px |
  | Button | 上下 8px / 左右 12px / gap 8px |
  | Badge | 左右 8px / gap 4px |
  | Keyboard hint | 上下 4px / 左右 8px |

---

## 5. RADIUS（角丸）

| トークン | Neutral の値 | px | 用途 |
| --- | --- | --- | --- |
| `--radius-none` | 0px | 0 | |
| `--radius-inner` | 0.375rem | **6** | 入れ子の内側の角。Checkbox の箱、Token（チップ）。 |
| `--radius-element` | 0.625rem | **10** | Button、入力欄、Keyboard hint。ButtonGroup の端。 |
| `--radius-container` | 0.75rem | **12** | Card、Popover、DropdownMenu、Dialog、Tooltip |
| `--radius-page` | 1.75rem | **28** | ページ規模のコンテナ |
| `--radius-chat` | 28px | 28 | core の既定値（Neutral は上書きしていない）。チャットのバブルや入力欄。 |
| `--radius-full` | 9999px | ピル形 | Badge、Switch、丸ボタン |

core の既定値は 4 / 8 / 12 / 28px。Neutral は inner と element を **2px ずつ大きく**している。

---

## 6. SIZES / DENSITY（寸法と密度）

### 6.1 コントロールの高さ

| トークン | px | Button のアイコン箱 |
| --- | --- | --- |
| `--size-element-sm` | 28 | sm = 16px |
| `--size-element-md` | 32（既定） | sm = 16px |
| `--size-element-lg` | 36 | md = 20px |

- 適用先: Button、TextInput、Selector などの高さ。
- アイコンだけの Button は `aspect-ratio: 1/1` の正方形で、パディングは 0。
- Token（チップ）や Neutral の SegmentedControl アイテムは `size - 8px`（20 / 24 / 28px）。
- `--border-width`: 1px。

### 6.2 SizeContext（`packages/core/src/SizeContext/SizeContext.ts`）

- `ElementSize = 'sm' | 'md' | 'lg'`。
- サイズは「明示 prop → 祖先の `SizeProvider` → コンポーネントの既定値（通常 `'md'`）」の順に解決される（`useSize(sizeProp, 'md')`）。
- Toolbar や TopNav、Card ヘッダーなどのコンテナが、子要素へ一括でサイズを流すために使う。
- Layer をまたいでもスコープは保たれる（layer-scoped context）。

### 6.3 アイコンサイズ（`Icon/IconSize.stylex.ts`）

| 名前 | rem | px |
| --- | --- | --- |
| `xsm` | 0.75 | 12 |
| `sm` | 1 | 16 |
| `md` | 1.25 | 20（Icon 単体の既定） |
| `lg` | 1.5 | 24 |

`IconDefaultSizeProvider` があると、そこで指定された既定サイズが優先される。

### 6.4 ヒットターゲットとタッチ

- `MIN_TOUCH_TARGET = 44`（`utils/hitArea.ts`）。
  - 主ポインタが coarse のときだけ、見えないヒット領域を広げる。
  - 各辺を 44px に向けて伸ばすが、隣のターゲットとの間隔の半分で止める。
  - **見た目のサイズは変わらない。**
- `utils/touchPresentation.stylex.ts` が判定に使うクエリ:

  | 名前 | クエリ |
  | --- | --- |
  | touch | `@media (pointer: coarse)` |
  | compact | `@media (width < 768px), (height < 500px)` |
  | compactTouch | compact かつ coarse |
  | iosSupports | `@supports (-webkit-touch-callout: none)`（16px の入力フォント下限専用） |

- `packages/core/src/internal/touch.ts` は、ジェスチャー基盤（GestureArena、Tap / LongPress の認識器、幾何計算）を Lab 向けに再エクスポートしているだけ。視覚的な値は持っていない。
- 押下ジェスチャーの定数（`utils/pressGesture.ts`）:

  | 定数 | 値 | 意味 |
  | --- | --- | --- |
  | `PRESS_ONSET_MS` | 150 | タッチで押下表示が出るまでの遅延 |
  | `PRESS_SLOP_PX` | 10 | これ以上動いたら押下を取り消す距離 |
  | `PRESS_FLASH_MS` | 100 | |
  | `PRESS_SCROLL_BRAKE_MS` | 150 | |
  | 離したときのフェード | 200ms、`cubic-bezier(0, 0, 0.2, 1)` | |

- ブレークポイント: sm 640 / md 768 / lg 1024 / xl 1280 / 2xl 1536px。

---

## 7. ELEVATION / SHADOWS（高さと影）

### 7.1 Neutral の影トークン（完全な値）

oklch のアルファ表記は rgba と等価（`oklch(0 0 0 / 5%)` = `rgba(0,0,0,0.05)`、`oklch(1 0 0 / 8%)` = `rgba(255,255,255,0.08)`）。

**ライト**（内側リムの層は `transparent` で、実質 2 層）:

| トークン | box-shadow |
| --- | --- |
| `--shadow-low` | `0 2px 4px rgba(0,0,0,0.05), 0 4px 8px rgba(0,0,0,0.10), inset 0 0 0 1px transparent` |
| `--shadow-med` | `0 2px 4px rgba(0,0,0,0.05), 0 4px 12px rgba(0,0,0,0.10), inset 0 0 0 1px transparent` |
| `--shadow-high` | `0 4px 6px rgba(0,0,0,0.10), 0 12px 24px rgba(0,0,0,0.15), inset 0 0 0 1px transparent` |

**ダーク**（影を深くし、全周に 1px の白い内側リム「ベゼル」を付ける）:

| トークン | box-shadow |
| --- | --- |
| `--shadow-low` | `0 2px 4px rgba(0,0,0,0.25), 0 4px 8px rgba(0,0,0,0.40), inset 0 0 0 1px rgba(255,255,255,0.08)` |
| `--shadow-med` | `0 2px 4px rgba(0,0,0,0.35), 0 4px 12px rgba(0,0,0,0.50), inset 0 0 0 1px rgba(255,255,255,0.12)` |
| `--shadow-high` | `0 4px 6px rgba(0,0,0,0.50), 0 12px 24px rgba(0,0,0,0.70), inset 0 0 0 1px rgba(255,255,255,0.15)` |

**入力欄の状態リング**（inset 2px、ライトとダークで同じ値）:

| トークン | box-shadow |
| --- | --- |
| `--shadow-inset-hover` | `inset 0 0 0 2px #0074e24D`（rgba(0,116,226,0.302)） |
| `--shadow-inset-selected` | `inset 0 0 0 2px #0074e280`（rgba(0,116,226,0.502)） |
| `--shadow-inset-success` | `inset 0 0 0 2px #2f7d334D`（rgba(47,125,51,0.302)） |
| `--shadow-inset-warning` | `inset 0 0 0 2px #f8d36a4D`（rgba(248,211,106,0.302)） |
| `--shadow-inset-error` | `inset 0 0 0 2px #de47454D`（rgba(222,71,69,0.302)） |

- `inset-hover` と `inset-selected` は core のコンポーネントからは参照されていない（grep で確認）。入力欄のホバーは、代わりに `inset 0 0 0 2px color-mix(in srgb, var(--color-border-emphasized) 30%, transparent)` で描かれる。値は L rgba(212,212,212,0.3) / D rgba(82,82,82,0.3)。
- 実際に使われているのは success / warning / error の 3 つ。

### 7.2 どの面がどの影を使うか（core ソースの grep 結果）

| 影 | 使う面 |
| --- | --- |
| `--shadow-low` | Popover、ContextMenu、DropdownMenu のサブメニュー、Keyboard hint、TopNav のメガメニュー、Switch のつまみ、TabList、SegmentedControl の選択アイテム（**Neutral では none**）、ChatComposer |
| `--shadow-med` | HoverCard、Toast、Carousel、ChatLayout のスクロールボタン |
| `--shadow-high` | Dialog、Drawer、BottomSheet |

Button、ButtonGroup、Card、Banner の `elevation` prop で low / med / high を選べる。既定は none。

### 7.3 枠線と影の使い分け

- 平らな面（Card 既定、Section の区切り）: 1px の `--color-border` を使い、影は付けない。
- 浮かぶ面（Popover、Menu、Dialog）: 影を使い、外枠は付けない。ダークでは影に含まれる白い内側リムが枠の代わりをする。
- 入力欄: 1px の `--color-border-emphasized` の枠に、状態に応じて inset リングを足す。
- Tooltip は反転配色: 地が `--color-text-primary`、文字が `--color-background-surface`、角丸は `--radius-container`。

### 7.4 重なり順（Layer）

- Astryx は z-index の段階トークンを持たない。Dialog、Popover、Menu、Tooltip、Toast、Keyboard hint は、ブラウザの **top layer**（`<dialog>.showModal()` と Popover API）に昇格する。
- 順序（下から上）:
  1. 通常コンテンツ（z 0〜2 程度の局所スタック）
  2. スティッキー要素
  3. 非モーダルの浮遊レイヤー（Popover、Menu、Tooltip）。開いた順に積まれ、ネストしたものはさらに上
  4. モーダル Dialog / Drawer / BottomSheet（スクリム付き）
  5. Toast。最後に開いたモーダルより上に出るよう、top layer に入り直す（`docs/architecture/layer-runtime.md`）
- Escape キーで閉じる順番は、共有スタックが論理的な深さ順に管理する。
- 例外的に固定の z-index がある: AppShell のスキップリンクが 9999、BottomSheetSwitcher が 1000。
- Kivact では「base < sticky < popover < modal(+scrim) < toast」の 5 段階で実装すれば十分。

---

## 8. FOCUS（フォーカス）

### 8.1 フォーカスリング

| トークン | 値 |
| --- | --- |
| `--focus-outline-width` | **2px** |
| `--focus-outline-style` | solid |
| `--focus-outline-color` | `var(--color-accent)` → L #1b1b1b / D #f1f1f1 |
| `--focus-outline-offset` | **3px**（要素の外側に 3px の隙間を空ける） |

- **`:focus-visible` のときだけ描く。** キーボード操作時だけで、マウスでクリックしたときは出ない。この条件はテーマから変えられない。
- `outline` として描くので、角丸は要素の border-radius に沿う。Chromium では外側リングの半径が「要素の半径 + オフセット」に広がる。たとえば Button（10px）のリングは外周の半径がおよそ 13px〜15px（ブラウザの挙動であり、Astryx の仕様ではない）。
- Kivact での推奨描画: 要素の矩形を 3px 外側へ広げ、半径を「要素の半径 + 3px」にして、幅 2px のストロークを描く。
- 例外:
  - destructive Button のリング色は `--color-error`。
  - Switch は独自の条件でリングを描くが、値は同じトークンを使う。
  - 入力欄は外側リングを出さない（`outline: none`）。代わりに枠をアクセント色にし、`inset 0 0 0 2px var(--color-accent-muted)` を足す。Neutral のライトでは #f1f1f1 の内側リングで、ほとんど見えない。ダークでは #262626 で surface と同じ色になり、実質見えない。
  - フォーカスを示すのは主に、枠が #1b1b1b / #f1f1f1 に変わること。
- 1 回のフォーカス移動では 1 つのリングしか出さない（`focusWithinFirstChild` などで二重リングを避ける）。

### 8.2 useIndicatorFocusRing

Checkbox や Radio は、本物の input が透明（opacity 0）で見えない。そのためリングは横に描かれるインジケータ（箱や丸）の要素に、命令的にスタイルを書き込んで描く（`FOCUS_OUTLINE_PARTS`）。リングの形はインジケータ自身の border-radius に従い（Checkbox なら 6px）、条件は `:focus-visible` のときだけ。無効状態では描かない。

### 8.3 useKeyboardHint

- 矢印キーで移動する複合ウィジェット（roving tabindex）に、キーボードで初めてフォーカスが入ったとき、「← → で移動」というヒントを出す。
- 見た目:
  - 地 `--color-background-popover`、角丸 `--radius-element`（10px）、影 `--shadow-low`
  - 上下 4px / 左右 8px のパディング
  - 文字は supporting（12px / 20px）、色は `--color-text-secondary`
  - 矢印は `Kbd` コンポーネントで描き、キー同士の gap は 4px
- 不透明度 0 → 1 を 150ms で切り替える。
- 消える条件: 3000ms 経過、最初の矢印キー入力、フォーカスがウィジェットから出たとき。一度消えたら、そのインスタンスでは再表示しない。
- `pointer-events: none` で、`aria-hidden`。

---

## 9. STATES（状態モデル）

### 9.1 状態の一覧と描画

| 状態 | 描き方 |
| --- | --- |
| rest | 地の色のみ |
| hover | オーバーレイ層方式なら `--color-overlay-hover` を重ねる。color-mix 方式なら §9.3。`@media (hover: hover)` のときだけで、無効状態には出さない。入力欄は inset 2px の border-emphasized@30%。 |
| pressed | `--color-overlay-pressed`（10.2%）を重ねる。Button はさらに `transform: scale(0.98)`。マウスは `:active`、タッチは `data-astryx-press="on"`（150ms の遅延後に表示し、離すと 200ms でフェードアウト）。 |
| focus-visible | §8。外側 2px のリング、オフセット 3px、アクセント色 |
| disabled | **opacity 0.5**（core 全体の標準値。0.5 の使用が 39 か所）。カーソルは `default`（`not-allowed` ではない）。ホバーと押下の表示は出さない。Checkbox は枠を `--color-border` に、未チェック時の地を `--color-background-muted` にする。 |
| selected | リスト行: 地を `--color-accent-muted`（L #f1f1f1 / D #262626）にする。Checkbox / Radio / Switch: 地と枠を `--color-accent`、記号を `--color-on-accent` にする。SegmentedControl: 選択アイテムは surface の面（Neutral では影なし）。押下しても選択面は変えない。 |
| invalid / status | 入力欄の枠を `--color-border-error`（= text-error: L #c0262c / D #ffaea7）にし、ホバーでは `--shadow-inset-error` を出す。warning と success も同様。 |
| readOnly | **専用の見た目はない。** core は値を編集できなくし、`readonly` 属性や `aria-readonly` を出すだけ。Neutral もスタイルを付けていない。 |
| loading | Button は操作を止めるが、スピナーは暗くしない（明示的に disabled にした場合だけ暗くなる）。 |

- トランジション: 背景色、枠色、影、色を `--duration-fast`（125ms）+ `--ease-standard` で変化させる。`prefers-reduced-motion: reduce` のときは 0s。
- カーソル: 押せる要素は `pointer`、無効な要素は `default`。

### 9.2 InteractiveRoleContext / usePressFeedback

- **InteractiveRoleContext:** Popover や DropdownMenu が、トリガーになる子要素に「button として描画して」と伝える Context。子は `useInteractiveRole` で href / onClick とあわせて役割を判断する。見た目の値は持たない。
- **usePressFeedback:** 押下表示を描く要素に `pressableProps` を付け、ドキュメント全体で 1 つの押下コントローラーを登録する。描く側は `interactionOverlayStyles` の 4 種類のどれかと組み合わせる:

  | 種類 | 用途 |
  | --- | --- |
  | `backgroundColor` | 地の色そのものをオーバーレイ色にする |
  | `backgroundImage` | 地の色の上にグラデーション層を重ねる |
  | `backgroundImageOnNeutral` | neutral 層の上にオーバーレイを重ねる |
  | `pressedBackgroundColor` | 押下だけ描く（ホバー表示を持たない Link 向け） |

  タッチの押下の強さは `--astryx-press-alpha`（0〜1）。押下中は 1、離すと 200ms で 1 → 0 にフェードする。
- 押下を描く部位（`docs/design/user-states.md`）:

  | コンポーネント | 押下を描く部位 |
  | --- | --- |
  | Checkbox / Radio | インジケータ |
  | Collapsible | トリガー行 |
  | Link | 文字の面 |
  | Slider | ドラッグ中のつまみだけ |
  | Switch | トラックとつまみの両方 |
  | Tab | Tab の面 |
  | SegmentedControl | 未選択のアイテムだけ |

### 9.3 color-mix 方式のホバー（`color-mix(in srgb, X, tint N%)`、手計算）

| 対象 | 式 | Light | Dark |
| --- | --- | --- | --- |
| Checkbox / Radio 未チェックの枠 | border-emphasized + 20% | #aaaaaa | #757575 |
| Checkbox / Radio 未チェックの地 | surface + 5% | #f2f2f2 | #313131 |
| Checkbox / Radio チェック済み、Slider のつまみ、Switch ON | accent + 15% | #171717 | #f3f3f3 |
| Switch OFF のトラック（Neutral では border-emphasized に差し替え） | border-emphasized + 5% | #c9c9c9 | #5b5b5b |
| Link primary | text-primary + 15% | #000000 | #ffffff（変化なし） |
| Link secondary | text-secondary + 15% | #3c3c3c | #adadad |
| Link accent | text-accent + 15% | #171717 | #f3f3f3 |

---

## 10. ICONS（アイコン）

### 10.1 Neutral のアイコンセット

- Neutral は **Lucide**（`lucide-react`、lockfile 上のバージョンは 1.23.0）を使う。セマンティック名は 29 個で、`packages/themes/neutral/src/icons.tsx` に全件ある。
- すべて `size: '1em'` と `aria-hidden` を付けて描画する。

| セマンティック名 | import している名前 | Lucide の正規名（参考） |
| --- | --- | --- |
| close | X | x |
| chevronDown | ChevronDown | chevron-down |
| chevronLeft | ChevronLeft | chevron-left |
| chevronRight | ChevronRight | chevron-right |
| chevronsLeft | ChevronsLeft | chevrons-left |
| chevronsRight | ChevronsRight | chevrons-right |
| check | Check | check |
| success | CheckCircle | circle-check |
| error | XCircle | circle-x |
| warning | AlertTriangle | triangle-alert |
| info | Info | info |
| calendar | Calendar | calendar |
| clock | Clock | clock |
| externalLink | ExternalLink | external-link |
| menu | Menu | menu |
| moreHorizontal | MoreHorizontal | ellipsis |
| search | Search | search |
| upload | Upload | upload |
| arrowUp | ArrowUp | arrow-up |
| arrowDown | ArrowDown | arrow-down |
| arrowsUpDown | ArrowUpDown | arrow-up-down |
| funnel | Filter | funnel |
| eyeSlash | EyeOff | eye-off |
| viewColumns | Columns | columns-2 |
| copy | Copy | copy |
| checkDouble | CheckCheck | check-check |
| wrench | Wrench | wrench |
| stop | Square | square |
| microphone | Mic | mic |

- 「Lucide の正規名」の列は**一般知識による参考情報**。Lucide は旧名（CheckCircle、XCircle、AlertTriangle、MoreHorizontal、Filter、Columns）を別名として残している。リポジトリから確定できるのは import 名だけ。
- **SVG のパスデータは UNRESOLVED。** `node_modules` がなく、リポジトリにも入っていない。lucide-static などから入手すること。ライセンスは一般知識として ISC。
- `'richtext:bold'` のような名前空間付きのアイコンは、Neutral には定義がない。

### 10.2 Lucide の描画仕様（一般知識。Lucide の既定値）

- viewBox 24×24、`fill="none"`、`stroke="currentColor"`、**`stroke-width="2"`**、`stroke-linecap="round"`、`stroke-linejoin="round"`。
- `absoluteStrokeWidth` は既定で false。つまりストロークの太さはアイコンの大きさに比例する。

| アイコンサイズ | 実際の線幅 |
| --- | --- |
| xsm 12px | 1px |
| sm 16px | 約 1.33px |
| md 20px | 約 1.67px |
| lg 24px | 2px |

- 参考: core の既定アイコン（`Icon/defaultIcons.tsx`。Neutral は全部上書きしている）は heroicons outline 風で、線幅 1.5（menu だけ 2）。

### 10.3 Icon コンポーネントのサイズ決定と色

- サイズは「`size` prop → `IconDefaultSizeContext` → `md`（20px）」の順に決まる。
- レジストリのアイコン（名前で指定するもの）は `<span>` の箱で描く。箱のサイズは `width/height: var(--_icon-width, <size>)` で、`font-size` は幅と高さの小さいほう。1em 指定のアイコンがこの箱いっぱいに広がる。
- 色の既定は `inherit`（currentColor）。指定できる色は次のとおり:

  | color | 参照するトークン |
  | --- | --- |
  | primary | `--color-icon-primary` |
  | secondary / tertiary | `--color-icon-secondary` |
  | disabled | `--color-icon-disabled` |
  | accent | `--color-accent` |
  | info / success / warning / error | `--color-icon-<s>` |
  | blue / red / green / gray / cyan / teal / yellow / orange / pink / purple | `--color-icon-<hue>` |

- Button の中のアイコン: sm と md の Button では 16px、lg では 20px。
- サイズの役割（`docs/design/iconography.md`）:

  | サイズ | 役割 |
  | --- | --- |
  | 12px | 補助的なインジケータ |
  | 16px | 標準的なコントロール |
  | 20px | 目立つコントロール |
  | 24px | 表示用、オブジェクトの識別 |

---

## 11. MOTION TOKENS（値のみ）

Neutral の `motion: {fast: 125, medium: 300, slow: 700, ratio: 0.75}` から計算した値（手計算）:

- min = base × 0.75、max = base / 0.75
- どちらも 5ms 単位に丸める（`round(x / 5) × 5`）
- tempo は未設定なので 1、character も未設定なので standard

| トークン | 値 | トークン | 値 | トークン | 値 |
| --- | --- | --- | --- | --- | --- |
| `--duration-fast-min` | 95ms | `--duration-medium-min` | 225ms | `--duration-slow-min` | 525ms |
| `--duration-fast` | **125ms** | `--duration-medium` | **300ms** | `--duration-slow` | 700ms |
| `--duration-fast-max` | 165ms | `--duration-medium-max` | 400ms | `--duration-slow-max` | 935ms |

イージング:

| トークン | 値 |
| --- | --- |
| `--ease-standard` | `cubic-bezier(0.24, 1, 0.4, 1)`（強く減速する。到着と状態変化に使う） |
| `--ease-entrance` | 同上（character が standard なので standard と同じ） |
| `--ease-move` | 同上 |
| `--ease-exit` | `cubic-bezier(0.3, 0, 0.6, 0.6)`（加速する。退場に使う） |

その他の時間: 押下を離したときのフェードが 200ms + `cubic-bezier(0,0,0.2,1)`、Keyboard hint が 150ms。core の既定値（175 / 410 / 975ms）は Neutral では使われない。

---

## 12. 他のテーマとの違い（参考情報。目標は Neutral）

- **matcha** — 落ち着いた抹茶色の緑系。
  - 本文は DM Sans、見出しは手書き風の Playwrite US Trad、コードは JetBrains Mono。
  - 文字スケールは base 16 / ratio 1.25 で、Neutral より大きく、段差も大きい。
  - 角丸が大きい（inner 6 / element 12 / container 18 / page 42px）。
  - 影はニュートラルの濃い色（light20）に色付けしたもの。
  - アクセントと本文色はパレットの濃い緑灰（neutral.light20）で、Neutral ほど純黒ではない。
- **stone** — 暖かい石とスレートの色。
  - 本文は Figtree、見出しは Montserrat（h3 と h4 は bold）、コードは JetBrains Mono。
  - スケールは 14 / 1.25。角丸は 4 / 8 / 12 / 24px で、Neutral よりわずかに鋭い。
  - 枠線は不透明な neutral.light90。影は light15 の色で付ける。
  - 入力欄のステータス枠は、ステータス文字より柔らかい中間の stop を使う。
- **gothic** — **ダーク専用**。
  - 深い青灰色に、ディスプレイ用セリフ（Manufacturing Consent）を合わせる。本文は Fustat。
  - スケールは 16 / 1.25。モーションはやや遅く、fast 150 / medium 350。
  - surface と body が同じ色（neutral.dark10）。
  - カテゴリ色は、暗い地の上に明るい地 + 暗い文字を置く「pastel-on-dark」。
  - 影が濃い（20〜30%）。
- **chocolate** — チョコレートとキャラメルの茶色。
  - 本文は Albert Sans、見出しはセリフの Fraunces。
  - 背景はクリーム色（#FFFCF7）、アクセントは #8C5927、枠線は不透明の #C4AC95。
  - 影は茶色（#4a3520）に色付け。
  - スケール（14 / 1.2）と角丸（6 / 10 / 12 / 24px）は Neutral とほぼ同じ。
- **y2k** — 2000 年代初頭のポップ調。
  - Poppins を全体に使い、スケールは 16 / 1.25。
  - **角丸はすべて 0**。`radius-full` まで 0 にした、尖ったブルータリスト風。
  - ページの地はアクセント色の淡い stop（ペリウィンクル）。ネオングリーンを差し色にする。
  - 枠線は濃い（neutral.light20）。
  - モーションは速い（100 / 250 / 600、ratio 0.8）。
- **butter** — 暖かいバター色の黄色に、青いアクセント。
  - 見出しと本文は Outfit、ディスプレイは Sarina。スケールは 14 / 1.25。
  - 角丸は 6 / 8 / 12 / 24px。チャットの入力欄ではボタンを完全な丸にする。
  - アクセントは青（accent.light40）。ダークのアクセントは黄色（yellow.dark90）。
  - Badge の高さは 30px（Badge.tsx のコメントより）。

---

## 13. Kivact マッピング表（`Tokens` 構造体の提案）

- 色は `Color`（RGBA8）。半透明の色は 8 桁 hex のまま持ち、描画時にアルファ合成する。
- 長さは `f32` の論理 px（ルート 16px 前提）。
- `[L / D]` はライト用とダーク用の 2 インスタンスの値。値が 1 つなら両モード共通。

### 13.1 colors

| Rust フィールド | CSS トークン | Light | Dark |
| --- | --- | --- | --- |
| `colors.bg_body` | --color-background-body | #f1f1f1 | #1b1b1b |
| `colors.bg_surface` | --color-background-surface | #ffffff | #262626 |
| `colors.bg_card` | --color-background-card | #ffffff | #1b1b1b |
| `colors.bg_popover` | --color-background-popover | #ffffff | #1b1b1b |
| `colors.bg_muted` | --color-background-muted | #f1f1f1 | #1b1b1b |
| `colors.bg_inverted` | --color-background-inverted | #0a1317 | #ffffff |
| `colors.bg_error_inverted` | --color-background-error-inverted | #aa071e | #e3193b |
| `colors.neutral` | --color-neutral | #0000000f | #ffffff1a |
| `colors.accent` | --color-accent | #1b1b1b | #f1f1f1 |
| `colors.accent_muted` | --color-accent-muted | #f1f1f1 | #262626 |
| `colors.on_accent` | --color-on-accent | #ffffff | #111111 |
| `colors.on_dark` | --color-on-dark | #ffffff | #ffffff |
| `colors.on_light` | --color-on-light | #111111 | #111111 |
| `colors.text_primary` | --color-text-primary | #000000 | #ffffff |
| `colors.text_secondary` | --color-text-secondary | #474747 | #9e9e9e |
| `colors.text_disabled` | --color-text-disabled | #919191 | #525252 |
| `colors.text_accent` | --color-text-accent | #1b1b1b | #f1f1f1 |
| `colors.icon_primary` | --color-icon-primary | #000000 | #ffffff |
| `colors.icon_secondary` | --color-icon-secondary | #6a6a6a | #9e9e9e |
| `colors.icon_disabled` | --color-icon-disabled | #919191 | #525252 |
| `colors.icon_accent` | --color-icon-accent | #1b1b1b | #f1f1f1 |
| `colors.border` | --color-border | #00000014 | #ffffff1a |
| `colors.border_emphasized` | --color-border-emphasized | #d4d4d4 | #525252 |
| `colors.skeleton` | --color-skeleton | #f1f1f1 | #525252 |
| `colors.track` | --color-track | #ccd3db | #5a5e66 |
| `colors.shadow` | --color-shadow | #0000001a | #0000004d |
| `colors.tint_hover` | --color-tint-hover | #000000 | #ffffff |
| `colors.overlay_scrim` | --color-overlay | #00000080 | #000000cc |
| `colors.overlay_hover` | --color-overlay-hover | #0000000d | #ffffff0d |
| `colors.overlay_pressed` | --color-overlay-pressed | #0000001a | #ffffff1a |
| `colors.focus_ring` | --focus-outline-color | #1b1b1b | #f1f1f1 |
| `colors.info` | --color-info | #0074e2 | #6d9cfe |
| `colors.on_info` | --color-on-info | #ffffff | #111111 |
| `colors.info_muted` | --color-info-muted | #b9d7ff | #88bcff3d |
| `colors.on_info_muted` | --color-on-info-muted | #003978 | #a1caff |
| `colors.text_info` | --color-text-info（icon / border も同値） | #0067ce | #a1caff |
| `colors.success` | --color-success | #198100 | #64af4c |
| `colors.on_success` | --color-on-success | #ffffff | #111111 |
| `colors.success_muted` | --color-success-muted | #bce0bb | #90ca903d |
| `colors.on_success_muted` | --color-on-success-muted | #00490b | #90ca90 |
| `colors.text_success` | --color-text-success（icon / border も同値） | #2f7d33 | #90ca90 |
| `colors.warning` | --color-warning | #ffce2f | #ffce2f |
| `colors.on_warning` | --color-on-warning | #111111 | #111111 |
| `colors.warning_muted` | --color-warning-muted | #fae19e | #e2b6233d |
| `colors.on_warning_muted` | --color-on-warning-muted | #4b3900 | #eec448 |
| `colors.text_warning` | --color-text-warning（icon / border も同値） | #836700 | #eec448 |
| `colors.error` | --color-error | #c9303a | #ff705d |
| `colors.on_error` | --color-on-error | #ffffff | #111111 |
| `colors.error_muted` | --color-error-muted | #ffc4be | #ff98903d |
| `colors.on_error_muted` | --color-on-error-muted | #76000c | #ffaea7 |
| `colors.text_error` | --color-text-error（icon / border も同値） | #c0262c | #ffaea7 |
| `colors.hue.<h>.bg / border / icon / text` | --color-{background,border,icon,text}-<h> | §2.7 の表 | §2.7 の表 |
| `colors.data.*` | --color-data-* | §2.9 | §2.9 |
| `colors.syntax.*` | --color-syntax-* | §2.10 | §2.10 |

`hue` は red、orange、yellow、green、teal、cyan、blue、purple、pink、gray の 10 色。それぞれ `HueColors { bg, border, icon, text }` を持つ構造体にすることを推奨する。

### 13.2 typography

| Rust フィールド | CSS トークン | 値 |
| --- | --- | --- |
| `typography.family_body` | --font-family-body | "Figtree"（フォールバック: Segoe UI → Roboto → Helvetica → Arial → sans-serif） |
| `typography.family_heading` | --font-family-heading | "Figtree" |
| `typography.family_code` | --font-family-code | ui-monospace → "SF Mono" → Monaco → Consolas → "Liberation Mono" → "Courier New" → monospace |
| `typography.weight_normal` / `medium` / `semibold` / `bold` | --font-weight-* | 400 / 500 / 600 / 700 |
| `typography.size_4xs` … `size_5xl` | --font-size-* | 6, 7, 8, 10, 12, 14, 17, 20, 24, 29, 35, 42 |
| `typography.display1` | --text-display-1-* | 42 / 52 / 400 / heading |
| `typography.display2` | --text-display-2-* | 35 / 44 / 400 / heading |
| `typography.display3` | --text-display-3-* | 29 / 40 / 400 / heading |
| `typography.heading1` | --text-heading-1-* | 24 / 32 / 600 |
| `typography.heading2` | --text-heading-2-* | 20 / 28 / 600 |
| `typography.heading3` | --text-heading-3-* | 17 / 24 / 700 |
| `typography.heading4` | --text-heading-4-* | 14 / 20 / 700 |
| `typography.heading5` | --text-heading-5-* | 12 / 20 / 600 |
| `typography.heading6` | --text-heading-6-* | 10 / 16 / 600 |
| `typography.large` | --text-large-* | 17 / 24 / 600 / body |
| `typography.body` | --text-body-* | 14 / 20 / 400 / body |
| `typography.label` | --text-label-* | 14 / 20 / 500 / body |
| `typography.code` | --text-code-* | 14 / 20 / 400 / code |
| `typography.supporting` | --text-supporting-* | 12 / 20 / 400 / body（既定色 secondary） |
| `typography.letter_spacing` | （なし） | 0 |

- 各役割は `TextStyle { size_px, line_height_px, weight, family }` の形で持つことを推奨する。
- ライトとダークで値は変わらない。

### 13.3 spacing

| Rust フィールド | CSS トークン | 値 (px) |
| --- | --- | --- |
| `spacing.s0`, `s0_5`, `s1`, `s1_5`, `s2` | --spacing-0 … -2 | 0, 2, 4, 6, 8 |
| `spacing.s3`, `s4`, `s5`, `s6`, `s7` | --spacing-3 … -7 | 12, 16, 20, 24, 28 |
| `spacing.s8`, `s9`, `s10`, `s11`, `s12` | --spacing-8 … -12 | 32, 36, 40, 44, 48 |
| `spacing.card_padding` | (card base, Neutral) | 12 |
| `spacing.section_padding` | (section base, Neutral) | 12 |
| `spacing.dialog_padding` | --astryx-dialog-padding（既定） | 16 |
| `spacing.layer_viewport_gutter` | spacing-4 | 16 |

### 13.4 radius

| Rust フィールド | CSS トークン | 値 (px) |
| --- | --- | --- |
| `radius.none` | --radius-none | 0 |
| `radius.inner` | --radius-inner | 6 |
| `radius.element` | --radius-element | 10 |
| `radius.container` | --radius-container | 12 |
| `radius.page` | --radius-page | 28 |
| `radius.chat` | --radius-chat | 28 |
| `radius.full` | --radius-full | 9999（実装では高さの半分にする） |

### 13.5 sizes

| Rust フィールド | CSS トークン / 出典 | 値 |
| --- | --- | --- |
| `sizes.control_sm` | --size-element-sm | 28 |
| `sizes.control_md` | --size-element-md | 32 |
| `sizes.control_lg` | --size-element-lg | 36 |
| `sizes.border_width` | --border-width | 1 |
| `sizes.focus_ring_width` | --focus-outline-width | 2 |
| `sizes.focus_ring_offset` | --focus-outline-offset | 3 |
| `sizes.icon_xsm` / `icon_sm` / `icon_md` / `icon_lg` | Icon size | 12 / 16 / 20 / 24 |
| `sizes.icon_stroke_ratio` | Lucide | 2/24（線幅 = アイコンサイズ × 2/24） |
| `sizes.badge_height` | --spacing-5 | 20 |
| `sizes.min_touch_target` | hitArea.ts | 44 |
| `sizes.disabled_opacity` | 共通 | 0.5 |
| `sizes.press_scale` | Button | 0.98 |

### 13.6 shadows

`Shadow { x, y, blur, spread, color, inset }` の配列として持つ。

| Rust フィールド | CSS トークン | Light | Dark |
| --- | --- | --- | --- |
| `shadows.low` | --shadow-low | (0,2,4,0,#0000000d) + (0,4,8,0,#0000001a) | (0,2,4,0,#00000040) + (0,4,8,0,#00000066) + inset(0,0,0,1,#ffffff14) |
| `shadows.med` | --shadow-med | (0,2,4,0,#0000000d) + (0,4,12,0,#0000001a) | (0,2,4,0,#00000059) + (0,4,12,0,#00000080) + inset(0,0,0,1,#ffffff1f) |
| `shadows.high` | --shadow-high | (0,4,6,0,#0000001a) + (0,12,24,0,#00000026) | (0,4,6,0,#00000080) + (0,12,24,0,#000000b3) + inset(0,0,0,1,#ffffff26) |
| `shadows.inset_hover` | --shadow-inset-hover | inset(0,0,0,2,#0074e24d) | 同じ |
| `shadows.inset_selected` | --shadow-inset-selected | inset(0,0,0,2,#0074e280) | 同じ |
| `shadows.inset_success` | --shadow-inset-success | inset(0,0,0,2,#2f7d334d) | 同じ |
| `shadows.inset_warning` | --shadow-inset-warning | inset(0,0,0,2,#f8d36a4d) | 同じ |
| `shadows.inset_error` | --shadow-inset-error | inset(0,0,0,2,#de47454d) | 同じ |
| `shadows.input_hover_ring` | color-mix 由来 | inset(0,0,0,2,rgba(212,212,212,0.3)) | inset(0,0,0,2,rgba(82,82,82,0.3)) |
| `shadows.input_focus_ring` | accent-muted | inset(0,0,0,2,#f1f1f1) | inset(0,0,0,2,#262626) |

- アルファの hex 換算: 0.05 = 0d、0.10 = 1a、0.15 = 26、0.25 = 40、0.35 = 59、0.40 = 66、0.50 = 80、0.70 = b3、0.08 = 14、0.12 = 1f。ただし 0.05 は正確には 12.75/255 で、丸めて 0d にしている。
- CSS の blur 値をそのままガウスぼかしに使うなら、σ ≈ blur / 2。

### 13.7 motion

| Rust フィールド | CSS トークン | 値 |
| --- | --- | --- |
| `motion.fast_min` / `fast` / `fast_max` | --duration-fast* | 95 / 125 / 165 ms |
| `motion.medium_min` / `medium` / `medium_max` | --duration-medium* | 225 / 300 / 400 ms |
| `motion.slow_min` / `slow` / `slow_max` | --duration-slow* | 525 / 700 / 935 ms |
| `motion.ease_standard` | --ease-standard | cubic-bezier(0.24, 1, 0.4, 1) |
| `motion.ease_entrance` | --ease-entrance | cubic-bezier(0.24, 1, 0.4, 1) |
| `motion.ease_move` | --ease-move | cubic-bezier(0.24, 1, 0.4, 1) |
| `motion.ease_exit` | --ease-exit | cubic-bezier(0.3, 0, 0.6, 0.6) |
| `motion.press_onset` | PRESS_ONSET_MS | 150 ms |
| `motion.press_release` | pressConsts.releaseMs | 200 ms, cubic-bezier(0, 0, 0.2, 1) |

---

## 付録 A: UNRESOLVED と注意点の一覧

1. **Figtree のライセンス:** リポジトリに記載がない。一般知識では SIL OFL 1.1。フォントファイルも同梱されておらず、Google Fonts から読み込む方式。
2. **Lucide の SVG パスデータ:** `node_modules` がなく、リポジトリにもない。線幅 2、viewBox 24 などの描画属性は Lucide の一般的な既定値で、リポジトリ内では確認できていない。正規名の対応表も一般知識による。
3. **display-3 の行高:** core の既定は 1.2414 だが、Neutral では式から 1.3793（40px）になると手計算した。実際の描画では確認していない。
4. **`--color-track` と `--color-background-inverted`:** Neutral は上書きしていないので、core の青みがかった値が残る。docsite でもそう描画されるはず。描画では確認していない。
5. **フォーカスリングの外周半径:** ブラウザ依存（outline-offset に対する border-radius の広がり方）。Astryx としての仕様値ではない。
6. **Badge の gray バリアント:** Neutral は `variant:gray` のスタイルを定義しているが、core の BadgeVariantMap の 14 種には gray は含まれていない。拡張を前提にしたスタイル。
7. **レイアウト幅や z-index の段階トークン:** そもそも存在しない（UNRESOLVED ではなく、存在しないことを確認した）。
