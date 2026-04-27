# BoundryMap

BoundryMap is an Ethos widget that overlays editable boundary lines on a GPS map.

## Display Features

- Home and aircraft position use the same icon assets as GPS AccuMap when available, with drawn fallbacks if an icon cannot be loaded.
- The aircraft indicator can be configured as a dot or a heading arrow.
- GPS coordinate text is optional. When enabled, coordinates render in the lower-left area above distance text instead of the lower-right touch controls.

## Map Assets And Privacy

Maps usually identify a specific flying site, so the `maps/` directory is local-only and ignored by git. Do not commit personal field maps, generated metadata, or boundary sidecars to the public repository.

Install with Ethos Suite:

1. Build a package with `python tools/build.py --project BoundryMap --dist`
2. Import the generated ZIP through Ethos Suite Lua install/import.

When local maps exist under `scripts/BoundryMap/maps/`, the build scans them using the generic asset rules in [build.json](build.json):

- BMP and PNG files install to `/bitmaps/GPS/`
- matching JSON map metadata files install to `/documents/user/`
- generator metadata text files are ignored by the package because BoundryMap reads the JSON metadata
- built-in widget icons install under `/scripts/BoundryMap/icons/`

Multiple maps can be packaged together without adding each map filename to [build.json](build.json). Use one folder per map:

```text
scripts/BoundryMap/maps/
├── MyField/
│   ├── MyField.bmp
│   └── MyField.json
└── PracticeSite/
    ├── PracticeSite.png
    └── PracticeSite.json
```

Clean checkouts without a local `maps/` directory still build the widget ZIP; they just do not include private map assets.

## Creating Maps

Create map files with the [Ethos GPS Map Generator](https://martinovem.github.io/Ethos-GPS-Map-Generator/), the same tool used by AccuMap. Choose the flying area, export the map, and place the generated bitmap plus JSON metadata in a local folder under `scripts/BoundryMap/maps/`.

The generator may also create a human-readable metadata text file. Keep it locally if useful, but it is not installed by the build.

Boundary edits are stored per map in the radio's `/documents/user/` folder using the `<map-stem>.boundries.json` filename pattern.

Map functionality derived from: [AccuMap](https://github.com/MartinovEm/Ethos-GPS-AccuMap)
