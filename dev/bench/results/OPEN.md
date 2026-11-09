# OPEN: 設計者への差戻し

- U23 `crates/kivact-core/src/view/mount/semantics.rs:61` — STOP: button ごとに意味論 patch の Vec が mount 後も effect に保持される（割当て元 `kivact_core::view::element_data::Common::append_semantics`、1280.0 B/item、M-R5）/ 要素を消してよいかの設計判断はできないため変更せず報告。影響する設計書の節: 4.7 節 MEM-4（M-R5）。
- CP1 K5 `wheel_n3000` — B0 mean 24.436 ms / p95 28.291 ms、CP1 mean 27.555 ms / p95 29.168 ms（mean +12.8%、p95 +3.1%）。K1・K4 とほかの K5 行は改善。関連 unit: U21（paint facts）・U22（a11y geometry）。CP1 の規則に従い記録して先へ進む（W7 の B1 で再判定）。
- CP1 環境メモ — Windows のリンカ PDB 上限とディスク不足のため、checkpoint のテストは `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` で実行した（debug assertions は有効のまま。試験の内容・期待値は変えていない）。
