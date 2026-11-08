# DevTools を使う

[English](../../en/how-to-guide/devtools.md)

DevTools は、動いている Kivact のアプリに接続する別のアプリケーションです。すべてのウィンドウの要素ツリーとレイアウトを表示し、スタイルと emulation を編集でき、turn と frame を記録して画面が更新された理由を調べられます。コメントとスタイルの変更を review としてまとめ、ほかの人や AI エージェントと共有できます。同じ protocol は MCP の tool と Python の script からも使えます。

## ツールをインストールする

DevTools アプリと MCP サーバーを Kivact のリポジトリーからインストールします。2 つの binary は `PATH` に入り、target のアプリは `PATH` を DevTools アプリを探す 4 番目で最後の場所として使います。

```sh
cargo install --git https://github.com/Kashiwade-music/kivact kivact-devtools kivact-devtools-mcp
```

Python の client は pip でインストールします。この command は Kivact の checkout の外でも動きます。

```sh
pip install "kivact-devtools @ git+https://github.com/Kashiwade-music/kivact#subdirectory=python/kivact-devtools"
```

Kivact のリポジトリー自体を編集するときだけ、`pip install -e python/kivact-devtools` を使ってください。

## DevTools を有効にする

DevTools は `kivact` crate の `devtools` feature の後ろにあります。調べたいアプリで有効にします。

```toml
[dependencies]
kivact = { git = "https://github.com/Kashiwade-music/kivact" }

[features]
devtools = ["kivact/devtools"]
```

`cargo run --features devtools` で起動します。配布する release のビルドには feature を入れないでください。有効な release のビルドも動きますが、起動時に警告を出します。

`.kivact/review/` と `.kivact/traces/` を `.gitignore` に足してください。

## DevTools を開く

動いているアプリで F12 か Ctrl+Shift+I を押します。Ctrl+Shift+C は pick mode を始めると同時に DevTools を開きます。pick はキーを押したウィンドウで始まります。DevTools がすでに開いていれば、そのキーで前面に出ます。

キーと editor の command は workspace root の `.kivact/devtools.json` から読みます。

```json
{
  "$schema": "https://raw.githubusercontent.com/Kashiwade-music/kivact/main/crates/kivact-devtools-protocol/schema/devtools-config.schema.json",
  "keys": {
    "open": ["F12", "Ctrl+Shift+I"],
    "pick": ["Ctrl+Shift+C"]
  },
  "editor": ["code", "--goto", "{file}:{line}:{column}"]
}
```

chord は修飾キーの `Ctrl`・`Shift`・`Alt`・`Meta` を `+` でつなぎ、最後にキーの名前（`F12`、`Enter`、1 文字）を書きます。`keys.open` と `keys.pick` は別々です。片方を省くとその既定のまま、`[]` にするとそのキーだけを無効にできます。`editor` の既定は `["code", "--goto", "{file}:{line}:{column}"]` で、`{file}` は workspace root からの絶対パスに置き換わります。Windows で `code` を起動できないときは、`code.cmd` を 1 回だけ試します。

target のアプリは DevTools アプリを、`.kivact/devtools.json` の `app`、環境変数 `KIVACT_DEVTOOLS_APP`、target の実行ファイルと同じディレクトリ、`PATH` の順に探します。見つからないときは `DevTools app not found.` と報告します。上の `cargo install` で入れるか、`app` に実行ファイルのパスを設定してください。

`.kivact/` は、review か trace の file を最初に書くときに作られます。起動時には作りません。

## 要素を調べる

Elements パネルは、toolbar で選んだウィンドウの要素ツリーを表示します。行には role、名前、作成位置、更新回数（記録を始めてから数えたもの）が出ます。pick ボタン（または Ctrl+Shift+C）で target の要素をクリックして選びます。

- 作成位置をクリックすると、その file が editor で開きます。
- 「Copy locator」は `kivact::test` の式を clipboard にコピーします。test id があれば `screen.get_by_test_id("increment")`、無ければ role と名前で `screen.get_by_role(Role::Button, "Increment")` です。別のウィンドウの node には `screen.window("Settings").` が前に付きます。
- 「Owners」を切り替えると、要素ツリーがウィンドウの owner ツリーに変わります。行には owner の label と作成位置が出て、選ぶとその owner の最初の要素が選ばれます。
- Layout タブには flex/grid の重ね表示の toggle があります。子の箱と隙間を target の画面に重ねて描きます。

## スタイルを編集する

Styles タブはスタイルの層を優先の高い順に並べます。checkbox で各 override を有効・無効にし、値の入力は Enter で適用します。override の範囲は次の 3 つです。

- このノードだけ
- 同じ呼び出し位置から作られたすべてのノード
- theme token

同じタブで、hover や pressed などの interactive state を強制することもできます。空の一覧にすると解除されます。「Record changes in review」をオンにすると、変更が review の item に記録されます。

## review にコメントして書き出す

要素には Elements から、ウィンドウには toolbar からコメントを足せます。変更のたびに `review.json`（schema と版番号を持つ正本）と `review.md`（人向けの同じ内容）が `.kivact/review/<YYYYMMDD-HHMMSS>-<pid>-<port>/` に自動で保存されます。別の書き出し操作はありません。「Open folder」でそのディレクトリを開きます。ツリーから消えた node の item には「(removed)」が出て、強調はできなくなりますが、コメントは編集できます。

`review.json` の例:

```json
{
  "schema": "kivact-devtools-review/1",
  "app": {
    "name": "Counter",
    "pid": 12345,
    "kivact_version": "0.1.0",
    "build": "debug",
    "exe": "C:\\dev\\counter\\target\\debug\\counter.exe",
    "cwd": "C:\\dev\\counter",
    "workspace_root": "C:\\dev\\counter"
  },
  "session": "20261108-053200-12345-49152",
  "created": "2026-11-08T05:32:00Z",
  "updated": "2026-11-08T05:34:10Z",
  "environment": {
    "pseudo_locale": "off"
  },
  "items": [
    {
      "id": "C1",
      "window_title": "Counter",
      "window": 1,
      "target": {
        "kind": "node",
        "node": 17,
        "role": "button",
        "name": "Increment",
        "source": { "file": "src/main.rs", "line": 42, "column": 9 },
        "call_chain": [
          {
            "location": { "file": "src/main.rs", "line": 42, "column": 9 },
            "app_code": true
          },
          {
            "location": { "file": "src/main.rs", "line": 12, "column": 5 },
            "app_code": true
          }
        ],
        "bounds": { "x": 24, "y": 96, "width": 96, "height": 32 },
        "states": ["hovered"]
      },
      "style_changes": [
        {
          "scope": { "kind": "node", "node": 17 },
          "property": "padding",
          "after": "12"
        }
      ],
      "image": "images/C1.png",
      "comments": [
        {
          "id": "C1.1",
          "text": "Rename this to \"Add one\".",
          "created": "2026-11-08T05:33:01Z"
        }
      ]
    }
  ]
}
```

対応する `review.md`:

```markdown
# DevTools review: Counter

- Session: 20261108-053200-12345-49152 / Updated: 2026-11-08T05:34:10Z
- Build: debug / Kivact 0.1.0
- Emulation: none

## C1 Button "Increment"  (window "Counter")

- Source: src/main.rs:42:9
- Call chain: src/main.rs:42 ← src/main.rs:12
- Query: screen.get_by_role(Role::Button, "Increment")
- State: hovered / Bounds: x 24, y 96, 96×32
- Image: images/C1.png
- Style changes:
  - padding: (unset) → 12  (node #17)
- Comments:
  1. Rename this to "Add one".  (2026-11-08T05:33:01Z)
```

## 画面が更新された理由を見る

toolbar で記録を始めてからアプリを操作します。Updates パネルは、入力・更新・frame のレーンを持つ時間軸を表示します。turn はきっかけで色分けされ、frame の予算線と比べられます。範囲を選ぶと拡大できます。下には選んだ turn の原因グラフが出て、きっかけから状態の変化、再計算、変わった要素、layout/paint/frame の帯へと進みます。カードをクリックすると target の要素が強調され、ダブルクリックで editor が開きます。まとめた node は「×N」で展開できます。「画面で点滅」と「選んだ要素だけ」を切り替えられ、画面上のヒートマップも使えます。

## パフォーマンスを測る

Performance パネルは、frame の時間をウィンドウごとの予算線付きの棒で表示し、turn の Stage と frame の段階を flame chart にし、GPU と CPU のどちらで描いたか（fallback を含む）を示します。Counters は折れ線で、snapshot を取って 2 つの差分を見れば、何が増えて何が消えたかが分かります。paint flashing の toggle は、描き直した箱を点滅させます。「Perfetto 形式で書き出す」は `.kivact/traces/` に番号付きの名前で file を書きます。

## アクセシビリティを確かめる

Accessibility パネルは、ウィンドウの accessibility tree、audit の結果の表（行を選ぶと target で強調されます）、番号付きの Tab 順、focus の履歴を表示します。

## 環境をエミュレートする

Emulation パネルは、theme（なし・Light・Dark）、reduced motion、locale、pseudo-locale、DPI の倍率、touch を設定します。設定していない項目は上書きしません。theme の emulation は `platform::theme_hint()` に従って theme を選ぶアプリにだけ効き、パネルにもその注意が出ます。pseudo-locale は `message()` と `message_with()` を通る文字列だけを変えるので、`text("...")` に直接書いた文字列は変わりません。

## AI エージェントから DevTools を使う（MCP）

`kivact-devtools-mcp` は stdio の MCP サーバーです。次のように登録します。

```sh
claude mcp add kivact-devtools -- kivact-devtools-mcp
```

`initialize`・`subscribe`・`show_table` を除くすべての protocol の method が、同じ名前の tool になります。これに local の 2 つを足したものが tool の一覧です。`list_targets` は起動中の target（path、アプリ名、pid、port、版、起動時刻、workspace root）を返します。token は含みません。`select_target` は `{ "target": "<path>" }` を受け取ります。選ばないときは `KIVACT_DEVTOOLS_TARGET`、起動中の target が 1 つだけならそれを使います。0 個か 2 個以上なら、`list_targets` を呼ぶよう促す error を返します。

tool はパネルと同じ範囲を覆います。要素とレイアウト（`get_tree`、`get_node`、12 の query、`get_owners`、`screenshot`、`highlight`、`start_pick`/`stop_pick`、`lint_layout`）、更新と性能（記録、`list_turns`、`list_frames`、`get_counters`、snapshot、`export_trace`）、スタイルと emulation（`set_style_override`、`force_state`、`set_emulation`、`set_overlays`、`set_time_scale`）、入力（`inject_input`、`list_inputs`）、review（`get_review`、`add_comment`、`edit_comment`、`remove_comment`、`export_review`）です。

`inject_input` の後の turn は、`list_turns` に `wait_ms` を付けて待ちます。

## Python で DevTools を操作する

Python の package は[ツールをインストールする](#ツールをインストールする)の pip の command で入ります。VS Code では Pylance と pyright が型ヒントと `py.typed` を読むので、stub は要りません。

```python
import kivact_devtools as kd

app = kd.connect()                       # the only running target
win = app.window("Counter")              # by title
button = win.get_by_role(kd.Role.BUTTON, "Increment")
buttons = win.all_by_role(kd.Role.BUTTON)
button.highlight()
app.clear_highlight()
button.click()
win.type_text("abc")
win.press_key("Ctrl+Enter")              # the chord syntax of the config file

with app.record() as rec:
    button.click()
for turn in rec.turns:
    print(turn.turn, turn.duration_ns, len(turn.changed_nodes))

app.export_review()                      # [review.json, review.md] paths
```

`connect(target=...)` は discovery file の path を受け取り、`name="Counter"` はアプリ名で選びます。どちらも無いときは `KIVACT_DEVTOOLS_TARGET`、起動中の target が 1 つだけならそれを使います。`Window` の query は protocol と同じ `get_`/`query_`/`find_`/`all_` の形です。`app.rpc` は低水準の型付き method で、protocol の dataclass を受け取って返します（`app.rpc.get_tree(...)`）。手書きの `App`・`Window`・`Element`・`Recording` はそれらを継承しません。

Scripts パネルは `.kivact/devtools/scripts/` の `.py` を一覧して実行します。interpreter は設定の `python`、workspace root の `.venv`、`python3`、`python` の順に探します。標準出力と標準エラーを行ごとに表示し、`show_table` の表も表示します。

## 制限

- embed のアプリでは `inject_input` が `UNSUPPORTED`（-32015）で断られます。
- すべての client は同じ状態を共有します。2 つの client が同時に編集すると、最後の書き込みが勝ちます。
- 最後の client が切れると、DevTools が作った状態（pick、重ね表示と強調、style の override、強制状態、emulation、時間の倍率、記録）は元に戻ります。review はディスクに残ります。
- layer の境界の表示はありません。paint flashing と flex/grid の重ね表示を使ってください。
- `focus_window` は最善の努力です。Wayland など効かない環境では、DevTools アプリは前面に出ません。

## 関連項目

- [エラーや更新の原因を調べる](diagnostics.md): DevTools が読むエラー、警告、記録。
- [アプリをテストする](testing.md): 「Copy locator」が書く query。
- [複数のウィンドウを開く](windows.md): DevTools が一覧して前面に出すウィンドウ。
- [プラットフォームのサービスを使う](platform-services.md): ファイルダイアログやパスを開く操作などの OS の機能。
