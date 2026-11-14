# ノブとフェーダーの表示・写像・微調整

[English](../../../en/creative/how-to-guide/knobs-and-faders.md)

Creative キットの数値部品（`number`、`value_slider`、`knob`、`fader`）は一つの実装を共有します。binding、範囲、ステップ、キー、ホイール、編集フェーズは同じです。このガイドでは、利用者が考える単位で値を表示し、正しいカーブに従わせ、ポインター操作をプラグインの部品らしくする方法を説明します。

## 利用者が読む形で値を表示する

`format` は部品の表示、テキスト欄、支援技術が読む値を置き換えます。値の単位ではない文字列（リニアなゲインを dB で見せる場合など）は、`text_parser` も渡して、入力した文字列と支援技術の `SetValue` が同じように読めるようにします。

```rust,test_harness
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn gain() -> impl IntoView {
    let gain = signal(0.5);
    ui::knob("Gain", gain)
        .range(0.0..=2.0)
        .reset_to(1.0)
        .format(|gain| {
            if gain > 0.0 {
                format!("{:.1} dB", 20.0 * gain.log10())
            } else {
                "-inf dB".to_owned()
            }
        })
        .text_parser(|text, _| {
            let db: f64 = text
                .trim()
                .trim_end_matches("dB")
                .trim()
                .parse()
                .map_err(|_| "Enter decibels, such as -6.0".to_owned())?;
            Ok(10f64.powf(db / 20.0))
        })
}
```

優先順位は固定です。

| 表示する文字列 | 出どころ |
|---|---|
| 部品の表示、テキスト欄の値、支援技術の値テキスト | `mixed(true)` の間は mixed のメッセージ、それ以外は `format`、なければ `timecode`、なければ `precision` と `unit` |
| テキスト欄を開いたときの文字列 | `format(value)` があればそれ、なければ単位を付けない組み込みの表示 |
| 入力した文字列と、数値でない支援技術の `SetValue` | `text_parser` があればそれ、なければ組み込みの数値・四則演算・タイムコード |

`format` は範囲にクランプした値を受け取り、signal を読んでもかまいません。単位の切替やロケールが、部品を作り直さずに反映されます。`precision` と `unit` はステップと組み込みの表示の設定として残ります。

## 値を移動量に写す

既定では、同じ移動量が同じ値の変化になります。`mapping` は binding の単位を変えずに、その関係だけを変えます。

| 写像 | 位置 |
|---|---|
| `ScalarMapping::linear()` | 既定。同じ移動量で同じ値の刻み |
| `ScalarMapping::logarithmic()` | 同じ移動量で同じ比（周波数、時間）。範囲の最小が正でなければ線形 |
| `ScalarMapping::skewed(0.5)` | フェーダーが使う平方根カーブ。1 未満で低域側、1 超で高域側の移動量が増える |
| `ScalarMapping::custom(to_position, to_value)` | パラメータ自身の正規化 |

フェーダーはすでに `ScalarMapping::skewed(0.5)` を使います。置き換えるには `mapping` を呼びます。`step` は `ScalarStep` を受け取ります。`Value(..)` は値の単位（既定）、`Position(..)` は移動量の割合です。`Position` のステップは、対数ノブのどこでも矢印キー 1 回で移動量の 1 % を動かす方法です。

```rust,test_harness
use kivact::interaction::{ScalarMapping, ScalarStep};
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn frequency() -> impl IntoView {
    ui::knob("Frequency", signal(440.0))
        .range(20.0..=20_000.0)
        .mapping(ScalarMapping::logarithmic())
        .step(ScalarStep::Position(0.01))
        .unit(" Hz")
        .precision(0)
}
```

文字列、範囲、Home/End、値の単位のステップ、`reset_to` は値の単位のままです。ドラッグ、タッチ、`Position` のステップ、描画だけが位置を使います。`use_scalar` の `ScalarState::fraction` がその位置で、キャンバスのノブはこれで角度を決めます。

## ドラッグの距離を決める

ノブは自分の高さ（最低 60 px）で範囲全体を動かします。プラグインのノブは感度を固定したいことが多いので、`drag_distance` に、写像を通して範囲全体を動かすポインターの移動量を論理ピクセルで指定します。

```rust,test_harness
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn threshold() -> impl IntoView {
    ui::knob("Threshold", signal(-18.0))
        .range(-60.0..=0.0)
        .unit(" dB")
        .drag_distance(200.0)
}
```

正でない、または有限でない値は既定として扱われるので、設定から reactive な `drag_distance` を渡せます。数値欄は `drag_distance` を指定しない限り 1 ピクセルで 1 ステップ動きます。

## 微調整・粗調整のキーとリセットクリック

スクラブ、矢印キー、ホイールの修飾キーは一つのテーマポリシー `ScrubPolicy` です。既定は Ctrl で微調整（×0.1）、Shift で粗調整（×10）、リセットクリックなしです。エディタのテーマで一度設定します。

```rust,no_run
use kivact::interaction::{InteractionPolicy, ModifierKey};
use kivact::style::ThemeMode;
use kivact_ui_creative as ui;

fn editor_theme() -> kivact::style::Theme {
    let theme = ui::theme(ThemeMode::Dark);
    let mut policy = (*theme.ext::<InteractionPolicy>()).clone();
    policy.scrub.fine = Some(ModifierKey::Shift);
    policy.scrub.coarse = None;
    policy.scrub.reset_click = Some(ModifierKey::Alt);
    theme.with(policy)
}
```

スクラブの途中で倍率が変わっても値は飛びません。それまでの移動は適用済みの倍率のままで、新しい倍率は以後の移動だけに掛かります。境界で飽和した値や量子化された値は、今いる値から再開します。修飾キーの押下・解放だけでも同じポインター位置の更新が届くので、次の移動は新しい起点から始まります。

ホイールは微調整キーだけを使い、粗調整キーは無視します。一部の環境では Shift がホイールを横方向に変えるためです。ホイール 1 ノッチは 60 論理ピクセルのスクロール（`wheel_notch`）なので、タッチパッドの小さなデルタは積み上がります。20 px のイベント 2 回では何も書かず、3 回目で 1 ステップ動きます。逆方向のスクロールは新しい積み上げを始めます。1 ノッチ以上書いたイベントはそれぞれ 1 編集で、1 ノッチ未満のイベントは何も報告しません。

## ノブで値を入力する

数値欄と value_slider は Enter と、動かないクリックでテキスト欄を開きます。ノブとフェーダーは `text_entry(true)` のとき Enter で開き、欄は部品の上に重ねて表示されます。入力、Enter か欄の外への移動での確定、Escape は数値欄と同じです。確定は 1 編集、Escape は下書きを捨てて何も報告しません。

```rust,test_harness
use kivact::prelude::*;
use kivact_ui_creative as ui;

fn mix() -> impl IntoView {
    ui::knob("Mix", signal(100.0))
        .range(0.0..=100.0)
        .reset_to(100.0)
        .unit(" %")
        .text_entry(true)
}
```

## キー、PageUp/PageDown、リセット

| 入力 | 動作 |
|---|---|
| ArrowUp / ArrowRight | キーステップ 1 回分 |
| ArrowDown / ArrowLeft | キーステップ 1 回分だけ減らす |
| PageUp / PageDown | キーステップ 10 回分 |
| Home / End | 範囲の最小 / 最大 |
| Enter | テキスト欄を開く（`text_entry` のとき） |
| Backspace、ダブルクリック、修飾キー付きの主ボタン押下 | `reset_to` |

いずれも 1 編集です。スクラブは押下から解放まで値を所有し、その間はキー、ホイール、支援技術は書きません。

## 編集フェーズと混在値

すべての書き込みは `use_scalar` を通るので、`on_edit` はスクラブ全体、キー 1 回・ホイール 1 ノッチ、確定したテキストのそれぞれに `Begin`、`Commit`、`Cancel` を報告します。`Cancel` は開始時の値を書き戻したあとに届きます。

`mixed(true)` は混在のメッセージを表示し、部品を不確定として扱い、スクラブ、ステップ、ホイール、リセットクリックを止めます。書き込めるのは入力したテキストか支援技術の `SetValue` だけです。数値欄と value_slider は押下で、`text_entry` のあるノブとフェーダーは Enter でテキスト欄を開きます。
