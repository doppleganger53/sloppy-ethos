# Make your first widget

What would you like to see on your transmitter? You describe the behavior and
decide whether it helps. Your coding assistant handles the files and checks.

**Open your own copy.** In a coding editor's “Clone Repository” or “Get from
Version Control” box, paste `https://github.com/doppleganger53/sloppy-ethos`.
Choose an empty folder, then open the downloaded repository folder (the one
containing `AGENTS.md`) in your coding assistant. You don't need a separate
workspace, ETHOS Suite profile, connected radio, or permission to publish here.

**Paste this starter** into Codex or another coding assistant with that folder
open. Add your idea in your own words.

```text
Read AGENTS.md in the open repository and follow it. Help me make my first
widget in my own local copy, on a new working branch. Ask what I want to see
or do, which radio and ETHOS version I know, and how I'll tell it works.
Wait for my answers; “I don't know” is fine. Use the widget prompt template
as your technical reference. Check which tools you have, find firmware-matched
FrSky references, implement one small behavior, and use this repo's checks
and packaging tools. Give me an isolated preview or a named install ZIP and
one thing to look for. Tell me what you actually tested. If you can't do a
step, give me just the next action, its expected result and what to send back.
Keep this local; leave my Suite setup and radio alone.
```

**Worked example: receiver voltage**

- **Pilot:** “Show my receiver voltage in large numbers. If the sensor is
  missing, show a dash.” Example answers: “X20RS, ETHOS 1.6.6. I want to
  read it at arm's length. The sensor is called RxBatt.”
- **Assistant:** Create a widget called **RX Voltage**. With a sample receiver
  reading of 5.2 volts it should draw **5.2 V**; with no reading it should draw
  a single dash (`-`). Use the [widget template](../deslopification/prompts/templates/EthosWidgetPromptTemplate.md)
  for implementation details. Public [FrSky releases](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases)
  list the docs and simulators by firmware; for this example use the
  [1.6.6 Lua docs](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/download/1.6.6/lua-doc.zip)
  and the X20RS-FCC WebSimulator from [that release](https://github.com/FrSkyRC/ETHOS-Feedback-Community/releases/tag/1.6.6).
  A missing sensor or a different source name on the pilot's model needs a
  separate check.
- **Try it:** In the isolated simulator preview, add **RX Voltage** to a
  widget area. With no receiver sensor, look for the dash. The assistant can
  also give you the install file `dist/RxVoltage-0.1.0.zip`. A successful
  build or simulator startup alone does not confirm the display.
- **Pilot's next request:** “Make the reading one size larger; keep the dash
  when there's no sensor.” Ask for a fresh preview, then decide if it's readable.

**If the assistant has no terminal:** It can still read the policy and prepare
the files. For this example, once it has created the widget and checked the
prerequisites with you, it can ask: “In your editor's terminal in the open
folder, paste `python tools/build.py --project RxVoltage --dist`. You should
see `Packaged widget ZIP:` and the filename. Paste that line back, or the first
error.” If no terminal or required program is available, it should explain
the one setup step you need, or hand the same folder and starter to a local
assistant that can run it.

For a repeatable example without generating new code, an assistant can use
the [sample and its display check](examples/rx-voltage/README.md).
