# 作者による手動確認

この一覧は、自動 E2E では確かめられない native 経路を作者が実機で確かめるための、リリース前の確認項目です。
入力・支援技術・埋込み host・描画の寿命に関わる native 経路を変えたときに、該当する行を実施します。
自動 E2E・静的検査・doctest の成功は、実機での確認済みとは数えません。
境界と既知の対応範囲は [アーキテクチャ](architecture.md)、理由は [設計の決定](decisions.md) を参照します。

各行の確認後に、commit、OS・表示 backend、IME／reader／DAW の製品と版、GPU・driver、
操作結果、必要なら画面・読み上げ・診断を PR の説明かリリースの記録に残します。利用できない環境は「未確認」とし、
成功に数えません。アプリや mirror を開く操作は共通の `dev/tools/window-lock.sh` を通します。

## 文字入力と native accessibility

実際に操作する native アプリで行います。mirror は観察用なので、IME や reader の操作確認の代用にはしません。
Windows と、利用可能な Linux Wayland／X11 の各環境を区別して記録します。

| 対象・環境 | 操作 | 期待する結果 | 変更経路の参照 |
|---|---|---|---|
| 日本語 IME：Windows の日本語 IME、Linux の利用可能な IME | 既存の入力例で「東京」、絵文字、結合文字を含む文を入力する。未確定文字列の範囲を変え、候補選択・確定・取消を行う。scroll と表示倍率変更後も続ける。 | 未確定表示と確定値が二重にならない。候補位置が caret に追従し、UTF-16 と UTF-8 の境界の違いで文字や選択範囲がずれない。取消で確定済みの文を消さない。 | [入力正規化](../../crates/arvane-shell/src/pipeline/ime.rs)、[Windows IME](../../crates/arvane-native/src/port/win32/ime.rs)、[X11 IME](../../crates/arvane-native/src/port/x11/ime.rs)、[engine IME](../../crates/kivact-engine/src/model/edit/ime.rs) |
| 選択・undo と custom `TextEditing`：同じ実窓 | 通常の入力欄と custom editor で、日本語確定後に Shift＋矢印、ドラッグ選択、コピー・貼付け、undo／redo を行う。変換中の Enter と Escape、別の欄へ移った後の key-up も確認する。 | native の選択・caret と描画が一致する。編集が重複せず、undo／redo が受理した編集単位を復元する。変換の確定キーで別の操作を誤起動せず、移動先や破棄済み editor へ古い入力を配送しない。 | [編集の公開口](../../crates/kivact-core/src/custom/text_editing.rs)、[editing journal](../../crates/kivact-editing/src/journal.rs)、[custom editor の操作契約](../../dev/tests/kivact-public-api/tests/e2e/custom_text_field.rs) |
| reader と論理構造：Windows UIA の reader、Linux の利用可能な AT-SPI reader | 名前付き Group の中に標準部品・`Contents`・動的な子を持つ画面を読む。Tab で移動し、子を追加・非表示・除去する。名前とは別の description がある入力欄も読む。 | layout が透明でも意味のある Group と子の順序を保つ。名前・役割・状態・description の関係を読み、同じ文を重複して名前に混ぜない。非表示・除去済みの対象を操作先に残さない。 | [a11y tree](../../crates/kivact-engine/src/a11y/build/node.rs)、[native 変換](../../crates/arvane-native/src/a11y/conversion.rs)、[description の契約](../../dev/tests/kivact-public-api/tests/e2e/metrics_and_description.rs) |
| canvas／viewport の領域：同じ reader と native 入力 | 独立した名前・action を持つ二つの描画領域へ順に移動し、片方を操作する。無効化、未描画、除去も行う。 | 宣言した論理順序、領域ごとの名前・bounds・focus・action が一致する。隣の領域の action を実行せず、未描画や消えた領域を reader が操作しない。 | [region の契約](../../dev/tests/kivact-public-api/tests/e2e/parts.rs)、[native a11y](../../crates/arvane-native/src/a11y/native.rs) |
| 数値部品の表示・写像・微調整：Creative カタログの Rotary Knob / Channel Fader と実アプリ | Gain ノブが dB 表示になり、Drive ノブは Enter で中央のオーバーレイに型入力、周波数ノブは対数で中央付近が約 632 Hz、Fader は下半分で 0.25。Ctrl を押したままスクラブしても飛ばない。Shift+ホイールで増加しない。小さなホイールデルタ 3 回で 1 ステップ。PageUp/PageDown で 10 ステップ。Alt+クリックで reset_to。 | 表示・a11y の値テキスト・入力欄の初期文字列が同じ書式になる。スクラブ中に修飾キーを押しても値が跳ねない。1 編集 = 1 EditPhase（スクラブは押下から解放まで）。 | [scrub](../../crates/kivact-ui-creative/src/numeric/scrub.rs)、[mapping](../../crates/kivact-interaction/src/policy/scalar_mapping.rs)、[modifier re-anchor](../../crates/kivact-interaction/src/drag/handle.rs) |

## 埋込み host と描画の寿命

| 対象・環境 | 操作 | 期待する結果 | 変更経路の参照 |
|---|---|---|---|
| DAW の実 editor：Windows の対象 DAW、利用可能なら Linux X11 host | plugin editor を開き、host から resize する。要求した寸法を host が補正する場合、補正後に再度 resize する。host と editor の間で focus を移す。 | 受理・補正された実寸法で layout と入力座標が一致し、古い要求値へ戻す resize loop が起きない。editor を開く・操作する際の focus が host と競合しない。 | [embed 寸法](../../crates/arvane-native/src/port/embed/geometry.rs)、[resize 補正の契約](../../dev/tests/kivact-public-api/tests/e2e/embed/contract.rs) |
| DAW の close／再open と background wake：同じ host | 非同期処理または協調キャンセルする blocking 処理の実行中に editor を閉じ、再open する。結果の到着と close が前後する場合も確認する。 | 閉じた UI へ再描画・focus・callback を戻さず、再open した owner と混同しない。GUI waker が host を固めない。停止しない worker は規定の待機後に診断・detach し、実行中の code module を早く unload しない。 | [embed session](../../crates/arvane-native/src/port/embed/session.rs)、[GUI wake](../../crates/arvane-native/src/wake.rs)、[blocking shutdown](../../crates/kivact-runtime/src/task/blocking.rs) |
| `viewport` の CPU 経路：Windows／Linux、GPU 使用時と CPU 指定時 | 同じ画像・overlay・zoom／pan を表示し、resize と表示倍率変更を行う。CPU renderer でも提供した `Pixels` を表示する。 | 表示範囲、拡大率、pointer の content 座標が一致する。CPU fallback でも用意した pixel 表示が続き、GPU frame を CPU pixel と誤って扱わない。 | [viewport](../../crates/kivact-core/src/view/elements/viewport.rs)、[座標と CPU 出力の契約](../../dev/tests/kivact-public-api/tests/e2e/viewport.rs) |
| 共有 texture：Windows D3D12 同士／Vulkan 同士、Linux Vulkan の対応環境 | 外部 renderer の連続 frame を表示しながら resize、pool の交換、owner の破棄を行う。再open 後は新しい pool で再開する。 | slot／fence の順序を保ち、古い pool や破棄した画像を再利用しない。resize 後の寸法が一致し、close 後は producer が終了を観測する。対応しない import は `Unsupported` 等の理由を返し、CPU pixel 経路を持つ構成ではその経路へ移る。 | [共有契約](../../crates/kivact-core/src/gpu_interop.rs)、[import](../../crates/kivact-core/src/gpu_interop/import.rs)、[共有 viewport の契約](../../dev/tests/kivact-public-api/tests/e2e/viewport_gpu_interop.rs) |
| 埋め込みの表示切替：Windows の CLAP/VST3 ホスト、Linux X11 host | ホストの gui.hide / gui.show（VST3 はウィンドウ表示）でエディターを隠して再表示する。`Parent::hidden` で開くホストと、非表示中の Space も確認する。 | 非表示中は描画と frame loop が止まり `platform::is_surface_active()` が false。再表示で描画が戻り、`Embedded::is_visible` が一致する。非表示にしたときフォーカスが親へ戻り、押していたポインターが取り消される。 | [surface](../../crates/arvane-native/src/port/embed/surface.rs)、[embedded](../../crates/kivact-core/src/embed/embedded.rs) |
| 確定待ちリサイズ：Windows の VST3/CLAP ホスト、Linux X11 host | UI から `platform::request_window_size` を要求し、ホストが `Pending`（VST3 は `resizeView`→`onSize`、CLAP X11 は post した `request_resize`）で応答する。確定前に子の寸法が変わらないこと、確定後に一致すること、別サイズを選ばせたときと 2 秒応答なしのときも確認する。 | Pending 中は子の寸法が変わらず、onSize で要求サイズに一致して成功する。ホストが別サイズを選ぶと Platform エラー、2 秒応答なしで Timeout。子はその間も描画を続ける。 | [resize](../../crates/arvane-native/src/port/embed/resize.rs)、[window](../../crates/arvane-shell/src/window.rs) |
| 未処理キー：Windows の CLAP/VST3 ホスト、Linux X11 host | `Parent::forward_unhandled_keys` を付けたエディターで、フォーカスがエディターにある状態の Space、テキスト入力中の文字、Enter でのボタン活性化、Escape での popover 閉鎖を順に行う。入力欄にフォーカスがある状態で Space が文字入力になることも確認する。 | 未処理の Space はホストの再生/停止を動かし、押下と release が対で届く。入力・活性化・破棄に使ったキーはホストへ返らない。IME 変換中のキーは返らない。 | [key dispatch](../../crates/kivact-core/src/client/dispatch/key.rs)、[keys](../../crates/arvane-native/src/port/embed/keys.rs) |
| DAW の panic の報告：Windows の対象 DAW と Linux X11 host | plugin editor を開いた状態で editor の handler を panic させ（試験用の build か、panic を起こす操作）、そのまま host の GUI を操作し続ける。root を panic させた editor も開く。 | host の process が abort せず、`Embedded::status` が `Panicked` を返し、以後の入力で editor のコードが動かない。root の panic では `App::embed` が `Err(ErrorKind::Panicked)` を返し、子ウィンドウが残らない。close 中の cleanup の panic でも drop が戻る。 | [panic 境界](../../crates/arvane-native/src/port/win32/proc.rs)、[PanicSlot](../../crates/arvane-shell/src/child.rs)、[子プロセスの試験](../../dev/tests/kivact-public-api/tests/e2e/embed/panic_boundary.rs) |
| OS のスケール変更の通知：Windows の対象 DAW | editor を開いたまま、モニターの表示スケールを変える（または DPI の異なるモニターへウィンドウを移す）。 | `Parent::on_notice` が `EmbedNotice::ScaleChanged` を 1 回受け取り、wrapper が `scale()` を読んで新しい物理サイズを要求でき、論理サイズが保たれる。X11 では通知が来ない。 | [Win32 のスケール変更](../../crates/arvane-native/src/port/win32/proc.rs)、[notice](../../crates/kivact-core/src/embed/notice.rs) |

## mirror と実アプリの入力の分離

| 環境・操作 | 期待する結果 | 変更経路の参照 |
|---|---|---|
| 利用可能な Windows／Wayland／X11 で、`KIVACT_TEST_MIRROR=required` の台本 E2E を観察する。frame 更新、resize、scale、focus・選択の変化を見る。 | mirror は同じ App の実際の frame と a11y tree を表示する。別の見本画面や古い frame に固定されず、仮想時計で進めた結果に追従する。 | [virtual mirror](../../crates/arvane-virtual/src/mirror.rs)、[native mirror](../../crates/arvane-native/src/mirror.rs) |
| mirror に物理 mouse／key や reader の action を送り、その後に台本の入力を続ける。別途、通常起動した native アプリを同じ物理入力で操作する。 | mirror の OS 入力・支援技術 action は台本へ混入しない。通常の native アプリでは入力を受け、二重配送せずに同じ操作を行える。mirror の観察成功と native 入力の確認成功を別々に記録する。 | [mirror の入力契約](../../crates/arvane-native/src/mirror.rs)、[native event](../../crates/arvane-native/src/port/x11/event.rs)、[共通入力](../../crates/arvane-shell/src/pipeline/ime.rs) |

## 描画と動きの設定

| 対象・環境 | 操作 | 期待する結果 | 変更経路の参照 |
|---|---|---|---|
| shader の見た目：Windows の GPU 描画と CPU 指定、Linux の GPU 描画 | Catalog の Effects にある scanline 背景（`PaintShader` と `fallback`）を GPU と CPU で表示し、時間の経過を見る。pipeline の作成に失敗する構成で fallback の見た目を確かめる。 | GPU では WGSL の結果が時間とともに動き、CPU と fallback では宣言した代わりの見た目が描かれる。色は theme の token の値を保つ。 | [shader の検証](../../crates/kivact-shader/src/lib.rs)、[paint shader](../../crates/kivact-style/src/shader/paint_shader.rs)、[GPU の compositor](../../crates/arvane-compose/src/gpu.rs) |
| Linux の reduced motion：GNOME と KDE | 各デスクトップの「アニメーションを減らす」設定を切り替え、Effects の boot sequence と parallax HUD の動きを見る。 | 設定が reduced motion として反映され、装飾の動きは止まる。essential と指定した動きは続く。設定を読めない環境では既定の動きのまま、理由を「未確認」として記録する。 | [Linux の設定](../../crates/arvane-native/src/port/linux_settings.rs) |
| X11 の refresh rate：Linux X11 の 60 Hz と 120 Hz 以上の display | Effects の scanline 背景（`FrameRate::Display`）の frame 間隔を見る。display を別の refresh rate へ移す。 | `FrameRate::Display` は表示の refresh rate に合い、`FrameRate::Hz` の指定は指定した値に近い。移動後も追従する。 | [X11 window](../../crates/arvane-native/src/port/x11/window.rs) |
| drawable の GPU export：Windows の D3D12・Vulkan、Linux の Vulkan | Effects の 3D 設定パネル（drawable）を `Viewport::drawable` で export し、外部 renderer の共有 texture に表示したまま、回転、resize、pool の交換、owner の破棄を行う。 | export の画像は drawable の表示と一致する。consumer が import の fence を signal するまで slot を書き換えず、close 後は producer が終了を観測する。対応しない import は理由を返す。 | [drawable の share](../../crates/kivact-core/src/view/drawable_share.rs)、[drawable の mount](../../crates/kivact-core/src/view/drawable/mount.rs) |

## 複数 window と DevTools

native でしか確かめられない項目です。自動 E2E では確認できません。

| 環境 | 確かめること |
|---|---|
| Windows | `Tool` の window が親の上に残り taskbar に出ない。`Modal` は親が無効になり、閉じると戻る。`CenterOnParent`・`NextToParent` の位置。window ごとの focus と Tab 順。2 つ目の window の IME。title bar の閉じるが `on_dismiss` に届く。`platform::focus_window()` が window を前面に出す（出せないときは task bar の点滅） |
| X11 | `Utility`・`Dialog` の window type |
| Wayland | 位置が `Auto` に落ちる。`focus_window()` は何も起きないが error にならない |
| DevTools | 実際の native の App で F12・Ctrl+Shift+I・Ctrl+Shift+C が DevTools アプリを起動する／すでに開いていれば前面に出る。Ctrl+Shift+C は押した window で pick を始める。pick と重ね表示が内容の上に描かれる。DevTools アプリの「Open source」が editor を開く（Windows の `code.cmd` を含む） |
| embed（DAW など） | host の中でも F12・Ctrl+Shift+I・Ctrl+Shift+C が DevTools に届く。`inject_input` は `-32015` で断られる |
