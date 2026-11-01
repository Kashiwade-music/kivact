# workspace 外の consumer

この package は自身の空の `[workspace]` を持ち、root workspace の依存設定や lint を継承しません。依存は `kivact` と公式 UI の3 crate だけです。独自の `ExternalAction` も公開 `Node` / `NodeBuilder` / `IntoView` だけで実装し、公式部品と同じ modifier・入力・支援技術の経路を試験します。

`interactions::calibration` は Scalar/Drag の一つの操作本体に色だけ異なる二つのスキンを付けます。`device_picker` は String ID の ListItem を独立 package で組み立て、並べ替えても caller の選択を保ちます。キーと AT SetValue が同じ Binding writer へ届くことを実ウィンドウで確認します。これは外部 package の利用可能性の検証で、第三者による採用や初学者本人の理解の検証ではありません。

`official_device_choices` は同じ `ListItem<String>` と `Binding<Option<String>>` を、
`kivact::ui::tabs`・`select` と kit 固有の `radio_group` へ渡します。標準部品の実装は
一つで、Pop/Game/Creative の theme を `provide_theme` で切り替えます。tabs は
見出しだけを描画し、空の panel は作りません。button と checkbox も `kivact::ui` を
使用し、独自の `ExternalAction` は引き続き公開 NodeBuilder だけで実装します。
すべて通常の API がアプリの String キーを受け取り、位置 index への変換はアプリ側に
ありません。公開試験は各 theme で並べ替え、filter、削除・再追加、disabled の除外、
不可視の選択を保持したままの Tab 移動、setter の拒否と Err を確認します。
新しい test binary は増やさず、既存 `public_api` の module として実行します。

```powershell
$env:CARGO_TARGET_DIR = "$PWD/target/official-ui-consumer"
$env:KIVACT_TEST_GPU = "1"
& "C:/Program Files/Git/bin/bash.exe" dev/tools/window-lock.sh --timeout 3000 -- cargo test --manifest-path dev/tests/fixtures/official-ui-consumer/Cargo.toml
Remove-Item Env:KIVACT_TEST_GPU
& "C:/Program Files/Git/bin/bash.exe" dev/tools/window-lock.sh --timeout 3000 -- cargo test --manifest-path dev/tests/fixtures/official-ui-consumer/Cargo.toml --no-default-features
```

全試験は公開 `WindowMode::Real` で実ウィンドウを作り、公開 `RenderOptions::gpu` で描画経路を選ぶ。通常はCPU、環境変数を指定した実行はGPU。motion試験は50msの中間frameと終了frameを区別し、reduced motionでは中間frameを飛ばすことも検証する。OSの設定は変更しない。

workspace-policy の試験（`dev/tests/workspace-policy/tests/dependencies.rs`）は、3 UI crate の normal/dev/build/target 別依存を外部 package も含めて `kivact` に制限します。consumer 自体の metadata は別に `cargo metadata --manifest-path dev/tests/fixtures/official-ui-consumer/Cargo.toml --no-deps` で読み出せます。これは CI workflow を追加するものではなく、外部開発者と同条件の構築・実 window の E2E 検査です。
