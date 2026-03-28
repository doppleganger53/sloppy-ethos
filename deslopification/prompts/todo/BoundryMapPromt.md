# Prompt

You are generating code for a FrSky Ethos Lua widget.

## Goal

Create a widget named `BoundryMap` (`{WIDGET_NAME}`) that implements:

*   Primary function: Enhance existing GPS AccuMap widget to allow the user to draw boundy lines on the map display. Audio and tactile warnings will be triggered by the aircraft crossing the boundries relative to the home location.
*   Primary user value: user feedback of aircraft location

## Environment

*   Radio target(s): X18 and X20 family
*   Ethos version target: `1.6.4+` (`{ETHOS_VERSION}`)
*   Project layout:
    *   Script entry point: `scripts/{WIDGET_NAME}/main.lua`
    *   Optional images: `scripts/{WIDGET_NAME}/images/`
    *   Map location: `/bitmaps/GPS`
    *   `{METADATA}`: `/documents/user/`
*   Reference Project: `referenceProjects\Ethos-GPS-AccuMap` (`{REFERENCE_PROJECT}`)

## Ethos API Requirements (Critical)

*   Use Ethos-native widget registration:
    *   `init()` calling `system.registerWidget({...})`
    *   `return { init = init }`
*   Use Ethos callbacks only (as needed): `create`, `paint`, `wakeup`,
    `configure`, `read`, `write`
*   Do not assume OpenTX/EdgeTX callback naming (`refresh`, `background`,
    etc.) unless explicitly requested.
*   Callback safety:
    *   Defensively handle unexpected `nil` callback args.
    *   Never crash on startup or page navigation if context is incomplete.

## Functional Requirements

Use `{REFERENCE_PROJECT}` as a starting point. Obey LICENSE and document appropriately.

###   User can define up to 6 2-point boundry lines on the map

1.  Control Interface graphical touch buttons/toggles on bottom right of display
    1.  Draw (toggle) - enables touch editing of boundries
    2.  Delete (toggle) - touching existing boundry removes it
    3.  Save (button) - stores data in json file in `{METADATA}` location associated to currently visible map
2.  While Draw is selected, user can tap, drag, release to define a line that will show on the map in real time
    1.  Tap defines point 1 of the line.
    2.  point 2 is updated dynamically as the drag continues with a 5hz refresh rate & finalized when touch is released
    3.  a subsequent tap will start drawing a new line

### Warning behavior

1. A boundry warning occurs when the current aircraft position crosses a boundry line in the direction heading away from the home location
2. The aircraft is no longer exceeding a boundry when there are no boundry lines between the current aircraft position and the home location.

### Widget Configuration

1. Add dropdown to select 'Boundry Warning' Options:
    1. None (Default) - no behavior change
2. Add dropdown to select 'Warning Type' Options:
    1. Momentary - warning is played once
    2. Constant - warning is played continuously until aircraft position is no longer exceeding boundry

### Interaction model: follow `{REFERENCE_PROJECT}` where applicable

## Data Access Requirements

*   Handle simulator-vs-radio API differences safely.
*   If data handles may be table or userdata, support both where practical.
*   Avoid expensive rescans every frame; use cached discovery where possible.

## Performance & Stability Requirements

*   Keep `wakeup` lightweight; avoid heavy loops each call.
*   UI should be stable and deterministic (no forced auto-scroll unless requested).
*   Avoid instruction-budget issues (`max instructions count reached`).

## UI Requirements

*   follow `{REFERENCE_PROJECT}`

## Technical Constraints

*   Keep implementation intentionally simple and maintainable.
*   Prefer root-cause solutions over compatibility shims or temporary workarounds.
*   Avoid unnecessary global state.
*   Commenting: follow `{REFERENCE_PROJECT}`
*   Do not add unrelated features unless requested (alarms, logs, advanced menus).

## Packaging & Delivery Requirements

*   Provide complete `main.lua`.
*   If requested, include/update packaging script so output ZIP installs through
    Ethos Suite Lua import.
*   Expected ZIP layout: `scripts/{WIDGET_NAME}/...`

## Validation Requirements

*   Script compiles with `luac -p`.
*   Widget appears in Ethos widget picker.
*   Widget renders without runtime errors in simulator and/or target radio.
*   Data display path and empty-state path both validated.
*   Interaction behavior validated (for example: manual scroll input works).
*   Tests should be able to mock aircraft movement across boundries

## Output Format

Return:

1.  Complete `main.lua` implementation.
2.  Short explanation of architecture and data flow.
3.  Explicit test checklist (simulator + radio when applicable).
4.  Any assumptions made due to API/version uncertainty.
