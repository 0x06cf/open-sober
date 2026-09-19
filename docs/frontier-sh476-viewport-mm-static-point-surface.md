# Frontier SH476 — wire the PlatformParams viewport{Width,Height}Mm display surface (getScreenPhysicalSizeInMillimeters -> Point -> x/y int fields)

Date: 2026-09-20, hermes-worker, single-agent (cone suppressed). Workspace green
before/after (`cargo test --workspace` EXIT 0; arm64jit lib 681 -> 682 incl. 1
new SH476 hermetic; `cargo build --workspace` + `--example elfjit` OK).

## Why this cycle

STATUS.md next-forward #4 named the one remaining unclosed, live-DM-independent,
BUILD-THE-RUNTIME display item: the PlatformParams viewport{Width,Height}Mm
(=338/190) surface. It was UNREACHABLE headlessly: the engine reaches it via a
JAVA STATIC call `DeviceUtils.getScreenPhysicalSizeInMillimeters()` (returns an
`android/graphics/Point`), then reads the Point's `x`/`y` INT FIELDS. The harness
serviced `CallStaticObjectMethod` (JNI slot 114) with the shared NULL stub
(`jni_voidp_0`), so the static call returned NULL and the whole chain died before
any field read. This is the same class of defect SH469 closed for
DisplayMetrics/Configuration — a session-content surface the engine reads to lay
its OWN UI out collapsing to 0.

## What landed (SH476) — crates/arm64jit/src/jni.rs

Real descriptors MEASURED in the .so first (literal-preservation, not assumed):
`com/roblox/platform/util/DeviceUtils`, `getScreenPhysicalSizeInMillimeters`
(static, `(Landroid/content/Context;)Landroid/graphics/Point;`), `viewportWidthMm` /
`viewportHeightMm` strings, and the log lines `getViewportDisplaySize: (in
millimeters): x = {}, y = {}` + `Failed to find class 'DeviceUtils'`.

1. `VIEWPORT_WIDTH_MM = 338`, `VIEWPORT_HEIGHT_MM = 190` (recon-framework-boot-order
   PlatformParams.viewport{Width,Height}Mm).
2. `viewport_point_handle()` — a DEDICATED fake `android/graphics/Point` object
   (OnceLock<new_fake_object>), distinct from the generic fake object. This is the
   load-bearing SCOPING decision: `x`/`y` are generic field names, and a name-only
   dispatch would wrongly return Mm for ANY object named x/y. Scoping the Mm values
   to this one Point object keeps every other object's `x`/`y` honestly 0.
3. `jni_call_static_object_method` (slot 114, previously the dead NULL stub): the
   one statically-reached object return is `getScreenPhysicalSizeInMillimeters` ->
   the viewport Point; every other static-object call keeps the honest 0.
4. `jni_get_int_field`: when `obj == viewport_point_handle()`, serve `x`->338,
   `y`->190 (and 0 for any other Point field); the existing DisplayMetrics/etc.
   name dispatch is unchanged for every non-Point object.

## The hermetic (pins the REAL JNI dispatch chain end-to-end)

`viewport_point_mm_surface_resolves_through_dispatch`: GetStaticMethodID ->
CallStaticObjectMethod (slot 114, a real returned Point, stable identical to
`viewport_point_handle()`) -> GetFieldID x/y/z -> GetIntField (slot 100) on the
Point = 338/190/0. CRITICAL SCOPING asserts: a generic fake object's read of the
SAME x/y field IDs returns 0 (never Mm) — proving the name-only collapse is
prevented. An unrecognized static-object method returns NULL/0. Hermetic, no
image, no env, parallel-safe — it resolves the real slot thunks from the fn table.

## Honest

- NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root
  [0x106a68818]=0 structural per SH462/467). recon-v3 immediate-priority
  deliverables re-verified GREEN at this HEAD first (capture_taskv4_frame.sh
  attempt 1: 24 real task-driven frames `present swap Ok(0x1)`, 197 node pops,
  0 json abort, 0 crash, EXIT 124 = stable idle).
- BUILD-THE-RUNTIME session-content completion: when a live session advances and
  the engine's UI requests its physical size in mm, it gets 338x190 instead of a
  0 collapse. Latent-but-correct like every axis. No re-treads (SH469 covered the
  DisplayMetrics/Configuration INSTANCE-object+field chain; this is the STATIC
  object -> Point -> field chain, the one remaining recon-named display value).
- Production code only in jni.rs (off the 1MiB hooks; jit.rs/elfjit.rs untouched).

## Files / commit

- crates/arm64jit/src/jni.rs (viewport Point + static-object + int-field dispatch
  + hermetic). Commit (SH476).