# OPEN: 設計者への差戻し

- U23 `crates/kivact-core/src/view/mount/semantics.rs:61` — STOP: button ごとに意味論 patch の Vec が mount 後も effect に保持される（割当て元 `kivact_core::view::element_data::Common::append_semantics`、1280.0 B/item、M-R5）/ 要素を消してよいかの設計判断はできないため変更せず報告。影響する設計書の節: 4.7 節 MEM-4（M-R5）。
