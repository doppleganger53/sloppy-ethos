# BoundryMap

BoundryMap is an Ethos widget for flying with a GPS map and editable boundary
lines. It shows your home point, aircraft position, optional coordinates and
distance, and warns when a boundary is crossed or lies directly ahead on the
current flight path.

The screenshots below were captured from the actual Ethos 1.6.6 X20RS-FCC
WebSimulator canvas using a neutral demo map named `GuideField`, so no private
flying-site map or real field coordinates are published in this repository.
The map views use Ethos' `Full screen` layout and a three-sided, non-crossing
demo boundary. The boundary-warning screenshot uses a run-scoped simulated GPS
state because the stock simulator model used for capture does not expose a GPS
telemetry source.

## Quick Start

1. Create a map with the
   [Ethos GPS Map Generator](https://martinovem.github.io/Ethos-GPS-Map-Generator/).
2. Put the exported bitmap and JSON metadata in a local folder under
   `scripts/BoundryMap/assets/maps/`.
3. Build the install ZIP:

   ```powershell
   python tools/build.py --project BoundryMap --dist
   ```

4. When updating an existing radio install, delete
   `/scripts/BoundryMap/main.luac` from the radio first. Ethos prefers that
   cached bytecode over the updated `scripts/BoundryMap/main.lua` in the ZIP.
5. Import the generated ZIP with Ethos Suite.
6. Add the `BoundryMap` widget to a screen.
7. Open the widget settings, choose your map and GPS source, then enable the
   options you want.

## Map Files

BoundryMap expects each map bitmap to have a matching JSON metadata file with
the same stem:

```text
scripts/BoundryMap/assets/maps/
├── MyField/
│   ├── MyField.bmp
│   └── MyField.json
└── PracticeSite/
    ├── PracticeSite.png
    └── PracticeSite.json
```

The build scans `assets/maps/` automatically:

- BMP and PNG map files install to `/scripts/BoundryMap/assets/maps/`.
- matching JSON metadata files install to the same radio folder.
- generator text metadata is not packaged because the widget reads the JSON.
- boundary sidecars are saved as `<map-stem>.boundries.json`.

Maps usually identify a specific flying site. The local map folders are ignored
by Git; do not commit personal maps, generated metadata, or boundary sidecars
to the public repository.

## Widget Settings

![BoundryMap settings screen](docs/images/boundrymap-settings.png)

Use the widget settings to choose the map and tune how the in-flight overlay
behaves.

| Setting | Use |
| --- | --- |
| `Map` | Select the packaged bitmap map file. |
| `Map rotation (degrees CW)` | Turn the map clockwise by 0–359 degrees. Zero keeps north up. |
| `GPS Source` | Select the GPS telemetry source used for latitude and longitude. |
| `Heading Indicator` | Choose `Dot` or `Arrow` for the aircraft marker. |
| `Signal Timeout (s)` | Mark GPS data stale after this many seconds without a fresh fix. |
| `Coordinates` | Show or hide the current GPS coordinates on the map. |
| `Distance` | Show or hide distance readouts. Without altitude, this is 2D ground distance. |
| `Altitude Source` | Select a source in metres or feet before home is acquired to include height above home in distance. |
| `Reset Home` | Clear the learned home point so the widget can learn it again. |
| `Boundry Warning` | Choose `None`, `Audio`, `Haptic`, or `Both`. |
| `Warning Type` | Choose one-time `Momentary` feedback or repeating `Constant` feedback. |
| `Speed Source` | Optional ground-speed or airspeed telemetry source used for predictive warnings. |
| `Pre-warning Time` | Keep prediction `Off`, or warn when the projected path reaches a boundary in 1 to 10 seconds. |
| `Diagnostics` | Open a status page for GPS, map, metadata, sidecar, and last error. |

On firmware without a Boolean control, **Coordinates** and **Distance** use an
Off/On selector. Both controls read and save the same settings.
## Telemetry overlays and flight statistics

Open **Telemetry overlays → Set up** to show altitude or speed, select up to
three extra sensors, or choose live and stale text colors. Values use the
sensor's displayed unit. Missing, stale, or nonnumeric values show `--`.
Coordinates and distance keep their existing colors with the default settings.
Boundary warnings always retain their red and yellow colors.

Compact layouts shorten labels (for example, `Spd` and `F1`) and reserve room
for boundary controls, warnings, coordinates, and distance. When fewer telemetry
rows fit than are enabled, the rows cycle every three seconds.

Open **Flight statistics → View** for moving time, maximum horizontal distance
from home, and minimum, arithmetic mean, and maximum for each selected sensor.
The widget samples at most once per second. A sample counts as moving when the
selected speed is fresh and at least **3 m/s** (10.8 km/h or about 6.7 mph).
Time accumulates between consecutive moving samples at most two seconds apart.
Gaps and stationary periods contribute no time. Max distance needs a current
GPS fix. Each sensor's mean gives equal weight to each valid moving sample.
Changing a sensor or its units starts that sensor's values over. Hiding an
overlay leaves its collection active.

The statistics page shows a snapshot. **Refresh** updates it, **Reset** clears
the session statistics, and **Back** returns to settings. Resetting statistics
preserves home and pending boundary edits. Restarting the widget starts a new
session. Only overlay settings and selected source names are saved.
### Map rotation

Set the angle to match your view from the flight line. For example, at 90°
north points toward the right edge. The top-right `N` readout shows the
clockwise direction of north. The widget centers the bitmap at its original
pixel scale. The window clips a map larger than its widget. Turning the map also clips its corners at the edges of the centered bitmap. Touches in
blank corners leave the boundary unchanged.

You can draw, delete, and save with the map rotated. Changing the angle keeps
drafts and saved lines in their original map coordinates. The requested angle
is saved with this widget. If the bitmap cannot be rotated, `N up!` indicates
that the map and controls are using north-up coordinates. A smaller map can
reduce the memory needed while switching angles. Use a 32-bit BMP or PNG if
the simulator reports `Map bitmap unavailable`.

## Diagnostics

![BoundryMap diagnostics screen](docs/images/boundrymap-diagnostics.png)

Run diagnostics when the widget is not showing the map or telemetry you expect.
The diagnostics page checks:

- GPS source selection and current latitude/longitude availability.
- selected map bitmap availability.
- JSON metadata availability and shape.
- boundary sidecar availability and line count.
- the last runtime error captured by the widget.

Use `Back` to return to the main settings page.

## In-Flight Display

![BoundryMap normal flight display](docs/images/boundrymap-normal-flight.png)

The map screen shows the selected map and boundary lines. With GPS telemetry,
it also shows the home icon and aircraft indicator. The lower-left overlays
show coordinates and distance when those settings are enabled. The top-left
status shows how many boundary lines are on the map out of the six-line limit,
or `Unsaved *` when changes need to be saved.

Home is learned after 100 consecutive widget updates with a fresh, stable GPS
position. Changing or clearing the GPS source clears the previous home and
position. Use **Reset Home** on the ground to learn a new reference.

**Lost or implausible GPS.** The widget keeps the last accepted coordinates and
ground distance, shows the aircraft in red, and suspends warnings. It rejects
inactive or expired telemetry, invalid coordinates, the receiver's `0,0`
no-fix pair, and the two poles where the map projection is undefined. Flying
on the equator or prime meridian is supported.

Between fixes, the permitted displacement is 25 m plus 150 m/s of elapsed
sample time, capped at two seconds of travel. A larger jump requires at least
three mutually consistent samples spanning three seconds. A change in
coordinates or a drop in source age identifies a refreshed sample. Polling a
single unchanged fix cannot confirm a jump.

**Distance from home.** The ground-distance display suppresses horizontal
jitter under 5 m. Home acquisition also captures a fresh altitude in metres or
feet, even with `Distance` turned off. Live distance then includes the
difference from that altitude. At home, a 300 m sea-level reading therefore
shows 0 m. A 100 m climb and 100 m of horizontal travel show about 141 m.

Missing or expired altitude uses ground distance. If altitude was unavailable
at home, or you change the altitude source or unit, the widget keeps using
ground distance until you reset home on the ground. After changing a sensor's
offset or zeroing the altimeter, also use **Reset Home**. ETHOS does not expose
that calibration change to this widget. The stale-GPS readout always retains
the last accepted 2D distance, including with `Distance` turned off.

## Drawing Boundaries

![BoundryMap draw mode](docs/images/boundrymap-draw-boundary.png)

Tap `Draw` to enter draw mode. Drag on the map from the start of a boundary
line to the end of it. The draft line is yellow while you are drawing. When you
release, the line is added and the status changes to `Unsaved *`.

BoundryMap stores up to six boundary lines per map. Short accidental drags are
ignored.

## Deleting And Saving

![BoundryMap delete and save controls](docs/images/boundrymap-delete-save.png)

Tap `Delete` to enter delete mode, then tap near a boundary line to remove it.
Tap `Save` after adding or deleting lines. Saved boundaries are written next to
the selected map as `<map-stem>.boundries.json`, so each map keeps its own
boundary set.

Unsaved edits remain visible while the widget is running, but they are not
persisted until you tap `Save`.

## Boundary Warnings

![BoundryMap boundary warning](docs/images/boundrymap-warning.png)

The red `Boundary exceeded` overlay appears when the line from home to the
aircraft crosses one of the current boundary lines. When `Speed Source` and a
1-to-10-second `Pre-warning Time` are configured, a yellow `Boundary ahead`
overlay appears when the current heading and speed project an outbound crossing
within that time. Unsaved edits can trigger warnings while the widget is
running, but they must be saved to persist after restart. Warning feedback
depends on the settings:

- `None`: show the overlay only.
- `Audio`: play tone feedback.
- `Haptic`: play haptic feedback.
- `Both`: play tone and haptic feedback.
- `Momentary`: alert once when first entering an ahead or exceeded state while
  moving away from home.
- `Constant`: repeat feedback while the ahead or exceeded state remains active.

Predictive warnings require fresh GPS and speed telemetry plus a heading derived
from recent GPS movement. GPS ground speed is usually the best source for the
projected ground track. Airspeed is supported, but wind can make an airspeed-only
projection differ from the actual ground path. A turn after the last GPS heading
update can also briefly make the projection lag the aircraft.

For best results, keep boundary sets simple and non-crossing. A three-sided box
is a practical default when you want a readable keep-out shape without closing a
full perimeter. Closing the shape is optional, but it does not enable polygon
containment; the widget still compares the home-to-aircraft line against each
saved segment independently.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `No map selected` | Select a map in the widget settings. |
| `Map metadata unavailable` | Confirm the bitmap and JSON metadata have the same file stem. |
| GPS coordinates do not update | Run diagnostics and confirm the selected GPS source reports latitude and longitude. |
| Distance is hidden | Enable `Distance`; select an altitude source only when you want 3D distance. |
| Boundary edits disappear after restart | Draw or delete the lines again, then tap `Save`. |
| No warning feedback | Confirm `Boundry Warning` is not `None` and the aircraft is moving farther from home when entering the exceeded state. |
| No predictive warning | Select a supported speed source, choose a 1-to-10-second pre-warning time, and confirm both GPS and speed telemetry are fresh. |

## Attribution

Map handling is derived from
[AccuMap](https://github.com/MartinovEm/Ethos-GPS-AccuMap). See
[LICENSE](LICENSE) for BoundryMap licensing details.
