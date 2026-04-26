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

Map functionality derived from: [AccuMap](https://github.com/MartinovEm/Ethos-GPS-AccuMap)
