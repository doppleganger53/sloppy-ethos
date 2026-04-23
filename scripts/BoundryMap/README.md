# BoundryMap

BoundryMap is an Ethos widget that overlays editable boundary lines on a GPS map.

Install with Ethos Suite:

1. Build a package with `python tools/build.py --project BoundryMap --dist`
2. Import the generated ZIP through Ethos Suite Lua install/import.
3. The package now includes the example `maps/WJRC` files declared in [build.json](build.json):
   - `WJRC.bmp` installs to `/bitmaps/GPS/WJRC.bmp`
   - the matching WJRC metadata JSON installs to the radio's `/documents/user/` folder

Additional map assets can be packaged and simulator-deployed by adding entries to [build.json](build.json) under `radioFiles`.

Boundary edits are stored per map in the radio's `/documents/user/` folder using the `<map-stem>.boundries.json` filename pattern.

## User Guide

Add the BoundryMap widget to a screen, then open widget configuration before flying:

- **Map**: choose the GPS bitmap to display. The widget expects a matching metadata JSON with the same map stem in the radio's user documents folder.
- **GPS Source**: choose the GPS telemetry source used to place the aircraft on the map and set the home point.
- **Heading Indicator**: choose Dot for a compact position marker or Arrow for a direction marker when movement gives the widget a usable heading.
- **Signal Timeout (s)**: set how long GPS telemetry can go stale before the aircraft marker is shown as stale.
- **Distance**: enable distance text from the home point. Add an altitude source when you want slant distance instead of ground distance.
- **Reset Home**: clear the current home point so the widget can set a new one after GPS stabilizes.
- **Boundry Warning**: choose no warning, audio, haptic, or both when the aircraft crosses a saved boundary line from home.
- **Warning Type**: choose Momentary for a one-time warning when first crossing while moving away, or Constant for repeated warnings while the boundary remains exceeded.

On the widget screen, use the on-screen controls:

- **Draw**: tap Draw, then drag across the map to add a boundary line. Up to six lines can be stored for each map.
- **Delete**: tap Delete, then tap near an existing line to remove it.
- **Save**: tap Save after drawing or deleting lines. Unsaved edits are marked with `Unsaved *`; saved maps show the current line count.

The widget draws saved boundary lines in cyan, the active draft line in yellow, home as `H`, and the aircraft marker in orange or red when GPS is stale. When a boundary is exceeded, the overlay shows `Boundary exceeded` and uses the configured warning feedback.

Map functionality derived from: [AccuMap](https://github.com/MartinovEm/Ethos-GPS-AccuMap)
