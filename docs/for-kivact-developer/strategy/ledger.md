# 見送りと実アプリ要求の台帳

[製品戦略](20261009_strategy.md) §3・§9 の記録先。状態は実装と検証の結果で更新する。

## 見送り

| 項目 | 見送った理由 | アプリでの代替手段 | 再検討の条件 | 日付 |
|---|---|---|---|---|
| 日付と数値の書式 | 中核の担当だが、書式の API は今回のリファクタリングの対象外。日付の値（`CalendarDate`）は `kivact::i18n` に置いた（Phase 5.4） | アプリの書式関数を使う | 地域書式の API を設計するとき | 2026-10-27 |
| float・HDR の描画（旧 `render_float`） | 戦略 4 章「HDR・浮動小数点の出力はアプリ」 | `Offscreen` の RGBA8。HDR はアプリの CPU renderer | 2 つ以上のアプリが HDR の書出しを要求 | 2026-10-18 |
| audio（旧 `arvane-audio`、PCM 再生、worker） | 戦略 4 章「音声はアプリ」 | アプリが rodio などを直接使う（`dev/apps/music-player`） | 4 章の「UI の lifecycle と統合できる形」 | 2026-10-18 |
| display UUID による window の配置と復元 | 戦略 4 章「位置と display は、移植可能な不透明な値ができるまで入れない」 | `platform::window_size()` で寸法だけを保存し、`App::window_size` で戻す。位置は C-6 の中央配置 | 移植可能な不透明な display の値の設計ができる | 2026-10-18 |
| before-action input guard（`on_before_leave`・`on_unavailable`） | 経路が増え、focus の規則と二重になる | `on_commit`・`on_composition_change` | 確定を保証できない具体的な操作が報告される | 2026-10-18 |
| path の boolean 演算（旧 `i_overlay`） | 依存が大きく、要求が 1 つ | C-4 の `Path::flattened` の点列をアプリが処理する | 2 つ以上のアプリが要求 | 2026-10-18 |
| `font_sources` | `App::font`・`Offscreen::font` で足りる | family 名で指定する | — | 2026-10-18 |
| owned な text layout、glyph の輪郭、`StrokeGlyphs` | 戦略 4 章「テキスト編集の拡張口」の設計と一体で決める。旧の形は 6 crate・公開型約 9 つに及ぶ | `Draw::measure_text` | テキスト編集の拡張口（行単位の配置結果）を設計するとき | 2026-10-18 |
| `CanvasGradient::transform` | HTML の CanvasGradient に transform はなく、対応が崩れる | Draw の transform scope | — | 2026-10-18 |
| creative editor の compact sidebar（lane の行の metrics、`on_lane_drop`、sidebar の resize） | 特定アプリの画面構成に寄っている | `dock`・`kivact::ui::split` と D-7 の lane のセルで組む | 2 つ目のアプリが同じ構成を要求 | 2026-10-18 |
| outliner の複数列・sort 付きの見出し | 表（E）と同じ系統を 2 つ作ることになる | D-7 の固定幅のセル | E の結論が出たとき | 2026-10-18 |
| 期間入力の「開始日が存在しない」「終了日が存在しない」の別の文言 | 要求が 1 つで、1 つの文言で用が足りる | `date.error.invalid` を差し替える | 要求が出たとき | 2026-10-18 |
| overlay 内の百分率による最大幅の基準 | C-3 の調査で、window を 400px に縮めても幅 560px の dialog が `max_width(pct(100))` で縮まらない既存の問題を確認。overlay の配置と百分率の基準の検討が必要で、幅の指定口とは別に扱う | アプリで `platform::window_size().width` 以下の明示幅を渡す | overlay の百分率の基準を整えるとき。新設の幅の指定と反応更新の試験は独立して検証する | 2026-10-18 |
| 編集できる表の統合（E-2） | 現在の `TableFocus::Cells` はセルの focus を持つが、矩形選択・draft・確定と取消しは持たない。設計では中核の選択と draft の寿命に約300行、TSVの複数セルへの一括書込みに約140行、kit の編集表示に約250行、行IDの割当てと追加・削除・移動の binding 接続に約90行を見積もる。計約780 code行と試験になり、指示の約600行の単位を超える。command/history の別経路を作って縮めない | creative の `DataTable::cell_controls` に標準 `text_input` 等を置き、アプリの binding で確定する。TSVと行IDの生成・行操作はアプリが所有し、`platform::copy_text` / `read_text` を使う | 複数セルの選択・draft・TSV・行操作のうち、共通化する最小の一組を約600行に絞れる具体的な要求があるとき | 2026-10-18 |
| macOS の埋め込み（NSView） | 実機と保守者の条件が未成立（[設計の埋め込み](../design.md#埋め込み)） | なし | 戦略 8 章の macOS の条件（実機と保守者）。設計の埋め込みの章にある backend の契約と、共通の契約試験の通過を完成の条件とする | 2026-10-31 |
| Wayland の埋め込み（VST3 3.8 の `WaylandSurfaceID`） | 現行の wrapper/host との接続経路を X11 に限定（[設計の埋め込み](../design.md#埋め込み)） | X11（XWayland）のホストを使う | wrapper（nih-plug）とホストが Wayland の埋め込みを提供する | 2026-10-31 |
| Linux でホストの run loop（fd の登録）を使う方式 | 現行実装は独立した XCB 接続と専用 UI thread を所有（[設計の埋め込み](../design.md#埋め込み)） | 専用 thread（D3） | wrapper が run loop を渡し、VST3 の UI thread の規則を守る要求が出る | 2026-10-31 |
| GPU の device を process で共有 | window ごとの Presenter の寿命を維持し、共有の必要性は計測で判断（[設計の埋め込み](../design.md#埋め込み)） | 開くたびに CPU で描き始め、GPU に移る | 計測で、開く時間や資源が問題になる | 2026-10-31 |
| 未処理のキーをホストへ返す | engine からキー処理結果を同期に返す契約がない（[設計の埋め込み](../design.md#埋め込み)） | なし（editor が focus を持つ間、DAW の shortcut は効かない） | engine がキーの処理の有無を同期的に返せる設計ができる | 2026-10-31 |
| DAW のキーの横取りへの対策（`WH_GETMESSAGE` の hook） | 実機で必要と確かめる前に process の hook を導入しない（[設計の埋め込み](../design.md#埋め込み)） | クリックで focus を取る | 実機の確認で必要と分かる（段階 6 で判断） | 2026-10-31 |
| 特定の枠組み（nih-plug・clack など）の adapter と、テンプレートのプラグイン（D8） | 枠組みの依存と glue はアプリの所有範囲（[設計の埋め込み](../design.md#埋め込み)） | アプリが how-to の手順で wrapper を書く | 二つ以上のアプリが同じ glue を書く、または枠組みを決めた実アプリが要求する（戦略 3 章の 1 と 6）。所有者の判断で見送り（2026-10-31） | 2026-10-31 |
| DAW に読み込める確認用のプラグイン | repository は DAW plugin を持たず、framework-free の embed-host で確認（[設計の埋め込み](../design.md#埋め込み)） | 所有者が自分の wrapper で DAW の確認をする（任意） | 上の adapter を作るとき、または DAW での回帰が問題になったとき | 2026-10-31 |
| ARA の wrapper と、ARA の model の adapter（D10） | ARA SDK と model はアプリ固有で、Rust binding を中核に持たない（[設計の埋め込み](../design.md#埋め込み)） | アプリが ARA SDK（C++）の binding を持ち、`App::embed` で editor を開く | Rust から使える ARA の binding が現れる、または実アプリの要求 | 2026-10-31 |
| 外部の値を追跡する中核の API（`useSyncExternalStore` に当たるもの） | 既存の frame_loop/channel で実現でき、共通化の要求は未成立（[設計の埋め込み](../design.md#埋め込み)） | アプリの `frame_loop`、または channel と `spawn` | 二つ以上のアプリや kit が同じ仕組みを書く（戦略 3 章の 6） | 2026-10-31 |
| Windows の drag の hover と cancel | `RegisterDragDrop` は `OleInitialize` を要し、ホストの thread の状態を変える | file の drop だけを受け取る | ホストの thread の状態を変えない経路と実アプリの要求がある | 2026-10-31 |
| 埋め込みの touch・pen | 埋め込みの要求がない | mouse・keyboard を使う | 埋め込みの実アプリが touch・pen を要求する | 2026-10-31 |

## 実アプリの要求

出典がない項目は今回の再調査を根拠とし、旧 commit を推測しない。取込済みは現行実装で確認したもの。

| 操作 | アプリ | 出典（commit・issue） | 取込先（単位） | 状態 |
|---|---|---|---|---|
| DAW プラグインの editor の UI | 外部（所有者が確認） | 2026-10-31 の所有者の判断 | 埋め込みの改修（2026-10-31） | 取込済み（Windows/Linux X11 の実験。主要 DAW と両 OS の G-03 実機確認は未実施。制限は[埋め込みの手順](../../for-kivact-user/how-to-guide/embed.md)） |
| 取消しの共通セッション | 外部 | 810457ea、9f4553e0 | C-1 | 取込済み |
| 最後の client turn が layout だけを要求した後の host 起床 | 外部 | CODEX_REVIEW_FOLLOWUP.md Z の制御経路調査 | Z | 厳密条件は未再現。通常の `Host::turn` は `client.turn` 後に必ず `effects`・`layout` を実行する。既存 focus 試験は a11y 入力由来であり証拠に代用しない。post-layout の `on_mount` から幅を変更し、追加入力なしで反映する public E2E を追加。ただし waker/redraw と control 単独の起床を分離しないため、起床条件は変更しない |
| closure の直接指定 | 外部 | 今回の再調査（入門との不一致） | C-2 | 取込済み |
| dialog の幅 | 業務系 | 今回の再調査（固定幅） | C-3 | 取込済み |
| path の輪郭の読取り | モーショングラフィックス | ee54d2af | C-4 | 取込済み |
| font family の一覧 | モーショングラフィックス | 9cde3651、1b7cabd0 | C-5 | 取込済み |
| window の中央配置 | 業務系 | dea1f644 | C-6 | 取込済み |
| open・reveal・folder picking の手順 | 外部 | a5f33eb4 | C-7 | 取込済み |
| 拡大・pan・座標変換 | 外部 | 今回の再調査（editor の重複） | C-8 | 取込済み |
| Canvas region 上の共通 pan と標準 menu | 外部 | 今回の回帰試験（Node 見出し上で middle/Space+primary/wheel が届かない、region の右クリックで標準 menu が開かない） | C-8・D-7 | 取込済み。`Part::region_events()` はユーザーが承認した中核の追加。明示した部位だけ未処理region入力を受け、kit の独自転送と menu 専用配送の二重化を避ける |
| 拡大・pan の面で focus 中の部位を Space で起動しない | 外部 | 今回の回帰試験（node editor で node に focus を置き Space を押すと node が起動する） | C-8 | 取込済み。`Handled::PreventDefault` はユーザーが承認した中核の追加（2026-10-18）。伝播は続け、Enter/Space の activate などの既定動作だけを止める |
| worker の frame の受渡し | 外部 | 今回の再調査（renderer の所有境界） | C-9 | 取込済み |
| editor の操作の回帰試験 | 外部 | 今回の再調査（試験の不足） | D-1 | 取込済み（D-3で既知の再現2件も有効化し成功） |
| 用途別 editor とアプリの絵の表示 | 外部 | 3c6948cb、8404ff22、公式 kit の境界見直し（2026-10-19） | D-2 | 項目列は読取専用。EditorEvent の要求をアプリが反映・拒否し、Cancel で項目を復元する。画像・ID・名前・接続規則・再生クロックはアプリが所有 |
| 取消しのできる編集 | モーショングラフィックス | 810457ea、9f4553e0 | D-3 | 取込済み（全対象の取消し、D-1のignore2件を含むconsumer238件が成功） |
| 実単位の graph | モーショングラフィックス | 14f326c0、451f3e62、0e5c4d67、5da667f0、9f4553e0 | D-4 | 取込済み |
| 独立した color wheel | モーショングラフィックス | 3c6948cb | D-5 | 取込済み |
| Bezier とプレビュー | モーショングラフィックス | fce90184、bb85a142 | D-6 | 取込済み |
| gradient の編集 | モーショングラフィックス | 9cde3651 | D-6 | 取込済み |
| outliner の tone・placeholder・セル | モーショングラフィックス | 791f0f9a、3c6948cb | D-7 | 取込済み |
| lane セル・activation・context menu | モーショングラフィックス | 8404ff22、791f0f9a、c7279193 | D-7 | 取込済み |
| picker の複数列 | モーショングラフィックス | c40ea401 | D-8・review K-5 | 取込済み。`forward_keys` の3 methodsはユーザーが承認した中核の追加。引数を `impl IntoValue<Vec<Key>>` に揃え、実行中の一覧変更にも対応 |
| creative 表の共通の振る舞い | 外部 | 今回の再調査（表の重複） | E-1 | 取込済み（既存の共通table経路を確認） |
| セルの選択・draft・TSV・行の編集 | モーショングラフィックス | c7f90e7e | E-2 | 見送り（設計が約600行の単位を超える） |
| calendar の年・月の選択 | 業務系 | ccf5a2ff | F-1 | 取込済み |
| 期間入力の案内と閉じるボタン | 業務系 | ccf5a2ff | F-2 | 取込済み |
| tabbed dock | 外部 | 46e2c18a（現行 `dock.rs` を確認） | 既存 | 取込済み |
| outliner の drop | 外部 | 05907f55、公式 kit の境界見直し（2026-10-19） | 既存 | on_item_drop がある場合だけ drag を有効にして要求を報告。親と並び順の反映はアプリが所有 |
| Numeric の draft | 外部 | dba09ea8（現行 `numeric.rs` を確認） | 既存 | 取込済み |
| window の寸法 | 外部 | 345997b9（現行 `view/platform.rs` を確認） | 既存 | 取込済み |
| 日付の途中入力の保持 | 外部 | 2e0380d2（現行 `date.rs` を確認） | 既存 | 取込済み |
| 期間入力 | 外部 | ccf5a2ff（現行 `date.rs` を確認） | 既存 | 取込済み |
| 右クリックメニュー | 外部 | c7279193（現行 `interaction/menu.rs` を確認） | 既存 | 取込済み |
| 残り続ける toast | 外部 | dea1f644（現行 `interaction/toaster.rs` を確認） | 既存 | 取込済み |
| アプリの GPU texture を読み戻さずに表示する | クリエイティブ | 2026-10-26 の要求と測定（4K で CPU 経路 49〜66 ms）、spike branch `spike/kashiwade/gpu-interop` | 実験 feature `gpu-interop`、中立な共有 crate `gpu-share`（engine は feature `export`、wgpu の engine は `wgpu`、renderer は `import`） | 取込済み（実験）。既定で有効にする条件は、次のすべて：Linux 実機での e2e が通る（2026-10-26、Surface Pro 6・Intel UHD 620・Mesa 26.2.5 で達成）、Vulkan の validation layer で受け渡しの error が 0 件（同日、Linux で達成。naga の SPIR-V による `VUID-StandaloneSpirv-None-10684` は受け渡しの外で、別に扱う）、device の喪失と release の時間切れの e2e がある、実アプリ 1 つで使われる、wgpu の版上げを 1 回乗り越える |
