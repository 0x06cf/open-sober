# SH195 — scene-attach ABI gate: the self-constructed PlayerGui's live vtable is
# genuinely populated, but its draw slot is an x1-forwarding adapter the render
# scene's walker cannot direct-dispatch.

## Date / context
Sep 16, 2026 (hermes-worker). Route-B line after SH194. SH194 closed the name→classid
RESOLUTION lever (resolver map 0x106dca0e70 = live-world-build-gated) and its "NEXT"
named scene-attach (R+0x180/0x188) via "a direct parent hook that bypasses getService
name-resolution, OR the live-DM session (migration)". This cycle attacked that scene-attach
step directly on the real binary, at the ABI level.

## Empirical (real libroblox.so, llvmpipe; the SH191/SH190d/e self-construction drive)
`capture_sh191_service_node.sh` at HEAD with a new benign read-back in
`routeb_dm_service_resolve_guard`:

```
[routeb-dmsvc] SH195: gen PlayerGui 0x7f..720 vptr 0x106648950 live vtable slots[0..16] =
  [0x104c25394, 0x104c253e8, 0x1025a9fa4, 0x105fb2dac, 0x105e0c5d8, 0x101dc7428,
   0x101db2cf0, 0x10229c2a4, 0x104ba5e2c, 0x102376aa4, 0x104c25170, 0x1024bfe18,
   0x10246f37c, 0x101db2cf0, 0x102c226cc, 0x102c226f0]
```

Decoded (guest 0x106648950 + N*8):
- **+24 (the per-item draw the scene present-walker 0x5b2eec0 blr's as `draw(render_obj)`)** =
  guest 0x105fb2dac, file 0x5fb2dac:
  ```
  5fb2dac:  ldr x9,[x1]        ; x9 = *x1  (SECOND object)
  5fb2db0:  mov x8,x0          ; save this(x0)
  5fb2db4:  mov x0,x1          ; x0 = x1 (the second object becomes `this`)
  5fb2db8:  mov x1,x8          ; x1 = original x0 (PlayerGui)
  5fb2dbc:  ldr x3,[x9,#56]
  5fb2dc0:  br  x3             ; dispatch through *x1[..]->vt+56
  ```
  → This is a **bouncing adapter**: the PlayerGui's draw entry does NOT render itself; it
  forwards to a *base/enclosing* render object in x1 via `[x1]->vt[+56]`, passing the original
  x0 back in x1. The scene present-walker's per-node loop (SH64 pin:
  `ldr x0,[x20,#8]; ldr x8,[x0]; ldr x8,[x8,#24]; blr x8`) supplies ONLY x0=render_obj, never
  x1 ⇒ calling the genuine PlayerGui draw through the renderer faults / dispatches to a stale
  register. A genuine GuiObject cannot be dropped into the R+0x180/0x188 scene node list as
  a bare render-obj.
- **+64 (the dims-query)** = guest 0x104ba5e2c = a tail-`b 0x4c25364`, a leaf getter returning
  a cached singleton from [0x6c97000+3864] (harmless, but not the scene node's dims interface).
- **+56** = guest 0x10229c2a4 — the `mov x0,xzr; ret` NULL stub (same benign dead-end vtable row
  seen in the DM family per SH186c).

## Conclusion — scene-attach lever closed at the ABI level
The self-constructed PlayerGui is a REAL, runtime-vtabled engine object (positive: its vtable
is populated, not zeros — first direct live-vtable read of this class). But its render entry
(x1-forwarding bounce) means the engine's own scene present-walker cannot draw it when installed
as a raw scene node: the walker has no way to supply the base object in x1. That base object
(enclosing gui/render entry with a real vt+56 leaf) is built only during a live scene-graph /
world-build — the same live-DM migration gate. No host-side seed constructs it; a manufactured
forwarder for it is a live-render-entry, out of reach headlessly.

**Confirmed delegating thunk (disasm, this cycle):** 0x5fb2dac is an Itanium-style delegate —
`x0=PlayerGui(this)`, `x1=base`; `ldr x9,[x1]; mov x8,x0(orig); mov x0,x1(base); mov x1,x8(PGI);
ldr x3,[x9,#56]; br x3` — i.e. the honest PlayerGui draw forwards to the **base object's vtable
slot +56**, moving the PlayerGui into x1. This is a *mechanism-level proof* (not a judgment) that
the standalone self-constructed instance cannot be the thing a scene walker draws: its real draw
contract requires the base/container render entry only a live scene-graph build provides. This is
the operator-requested "concrete proof-of-dead-end" for the scene-attach sub-frontier (vs. an ROI
call) — do NOT re-attempt under a "maybe we can fake x1" framing (faking x1's vt+56 = host thunk =
Route-A host geometry, not engine-authored).

## Do-not-re-tread
- Do NOT write the genuine PlayerGui (or ScreenGui) instance pointer into an R+0x180/0x188
  scene node's [node+8] and drive the present-walker — it jumps to the x1-stale bounce
  (FTL / wrong-dispatch), not a leaf draw.
- Do NOT manufacture a fake base object to satisfy the bounce unless a real vt+56 leaf draw
  exists; a fabricated object's vt+56 would be a host thunk, again host-composed geometry
  (Route-A), not an engine-authored draw.

## What IS shipped
- A benign, default-inert (JIT_ROUTEB_DM_SERVICE_NODE) live-vtable read-back of the
  self-constructed PlayerGui (16 slots), so future runs cheaply confirm the vtable is populated
  and pin the draw/dims slots. Hermetic sh191 test still passes (380/0 arm64jit lib; 558/0 ws).
- The ABI characterization above (bouncing-adapter draw ⇒ scene present-walker single-x0 cannot
  dispatch a genuine GuiObject) as the standing Route-B scene-attach gate.