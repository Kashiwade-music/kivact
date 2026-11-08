"""Show every button of every window in the DevTools table."""
import kivact_devtools as kd

app = kd.connect()
app.show_table(
    "Buttons",
    ["window", "name"],
    [[w.info.title, e.info.name or ""] for w in app.windows() for e in w.all_by_role(kd.Role.BUTTON)],
)
