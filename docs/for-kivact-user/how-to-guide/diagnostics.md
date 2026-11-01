# エラーと更新の理由を調べる

最初に、一回の操作で再現できる画面と、変化するはずの値を絞ります。`Screen::errors()` で診断を確認し、`ErrorKind` と作成・操作位置から調べます。`Unsupported` を無視する前に、その node や region が指定した機能を受けられるかを確認してください。

表示が変わらない場合は、`text(signal.get())` のように setup の snapshot を渡していないかを調べます。動的な値は closure や `memo` から読みます。部品が二重に動く場合は、派生値を別の signal へ effect でコピーしていないかを調べます。借用中の書込みは、`with` で判断を返してから行います。

`NodeRef` に結んだ node は `diagnostics::why(&reference)`、試験で得た `Element` は `.why()` で、動的 property を書いた effect と、その依存の作成・更新位置を読めます。未 mount の参照には説明がありません。依存の値そのものは記録しません。

`Screen::trace()` は turn の原因、書込み、再評価を返し、`Screen::stats()` は node・owner・timer などの計数を返します。操作の前後を比べ、変更していない部分まで再作成していないか、画面を閉じた後に owner や timer が残っていないかを確認します。計数は表示結果の代わりにはしないでください。

まず [画面のテスト](../getting-started/testing.md)で利用者の観測結果を固定し、必要な場合だけ計数を調べます。性能の測定方法と window-lock は [開発者向け設計](../../for-kivact-developer/design.md#host-の性能hostbench)、OS 入力を伴う確認は [手動検証](../../for-kivact-developer/manual-verification.md)にあります。
