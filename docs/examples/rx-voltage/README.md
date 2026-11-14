# Repeat the RX Voltage example

These are **assistant steps** for the [pilot walkthrough](../../FIRST_WIDGET.md).
The sample uses the `RxBatt` source from FrSky's
[ETHOS 1.6.6 Lua documentation](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/1.6.6/lua-doc.zip).
It is an example, not a published release. Work from the root of a fresh
standalone checkout on your own working branch. Check for Python, Lua
(`lua` and `luac`), and Node.js for a headless run. If the project already
exists, open a new copy instead of overwriting it.

```powershell
Copy-Item -Recurse docs/examples/rx-voltage/RxVoltage scripts/
lua docs/examples/rx-voltage/check.lua
python tools/build.py --project RxVoltage --dist
```

The Lua fixture runs the widget's registered callbacks with supplied readings;
it checks the font, a missing source, 5.2 volts, zero volts and a lost signal.
The ZIP is **dist/RxVoltage-0.1.0.zip**. No Suite configuration is needed.

To preview it on **X20RS-FCC / 1.6.6**, use a fresh path for each run.
The harness downloads the public radio runtime on first use.

```powershell
$smoke = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py headless --project RxVoltage --radio X20RS-FCC --ethos-version 1.6.6 --persist-dir "$smoke"
$preview = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
python tools/sim/harness/run.py gui --project RxVoltage --radio X20RS-FCC --ethos-version 1.6.6 --persist-dir "$preview"
```

On a fresh **1.6.6** preview:

1. Choose a language, acknowledge the fresh-profile notices, and use the
   top-left back arrow to leave the model wizard.
2. On the home screen, tap the **overlapping screens** icon at the bottom.
   Under **Screen1**, tap **Configure** in the large left widget area.
3. Open the **Widget** list, scroll down, and choose **RX Voltage**. Tap
   **Back**, then the **Home** icon. With no active RxBatt source, look for
   a dash under **RX Voltage**. If the model has an active sensor with that
   name, compare the displayed number to the telemetry reading.

A simulator startup does not check either visible result. If the GUI or
sensor is unavailable, report the fixture result and the observation still
needed. Ctrl+C stops the GUI server.

For “one size larger,” change the working widget's `FONT_XL` to `FONT_XXL`.
Run `lua docs/examples/rx-voltage/check.lua larger`, rebuild the ZIP, and
open a fresh preview. The pilot decides if that size fits and is easier to read.
