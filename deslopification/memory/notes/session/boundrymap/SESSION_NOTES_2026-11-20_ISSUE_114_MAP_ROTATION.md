# Session Notes 2026-11-20 - BoundryMap rotation on X20RS

## Note Placement

- Artifact: `session`
- Scope: `boundrymap`
- Concern: `testing`

## Display and input contract

The X20RS-FCC WebSimulators for ETHOS 1.6.8 (a6b30b5e) and 26.2.0
(0a6b337c) rotate positive angles clockwise. A 240×120 synthetic 32-bit BMP
had colored centers at (80,30), (160,30), and (90,85). In both a 300×280
widget and the 800×480 fullscreen layout, a synthetic drag between the first
two landmarks returned the original endpoints. Runs covered 0°, 37°, 90°,
180°, and 270°. The cyan line, home, aircraft marker, and north readout were
checked on the native framebuffer.

The centered rotation keeps the source image's dimensions. A rotated buffer
and the source coexist during conversion; the widget retains the display
buffer afterward. Display-only changes leave sidecar coordinates untouched.
The inverse transform rejects the blank corners around an oblique image.

## Firmware constraint and removal condition

Use the equivalent bearing plus 360° at nonzero cardinal angles to request
the centered native rotation. The 1.6.8 runtime's special 90° path failed
with a representative 480×272 map. The centered path produced the expected
landmark centers at 90°, 180°, and 270° on both builds. Keep this handling
until all supported runtimes pass the large-map cardinal and pixel-alignment
checks using ordinary cardinal angles.

On 1.6.8 an unsupported 24-bit BMP returned a truthy zero-size Bitmap.
A valid map and a valid rotated result must both have positive dimensions.

## Resource observations

In disposable X20RS-FCC simulator profiles, an 800×480 synthetic map cycled
through 90°, 37°, 180°, 270°, 90°, 37°, and 0°. The available bitmap memory
on both versions was 1,853,440 bytes after collection at every angle and
1,085,440 bytes while rotating. The widest observed switch interval in that
run was 14 ms on 1.6.8 and 26 ms on 26.2.0, including map reload, rotation,
and collection. These were simulator measurements. Physical-radio memory and
timing still need a pilot check with the installed widget set and map size.
