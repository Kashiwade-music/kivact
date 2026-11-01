# Creative kit how-to guides

[日本語](../../../jp/creative/how-to-guide/README.md)

Task-oriented guides for the Creative kit (`kivact-ui-creative`). Each page starts with the shortest working solution, then covers the variants and the details you are likely to hit. They assume you finished the [tutorial](../getting-started/README.md). The full API is in rustdoc (`cargo doc -p kivact-ui-creative --open`), and every control runs in the catalog: `cargo run -p kivact-ui-creative-catalog`.

| Guide | What it shows |
|-------|---------------|
| [Theme the Creative kit, set fonts and add icons](theme-and-icons.md) | Install the theme, choose an accent and density, switch light and dark, use your fonts, add icons, translate the kit's texts |
| [Edit a timeline, dope sheet, graph or arrangement](timeline-and-arrangement.md) | Show clips, keys, curves and tracks, apply or refuse the editor's requests, keep the playhead in your app, transport and mixer widgets |
| [Connect nodes in a node graph](node-graph.md) | Typed sockets, links, node parameters and previews, connection rules, undo of a gesture |
| [Edit colors, easing curves and gradients](color-curves-and-gradients.md) | Color popup and wheel, cubic Bezier easing with a preview, gradient stops, one undo step per edit |
| [Show frames your application renders](show-app-rendered-images.md) | Display your own pixels with zoom, pan, overlays and pointer coordinates; render on a worker thread; experimental GPU frames |
| [Build docks, panel groups and outliners](panels-docks-and-outliners.md) | Dock layouts you own, outliner rename, reparent and restrictions, inspector sections, searchable pickers |
