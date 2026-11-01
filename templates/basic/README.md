# Kivact basic template

The smallest Kivact app: one component, a window and a real-window test.

```text
cargo run -p kivact-template-basic      # window (Arvane host, Windows verified)
cargo test -p kivact-template-basic     # the same UI in a test window (kivact::test)
```

To start a new app, use this directory as the independent-package manifest, rename the package and point the
`kivact` dependency at your checkout (or a git dependency). The app uses `use kivact::prelude::*;` including the standard components of `kivact::ui`: a button with a Button role, focus, accessible name and an Activate action, in the theme's look. Add `features = ["test"]` in `dev-dependencies` for `kivact::test`.

Guides: [tutorial](../../docs/for-kivact-user/en/getting-started/README.md).
