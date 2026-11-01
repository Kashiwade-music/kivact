# Mini Postman

GET/POST を送る小さな HTTP クライアントです。メソッド、URL、ヘッダー、本文を入力し、応答のヘッダーと本文を表示します。送信履歴からフォームを復元でき、再試行、キャンセル、応答のコピー、URL 欄へのフォーカスを試せます。

```powershell
cargo run -p kivact-example-mini-postman
```

HTTP 送信には Windows の `curl.exe` を使います。ヘッダーは `名前: 値` を1行に1件書きます。応答は最大1MiB（画面には先頭64K文字まで）、接続待ちは3秒、要求全体は12秒で打ち切ります。

直近30件の履歴を JSON で保存し、別プロセスで起動しても復元します。保存先は `%LOCALAPPDATA%\kivact\mini-postman\history.json` で、`KIVACT_POSTMAN_HISTORY` 環境変数で変えられます。

## 構成（1ファイル1コンポーネント）

| ファイル | 内容 |
|---|---|
| `components/app.rs` | ルート。履歴の読込み（`spawn` ＋ `blocking`）、保存（`action(..).policy(ActionPolicy::QueueLatest)`）、HTTP の `action`、context の提供（`provide`） |
| `components/request_form.rs` | メソッド（`select`）、URL・ヘッダー・本文（`Signal::field` の `Binding`）、送信・再試行・キャンセル・URLへ移動 |
| `components/response.rs` | 状態行、応答、応答のコピー（`platform::copy_text`） |
| `components/history.rs` | 履歴の `for_each`。選ぶとフォーム全体を復元する |
| `form.rs` | フォームの値と `Postman` context（Copy handle だけ） |
| `http.rs` | curl による送信（`Transport`）と入力検証 |
| `storage.rs` | 履歴 JSON の読み書き |

- **最新の要求が勝つ:** `send: Action<Arc<Response>, Request>` が唯一の状態源です。送信中に再度送信すると前の future を drop し、`blocking` の `CancelToken` が立って curl プロセスを止めます。「送信中」「キャンセルボタンの有効/無効」「HTTP 200 · n ms」「失敗: ..」「キャンセルしました」はすべて `send.state()` から導きます（手動の pending 同期はありません）。
- **履歴:** 完了した要求（取り消されたもの・追い越されたものは除く）だけを `send.on_done` で履歴に入れ、queue で順序どおりに保存します。
- **再試行:** この起動で送った要求があれば `send.retry()`、なければ履歴の先頭を送ります。
- **URL へ移動:** `NodeRef::focus()` で移動し、状態行に出します。失敗は `on_error` に届きます。文字列 key は使いません。
- **送信方法の差し替え:** `postman(history_path, transport)` の `transport` は `fn(Request) -> PendingResponse` です。アプリは `curl`、試験は仮想時計の fake を渡します。

## 試験

`tests/ui.rs` は `kivact::test` で、入力エラー、ローカル HTTP サーバーへの実 GET/POST（curl）、応答のコピー、履歴の保存と別 App での復元、最新要求の優先、キャンセル・再試行・送信失敗、履歴行からのフォーム復元、URL 欄へのフォーカス、35件連続送信時の履歴順序を確かめます。`src/http.rs` の単体試験は curl の送受信と、キャンセルでの子プロセス停止を確かめます。

```powershell
cargo test -p kivact-example-mini-postman
```
