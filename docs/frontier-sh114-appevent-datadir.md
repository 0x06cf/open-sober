# SH114 — SendAppEventOnAppReady rung + data-dir getters; scoped-blr empirically rejected

## Outcome
Two clean, tested advances from disasm-backed read-only subagents (deleg_0d4e5597,
deleg_83e74525, deleg_f84d9f90). Workspace 50 suites / ~475 tests green; real
libroblox.so combined v2boot+render ladder EXIT 0, 0 crash, real task-frame present
swap Ok(0x1).

## 1. New `--v2boot-send-appevent` rung (the missing Java→engine bridge response)
Root cause of the session-advance wall (deleg_0d4e5597): the NativeHelper
`gameActivity_*` milestones are SET-ONLY — the jni.rs shim stores host AtomicU8
atoms (MH_FLAGS_LOADED/APP_READY/...) that NOTHING in guest memory polls. The
missing Java→engine response is `nativeAppBridgeV2SendAppEventOnAppReady`
(guest 0x102bb463c): it parses the first 4 bytes of the event-name jstring (x2)
vs 4-byte magics ("Home" LE 0x656d6f48 → w19=4), builds a 0x50 app-event struct,
dispatches through the shared app-bridge pipe 0x102baeeec into the
LuaAppExperienceController → NativeHelper/AppShell → DataModel path that would
construct the shell UI.

New opt-in `--v2boot-send-appevent` drives it as a SEQUENTIAL ladder rung after
`--v2boot-surface-handoff`, on the same thread (avoids the SH44/49 block-cache
SIGSEGV of a shim-internal nested jit_run). ABI (6-arg JNI): x0=env, x1=thiz,
x2=event jstring, x3/x4/x5=readable jstrings (new_string_utf_handle). Gotcha:
each trailing jstring must be non-NULL (GetStringUTFChars resolves them).
VERIFIED (real binary): the rung runs clean, logs `driving SendAppEventOnAppReady
@ guest 0x102bb463c (event="Home")`, then soft-returns at the same singleton-
latent leak pc as setTaskSchedulerBM/V2Init/V2Start (the too-short 0x60 vtable) —
so its BODY still does not complete. It remains the correct harness action for
when the singleton dispatch is resolved; wiring it as a ladder rung (not a
shim-internal drive) is the desync-safe form.

## 2. Data-dir getters (persistence objective 2b)
deleg_f84d9f90 (disasm): the REAL client never hardcodes its data dir; it gets it
via Java Context.getFilesDir()/getCacheDir()/getDatabasePath() (rodata 0x244d48)
→ nativeSetFilesDirectory (guest 0x1021f7654) → global std::string 0x1026d600.
The engine's OWN SQLite datastore (rbx-storage.db, NOT session.db) is the real
"remember sign-in" plane. Before this, the getter registry returned 0, so the
engine never built a base path and nothing reached the fsmap store.

Fix: added getFilesDir/getCacheDir/getFilesDirectory/getCacheDirectory/
getDatabasePath to `auto_value_string_getter` (crates/arm64jit/src/jni.rs),
returning guest-absolute dir jstrings under /data/user/0/com.roblox.client/ so the
engine's own .db opens route through fsmap onto persistent host disk.
NEW regression `sh114_context_data_dir_getters_resolve_via_fn_table` (arm64jit 342/0):
each resolves a readable jstring of the exact expected length through the official
CALL_OBJECT_METHOD slot. NOTE (honest): the getters did NOT fire in the combined
run — StartLuaAppDM's session soft-returns before building the app data model, so
the engine does not yet call them. The fix closes a real hole but stays latent
until the session advances; it is boot-safe (getter additions only, no path
change) and test-covered.

## 3. Scoped-blr patch EMPIRICALLY REJECTED (do not re-attempt with 0/xzr)
The SH111 next-gate suggestion to scoped-patch the 3 lazy-singleton dispatch sites
(0x62517c4/0x6251aa8/0x6260948) was implemented as `blr x8 → mov x0,xzr` and TESTED
on the real binary: the patch applied cleanly (all 3 sites) but nativeInitialize-
NativeFlags then SIGSEGVs with fault=0x28 (NULL+0x28 deref). Root cause (matches
deleg_83e74525's warning): the virtual's return IS deref'd by SOME caller (nativeInit
reads [out]+0x28), so returning 0/xzr reproduces the exact SH110 crash class that
vtable widening hit. A scoped leaf MUST return a stable zeroed guest object
(routeb_singleton_obj_leaf), NOT 0. Left as the next gate with this precise
constraint. The baseline 0x60 vtable soft-return remains benign (ladder EXIT 0).

## Repro / verification
- Unit: `cargo test --workspace` → 50 suites, 0 failures (arm64jit 342 incl. sh114 getter test).
- Real binary clean baseline (EXIT 0, 0 crash, persist byte-exact, present #0 swap Ok(0x1),
  app-event rung soft-returns benignly, [0x106829ea8] stays 0):
  `timeout 60 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=4000 JIT_ROUTEB_HASHFIX=1`
  `./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4`
  `--jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent`
  `--renderinit 0x105b3a280 --renderthunk --renderframe --persist-roundtrip --kicker 0x106863af8`
  Log: runs/sh114-appevent.txt (no scoped-blr flag → clean). Reverted scoped-blr; doc and
  constraint retained in code comment.