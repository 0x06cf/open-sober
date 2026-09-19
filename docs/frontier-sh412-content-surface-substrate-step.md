# Frontier SH412 — the G3 content surface as a first-class driven substrate step (files-dir + R1 CoreScript that SH407/408 measured never firing, now wired into the ordered drive)

Date: 2026-09-21, hermes-worker, single-agent (cone suppressed). recon-v3 immediate-priority
deliverables re-verified green at this exact HEAD first (`capture_taskv4_frame.sh` attempt 1 =
24 real task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; JSON
len-clamp present). Workspace green (`cargo test --workspace` EXIT 0; arm64jit lib 459/0 with the
new sh412 hermetic). Production code only in session.rs (off the 1MiB hooks);
jit.rs/elfjit.rs untouched (both at/near the hook, byte-unchanged).

## Why this cycle

The SEP-18 BUILD-THE-RUNTIME ordered substrate (`session::drive_routeb_session_substrate`,
SH400) drove G1 (V2UpdateSurface 0x1025f5fec) + G2 (SendAppEventOnAppReady "Home" in x5 +
host-driven onAppReady lifecycle), but the operator's recon-routeB **G3** — the ENGINE's OWN
files-dir libc++ std::string at **[0x10726d600]** ("the structural content gate that turns a
nonzero probe into rendered home UI") plus the R1 synthetic CoreScript module — was reachable
ONLY via the `--v2boot-set-filesdir` / `--v2boot-r1-stage` elfjit rungs. SH407/408 MEASURED those
rungs NEVER fire on the reaching full-ladder env (the run faults into the LSM persistence lane
before the rung lines execute). So the content half — the very thing that turns a session into
rendered home UI — was staged-but-unwired: present as hermetics (SH351/354) but not a driven
runtime step. This cycle promotes it into the ordered substrate exactly as SH411/411b promoted the
dataModel binder + nativeAppBridgeAppStart.

## What landed

- `session::drive_content_surface()` (new, off-hook): seeds the engine's OWN files-dir libc++
  string at [0x10726d600] = "/data/user/0/com.roblox.client/files" (long-form, read-back-verified,
  wrapped in routeb_ensure_writable so a no-image cell is never SIGSEGV) AND calls
  `jit::stage_r1_core_scripts()` to stage the R1 CoreScript into the fsmap mirror.
- Wired into `drive_routeb_session_substrate` right after the `MessageBus.subscribe` atom
  (0x102ba5bb8), the same post-bus step that drives the binder + app-start — so the CONTENT
  surface is now a first-class driven step of the runtime, not a stranded rung.
- New hermetic `sh412_content_surface_is_first_class_substrate_step` (arm64jit lib 458->459):
  compile-pins the drive signature, asserts the MessageBus trigger atom stays in the substrate
  table, and proves the drive SEEDS the files-dir cell (maps via routeb_ensure_writable even with
  no image, read-back verifies, returns 1) without SIGSEGV.

## MEASURED (real libroblox.so, SH400 capture env, EXIT 124, 0 crash)

```
[session-drive] [16/16] MessageBus.subscribe(experience-launch) @ guest 0x102ba5bb8
[session-drive] nativeAppBridgeAppStart returned Ok(0x3e8); registry=12 DM-root=0x0
[session-drive] SH412 G3 files-dir: seeded libc++ string @ [0x10726d600] = "/data/user/0/com.roblox.client/files" SEEDED
[session-drive] SH412 R1 content surface: staged 2 candidates (STAGED AppShell.lua, STAGED CoreScripts.lua)
[session-drive] substrate complete: 11/16 atoms returned non-zero Ok
```

The substrate completes 11/16 Ok (no regression from the new step); the G3 files-dir gate
SEEDS with verified read-back and the R1 module STAGES — for the first time these fire inside a
driven headless run, not as a never-reached rung. EXIT 124 stable, 0 SIGSEGV/SIGABRT.

## Honest

Not a DataModel manufacture: DM-root [0x106a68818]=0, MH_GAME_LOADED false, AppBridgeV2 stays at
genuine vt 0x1063a3410. The content surface is the repository the engine draws FROM the instant a
completed do-init owns a live DM (latent-but-correct, fires at the right time — exactly the SEP-18
"build the runtime, not the DM" deliverable). The Route-B live-DM structural gate is UNCHANGED.
The `runs/capture_sh412_content_surface.sh` capture script cleans its SOBER_ANDROID_ROOT (the
"STAGED" strings are only returned on a real fs::write; sh351/sh354 already prove the staging +
fsmap serve resolution end-to-end).

## Files

- `crates/arm64jit/src/session.rs`: +`drive_content_surface` + wiring + hermetic sh412 (off-hook).
- `runs/capture_sh412_content_surface.sh`: real-binary capture + verify markers.
- Real-binary log: /tmp/cap_sh412.out (outside repo).

## Next

Keep building the SEP-18 runtime surface. The content gate now fires headlessly; the standing
Route-B live-DM gate (a completed do-init owning a live DM, which would then drive the resolver
to SERVE the staged R1 module) is unchanged and remains the SESSION-CTOR wall. Do-not-re-tread
unchanged (no LSM skips, no map manufacture, no setDataModelToCurrent, no single-object DM seeds).