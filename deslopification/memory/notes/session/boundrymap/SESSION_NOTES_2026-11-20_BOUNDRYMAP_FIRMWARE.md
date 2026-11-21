# BoundryMap firmware and asset checks, November 20, 2026

## Note Placement

- Artifact: `session`
- Scope: `boundrymap`
- Concern: `testing`

## Sensor settings

On the X20RS-FCC 26.2.0 WebSimulator, opening the original widget's
configuration stopped at GPS Source. The simulator created the dropdown and
then raised `bad argument #5 to 'addSensorField' (function expected, got SensorChoice)`.
The native API returned a SensorChoice for an explicitly supplied fifth
argument of either nil or a filter callback. Using nil preserves the unfiltered
selector and requires only one field creation. A blanket error handler would
also have hidden unrelated errors. FrSky marked the upstream fix for 26.2.1.
Keep the explicit parameter while supporting 26.2.0.
[FrSky issue 6443](https://github.com/FrSkyRC/ETHOS-Feedback-Community/issues/6443)

On 26.1.2 the native Boolean field exists. If a runtime lacks that control,
the widget offers Off/On choices backed by the same stored Boolean.

## GPS and assets

With an S.Port GPS simulated on X20RS-FCC 26.1.2 and 26.2.0, optioned reads
on a selected source and optioned source lookup both returned the injected
37.25°, −121.75° fix. Before the first frame the native source returned nil
coordinates and age −1.

On 26.1.2, relative and absolute loads of all three tracked icons returned
bitmaps. A synthetic GuideField image and matching JSON loaded from the
absolute script-local map path. A native sidecar write landed in the same
directory. A build using nested synthetic map inputs placed the flattened
bitmap, metadata, and sidecar in `scripts/BoundryMap/assets/maps/`. all ZIP
entries matched the bytes in an isolated simulator deployment. Personal maps
and radio model files stayed outside Git.

## Widget settings and Diagnostics

With the same staged Lua on X20RS-FCC 1.6.8, 26.1.2, and 26.2.0, the settings
page opened through the normal widget menu and all rows through Diagnostics
were accessible. Selecting GPS and enabling Coordinates and Distance worked.
Diagnostics reported the demo map, metadata, two sidecar lines and no runtime
error on all three. On 26.1.2 and 26.2.0 it displayed the injected GPS fix.
On 1.6.8 it correctly showed the selected sensor without a fix. That
WebSimulator does not export the S.Port injection entry point.

The 26.2.0 simulator reproduces the settings failure. The original report
names 26.2.0 RC5/RC6. No runnable X20RS packages for those candidates were
available from the verified FrSky release feeds during validation.

## Follow-up

Real receiver data and physical-radio interaction remain pilot checks.
