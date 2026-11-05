# タイムライン・ドープシート・グラフ・アレンジメントを編集する

[English](../../../en/creative/how-to-guide/timeline-and-arrangement.md)

目的: クリップ・キー・曲線をタイムライン型のエディタに表示し、利用者が移動・トリム・削除できるようにしつつ、時間（再生ヘッド）をアプリの管理下に置きます。

これらのエディタは、描画とジェスチャーの解釈だけを行い、データは何も保存しません。アイテムを渡すと、エディタは利用者が*求めている*変更を報告します。それを適用するかどうかはアプリが決めます。このガイドは、[チュートリアル](../getting-started/README.md)を終えていることを前提にしています。

## クリップを表示して、編集要求を適用する

```rust,no_run
use kivact::prelude::*;
use kivact::style::{ThemeMode, rgb};
use kivact_ui_creative as ui;

/// `lane` の `start..start + duration` に置いたクリップは、ほかのクリップと重なるか。
fn collides(clips: &[ui::Clip], id: u64, lane: u64, start: f64, duration: f64) -> bool {
    clips.iter().any(|other| {
        other.id != id
            && other.lane == lane
            && start < other.start + other.duration
            && other.start < start + duration
    })
}

fn apply(clips: &mut Vec<ui::Clip>, request: ui::TimeRequest) {
    match request {
        ui::TimeRequest::Retime(times) => {
            for time in times {
                let Some(index) = clips.iter().position(|c| c.id == time.id) else {
                    continue;
                };
                let lane = clips[index].lane;
                // アプリの規則: 開始が負にならない、同じレーンで重ならない。
                if time.time >= 0.0 && !collides(clips, time.id, lane, time.time, time.duration) {
                    clips[index].start = time.time;
                    clips[index].duration = time.duration;
                }
            }
        }
        ui::TimeRequest::Delete(ids) => clips.retain(|c| !ids.contains(&c.id)),
    }
}

fn timeline_panel() -> impl IntoView {
    let clips = signal(vec![
        ui::Clip::new(1, 10, 0.0, 48.0, "イントロ").color(rgb(0x8b7cff)),
        ui::Clip::new(2, 10, 60.0, 72.0, "Aメロ").color(rgb(0x3ad3e0)),
        ui::Clip::new(3, 20, 24.0, 96.0, "ナレーション").color(rgb(0xffb224)),
    ]);
    let lanes = signal(vec![ui::Lane::new(10, "ビデオ 1"), ui::Lane::new(20, "オーディオ 1")]);

    // アプリが持つ表示状態: 再生ヘッドはアプリの時間。
    let playhead = signal(0.0);
    let state = ui::TimeState {
        playhead: playhead.into_binding(),
        ..ui::TimeState::new()
    };

    // ジェスチャー 1 回につき元に戻す手順を 1 つ記録する。
    let snapshot = signal(None::<Vec<ui::Clip>>);
    let undo = signal(Vec::<Vec<ui::Clip>>::new());
    let on_edit = move |event: ui::EditorEvent<ui::TimeRequest>| match event {
        ui::EditorEvent::Begin => snapshot.set(Some(clips.get_untracked())),
        ui::EditorEvent::Request(request) => clips.update(|all| apply(all, request)),
        ui::EditorEvent::Commit => {
            if let Some(before) = snapshot.update(|saved| saved.take()) {
                undo.update(|steps| steps.push(before));
            }
        }
        ui::EditorEvent::Cancel => {
            if let Some(before) = snapshot.update(|saved| saved.take()) {
                clips.set(before);
            }
        }
    };

    column((
        ui::timeline(clips, state)
            .lanes(lanes)
            .step(1.0)
            .snap(true)
            .time_format(|frame| format!("{:.1}s", frame / 24.0))
            .on_edit(on_edit)
            .name("タイムライン"),
        text(move || format!("フレーム {:.0}、元に戻せる手順 {} 件", playhead.get(), undo.get().len())),
    ))
    .style(style().gap(8).padding(12))
}

fn main() -> kivact::Result<()> {
    App::new("タイムライン")
        .theme(ui::theme(ThemeMode::Dark))
        .run(timeline_panel)
}
```

クリップをドラッグし、端をトリムし、ルーラーをクリックして再生ヘッドを動かし、クリップを選んで `Delete` を押してみてください。クリップをほかのクリップの上へドラッグすると、要求は届きますが `collides` が拒否するので、クリップは動きません。

### 契約の仕組み

- **アイテムは読み取り専用の入力です。** `ui::timeline(clips, state)` は、シグナル、クロージャ、通常の `Vec<Clip>` のどれでも受け取ります。エディタがそれに書き込むことはありません。
- **`on_edit` は `EditorEvent` を受け取ります。** ジェスチャーは `Begin`、任意の数の `Request(..)`、そして `Commit`（利用者が離した）か `Cancel`（`Esc` を押した、またはジェスチャーが中断された）の順に流れます。キー操作のような単発の操作も、要求が 1 つだけの同じ流れで届きます。
- **要求は絶対値です。** `Retime` には、関係するすべてのアイテムの新しい開始位置と長さが、`Begin` 時点のアイテムから計算して入っています。差分を積み上げる必要はなく、最新の要求だけを適用すれば足ります。要求を無視してもかまいません。その場合、アイテムはモデルの値のまま描かれます。
- **スナップショットは `Begin` で取ります。** `Cancel` では、そのスナップショットを書き戻します。エディタは自分が持つ値（再生ヘッド、ズーム、スクロール）は復元しますが、アイテムの復元はアプリの仕事です。
- **`Begin` は押した瞬間ではなく、最初に変化が起きたときに届きます。** 動かさずに離した押下はクリップの選択で、ダブルクリックは起動です。どちらもイベントを送りません。
- **ID は安定したキーです。** `Clip::id` と `Lane::id` はあなたが割り当てる数値です。行を並べ替えても変わらず、インデックスではありません。

### 時間はアプリのものです

エディタが時間を進めることはありません。再生ヘッドは、*あなたが*渡す `TimeState` のバインディングです。

- 利用者が再生ヘッドを動かすと、エディタがそれに書き込みます。プレイヤーのシークなどのために読み取ってください。
- 時計が進むときは、あなたが書き込みます。再生中は、プレイヤーからこれを更新します。

```rust,no_run
use kivact::prelude::*;
use std::time::Duration;

async fn run_clock(playing: Signal<bool>, playhead: Signal<f64>) {
    loop {
        kivact::task::sleep(Duration::from_millis(1000 / 24)).await;
        if playing.get_untracked() {
            playhead.update(|frame| *frame += 1.0);
        }
    }
}

fn start_clock(playing: Signal<bool>, playhead: Signal<f64>) {
    spawn(run_clock(playing, playhead));
}
```

`start_clock` は、コンポーネントのセットアップで 1 回だけ呼びます。このタスクは、起動したコンポーネントとともに終わります。`TimeState` には `selection`・`transform` もあります。どのフィールドも自分のシグナルに差し替えて、アプリのほかの部分と共有できます（`TimeState { selection: picked.into(), ..ui::TimeState::new() }`）。時間の単位は自由で、フレーム番号でも、ティックでも、秒でもかまいません。`time_format` は、ルーラーと再生ヘッドの表示の書式だけを決めます。

### そのほかのビルダーのオプション

| メソッド | 効果 |
|----------|------|
| `.lanes(..)` | 行とそのラベルを順番に指定します。リストにある行だけが存在し、空のリストならすべてのアイテムが隠れます。指定しない場合は、アイテムの `lane` の値から行が決まります。 |
| `.snap(true)` と `.step(n)` | ポインター操作を `n` の倍数に丸めます。矢印キーは `n` ずつ、`Shift` で `10 n` ずつ動かします。 |
| `.zoom_range(min, max)` | ズームの許容範囲です（既定は `0.25..=4`）。 |
| `.min_duration(n)` | トリムで提案される最短のクリップ長です（既定は `1`）。 |
| `.ruler_ticks(ui::RulerTicks::new([30.0, 60.0, 150.0], 4))` | 主目盛りの間隔と補助目盛りの数を自分で指定します。 |
| `.lane_cell(width, build)` と `.context_menu(entries, choose)` | 各レーンのヘッダーに置く部品と、右クリックのメニューです。メニューの項目は `EditorContext`（アイテム・レーン・時間）から組み立てます。 |

## キーフレーム: ドープシート

`ui::dope_sheet(keys, state)` は、`Keyframe` を対象にした同じエディタです。報告する `TimeRequest` も同じなので、`on_edit` の形はどちらにも使えます。フィールド名だけが違います（`start` ではなく `time`）。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn keys_panel() -> impl IntoView {
    let keys = signal(vec![
        ui::Keyframe::new(1, 1, 0.0, "開始"),
        ui::Keyframe::new(2, 1, 24.0, "ホールド").duration(12.0),
        ui::Keyframe::new(3, 2, 12.0, "ポップ").glyph(ui::KeyGlyph::Circle),
    ]);
    ui::dope_sheet(keys, ui::TimeState::new()).on_edit(move |event| {
        if let ui::EditorEvent::Request(request) = event {
            keys.update(|all| match request {
                ui::TimeRequest::Retime(times) => {
                    for time in times {
                        if let Some(key) = all.iter_mut().find(|k| k.id == time.id) {
                            key.time = time.time;
                            key.duration = time.duration;
                        }
                    }
                }
                ui::TimeRequest::Delete(ids) => all.retain(|k| !ids.contains(&k.id)),
            });
        }
    })
}
```

`duration` が正のキーは、バーとして描かれます。この例はスナップショットを省いています。`Cancel` でキーを元に戻したい場合は、上のタイムラインと同じように追加してください。

## 曲線: グラフエディタ

`ui::graph_editor(curves, state)` は、縦軸を値にして曲線を表示します。`CurveKey` は、時間、値、`Interpolation`（`Constant`・`Linear`・`Bezier`）、相対位置で表すタンジェントのハンドルを持ちます。

```rust,no_run
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn apply(curves: &mut [ui::Curve], request: &ui::GraphRequest) {
    for key in curves.iter_mut().flat_map(|curve| curve.keys.iter_mut()) {
        match request {
            ui::GraphRequest::Move(moves) => {
                if let Some(m) = moves.iter().find(|m| m.id == key.id) {
                    key.time = m.time;
                    key.value = m.value;
                    key.in_tangent = m.in_tangent;
                    key.out_tangent = m.out_tangent;
                }
            }
            ui::GraphRequest::Interpolate(ids, mode) if ids.contains(&key.id) => {
                key.interpolation = *mode;
            }
            _ => {}
        }
    }
    if let ui::GraphRequest::Delete(ids) = request {
        for curve in curves.iter_mut() {
            curve.keys.retain(|key| !ids.contains(&key.id));
        }
    }
}

fn graph_panel() -> impl IntoView {
    let curves = signal(vec![ui::Curve::new(
        1,
        "不透明度",
        vec![ui::CurveKey::new(1, 0.0, 0.0), ui::CurveKey::new(2, 48.0, 1.0)],
    )]);
    ui::graph_editor(curves, ui::GraphState::new())
        .step(1.0, 0.01)
        .on_edit(move |event| {
            if let ui::EditorEvent::Request(request) = event {
                curves.update(|all| apply(all, &request));
            }
        })
}
```

キーの ID は、グラフ内のすべての曲線を通して一意でなければなりません。`GraphState::view` は表示する時間と値の範囲（`GraphView { time, value }`）を持ちます。`F` で全体を表示し、`Home` で選択したキーに合わせます。`Ctrl` を押しながらホイールで時間軸を、`Ctrl+Shift` を押しながらホイールで値の軸を拡大縮小します。

## 楽曲型のアレンジメント

`ui::arrangement(clips, tracks, state)` は、音楽シーケンサー型のレイアウトです。ミュート・ソロ・録音待機を持つトラックヘッダー、拍で表すクリップ、ループ範囲、マーカー、オートメーションのレーンがあります。クリップの `lane` は、そのクリップが載るトラックの ID です。

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn apply(
    clips: &mut Vec<ui::Clip>,
    tracks: &mut [ui::TimelineTrack],
    request: ui::ArrangementRequest,
) {
    use ui::ArrangementRequest as R;
    match request {
        R::Retime(times) => {
            for time in times {
                if let Some(clip) = clips.iter_mut().find(|c| c.id == time.id) {
                    clip.start = time.time;
                    clip.duration = time.duration;
                }
            }
        }
        R::Delete(ids) => clips.retain(|c| !ids.contains(&c.id)),
        R::Split { clip, at } => {
            // 新しいクリップの ID と名前はアプリが決める。
            let new_id = clips.iter().map(|c| c.id).max().unwrap_or(0) + 1;
            if let Some(left) = clips.iter_mut().find(|c| c.id == clip) {
                if at > left.start && at < left.start + left.duration {
                    let mut right = left.clone();
                    right.id = new_id;
                    right.start = at;
                    right.duration = left.start + left.duration - at;
                    left.duration = at - left.start;
                    clips.push(right);
                }
            }
        }
        R::TrackFlag { track, flag, value } => {
            if let Some(t) = tracks.iter_mut().find(|t| t.id == track) {
                match flag {
                    ui::TrackFlag::Mute => t.muted = value,
                    ui::TrackFlag::Solo => t.solo = value,
                    ui::TrackFlag::Arm => t.armed = value,
                    ui::TrackFlag::AutomationRead => t.automation_read = value,
                }
            }
        }
        // Erase・ToggleClipMute・AddPoint・MovePoint・DeletePoint は残りの要求。
        // このアプリはまだ対応していない。
        _ => {}
    }
}

fn arrangement_panel() -> impl IntoView {
    let clips = signal(vec![
        ui::Clip::new(1, 1, 0.0, 8.0, "ドラム"),
        ui::Clip::new(2, 2, 4.0, 8.0, "ベース"),
    ]);
    let tracks = signal(vec![
        ui::TimelineTrack::new(1, "ドラム", ui::TrackKind::Audio, rgb(0x8b7cff)),
        ui::TimelineTrack::new(2, "ベース", ui::TrackKind::Midi, rgb(0x3ad3e0)),
    ]);
    let state = ui::ArrangementState::new();
    ui::arrangement(clips, tracks, state)
        .length(32.0)
        .markers(vec![ui::TimelineMarker::new(16.0, "サビ", rgb(0xffb224))])
        .on_edit(move |event| {
            if let ui::EditorEvent::Request(request) = event {
                clips.update(|c| tracks.update(|t| apply(c, t, request)));
            }
        })
}
```

位置の単位は拍で、`ArrangementState` のズームは 1 拍あたりのピクセル数です。`ArrangementState` には、ループ範囲、`cycling` フラグ、ツール（`TimelineTool::Select`・`Split`・`Mute`・`Erase`）もあります。ツールを選んでも変わるのは、クリックが*要求する*内容だけです。分割ツールは `Split { clip, at }` を送り、切るかどうか、どう切るかはあなたが決めます。元に戻す機能が必要なら、このコードに上のタイムラインのスナップショットの手順を組み合わせてください。

## 再生コントロールとメーター

キットはトランスポートのボタン、チャンネルストリップ、メーターを描きますが、音は再生しません。状態を渡して、変更に反応するのはアプリの役目です。

```rust,no_run
use kivact::prelude::*;
use kivact::style::rgb;
use kivact_ui_creative as ui;

fn mixer_panel() -> impl IntoView {
    let transport = signal(ui::TransportState::default());
    let drums = ui::ChannelState::new();
    let level = signal(0.4); // オーディオエンジンが測った最終レベル（0..1）
    row((
        ui::transport(transport).on_to_start(move || {
            // ここでプレイヤーを先頭へシークする。
        }),
        ui::mixer_strip("ドラム", rgb(0x8b7cff), drums)
            .gain_range(kivact::interaction::ScalarRange::new(-60.0, 6.0).expect("valid range"))
            .level(level),
        ui::meter("マスター", level).orientation(Orientation::Horizontal),
    ))
    .style(style().gap(12).padding(12))
}
```

`TransportState` が持つのは `playing`・`recording`・`cycling` だけです。`ChannelState` は `gain`・`pan`・`mute`・`solo` をバインディングとして持ちます。ストリップは渡された `level` を、ゲインやミュートを掛けずにそのまま表示します。DSP はエンジン側で行い、その結果を渡してください。ルーティングの選択肢とインサートの名前もアプリのものです。`.routes(labels, route)` は `labels` が空でないときだけセレクターを表示し、`.inserts(names)` は名前を一覧します。

## 関連項目

- [アプリが描いたフレームを表示する](show-app-rendered-images.md)
- [ドック・パネルグループ・アウトライナーを作る](panels-docks-and-outliners.md)
- [ノードグラフでノードをつなぐ](node-graph.md)
- API の全体: `cargo doc -p kivact-ui-creative --open`
- カタログのタイムライン・アニメーション・ミキサーの章: `cargo run -p kivact-ui-creative-catalog`

時間エディタと arrangement は、単一の `transform: Binding<kivact::interaction::ViewTransform>` を使います。`scale` は論理時間単位あたりの横方向のピクセル数（arrangement では拍あたり）、`offset` はピクセル単位のスクロールです。レーンの高さは拡大縮小しません。セットアップ時に `transform.project(|pose| &pose.scale, |pose| &mut pose.scale)` を作ると、ズームスライダーも同じ表示状態に接続できます。ホイールの割り当てとサイドバーを基準にしたズームは時間エディタが保持します。
