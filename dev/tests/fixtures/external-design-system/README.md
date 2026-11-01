# 外部デザインシステムの独立 fixture

この package は独立した Cargo workspace です。直接依存は公開 facade の
`kivact` だけで、公式 kit、engine、reactive には依存しません。
`kivact::ui` を使わず、通常の公開 node で button を、
`interaction::use_dialog` の部位（content・title）に、自前の scrim・panel・見出しの style を付けて modal dialog を作ります。
button の Enter・Space・Activate は core の既存の起動経路を使います。
select と menu も interaction の trigger・content・item の部位を使い、
独自の角の小さい枠とアクセント色で描画します。項目は `for_each` で描画し、
menu は `submenu(key)` の子 handle でサブメニューも構成します。
選択・キー操作・focus の管理を fixture 側で重複実装しません。

実 window の公開 driver で、button の role・名前・click・Enter・Space・Activate の
一度だけの起動と disabled、dialog の role・名前・初期 focus・Tab と Shift+Tab の
閉じ込め・Escape と opener への focus 復帰を確かめます。dialog を再度開き、
`primitive::Owner` が作った状態の破棄と新しい node token も確かめます。
select は Alt+Down と click による開閉、矢印と Enter による選択、閉じた状態の
矢印操作、Escape と trigger への focus 復帰を確かめます。menu は Enter、
矢印の折返し、typeahead、左右キーでのサブメニュー開閉、callback の一度だけの
起動、選択後と Escape 後の focus 復帰を確かめます。

このディレクトリで次を実行します。window を開く試験はすべての worktree で共通の
lock を通します。既定の試験は GPU を使い、最後のコマンドは GPU feature を無効にした
CPU 構成です。

```sh
export CARGO_TARGET_DIR="$(git rev-parse --show-toplevel)/target"
cargo fmt -- --check
cargo check --tests
cargo clippy --all-targets --all-features -- -D warnings
../../../../dev/tools/window-lock.sh --timeout 3000 -- cargo test
../../../../dev/tools/window-lock.sh --timeout 3000 -- cargo test --no-default-features
```

GUI harness、OS の実入力、IME、スクリーンリーダーの手動確認は含みません。
