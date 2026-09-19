# Frontier SH414 — the host-input axis wired to a real event source (input delivered, not latent)

Date: 2026-09-22, hermes-worker, single-agent (cone suppressed). Workspace green
before + after (`cargo test --workspace` EXIT 0; arm64jit lib 468/0 with the 3 new
SH414 hermetics). Capture runs/capture_sh414_input_axis.sh. No production path /
JIT hook default / guest byte changed (new code in ainput.rs + session.rs, both
off the 1MiB hooks).

## Why this cycle

The SEP-18 BUILD-THE-RUNTIME roadmap names *session boot -> screens -> audio ->
input* as the four runtime axes. SH400-413 landed session (ordered substrate),
screens (G3 content surface), audio (SH132 fake-libaaudio bridge), and the FIRST
half of input (SH413 ainput: marshal_touch / fire_touch, real-image-pinned).
But SH413 left the input axis **latent-and-orphaned**: `fire_touch` had ZERO
production callers, and the `input-wrapper` crate (the X11 event pump + the
`PointerTracker` mouse->ACTION_DOWN/MOVE/UP translation) was a **dev-only**
dependency — unreachable from any production runtime path. The real X11 window is
created + its XID registered as the guest ANativeWindow (shims.rs, SH112/SH303),
but no host loop delivers events to the guest. STATUS "next-forward #3" names
exactly this: *wire the input-wrapper X11 pump to fire_touch so a real host loop
delivers events to a constructed login/home screen.* SH414 is that missing
executable half — a cause-not-symptom build-the-runtime surface (not a DM seed).

## What landed

- **input-wrapper promoted from dev-dependency to a real arm64jit dependency**
  (Cargo.toml). The crate is no longer dead/orphaned — it is imported by
  production `session.rs`.
- **ainput.rs** (off-hook): `from_motion_event(&MotionEvent) -> TouchAction`
  — converts the (already-built) input-wrapper translated stream into the guest
  `nativePassInput` ABI, preserving the real pointer id (multi-touch survives)
  and coordinates. Plus `deliver_motion(img, base, tpidr, boot_sp, ev)` — the
  production delivery entry that marshals one translated motion event into the
  guest native via `fire_touch` (env-gated, inert without JIT_AINPUT_BRIDGE).
- **session.rs** (off-hook): `drive_host_input_pump(iimg, ib, tpidr, boot_sp,
  events) -> usize` — a first-class host-input runtime step: for each translated
  `MotionEvent` it delivers through the SH414 bridge into the guest. Inert
  (returns 0, no guest path) unless (a) `JIT_AINPUT_BRIDGE` is set AND (b) a
  real window XID is registered (shims::anativewindow_xid != 0) AND (c) a live
  input image is present. Guards tested a/b/c explicitly.
- **3 new hermetics**: `sh414_from_motion_event_preserves_pointer_and_pos`,
  `sh414_deliver_motion_inert_without_env`, and
  `sh414_host_input_pump_inert_without_arm` (all three inert guards fire in
  order; NO guest path touched while any guard trips).

## Measured (real libroblox.so, this HEAD)

- arm64jit lib 468/0 (3 new), workspace green.
- recon-v3 type4 self-driven frame deliverable re-verified green with the promoted
  dependency (no regression): 24 real task-driven frames `present swap Ok(0x1)`,
  197 node pops, 0 json abort, 0 crash, EXIT 124.
- real-image pin byte-exact: nativePassInput @0x2bbba88 `sub 0xd10143ff`,
  `bl consumer @+0x50` = 0x940a4aed.

## Honest

Latent-but-correct, exactly like SH132/SH413: input only lands on a screen once a
live session owns a DataModel (the guest `nativePassInput` -> `RobloxInput::processInputEvents`
consumer `0x2e4e68c` gate). On the current boot path the pump is inert by its
three guards, so the product path is byte-identical. This completes the input axis
as a REAL deliverable host capability (X11 pointer -> translated -> guest native)
instead of an orphaned dev-only translation this crate historically shipped dead.
Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, no make_shared,
MH_GAME_LOADED false).

## Files

- crates/arm64jit/Cargo.toml (input-wrapper promoted to a real dep)
- crates/arm64jit/src/ainput.rs (from_motion_event, deliver_motion + 2 hermetics)
- crates/arm64jit/src/session.rs (drive_host_input_pump + 1 hermetic)
- runs/capture_sh414_input_axis.sh
- Commit (pending).

## Next (standing)

The input axis is now a deliverable host loop. The one genuinely-open front
remains a completed do-init owning a live DM — that is what turns every latent
axis (audio, input, screens, the binder, the app-start) REAL. Do-not-re-tread
unchanged: do NOT re-drive LSM skips (SH349/350/358/373/375/377/378/385/396),
do NOT re-manufacture the map (SH396), setDataModelToCurrent (SH388), EC reader
(SH355/356/374), window-attach real (SH367), ALooper (SH365), governor gates
(SH379). SH174 capture-latch stays the single forward observer (must run with
JIT_DM_ALLOC_CAPTURE_DELEGATE=1, SH395).