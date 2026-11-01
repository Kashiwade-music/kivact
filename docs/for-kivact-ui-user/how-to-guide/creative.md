# Creative で共有する数値を編集する

number と slider に同じ `Binding<f64>` を構築関数の引数で渡すと、一方の編集が他方へ反映されます。部品ごとに値を複製して同期する effect は不要です。部品が値を隠れて持つことはありません。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as creative;

fn inspector() -> impl IntoView {
    let gain = signal(0.5_f64);
    column((
        creative::number("Gain", gain).range(0.0, 1.0).step(0.01).precision(2),
        creative::value_slider("Gain slider", gain).range(0.0, 1.0).step(0.01),
        text(move || format!("現在値: {:.2}", gain.get())),
    )).style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Inspector")
        .theme(creative::theme(kivact::style::ThemeMode::Dark))
        .run(inspector)
}
```

`creative::theme(mode)` は Iris の accent と通常の密度の `Theme` です。accent と密度は `ThemeOptions` の method で替えます（`use kivact_ui_creative::ThemeOptions as _;` のうえで `creative::theme(mode).accent(creative::Accent::Lime).density(creative::Density::Compact)`）。

range、step、単位、精度は用途に合わせて指定します。Backspace と double click で戻す値は `.reset_to(..)` です。数値の部品は中核の `use_scalar` の上にあり、drag・キー・wheel・支援技術の書込みはすべてその scalar の編集です。長い操作を履歴の一回にまとめる必要がある場合は `on_edit` の EditPhase を使い、履歴自体はアプリの model に持ちます。複数選択の異なる値を示す `mixed` と、実際の値の binding は区別します。

timeline、outliner、dock の項目と選択も caller が所有します。表示部品の中へ保存や業務上の正本を複製しません。テキストを直接編集する場合は標準 `ui::text_input` と headless `use_text_edit` が共通の IME/caret 経路を使います。独自の数値式を認める場合は parser と検証結果を明示します。

排他的な値の選択（mode や補間方法の切替）は、標準の `radio_group` に Creative の型 `Segmented` を `.variant(..)` で渡した segmented control です。既定の slot が各 segment の icon と label を描き、`.orientation(Orientation::Horizontal)` で横に並べます。tabs ではなく radio の組なので、矢印キーで移動と選択を行い、端では反対側へ回ります。選択ごとの編集の報告は `radio_group` の `.on_edit(..)` です。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as creative;

fn interpolation() -> impl IntoView {
    let mode = signal(Some("linear"));
    radio_group(
        "補間",
        vec![
            ListItem::new("constant", "Constant"),
            ListItem::new("linear", "Linear"),
            ListItem::new("bezier", "Bezier"),
        ],
        mode,
    )
    .orientation(Orientation::Horizontal)
    .variant(creative::Segmented)
}
```

`cargo run -p kivact-ui-creative-catalog` で数値編集、drag、直接入力、editor の選択を確認できます。pointer だけでなく矢印キー、focus、AT の値変更も、部品の公開契約を通して試験してください。

## 編集部品へ項目を渡す

`kivact_ui_creative` では項目列を読取専用の `impl IntoValue<Vec<T>>` で渡します。clip は `timeline(clips, state)`、key は `dope_sheet(keys, state)`、curve は `graph_editor(curves, state)`、typed node は `node_editor(nodes, links, state)` です。変更は `on_edit` の `EditorEvent` で要求され、アプリが適用または拒否します。Begin で項目を保存し、Cancel ではその項目へ戻します。選択・zoom・pan と playhead などの view の状態は、editor ごとの `TimeState`・`GraphState`・`NodeState`・`ArrangementState` に binding としてまとめ、構築関数の最後の引数で渡します。`TimeState::new()` は既定の値の signal を作ります。アプリが持つ値は field を置き換えて渡します（`TimeState { playhead: playhead.into(), ..TimeState::new() }`）。`Lane::new(id, label)` の ID を item に保存するため、表示順を変えても所属は変わりません。

アプリが描いた viewport や解析画像は `viewport_surface(IntoValue<Option<Pixels>>)`、色の編集は `color_wheel(Binding<Color>, Binding<ColorModel>)` を使います。画像の作成と worker からの受渡しは [Canvas](../../for-kivact-user/how-to-guide/canvas.md) を参照してください。

## 時間の変更要求をアプリで反映する

用途ごとの部品を選びます。`timeline` は `Clip`、`dope_sheet` は `Keyframe`、`graph_editor` は `Curve` の一覧を受けます。位置と長さはアプリの時間単位です。timeline と dope_sheet は一覧を読取専用で受け、`EditorEvent<TimeRequest>` を `on_edit` へ報告します。アプリが要求を反映した一覧を描画し、反映しなければ項目は動きません。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn animation() -> impl IntoView {
    let clips = signal(vec![ui::Clip::new(1, 10, 0.0, 2.0, "Intro")]);
    let selected = signal(Vec::new());
    let playhead = signal(0.0_f64);
    let lanes = signal(vec![ui::Lane::new(10, "Video")]);
    let initial = signal(None);
    let state = ui::TimeState {
        selection: selected.into(),
        playhead: playhead.into(),
        ..ui::TimeState::new()
    };
    ui::timeline(clips, state)
        .on_edit(move |event| match event {
            ui::EditorEvent::Begin => initial.set(Some(clips.get())),
            ui::EditorEvent::Request(request) => clips.update(|clips| match request {
                ui::TimeRequest::Retime(values) => {
                    for clip in clips {
                        if let Some(value) = values.iter().find(|value| value.id == clip.id) {
                            clip.start = value.time;
                            clip.duration = value.duration;
                        }
                    }
                }
                ui::TimeRequest::Delete(ids) => clips.retain(|clip| !ids.contains(&clip.id)),
            }),
            ui::EditorEvent::Commit => initial.set(None),
            ui::EditorEvent::Cancel => {
                if let Some(saved) = initial.get() { clips.set(saved); }
                initial.set(None);
            }
        })
        .lanes(lanes)
        .step(0.5)
        .time_format(|seconds| format!("{seconds:.1}s"))
        .name("Animation")
}
```

lane の `id` は一覧の index と分けます。並べ替えても同じ ID のままにし、非表示にする lane は一覧から外します。明示的な空一覧はすべて非表示です。`Keyframe::duration` が正なら、点の代わりに長さのあるバーになります。

`Timeline::step` と `DopeSheet::step` は時間の刻みを指定します。既定は `1.0` で、signal や closure などの反応する `f64` も渡せます。0 以下と非有限の値は `1.0` として扱います。`snap(true)` のときは、item の移動後の開始時刻、trim で動かした端、drag 中の playhead を `step` の倍数に丸めます。矢印キーの移動量は `step`、Shift を押した場合は `step * 10` です。例の `step(0.5)` なら、通常の矢印キーで 0.5 秒ずつ移動します。

`Arrangement::step` は beat の刻み（既定 `1.0`）、`GraphEditor::step(time, value)` は時間と値の刻み（既定 `1.0` と `0.01`）です。どちらも signal や closure などの反応する `f64` を渡せ、0 以下と非有限の値は既定値として扱います。Arrangement の矢印キーも `step` ずつ、Shift を押すと `step * 10` ずつ選択した clip を動かします。

動かさずに離した item の時刻は丸めません。trim では最小の長さを優先します。丸めの演算が非有限になる場合は丸め前の値を使い、キーや支援技術の増減が `f64` の表現範囲を超える場合は変更しません。

Graph は曲線一覧を読取専用で受け、`on_edit` へ `EditorEvent<GraphRequest>` を報告します。アプリは `Move` の絶対的な `KeyMove`、`Delete` の ID、`Interpolate` の ID と補間方法をモデルへ反映します。ドラッグの `Begin` で保存したモデルを `Cancel` で戻します。要求を無視すれば key と tangent の描画は動きません。`GraphView` の `time` と `value` が表示範囲です。F は全体、Home は選択した key を表示し、Ctrl+wheel は時間、Ctrl+Shift+wheel は値を拡大します。Meta でも Ctrl と同じ操作ができます。`time_format` と `value_format` で軸の文字を変えます。

timeline と dope_sheet の `on_edit` は Begin、絶対値の Request、Commit または Cancel を通知します。drag の各 Request は Begin 時の項目から計算するので、最新の要求だけを反映できます。Cancel では kit が playhead・zoom・pan を復元し、アプリが項目を復元します。キーと支援技術による項目の移動も Begin/Request/Commit の一組です。

node と arrangement も `EditorEvent` の同じ契約を使います。`node_editor(nodes, links, state)` の `NodeRequest` は移動・接続・切断・削除・ミュート・control の変更を報告します。`NodeSocket::new(label, type_name, color)` で型名と色を明示し、`.can_connect(..)` へ接続可否の関数を渡します。重複リンク、入力の置換と ID の採番はアプリが所有します。自己接続は描画上の制約として kit が報告しません。リンクの付け替えは一つの gesture で Disconnect と Connect を報告します。node の preview はアプリが `.preview(Pixels)` で渡した画像だけを描きます。

`arrangement(clips, tracks, state).length(length)` の `ArrangementRequest` は移動・分割・削除・ミュート、track の flag、automation point の追加・移動・削除を報告します。length と最短の長さはアプリが指定し、分割後の ID・名前・素材の開始位置をアプリで決めます。point の追加 drag は新しい ID を kit が作らず、Begin 時の状態に対する `AddPoint` の提案を更新します。アプリは同じ操作で point を重複追加せず、Cancel なら追加を取り消します。`TimelineTrack::automation_label` が空なら automation の見出しは描きません。

数値・range・色・Bezier・gradient など単一の値の部品の `on_edit` は `EditPhase` を報告します。Escape・PointerCancel・非表示・無効化・破棄では構築関数に渡した値を戻してから Cancel を通知します。drag の取消しは中核の `Drag::edit`（数値は `use_scalar` の `scrub`、range は `use_range_scalar` の track・thumb・body）が行い、kit は Escape を手で扱いません。`range(label, value)` は一つの focus を持ち、track の押下は近い端を、端か `movable(true)` の区間の押下はそれを動かします。キーは最後に押した、選んだ、または動いた端（か区間）を動かします。キーと支援技術の一操作は Begin/Commit の一組です。履歴を作るアプリは Begin で履歴用の値を保存し、Commit で一件を確定します。選択だけでは編集通知は出ません。Dock と標準の `split` の `on_edit` は splitter の編集を通知します。

timeline と dope sheet の時間 editor、graph、node、arrangement の4系統は、次の操作規則を共有します。drag は押した時ではなく、最初に値が変わる時に Begin を通知します。動かさずに離した押下は選択か起動で、編集通知を出しません。drag の取消しは中核の `Drag::edit_on_change` が行い、kit は Escape と編集の開始・確定を手で扱いません。

| 操作 | 規則 |
|---|---|
| double click | 押した対象を、押してから pointer が動いていない場合だけ起動する |
| 矩形選択（marquee） | 修飾キーなしでは選択を置き換える。Shift・Ctrl・Meta のいずれかを押していれば、現在の選択に追加する |
| Ctrl+A / Meta+A | すべての項目を選択する |
| Escape | drag 中なら drag を止め、値が変わっていれば Cancel を通知してアプリが項目を復元する。drag 中でなければ選択を空にする |
| Delete / Backspace | 選択した項目の Delete 要求を報告する。項目の削除はアプリが反映する |
| Ctrl+wheel / Meta+wheel | pointer の位置を中心に拡大縮小する |
| 選択済み項目を修飾キーなしで押す | 選択を保ったまま drag を始める。動かさずに離したときだけ、その項目の単独選択にする |

arrangement の空の loop ruler は、drag で新しい loop 範囲を作る領域です。読み上げ名は `App::messages` などの翻訳設定で差し替えられます。

| 翻訳 key | 既定の文言 |
|---|---|
| `arrangement.loop_ruler` | `Loop ruler` |

## 色・曲線・gradient

`color_wheel(color, model)` は輪と彩度・明度の面、RGB/HSV/HEX の入力を持ちます。`model` は入力欄が表示する channel のモデル（`ColorModel`）で、色を変えずに切り替えます。`fields(false)` は入力欄を隠し、`alpha(false)` は透明度の操作を隠します。`fields`・`alpha`・`wheel_size` は固定値に加え、signal や closure などの反応する値も受け取り、入力欄・透明度の表示や輪の大きさを実行中に変えられます。彩度がゼロになっても最後の hue を保ちます。`swatches(binding)` で palette を編集できます。HEX の不正な途中入力は色へ書かず、Enter で確定、Escape で取り消します。`color_popup` も同じ Color Wheel を使います。

```rust,no_run
use kivact::prelude::*;
use kivact::style::Color;
use kivact_ui_creative as ui;

fn appearance() -> impl IntoView {
    let easing = signal(ui::CubicBezier::new(0.25, 0.1, 0.25, 1.0));
    let stops = signal(vec![
        ui::GradientStop::new(0.0, Color::BLACK),
        ui::GradientStop::new(1.0, Color::WHITE),
    ]);
    column((
        ui::bezier_curve(easing),
        ui::curve_preview(move || easing.get().samples(64)),
        ui::gradient_editor(stops, signal(None)),
    ))
}
```

`CubicBezier::new` は x を、`GradientStop::new` は位置を 0〜1 に制限します。Bezier の y は overshoot を許します。`progress(time)` は時間を 0〜1 に制限し、その時間の進行値 y を返します。返す y は 0〜1 に制限しないため、overshoot を保ちます。`samples(count)` は時間方向に等間隔の標本を返し、個数は 2〜513 に制限します。例では編集と preview が同じ `easing` の binding を読み、曲線を編集すると `samples(64)` の表示も更新されます。gradient の stop は一覧内の index で識別し、選択した stop の index（`Option<usize>`）を構築関数の第2引数で渡します。追加と削除では選択の index を調整し、表示順のためにアプリの一覧を並べ替えません。`color_transform(to_display, from_display)` は状態を捕捉する closure を受けます。表示を開いたり取り消したりしただけでは、変換によって保存色を書き換えません。

## アプリの絵を表示する

3D、合成、scope の計算はアプリの renderer で行い、結果の `Option<Pixels>` を `viewport_surface` に渡します。最初の `None` は背景だけを表示します。transform を渡さなければ fit、`transform(binding)` を渡せば中ボタン・Space+drag の pan と wheel の拡大を使えます。

アプリの renderer が GPU で描いていて、`Pixels` への読み戻しが frame の予算を超えるときは、実験 feature `gpu-interop`（`kivact` と `kivact-ui-creative` の両方）の `viewport_surface(pixels).gpu_frame(frame)` で、読み戻さずに GPU の frame を表示できます。`kivact::gpu_interop::shared_image(w, h)` の `SharedImage` が、engine との取り込みと退避を引き受けます。`frame()` は取り込んだ GPU の frame（`Option<GpuFrame>`）、`pixels()` は engine が CPU で描いた絵です。`gpu_frame` が `Some` で window の GPU device が使えるときだけ GPU の frame を、そうでなければ `pixels` を表示します。fit・zoom・pan・pointer の座標・overlay・region は、表示している方の画像の寸法を基準にします。`pixelated` は両方に効きます。engine には `image.link()` の `ShareLink` を渡し、大きさは surface の `on_resize` の物理画素数を `set_size` で要求します。

```rust,ignore
// `kivact` と `kivact-ui-creative` の feature `gpu-interop` と、アプリの engine が要る。
let image = kivact::gpu_interop::shared_image(1280, 720);
let link = image.link();
std::thread::spawn(move || engine_thread(link));
let (frame, pixels, sized) = (image.clone(), image.clone(), image);
kivact_ui_creative::viewport_surface(move || pixels.pixels())
    .gpu_frame(move || frame.frame())
    .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
```

engine の thread の組み方（wgpu の例を含む）、約束、状態と失敗、制限は [Canvas の「GPU で描いた画像を読み戻さずに表示する（実験）」](../../for-kivact-user/how-to-guide/canvas.md#gpu-で描いた画像を読み戻さずに表示する実験)を参照してください。

node editor と `viewport_surface` の pan は、Space を押したまま主ボタンで drag して始めます。pan していないときの Space の押下と解放は、内部のキー状態を更新したうえでアプリの上位 handler へ伝わります。ただし、editor の中で focus 中の node やボタンを Space で起動することはありません。pan 中に届く Space だけを消費し、Space を解放しただけでは pan は終了しません。アプリが Space に再生・停止を割り当てた場合は、Space+drag で pan を始めるときにも、その再生・停止が起きます。

`on_resize` は論理寸法・物理画素数・DPI を渡します。`on_pointer` は拡大と pan を戻した絵の座標を渡し、`overlay` と `regions` も同じ絵の座標を使います。拡大した画素を補間しない場合は `pixelated(true)` を指定します。touch screen では二本の指の pinch で拡大と pan をします。touchpad の専用の pinch event は現在ありません（wheel として届く拡大は使えます）。

worker は所有する `Pixels` だけを channel へ送り、UI の `spawn` が受信して image の signal に書きます。Signal を worker へ移しません。[frame の受渡し](../../for-kivact-user/how-to-guide/canvas.md)と、catalog の [CPU image の例](../../../examples/kivact-ui-creative/src/creative/editor_images.rs)を参照してください。

## 見出しと検索を組み立てる

outliner の `cell(width, |item| ..)`、timeline・dope sheet・graph の `lane_cell(width, |lane| ..)` は複数回呼べます。セル内の button/toggle は行の選択と drag から独立します。`TreeItem::tone` は文字の色、`placeholder(true)` は仮の項目の表示を指定し、操作を無効にはしません。

`disabled` は通常の Node と同じく、その node 自身に適用されます。アプリが作るセル内の操作も無効にする場合は、同じ disabled の値をセルの button/toggle に渡します。`hidden` は子を含めて隠します。

editor の `on_lane_activate`・`on_item_activate` は Enter と double click を受けます。`context_menu(build, choose)` は `EditorContext { item, lane, time }` から標準の `MenuEntry<K>` を作り、選択時に `(EditorContext, K)` を返します。右クリックとコンテキストメニューキーは同じ中核の menu を使います。

選択 callback がエラーを返すと menu は開いたままです。支援技術から編集中に ShowMenu を要求した場合は、編集を取り消し、復元後の値から context を作って開きます。

`search_picker` の `.column(title, items)` で列を追加すると、共通の検索語で絞り込み、Tab/Shift+Tab と左右キーで列を移ります。単列はこれまでの Tab による終了を保ちます。

表は標準の `kivact::ui::data_table(rows, key, columns)` で、creative の theme が密な表の見た目（30 px の header、kit の行の高さ、小さい文字）で描きます。使い方は [一覧と表](../../for-kivact-user/how-to-guide/lists-tables.md) にあります。creative のエディタの表は、行をフォーカスの単位にして行の押下・Ctrl・Shift で選ぶので `.focus(TableFocus::Rows)` と `.policy(..)`、行の左端の色は `.row_color(..)`、高さは `.row_height(..)` で渡します。絞り込みの欄と選択数を出さないときは `.toolbar(|_| ())` と `.footer(|_| ())` で領域を空にします。行が変わっても mount したままにしたい control（入力欄や switch）は `.cell_controls(|row, column| view)`、行の下の内容は `.row_details(..)`、列幅の変更は `.resizable(widths)` の binding へ書きます。`on_reorder` のある表は、`.sort(..)` がなければ並べ替えません。セルの draft・矩形選択・TSV の一括編集は共通化を見送りました。必要な入力を `cell_controls` に置き、確定や行操作をアプリの binding で扱います。判断の範囲は[台帳](../../for-kivact-developer/strategy/ledger.md)に記録しています。

outliner の項目列は読み取り専用です。`outliner(items, state)` の `OutlinerState` に、展開した ID（`expanded`）・選択（`selection`）・改名中の ID（`renaming`）の binding をまとめて渡します。`OutlinerState::new(ids)` は `ids` を展開した状態から始めます。改名した名前は `on_rename`、親と並び順の変更は `on_item_drop` でアプリへ要求します。アプリが反映した項目列が表示されます。drop の callback を指定しなければドラッグは無効です。

制限の切替は `on_restriction` が `EditorEvent<Restriction>` として報告します。クリック・キー・支援技術の一操作は Begin、Request、Commit、塗る操作は Begin、累積した ID と同じ値の Request、Commit または Cancel です。選択した行は、押した行の開始時の値を反転した一つの値へ揃える要求になります。アプリは Begin で記録した項目を Cancel で復元します。

`table(..)` の行も読み取り専用です。`on_reorder` に行の key と元の行列での 0 始まりの移動先が届きます。アプリが移動を反映し、callback を省略した場合は行のドラッグを無効にします。

## 再生と音量のモデルをアプリで持つ

`transport(state)` の `TransportState` は playing・recording・cycling の flag だけです。先頭への移動は `.on_to_start(..)` で要求を受けます。frame・fps・loop の範囲と再生クロックをアプリで所有し、transport を表示しただけでは時間は進みません。

`mixer_strip(label, color, channel)` は `ChannelState` の gain・pan・mute・solo を編集します。`.level(..)` は gain や mute を掛け直さず表示します。DSP はアプリで行い、最終的な meter 値を渡します。gain は `.gain_range(min, max)` で指定し、既定は0..1です。routing の selector は `.routes(labels, route)` の `labels` が空でないときだけ表示し、選んだ index を `route` に書きます。insert のラベルと routing の効果もアプリが所有します。

`fader` は任意の `.range(min, max)` に対し、正規化した値 `t = (value - min) / (max - min)` の表示位置を `sqrt(t)`、位置 `n` から値への変換を `min + (max - min) * n²` とします。dB の用途なら `range(-60.0, 6.0)` をアプリで明示します。

## icon・通知・向き

Creative の icon は theme の `IconSet` の名前（`"play"`、`"chevron-down"`、閉じる印の `"x"` など）で、`kivact::control::icon("play")` で描きます。`icon_button`、`Toggle::icon`、`DockTab::icon`、`TreeItem`、`PieEntry`、`PickerItem::icon`、`WindowOptions::icon` も名前か `kivact::style::Icon` を受けます。アプリ独自の画像は theme の set に名前で足します（`theme.with_icon("node", asset)`）。名前は Creative の theme の下で描かれ、その名前を持たない theme では何も描かずに報告します。

通知は `Toast::new(title).tone(Tone::Success)` で作り、二行目は `.body(..)` で足します。`toaster().push(..)` で積み、`kivact::ui::toast_region(toaster)` が描きます。右クリックの menu は `kivact::ui::menu(label, entries, choose).attach_context(target)` です。`meter` と `fader` は `.orientation(Orientation::Horizontal)` で横向きになります。標準の `kivact::ui::split((first, second), sizes)` は左右に並び、`.orientation(Orientation::Vertical)` で上下に積みます。`panel(..).header(|| view)` と `property_row(..).leading(|| view)`・`.trailing(|| view)` の slot は、部品の mount のときに view を作る closure を受けます。
