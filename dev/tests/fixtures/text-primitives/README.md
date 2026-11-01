# テキストの primitive の独立 fixture

この package は独立した Cargo workspace です。直接依存は公開 facade の `kivact` だけです。
`TextEditing` と `text_input` を使わず、3つの primitive だけで 1 行の editor（`line_editor`）を作ります。

- **段落**：`Constraints::measure_paragraph` で node の大きさを決め、`Draw::layout_paragraph` と
  `Draw::paragraph` で描き、`TextParagraph` の `caret`・`hit`・`rects` で caret・クリック・選択の帯を扱います。
- **IME session**：`NodeExt::ime_session` で確定済みの text・選択・caret を渡し、`on_ime` で composition・確定・
  再変換の範囲を受けます。composition は editor が text の中に下線つきで表示します。
- **a11y の text**：`NodeExt::semantics_text` で、描いた段落から作った text run・選択・caret を支援技術へ出します。

実 window の公開 driver で、入力と削除、矢印と Shift による選択、クリックとドラッグ（`hit`）、IME の
composition・確定・取消し・再変換、a11y の text run と選択、focus の移動による IME の入切を確かめます。
caret は grapheme ではなく char 単位で動きます（primitive が grapheme の境界を持たないため）。

このディレクトリで次を実行します。window を開く試験はすべての worktree で共通の lock を通します。

```sh
export CARGO_TARGET_DIR="$(git rev-parse --show-toplevel)/target/fx-tp"
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
../../../../dev/tools/window-lock.sh --timeout 3000 -- cargo test
```

GUI harness、OS の実入力、IME、スクリーンリーダーの手動確認は含みません。
