# kivact-devtools (Python)

Typed Python client for the Kivact DevTools. It discovers a running app that was
built with the `devtools` feature, connects to its agent, and drives it from
`unittest` tests or scripts.

## Install

From the Kivact repository (no checkout needed):

```sh
pip install "kivact-devtools @ git+https://github.com/Kashiwade-music/kivact#subdirectory=python/kivact-devtools"
```

If you edit the Kivact repository itself, install the package in editable mode:

```sh
pip install -e python/kivact-devtools
```

## VS Code

Select the Python interpreter of the environment where `kivact-devtools` is
installed (Command Palette: "Python: Select Interpreter"). The type hints and
`py.typed` are read by Pylance without any other setting.

## Examples

Start the target app with the `devtools` feature first, then run one of the
examples:

```sh
python examples/list_buttons.py
python examples/record_click.py
```

```python
import kivact_devtools as kd

app = kd.connect(name="Counter")
window = app.window("Main")
button = window.get_by_role(kd.Role.BUTTON, "Increment")
button.highlight()
button.click()
app.clear_highlight()
app.close()
```
