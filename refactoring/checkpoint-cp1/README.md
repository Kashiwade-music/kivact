# CP1 中断時の記録

2026-11-04、ユーザーの「区切りの良いところで中断し、結果と記録をコミット・push」という指示で保存した作業途中の記録。**CP1 は未通過。W2 の実装は未着手。** W1 の語彙統合と、その検証で発見した回帰修正までを integration branch に保存する。

最新の検証結果と再開位置は `../PROGRESS.md` の「ユーザー指示による中断」を参照。Rust source の検証対象は `97689c4`。その後の変更は記録のみ。

## この bundle

- `TRACE-RESUME.md`: 比較結果、残る問題、再実行の手順。
- `numeric-provenance.md` / `numeric-review.md`: f32→f64 の型・計算経路、数値差の未解決項目。
- `schema-review.md`: 新しい field の全件集計と、旧 trace では観測できない範囲。
- `source-review.md`: caller location の確認と、異なる rerun の対応付けを未解決として残した理由。
- `values-review.md`: 型名・Debug 表記・診断文字列の分類要約。全件 JSON と raw の行番号付き証拠はローカル `target/foundation-v3/cp1/` に保存。
- `checks.json`: 最終 source の検査結果、件数、時間、trace manifest の hash。
- `w2-preparation/`: read-only の R-2 調査。構築・依存の問題点と選択肢であり、未採用の助言を含む。W2 の実装や新しい設計決定を表すものではない。
- `tools/`: 比較・ソース位置確認・集計に使用した診断スクリプトの保存版。golden test や製品 API ではない。CP0 自己比較は 915 files / 差分 0、最新 trace 比較は REVIEW_REQUIRED。

raw trace、大きな全件 JSON、build/test log は作業手順書どおり commit しない。`target/foundation-v3/traces/{cp0,cp1-after-boundary}` と各 manifest を保持して再開する。別環境では raw を別途移す必要がある。比較スクリプトだけで未解決の同値性を証明できるわけではない。

保存版スクリプトはこの配置から直接実行できる。例:

```text
python refactoring/checkpoint-cp1/tools/compare_traces.py --baseline target/foundation-v3/traces/cp0 --current target/foundation-v3/traces/cp1-after-boundary --output target/foundation-v3/cp1/comparison-resume-new
```

再開時はまず FrameTick epoch を考慮した対応付けを直し、自己比較を確認してから新しい出力 directory へ比較する。既存の原ログ・比較結果を上書きしない。残る差を分類・検証し CP1 を通すまでは W2 に進まない。

この bundle は一時的な refactoring 記録。長期の根拠を F-1 で移した後、F-3 の `refactoring/` 削除と一緒に削除する。
