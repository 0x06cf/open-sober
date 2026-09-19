# SH469 — DisplayMetrics/Configuration session-content field surface (recon-framework-boot-order): wire GetIntField/GetFloatField/GetLongField + the getResources→getDisplayMetrics/getConfiguration object chain so the engine self-constructs a live-sized login/home

Single-agent (cone suppressed). recon-v3 deliverable re-verified GREEN at this
fresh HEAD first (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames,
196 node pops, 0 json abort, 0 crash; artifact runs/sh60-taskv4-frame.txt on
disk). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 679->680
incl. 1 new sh469 hermetic; cargo build --workspace + --example elfjit OK).

## The gap closed

recon-framework-boot-order.md names the engine's DISPLAY/CONFIG session-content
reads as the substrate it lays its OWN UI out over:

- `Activity.getResources()` -> `Resources.getDisplayMetrics()` (DisplayMetrics:
  `density=1.0, scaledDensity=1.0, xdpi=96, ydpi=96, densityDpi=160,
  widthPixels=1280, heightPixels=720`) and `getConfiguration()` (Configuration:
  `screenWidthDp=1280, screenHeightDp=720, orientation=2` LANDSCAPE) +
  `getLocales()`.

Measured headlessly BEFORE SH469 this whole surface collapsed to 0/NULL:
- GetIntField (JNI slot 100) and GetObjectField/GetBooleanField were routed to
  `jni_field_0` (always 0); GetLongField (101) and **GetFloatField (102) were not
  serviced at all** (fell to the voidp default), so `density`/`xdpi`/`ydpi`
  returned 0 and widthPixels/heightPixels/orientation returned 0.
- CallObjectMethod did not resolve the object chain: `getResources` /
  `getDisplayMetrics` / `getConfiguration` / `getLocales` returned NULL (0), so
  the engine could never even obtain the DisplayMetrics/Configuration objects to
  read fields from them.

The result: any self-constructed login/home the engine builds would be laid out
at density=0 / 0x0 dims / orientation=0 — a collapsed, unusable screen. This is
exactly the BUILD-THE-RUNTIME surface the operator's SEP-18/SEP-17 directives
name (provide the session substrate so the ENGINE constructs its GuiObjects over
real geometry, host does zero layout).

## What was added

- slot consts `GET_LONG_FIELD=101`, `GET_FLOAT_FIELD=102` (authoritative NDK
  offsets).
- `jni_get_int_field` (GetIntField 100): field-name dispatch over the recon's
  real geometry — `widthPixels=1280, heightPixels=720, densityDpi=160,
  screenWidthDp=1280, screenHeightDp=720, orientation=2` (LANDSCAPE).
- `jni_get_long_field` (GetLongField 101): honest 0 (no known long display field).
- `jni_get_float_field` (GetFloatField 102): a **HostJniF32 s0-return bridge**
  (same transport as the SH134 getDpiScale CallFloatMethod), returning
  `density=1.0, scaledDensity=1.0, xdpi=96, ydpi=96`.
- CallObjectMethod now resolves the 4 object chain getters
  (`getResources/getDisplayMetrics/getConfiguration/getLocales`) to a
  `new_fake_object()` handle instead of NULL, so the chain
  `getResources()->getDisplayMetrics()->[density field]` and
  `->getConfiguration()->[orientation]` survives end-to-end.
- wired the three slots in build_jni (GetIntField/GetLongField/GetFloatField),
  replacing the GetIntField field0 default and the NULL GetFloatField default.

Because GetFieldID routes to `jni_get_method_id` (which interns the FIELD NAME as
a readable handle), the field getters dispatch on the same `method_id_name` path
as every other getter — no new ABI, symmetric with the existing params
string/boolean/int/long/float getter table.

## New hermetic (parallel-safe, no image/env)

`display_config_field_surface_roundtrip` pins: (1) the three field-getter slots
are serviced (not NULL; GetFloatField uses the s0-return bridge, not
CallObjectMethod); (2) the object-chain getters resolve to a fake object
(chain survives); (3) GetIntField field-name dispatch returns the exact recon
geometry (1280/720/160/1280/720/2) and unknown field -> 0; (4) GetFloatField
via the registered thunk returns density 1.0f32 (s0 return); (5) GetLongField ->
honest 0.

## Honest

NOT a DM / NOT a live-DM seed (Route-B live-DM gate UNCHANGED; DM-root
[0x106a68818]=0x0 structural per SH462/467). BUILD-THE-RUNTIME session-content
completion: the engine, when a completed session drives its UI layer, now reads
real display geometry (density 1.0, 1280x720, LANDSCAPE) instead of a 0/0/0
collapse — the substrate its OWN GuiObjects lay out over. Latent-but-correct
exactly like every axis: it only matters once a live DM advances the UI
construction, and it is the pre-condition for that UI to be correctly sized when
it does. No re-treads (no prior SH covered the field-getter path; SH134 covered
the getDpiScale METHOD, this covers the DisplayMetrics/Configuration FIELD reads
over the object chain). Test-only + production surface; recon-v3 deliverable
re-verified green.

Files: docs/frontier-sh469-displayconfig-field-surface.md + crates/arm64jit/
src/jni.rs (production field surface + 1 hermetic). Commit (SH469).