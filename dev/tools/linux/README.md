# Linux verification tools

- `test-wslg.sh`: WSLg Wayland / XWayland の検査runnerです。前提・コマンド・環境変数・ログ・未検証範囲は [開発者向け資料](../../../docs/for-kivact-developer/tools/linux-verification.md) を参照してください。
- `verify-gpu-interop.sh`: ハードウェア GPU のある Linux 実機で、GPU texture 共有（実験 feature `gpu-interop`）を clone から報告まで検証します。手順と確認する結果は [手動確認一覧](../../../docs/for-kivact-developer/manual-verification.md) の「GPU texture 共有」の節にあります。
