# BoundryMap

BoundryMap is an Ethos widget that overlays editable boundary lines on a GPS map.

Install with Ethos Suite:

1. Build a package with `python tools/build.py --project BoundryMap --dist`
2. Import the generated ZIP through Ethos Suite Lua install/import.
3. Place matching map bitmaps in `/bitmaps/GPS/` and metadata JSON files in `/documents/user/`.

Boundary edits are stored per map in `/documents/user/<map-stem>.boundries.json`.
