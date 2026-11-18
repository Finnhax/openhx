# Editing the Active Preset

This document describes how to change the device's active preset: setting a block parameter, switching a block on or off, and saving the preset with its changes.

> **Status:** HX Stomp only. Derived from USB captures of HX Edit on an HX Stomp (firmware 3.71) and validated with openhx on the same device. Other models are refused until they are verified.

---

## Overview

All three operations are single requests on the **edit channel** (routing `80 10 ED 03`, resource 6), the same channel used to [read the active preset](./blocks.md). Each runs in a fresh edit session:

```
[EDIT HANDSHAKE]      seq 0x00   (see blocks.md)
[OPEN RESOURCE 6]     seq 0x02
[WRITE]               seq 0x03   cmd 0x04, one of the requests below
[REPLY]               status 0 once the device has applied the change
[DRAIN]               notifications on the third channel
```

Changes take effect at once, but the device keeps them only in the active preset's working copy. Selecting another preset discards them unless the preset was [saved](#save-the-preset) first.

---

## Request Framing

A write uses the edit channel's usual framing; the packet length depends on the payload:

```
LL LL 00 18                 frame length (bytes after the routing field) + magic
80 10 ED 03                 edit-channel routing
00 03 00 04                 seq 0x03, cmd 0x04
09 10 00 00 01 00 06 00     inner resource header (resource 6)
PP PP PP PP                 MessagePack payload length (LE u32)
<payload>                   {102: txn, 100: command, 101: args}
00 …                        zero padding to a 4-byte boundary
```

HX Edit numbers its transactions (`102`) and fills bytes 12–13 with a counter; the device echoes the transaction ID and accepts fixed values for both.

---

## Addressing a Block

Writes address a block by its **slot** in the preset data's 20-slot array (see [blocks.md](./blocks.md#slots)), key `98`:

| Path | Positions 1–8 | Slot (`98`) |
|---|---|---|
| 1 | 1–8 | `1`–`8` |
| 2 | 1–8 | `11`–`18` |

A parameter is further addressed by:

| Key | Meaning |
|---|---|
| `26` | Model within the block: `0` for a single model or the amp of an amp+cab block, `1` for its cab |
| `28` | Parameter index, in the order the device stores the values (the order in `crates/core/src/blocks/catalog.rs`) |

---

## Set a Parameter

Command type **30**:

```
{102: txn, 100: 30, 101: {98: slot, 29: true, 26: model, 28: param, 119: value}}
```

| Key | Type | Meaning |
|---|---|---|
| `98` | uint | Block slot |
| `29` | bool | Always `true` in captured traffic, for any block, path and model |
| `26` | uint | Model within the block |
| `28` | uint | Parameter index |
| `119` | float32 / int / bool | New value, stored the same way the preset data holds it (a knob shown as `4.0` is `0.4`, a cut filter is in Hz) |

The reply echoes the arguments with status `0`:

```
{102: txn, 103: 0, 104: {98: slot, 29: true, 26: model, 28: param, 119: value}}
```

### Examples

Kinky Boost in path 1, position 1 of "Pop-Punk"; Drive (parameter 0) set to `3.5`:

```
83 66 CD 03 F5             {102: 1013,
   64 1E                    100: 30,
   65 85                    101: {
      62 01                     98: 1,
      1D C3                     29: true,
      1A 00                     26: 0,
      1C 00                     28: 0,
      77 CA 3E B3 33 33         119: 0.35 (float32)}}
```

Other captured writes:

| Change | Arguments |
|---|---|
| Kinky Boost "Boost" off | `{98: 1, 29: true, 26: 0, 28: 1, 119: false}` |
| EQ in path 1, position 4: Low Cut 70 Hz | `{98: 4, 29: true, 26: 0, 28: 0, 119: 70.0}` |
| Amp+cab in path 1, position 3: cab mic distance 1.25 | `{98: 3, 29: true, 26: 1, 28: 0, 119: 1.25}` |
| Pitch Echo in path 2, position 4: Note Sync | `{98: 14, 29: true, 26: 0, 28: 10, 119: 9}` |

HX Edit sends one request per slider step while a control is dragged. For a switch it repeats the final value once with `cmd = 0x0C` instead of `0x04`; the device answers both alike, and a single `0x04` request is enough.

---

## Switch a Block On or Off

Command type **41**:

```
{102: txn, 100: 41, 101: {98: slot, 59: enabled}}
```

The reply is `{102: txn, 103: 0, 104: nil}`. The device accepts the command for an empty slot too, without effect.

---

## Save the Preset

Command type **71** stores the active preset, with its changes, in a slot:

```
{102: txn, 100: 71, 101: {107: setlist, 108: preset, 109: name}}
```

The name is a NUL-terminated string, as in a [rename](./rename.md). HX Edit saves to the active preset's own slot under its current name. The reply is `{102: txn, 103: 0, 104: nil}`. Unlike a rename, success is status `0`.

---

## Notifications

After each applied change, the device sends a notification on a third channel (routing `F0 03 02 10`, host→device acknowledgements on `02 10 F0 03`):

```
{105: event, 106: {82: …, 68: …, 121: …, 106: details}}
```

| Event (`105`) | Sent after | Details (`106.106`) |
|---|---|---|
| `30` | Parameter change | The arguments of the write |
| `49` | Block on/off | `{98: slot, 59: enabled}` |
| `39` | Block selected in HX Edit (command `78`, `{98: slot, 26: model}`) | `{98: slot, 26: model}` |
| `5`, `6` | Save | `{107: setlist, 108: preset, 116: …}`, then the name |

openhx does not read these; it drains them after each write.

---

## Command Reference

| Command (`100`) | Operation | Arguments (`101`) | Success |
|---|---|---|---|
| 20 | Select preset (also accepted on the edit channel) | `{107: setlist, 108: preset}` | status `1` |
| 30 | Set parameter | `{98, 29, 26, 28, 119}` | status `0`, arguments echoed |
| 41 | Block on/off | `{98: slot, 59: enabled}` | status `0` |
| 71 | Save preset | `{107: setlist, 108: preset, 109: name}` | status `0` |
| 78 | Select block (cursor only) | `{98: slot, 26: model}` | status `0` |
