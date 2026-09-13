# SH76 — the login surface is now a LIVE animated film

## Result

`RENDEREMITTER_LIVE=1` (with `RENDEREMITTER_LOGIN=1` + `RENDEREMITTER_MULTI=1`)
makes the engine's real geometry emitter draw a **per-frame-animated** Roblox
login surface: the RO-BLOX wordmark gently bobs up/down between consecutive
present-walker frames, so the picture is not a single static frame but a live
film. Verification is the probe's own geometry: the wordmark probe y MOVES
between the two emitter drives in the same run while staying Byte-EXACT
present.

Real libroblox.so, exit 124 (stable idle), zero crash:
```
frame 1:  probe 'logo_white_1x.png' (560,496) got rgba(255,255,255,255) expect [255,255,255,255] diff=[0,0,0,0] present=true
frame 2:  probe 'logo_white_1x.png' (560,474) got rgba(255,255,255,255) expect [255,255,255,255] diff=[0,0,0,0] present=true
[elfjit:renderemitter-multi] engine emitter Ok(ret=0x0) count=36 sprites=5 swap=Ok(1)  x2
```
10/10 probes present=true (both frames × 5 sprites). The wordmark probe y
shifted **496 → 474 (+22px NDC bob)** across the two frames while remaining
byte-exact-white — the surface is animated, not a single frozen frame.

## What changed (elfjit.rs)

`render_engine_emitter_multi` now honors `RENDEREMITTER_LIVE=1` (login mode
only): a static `LIVE_N` counter advances each call; the wordmark placement
(index 1) gets a per-frame vertical bob (±0.03 NDC, alternating). A `pl(i)`
placement-resolver applies the bob consistently to BOTH the quad-build and the
probe geometry, so the probe tracks the moving wordmark and stays valid. The
emitter is re-driven once per present-walker frame, each re-uploading fresh
(animated) geometry — the same pattern SH71 used for the spinning loader,
proven not to trip the SH67c silent-drop class here.

## Honest scope

The motion is authored (a harness counter), not engine-self-driven. It is the
"live login film" the SH73 next-artifact doc recommended (B), and it
demonstrates the engine's emitter re-rasterizing fresh geometry frame after
frame with correct alpha compositing — the frame-loop plane a real session needs.
Standing structural wall unchanged (type-4 producer vector glue-only;
nativeGameGlobalInit parks; no in-image GuiObject->scene-list path).

## Repro

`env RENDEREMITTER_LIVE=1` on the `runs/capture_emitter_login.sh` recipe.
Log `runs/sh76-emitter-live.txt`. Workspace **509/0**; example tests **11 pass**.

## Next

(D) text labels on the field/button for a more legible form (needs glyph
rasterization of an extracted font); or the standing structural wall — a real
self-populated session behind the Lua app-shell.