# 独立した core 消費者

この fixture は独立した workspace です。Kivact の workspace lint、公式 UI kit、内部 crate を直接参照せず、`kivact` の公開 API だけでアプリを構築します。

```sh
cargo check --manifest-path dev/tests/fixtures/core-only-consumer/Cargo.toml
```

コンパイル検証はウィンドウを開きません。`cargo run` は実窓を開きます。
