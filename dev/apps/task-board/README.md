# Task Board

3列（予定・進行中・完了）のタスクボードです。カードの追加、選択、編集、左右移動、完了チェック、列の選択、検索、削除ができます。変更は `%LOCALAPPDATA%\KivactExamples\task-board.json` に自動保存され、別プロセスで再起動すると復元されます。

```powershell
cargo run -p kivact-example-task-board
```

「題名」と「メモ」を入力して「追加」を押します。カードの「選択」で同じ入力欄へ値が読み込まれ、上部に編集バー（更新・左へ・右へ・列の選択・削除）が表示されます。カード自身にも「右へ」ボタンと「完了」チェックボックスがあります。カードの「閲覧 n」は選択された回数で、列を移動しても残ります。

## 構成（1ファイル1コンポーネント）

| ファイル | 内容 |
|---|---|
| `components/app.rs` | ルート。`resource` でJSONをworker上で読み、読込中・失敗（再試行ボタン）・成功を `show`/`show_some` で切り替える |
| `components/board.rs` | 共有状態を `provide(state, || ..)` で子へ渡し、入力欄・編集バー・3列・保存状態を並べる。保存は `action(..).policy(ActionPolicy::QueueLatest)` |
| `components/lane.rs` | 1列。検索語と列で絞った `Vec<Arc<Task>>` の memo を `virtual_list` で表示する |
| `components/task_card.rs` | 1枚のカード。行の値は `Value<Arc<Task>>` で届く |
| `components/task_editor.rs` | 選択中タスクの編集バー（`select` で列を選ぶ） |
| `components/search.rs` | 検索欄 |
| `context.rs` | `BoardCx`（Copy handle だけの構造体）と、盤面を変える操作 |
| `model.rs` / `storage.rs` | 盤面（256件ごとの共有ページ）と、JSONの読み書き（一時ファイル＋rename、64MiB上限、破損ファイルの退避） |

- **状態:** すべて Copy の handle（`Signal`・`Value`・`Var`・`Selector`・`Action`）です。コンポーネントは `context::<BoardCx>()` で受け取り、`move` で捕獲します。`.clone()` や `Rc<RefCell<..>>` は使いません。
- **選択の強調:** `selector(selected)` の `is(&id)` は、真偽が変わった2枚のカードだけを再描画します。
- **カードの局所状態:** 「閲覧 n」は `visits: Signal<HashMap<u64, u32>>` としてデータ側に置きます。仮想一覧の行はスクロールや列移動で破棄されるため、残したい状態は行の外に置きます。
- **保存:** 盤面が変わるたびに `effect_on(board, ..)` が `saver.run(..)` を呼びます（`effect_on` は初回の値では走らないので、読み込んだ直後は保存しません）。`ActionPolicy::QueueLatest` の action は同時に1件だけ書き、待機中のスナップショットは最新1件に置き換えます。状態表示は `saver.state()`（準備完了・保存中・保存しました・保存できません）で、失敗は後の保存が成功するまで残り、その間「保存を再試行」が有効になります。ウィンドウを閉じると App が待ち行列を最大3秒 drain してから終了します。
- **10万件:** 各列は `virtual_list` なので可視範囲だけを mount します。1件の編集で複製されるタスクは1件です（単体試験で計測）。確認用に、保存ファイルがないときだけ生成ボードで起動できます。

```powershell
$env:KIVACT_TASK_BOARD_GENERATED = '100000'
cargo run -p kivact-example-task-board --release
```

## 試験

`tests/ui.rs` は `kivact::test`（Arvane の実 window・仮想時計）で、追加・選択・検索・編集・列移動（ボタン・select・checkbox・キーボード）・削除・再起動後の復元、破損ファイルの再試行、保存失敗と再試行、連続変更の合流、仮想一覧のスクロールを確かめます。`src/lib.rs` の単体試験は10万件で可視範囲だけが mount され、1件の移動で `Task` の複製が1件であることを確かめます。

```powershell
cargo test -p kivact-example-task-board
```

電源断時の耐久性（sync と rename の順序）は OS 実機での別検証が必要です。
