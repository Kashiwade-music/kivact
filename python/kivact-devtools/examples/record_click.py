"""Click the first button and print what the recording collected."""
import kivact_devtools as kd

app = kd.connect()
windows = app.windows()
button = next((e for e in windows[0].all_by_role(kd.Role.BUTTON)), None) if windows else None
if button is None:
    raise SystemExit("no button")
with app.record() as rec:
    button.click()
print(f"turns: {len(rec.turns)}")
print(f"duration_ns: {sum(turn.duration_ns for turn in rec.turns)}")
