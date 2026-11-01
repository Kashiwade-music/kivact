# 完了後の手動検証

Windows GUI harnessの既定一周（6アプリ＋workbench icon）は2026-10-29にPowerShell 7 STAで実行し、補修後283成功・失敗0でした。人によるOS IME、スクリーンリーダー、ブラウザ表示の確認は未実施です。以下は自動一周と別の手動確認も含む一覧です。

## 2026-10-29 の自動GUI一周

| 対象 | 成功 | 失敗 |
|---|---:|---:|
| task-board | 66 | 0 |
| mini-postman | 49 | 0 |
| music-player | 38 | 0 |
| workbench | 40 | 0 |
| workbench icons | 10 | 0 |
| log-viewer | 40 | 0 |
| device-monitor | 40 | 0 |

`target/r1026/gui-final-attempt2/` に結果・screenshots・状態を保存し、7集計行と子/親終了0を確認しました。初回235成功・2失敗の `gui-final/` も保持しています。保存中JSONのreaderとwheel前のfocus前提を補修し、保存のatomic rename・focused row保持・既存assertionは維持しました。手動IME/Narrator/Orca、実機DPI/monitor・touch/pen、追加catalog/starter GUI、ブラウザ表示を、この自動成功の代わりには数えません。

## 確認手順

| 対象 | 操作 | 確認する結果 |
|---|---|---|
| Windows GUI harness | [既存の手順](tools/gui-harness.md)に従って debug の task-board・mini-postman・music-player・workbench と、ext-apps の log-viewer・device-monitor の計6アプリをビルドし、PowerShell 7 の STA mode で `pwsh -STA -NoProfile -File` を使い、 `rerun-round.ps1 -Round 1` を実行する | 各 scenario の失敗数が0。GUI evidence は ignored の target/gui-evidence に残す |
| 公式3kitの catalog | 同じ harness の catalog/starter scenario を、対象アプリ・承認時間・専用保存先を指定して実行する | 検索・選択・dialog・保存が動作し、識別子とfocus復帰を保つ。外観と配置は現在のtheme recipeに従い、旧pixel配置の再現は求めない |
| 日本語 IME | form と3kitの text input、ext-apps が使う external-component の SearchBox で、変換中の入力・確定・Escape取消・focus離脱を試す | preedit は binding に届かず、確定後に一度届く。composition の開始/終了通知が対応する。独自デザインシステム入力の自動 fixture E2E とも結果を照合する |
| 編集と caret | 日本語・emoji・結合文字の選択、Shift+矢印、Home/End、copy/cut/paste、長さ制限、複数行、横 scroll、親の scroll/transform を試す | selection と painted caret、IME 候補位置が一致。read-only は選択とcopyのみ、disabled は編集不可、password はcopy/cutと公開textを抑止する |
| Windows IME 再変換と DPI | 確定済み範囲の再変換を行い、harness の Fin-Dpi と実際の表示倍率/monitor移動も確認する | 範囲と候補位置が一致。文字・caret・a11y の矩形が同じ倍率で追従する |
| Windows Narrator / Linux Orca | 入力欄へ Tab で移動し、文字/行の読上げ、キー操作による選択の向きと移動、空欄、複数行を確認。List/Table、dialog の初期focusと閉じた後の復帰も操作する | accessible name/role/value/selection を読める。password の値と text run を公開しない。focus が対応する操作対象に留まる |
| Linux desktop | GNOME/KDE の Wayland/X11 上で fcitx/ibus、日本語入力、clipboard、file dialog を確認する | WSLg の自動試験だけでは確かめていない desktop/IME/portal 接続が動作する |
| ブラウザと rustdoc | 生成した公開 rustdoc と、入門・how-to・部品作者ガイドを開き、相互リンクとコード表示を確認する | 削除した型/計画書へのリンクがなく、最終 API と各ガイドへ辿れる |

GUI harness は全体の keyboard/mouse を操作します。自動試験に使った window-lock と別に harness 自身の GUI lock を使い、実行中は他の入力を行いません。手順の詳細、対象ウィンドウ、承認期限の指定は上記の harness ガイドに従います。

## A-1 入力語彙の一本化後の Windows IME

入力欄で日本語の変換・確定・再変換を行い、候補と選択範囲、確定後の文字列が維持されることを人が確認する作業は未実施。自動GUI一周の結果は上記の対象と範囲に限る。


## B-9 spin button と開閉部位

以下の GUI harness・スクリーンリーダー・IME の手動確認は未実施。

| 対象 | 操作 | 確認する結果 |
|---|---|---|
| 標準の number_input（Pop と Game の theme） | 実際の入力 field に Tab で入り、Up/Down、PageUp/PageDown、Home/End、Left/Right、Shift+Home/End を操作する | 6 種の spin キーで数値が変わる。Left/Right と修飾 Home/End は値を変えず caret/選択を動かす。Game の外 row に余分な tab stop がなく、内 field は TextBox のまま |
| Windows Narrator / Linux Orca の spin button | Increment/Decrement/SetValue、整数表現・小数・範囲外値を指定する | 名前・role・値域を読める。有限 AT 値を min 基準の整数へ丸めて clamp する。Basic の field 自身と Game の外 row が数値の操作対象で、Game の row へ value text を追加しない |
| Creative numeric の編集中 field / 通常の TextBox | caret を中央へ置き Home/End、Left/Right、IME の変換・確定を操作する | native な caret と選択、未確定文字を保持し、Creative の確定前の domain 値を変えない |
| Popup / Tooltip / Dialog の catalog と外部デザインシステム | 開閉、hover/focus による tooltip、dialog の dismiss と focus 復帰を操作する | free show に移した後も branch owner の寿命と focus が対応する。popover の anchor 欠落は surface の部位の構築時に同じ misuse message となり、失敗した Dialog surface を公開しない（自動 E2E でも確認する） |

## 標準 UI への委譲後のアプリとガイド

下表の6アプリとworkbench iconは上記の既定GUI一周で確認済みです。追加のinteraction-starter・catalog GUIとブラウザ確認は未実施です。各アプリの操作名・role・test id・node referenceとfocus復帰先を、対応scenarioの検索条件と照合します。主操作以外のbuttonはSecondary/Ghost、外観は現在のtheme recipeを基準とし、旧レイアウトの模写は要求しません。

| 対象・scenario | 操作と確認 |
|---|---|
| task-board / `scenario-taskboard.ps1` | 作成・編集・完了・削除、lane select popupを操作し、名前/role/test idとcheckboxの業務データを確認。lane移動後はcheckboxまたは「右へ」ボタンへfocusが復帰することを確認 |
| mini-postman / `scenario-postman.ps1` | URL・本文入力、送信・取消、履歴、scroll内のresponse表示とコピー、disabled条件とfocus復帰を確認 |
| music-player / `scenario-music.ps1` | 再生/停止、選曲、音量と補助操作の識別子、keyboard focusとthemeの強調を確認 |
| workbench / `scenario-workbench.ps1`・`scenario-workbench-extra.ps1`・`scenario-workbench-icons.ps1` | 一覧/検索/toolbar/外部icon buttonを操作し、残したdomain操作と標準部品のrole・選択・focusを確認 |
| ext-apps の log-viewer / `scenario-logviewer.ps1`・`scenario-logviewer-extra.ps1` | 検索/絞込み/読取り欄を操作し、外部SearchBoxのIMEと識別子、閉じた後のfocusを確認 |
| ext-apps の device-monitor / `scenario-devicemon.ps1`・`scenario-devicemon-extra.ps1` | device選択/更新/`Skin:`切替を操作し、外部部品の名前/role/test id、値とfocusを確認 |
| interaction-starter / `scenario-interaction-starter.ps1` | 検索→device選択→noteをEnterで保存し、検証error・retry・Calibrationのdragを操作。対応する名前/role/test id、focusとthemeの配置を確認 |
| 3kit catalog / `scenario-official-ui.ps1` | 選択付きnavigation・複合部品、read-onlyを含む標準text_areaと固有の数値/domain編集の境界を確認 |
| React/Solidガイド / `getting-started/react-solid.md` | ブラウザでREADME目次から開き、対応表・Rust5点・3つのコード例・参照リンクを確認。コードは自動docs test対象だがブラウザ表示は未確認 |

## B-7 button のスクリーンリーダー操作

Windows Narrator / Linux Orcaから標準buttonと独自部品のbuttonへInvokeを送る確認は未実施。同じクリック処理が一度だけ実行され、on_pressの再登録では後のhandlerが使われること、disabledでは実行されないことを確認する。canvas regionのon_a11y_event(Activate)は独立した対象として残す。

## dogfooding 後続の部品と window 配置

以下は自動試験とは別の手動確認で、未実施。

| 対象 | 操作 | 確認する結果 |
|---|---|---|
| Windows の複数 monitor と異なる DPI | 主 monitor を切替え、通常寸法と作業領域より大きい window を起動する | taskbar を除く作業領域の中央へ配置し、負の座標と DPI を考慮する。大きい軸は作業領域の始点になる |
| Creative の timeline・dope sheet・graph・node・色・曲線・gradient・outliner | Narrator / Orca で領域を読み、playhead、color wheel、graph key に SetValue を送る | 名前・role・現在値・選択を読み、値と描画が一致する。lane のセルの操作は行の選択と独立する |
| Creative の連続編集 | 実際の pointer で drag し、途中で Escape を押す。pointer を取り消す場合も試す | 編集開始前の値へ戻り、取消し後の move/up が値を再変更しない |
| Creative の viewport_surface | touchpad の pinch、touch screen の二本指の pinch、中ボタン drag、Space と主ボタン drag を操作する | host が wheel として届ける拡大操作、二本の指の pinch と pan が絵・重ね描き・region を同じ座標へ移す。touchpad の専用 pinch event は現在の公開入力 API にないため、その配送は保証しない |
| 複数の pointer（Windows の touch screen と pen） | 二本の指で二つの drag 可能な node（splitter）を同時に動かし、片方の指を離す・system の gesture で取り消させる。mouse を button に置いたまま指で drag し、mouse で click する | 各指が自分の値だけを変え、片方の指の取消しが他方の drag を止めない。hover・click・focus は mouse と最初の指だけから起き、2本目の指は click しない |
| Pop の期間入力 | 開始日を選び、案内文を読み、Close で閉じる | polite の案内が終了日の選択へ変わり、閉じた後に開始日の入力欄へ focus が戻る |
| Creative の複数列 picker | Tab・Shift+Tab・左右キー、検索、pointer で列を移る | 有効候補のある列へ移り、現在の候補を読み、選択後は起点へ focus が戻る |

## Creative editor の操作規則

以下の実際の mouse・trackpad による確認は未実施。

| 対象 | 操作 | 確認する結果 |
|---|---|---|
| E-2：node editor / viewport_surface とアプリ全体の Space 操作 | Space に再生・停止を割り当て、editor に focus を置いて Space の押下・解放と Space+主ボタン drag を試す | 通常の Space がアプリへ届き、editor の中で focus 中の node は起動せず、Space+drag は pan する。pan 開始時にも全体の再生・停止が起き、Space を先に解放しても主ボタンを離すまで pan を続けられる |
| E-5：time（timeline / dope sheet）・graph・node・arrangement | 実際の mouse と trackpad で、[利用者ガイド](../for-kivact-ui-user/how-to-guide/creative.md)の6操作の表を各系統について試す | 押した対象の静止 double click、修飾 marquee、Ctrl/Meta+A、Escape、pointer 中心の拡大、複数選択からの drag と静止 release が表に従う |
| E-3：多数トラックの arrangement | 15 トラックを用意し、wheel と中ボタン pan で最下段まで移動する。描かれた loop の両端と本体を drag する | 最下段まで到達し、内容の上下限を越えない。loop の端の当たり判定が描画 region と一致し、本体の drag では範囲の長さを保つ |

## GPU texture 共有（実験 feature `gpu-interop`）の Linux 実機

Linux の実機では、2026-10-26 に Surface Pro 6（Intel UHD Graphics 620、Mesa 26.2.5）で全項目を確かめた。別の GPU や driver（NVIDIA、RADV）で行い、結果を報告する。ハードウェア GPU（NVIDIA、または Mesa の RADV・ANV）の Vulkan が必要で、WSLg など GPU を持たない環境では確かめられない。

[dev/tools/linux/verify-gpu-interop.sh](../../dev/tools/linux/verify-gpu-interop.sh) が、clone から下の表の試験、Khronos の validation layer の下での再実行、報告の作成までを行う。この 1 file を実機に写し、desktop の session の端末（`WAYLAND_DISPLAY` か `DISPLAY` がある所）で `bash verify-gpu-interop.sh` を実行する。Vulkan の loader・driver・`vulkan-tools`・validation layer がなければ `--install-deps`（sudo で apt・dnf・pacman）を付ける。private の repository には、`gh auth login` と `gh auth setup-git`、または `--repo git@github.com:Kashiwade-music/kivact.git` が要る。終わると clone の隣に `kivact-gpu-interop-report-<時刻>.tar.gz` ができる。

Linux では Direct3D 12 の試験が `SKIPPED ..._dx12` を出す（想定どおり）。storage の texture を共有できない driver では `..._storage_...` の試験も skip する（任意）。それ以外の `SKIPPED`（Vulkan の試験、backend を名前に持たない creative の試験）は、GPU の経路を通っていないことを表し、確認にならない。

| 対象 | 操作（手で行う場合） | 確認する結果 |
|---|---|---|
| Arvane の契約の試験 | `dev/tools/window-lock.sh -- cargo test -p arvane --features gpu-interop,testing --test e2e external -- --nocapture` を実行する | すべて `ok`。`SKIPPED` は `_dx12` と storage の試験だけ |
| 公開 API の試験 | `dev/tools/window-lock.sh -- cargo test -p kivact-public-api-tests --features gpu-interop --test e2e gpu_interop -- --nocapture` を実行する | すべて `ok`。`SKIPPED` は `_dx12` の試験だけ（Vulkan の試験と、`shared_image_follows_requests_with_a_wgpu_engine_on_vulkan` が実際に走る） |
| creative kit の consumer 試験 | `dev/tools/window-lock.sh -- cargo test -p kivact-ui-creative --features gpu-interop --test consumer gpu_interop -- --nocapture` を実行する | `SKIPPED` の行がなく、3 件とも `ok` |
| wgpu の engine の読み戻し（window なし） | `cargo test -p gpu-interop-producer --features wgpu -- --nocapture` を実行する | すべて `ok`。`SKIPPED` は `_dx12` の試験だけ |
| validation layer | 上の試験を `VK_LOADER_LAYERS_ENABLE='*validation'` と `VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation` を付けて実行する | `VUID-StandaloneSpirv-` 以外の `Validation Error` がない（shader の作成で出る `VUID-StandaloneSpirv-None-10684` は naga の SPIR-V の既知の問題で、受け渡しとは関係しない） |

報告してほしい項目：スクリプトが作る `.tar.gz`（GPU・driver・各試験の出力・validation layer の結果を含む）。手で行った場合は、GPU（名前と種類）、driver（NVIDIA の版、または Mesa の版）、上の command の出力（`SKIPPED` の行を含む）。

## 埋め込み（Windows、実験）

以下はすべて **未実施（実際の人による操作が必要）**。対象・操作への許可を得て実施し、OS・IME・倍率・結果を記録する。DAW の正式対応を宣言するには主要な DAW（REAPER、Bitwig Studio、Ableton Live、Cubase など）と両 OS の G-03 の実機確認が必要。自動 message の E2E は実機の IME/Narrator の確認を代替しない。 Alt+F4 のホストへの close request と、左 drag 中の右ボタンによる capture の維持は Windows の自動 E2E で確認する。

`dev/tools/window-lock.sh -- cargo run -p kivact-embed-host -- --scale 1.25` で起動する。通常終了は右上の閉じる操作。`--two` は同じ800×600物理pxの親に別々の App を左右に表示する。自動起動確認のみは `--smoke`（5秒後終了）。

| 対象 | 人が行う操作 | 期待する結果 | 状態 |
|---|---|---|---|
| MS-IME | 名前の textbox をクリックし、日本語入力に切替え「にほんご」を変換、候補を上下キーで選び Enter。続いて再入力中に Escape。確定文字を選択して再変換 | preedit と確定が重複せず、候補が caret 付近に出る。取消しで未確定文字だけ消え、再変換位置と確定結果が一致 | 未実施 |
| focus と IME | 入力途中で button をクリックし、textbox に戻る。Tab/Shift+Tab で移動し、125%・150% と `--two` でも繰返す | focus が正しく移り、IME 状態・caret・候補位置が別 editor と混ざらない | 未実施 |
| Narrator | Narrator を起動し Tab で textbox、Increment button、Gain slider をたどる。button を Space、slider を矢印キーで操作 | 役割・名前・値が読まれ、button の count と slider の値が操作後に反映される。二つの editor の focus を識別できる | 未実施 |
| file/folder/save dialog、STA/MTA | 確認用 wrapper の UI に公開 platform API で picker/save を開く button を用意する。ホスト自身が先に STA または MTA を設定した GUI thread で editor を開き、dialog の選択・取消し、日本語と空白を含む path、title・初期 directory・file name・filter を確認する。ホストの `CoGetApartmentType` を呼出前後に記録する | owner がホストに対応し、結果と取消しが一度だけ届き、App callback が再入しない。ホストの apartment が変わらない。通常の embed-host はこの button を持たないため、実施には確認用 wrapper が必要 | 未実施 |
| DAW（正式対応に必要な確認） | 自分の wrapper を DAW に読み込み、20回開閉、複数 editor、125%/150%、parameter 自動化、drag と Escape、textbox と DAW shortcut を試す | 値と倍率が復元され、gesture が begin/set/end、取消しは書戻し後に end。キーが奪われる条件を記録する | 未実施 |

Linux は WSLg の X11/XWayland の自動 E2E のみを検証対象とし、実機 DAW（Bitwig/REAPER）・IME・screen reader は未実施。実機の IME・AT-SPI・dialog と DAW は、下記手順による確認が必要。


## Linux の埋め込み（X11）

以下は未実施。自動試験は OS dialog と IME server を起動しない。2026-10-31 の WSLg の読み取り調査では `DISPLAY=:0`、session bus socket はあり、`XMODIFIERS` は空、root の `XIM_SERVERS` atom はなく、AT-SPI bus socket・fcitx・ibus・portal の process は見つからなかった。AT-SPI の D-Bus service 定義は存在した。bus の起動は行っていない。

- fcitx5 または ibus を利用者の desktop session で準備し、XIM を有効にする（それぞれ `XMODIFIERS=@im=fcitx`、`@im=ibus`）。同じ環境から `kivact-embed-host --scale 1.25 --two` を起動する。textbox に「にほんご」を入力して変換、候補の選択、Enter で確定、Escape で取消しを行う。preedit の途中の編集も試す。文字が二重に入らず、候補が caret に追従し、別 editor へ focus を移した後も独立して入力できることを確認する。XIM の reconversion は未対応。
- XIM の順序と保留（未実施）：上記の実 IME server を使い、UI の処理が忙しい間に未消費の通常キーの press/release を続けて入力し、同じ native batch からでも Key → KeyUp の順で届くことを確認する。同じキーを再度押したときに誤った repeat にならず、長押しでは repeat が続き、解放で止まることを確認する。XIM 応答待ちに別の入力欄へ focus を移す・IME を解除し、旧 pending の文字やキーが新しい欄へ届かないことを確認する。利用者が管理する検証用 XIM server で応答を保留した場合も描画・ほかの OS event・close が動き続け、保留キーが1024件を超えるか応答が500 ms 途絶えると、診断を出して XIM が無効になり、保留キーが順序どおり通常入力として届いて App は動き続けることを確認する。自動で server を起動する試験は追加しない。
- XIM 取消し時の release（未実施）：popup 内の textbox から Shift+Tab で trigger へ戻り、press/release が同じ native batch にある場合も Tab の release によって popup が閉じることを確認する。別のキーを長押ししたまま text と button の間を移動し、ほかの XIM flight／queue を取り消しても、無関係な held key の repeat が初回 press に戻らず、余分な activation が起きないことを確認する。取消した release は配送済み down だけを補償し、二重の KeyUp を出さないことも確認する。
- AT-SPI bus と Orca を利用者の desktop session で起動し、二つの editor の textbox・button・slider の名前、role、値、focus を読む。button の実行と slider の値変更が一度だけ App に届くことを確認する。`python3-pyatspi` がある環境では window lock の下で `cargo test -p kivact-public-api-tests --test e2e x11_atspi_client_reads_the_embedded_button -- --ignored --nocapture` を実行できる。この試験は bus を起動しない。
- 確認用 wrapper の UI に公開 platform API で file/folder picker と save dialog を開く button を用意し、選択・取消し・日本語と空白を含む path を確認する。通常の embed-host はこの button を持たない。XDG portal には editor の XCB child XID を親として渡す。rfd 0.17.2 の内部 zenity fallback は親 window を指定しない既知の制限がある。dialog は手動確認のみとし、自動試験では開かない。
- library の保持は arboard、accesskit_unix、rfd の初回利用前に行う。rfd の同期 portal 経路には専用 worker はないが、zenity fallback の worker は結果通知後の終了処理が caller 復帰後に残り得るため、dialog も保守的に保持する。

### module の保持と unload（未実施）

利用者が管理する DLL/so の wrapper と loader host を使う。static executable の embed-host と保持警告だけでは実 DLL の検証にはならない。通常の20回開閉に加え、取消しを無視して3秒続く blocking job、callback 内の editor close、Windows UIA の activation、Linux AT-SPI/clipboard の利用をそれぞれ確認する。editor を閉じてからホスト側で `FreeLibrary`/`dlclose` を行い、残る worker の終了、native window の消滅、終了後に App callback が届かないこと、再 load/作成後の操作を観測する。callback/root からホスト GUI thread を待つ実装は使わない。

待機上限や library のために保持した module は process 終了まで mapping が残るのが期待値であり、即座の unmap や更新版への hot reload は成功条件にしない。mapping は process の module 一覧や `/proc/<pid>/maps` などで読み取り、観測のために loader の参照を追加して保持を偽装しない。正常終了と上限超過を分けて、thread 名、保持理由、mapping と host の戻り値を記録する。
