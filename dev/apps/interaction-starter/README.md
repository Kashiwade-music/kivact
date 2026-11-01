# Interaction starter

検索 → 非同期一覧 → 詳細フォーム → エラー／再試行 → ファイル保存を一つの短い画面で辿ります。カタログの独立した UI 展示とは別に、公開 API を組み合わせる入門例です。

```text
cargo run -p kivact-interaction-starter -- path/to/device-note.txt
cargo test -p kivact-interaction-starter
```

親ディレクトリが存在する保存先を指定してください。引数なしなら作業ディレクトリの `interaction-notes.txt` です。検索後に行を選び、メモと calibration を編集し、Save note またはメモ欄の Enter で保存します。保存ファイルは stable key、calibration、メモの順です。これは教材の保存形式で、自動再読込や複数メモ管理は含みません。

ロードは 120 ms の遅延を持つ教材用 service です。Simulate load error、Cancel load、Retry load で状態の遷移を観察できます。保存は実ファイル I/O を blocking pool に渡します。失敗は `AsyncState` を表示し、入力と最後の成功一覧を保持します。Retry save は失敗した仕事の snapshot を再送します。失敗後の新しい本文を保存するには Save note を押します。

`signal`／`resource`／`action` は setup で一度作り、イベントは既存ハンドルを使います。`text(move || ...)` が追跡読出しで、`state()` と `latest()` は owned value の clone を返します。`show` の children は再生成する factory です。行の key は表示位置とは別の identity です。

公開実 window E2E は、検索・確定入力・custom key と submit の共存・validation failure・ファイル内容・load retry・cancel と、実 I/O 失敗後の同一 snapshot の再保存を検査します。GPU の描画は `KIVACT_TEST_GPU=1` で別に確かめます。初学者本人の完遂時間や、作者支援なしの利用実績を示す例ではありません。
