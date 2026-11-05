# アコーディオンとプレビュー

`disclosure_group` に `signal(None::<K>)` を渡すと単一選択、
`signal(Vec::<K>::new())` を渡すと複数選択になります。`.panel` にキー、
見出し、開くたびに内容を作る関数を渡します。展開状態は選択の binding だけが
保持します。見出しは Enter、Space、順方向と逆方向の Tab に対応し、
[APG Accordion](https://www.w3.org/WAI/ARIA/apg/patterns/accordion/) に従います。

```rust,no_run
use kivact::prelude::*;

fn sections() -> impl IntoView {
    kivact::ui::disclosure_group(signal(Some("overview")))
        .panel("overview", "Overview", || text("Project overview"))
        .panel("details", "Details", || text("Project details"))
}
```

`disclosure(title, open).always_open()` は構造として常に内容を開きます。
`.content(factory).attach(trigger)` で呼び出し側のトリガーを使えます。

`hover_card(factory).attach(trigger)` は theme の待ち時間後にプレビューを開き、
フォーカスでは直ちに開きます。プレビューに向かうポインターの移動は共有の
corridor が保護します。Escape と外側の押下は、現在の操作が終わるまで閉じます。
`placement`、`align`、`open_delay`、`close_delay` で調整し、`label` で操作可能な
プレビューに dialog 名を付けます。`focus_trigger` はフォーカスでの開閉、
`enabled` は利用可否を指定します。`default_open` は最初の訪問が終わるまで
初期表示を保ちます。明示した `open` binding が表示を決定します。ポインターとフォーカスは書き込みを
要求し、拒否された書き込みを尊重します。外部から閉じると現在の操作が
終わるまで開き直しません。
`on_open_change` は初期値を除いた表示の変化を通知します。

`dialog(title, open).mode(DialogMode::Drawer(edge))` にはドラッグ用ハンドルが
付きます。端に向かって引き、距離または flick の閾値を超えると閉じます。
取り消すと元の位置に戻ります。`InteractionPolicy.drawer` が閾値、
`hover_card` がプレビューの時間を指定します。

`use_copy_feedback(duration)` はクリップボードへの書き込みと確認表示をまとめます。
ハンドラーで `copy.copy(text)` を呼び、表示には `copy.is_copied()` を読みます。
成功するたびに待ち時間が始まり直します。失敗は owner のエラーハンドラーへ届き、
確認表示を開始しません。owner を破棄すると処理とタイマーを取り消します。
