# Frontier SH364 — the messageBus experience-launch RECEIVE cb-body-entry measurement + real-payload publish probe (closes SH347's "real-string receive probe" open surface)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). recon-v3 immediate-priority
deliverables re-verified green at HEAD (24 real task frames, swap Ok(0x1), dispatch #2638000,
197 node pops, 0 json abort, 0 crash). Workspace green (cargo test --workspace EXIT 0, 0 failures;
arm64jit lib + elfjit example build). elfjit.rs / jit.rs stay under the 1MB pre-commit hook.

## Why this cycle (the last un-driven SEP-17 RECEIVE surface)
SH347 measured the messageBus experience-launch RECEIVE path twice over (subscribe Ok(0x3e8),
publishRaw Ok(0x3e8)) but the cb's DM-holder read (file 0x2bd7474) NEVER fired. That left a
two-way ambiguity SH347 explicitly named as its open forward surface ("a future
receive-payload-with-real-string probe"): either (a) publishRaw never dispatches to the cb at all
(publish-side topic/registration mismatch), or (b) the cb is entered but its DM-holder is null
because [DataModelBindings+16] is a live-DM slot a static seed cannot populate. SH347 only ever
published an EMPTY payload and measured only the holder READ — never the cb body ENTRY. This
cycle closes that with two read-only observations.

## What landed
1. `routeb_busrecv_cb_entry_guard` (jit.rs, env `JIT_ROUTEB_BUSRECV=1`): READ-ONLY, fires ONCE at
   the cb BODY ENTRY (file 0x2bd744c, the `sub sp,#128` prologue the cb's static data is built
   around — BEFORE the DM-holder read at 0x2bd7474). Records x0/x1/x2. Distinguishes "cb never
   entered" (publish-side failure) from "cb entered, holder null" (live-DM-side gate). Wired into
   the same per-block-entry hook as the SH347 holder guard.
2. `drive_messagebus_publish_receive_payload` (jit.rs, pub): parameterizes the publishRaw driver —
   REAL structured payload + optional topic override — while preserving the legacy empty-payload
   wrapper `drive_messagebus_publish_receive` bit-for-bit (refactors it to call the payload form
   with b"" + experience-launch).
3. elfjit rung `--v2boot-session-pub-real`: publishes a realistic
   `{"requestId":"sh364","topic":"experience-launch","payload":{}}` envelope on the same topic
   after the SH347 subscribe rung, single ladder thread (SH55/64).
4. +2 hermetic tests (`busrecv_cb_entry_guard_inert_then_fires`,
   `busrecv_real_payload_driver_handle_shapes`); the existing 2 SH347 busrecv tests retain their
   behavior (re-verified). +capture script `runs/capture_sh364_busrecv_real.sh`.

## MEASURED (real libroblox.so, runbook capture, 1 clean bounded run)
- Subscribe registered: `MessageBus.subscribe returned Ok(0x3e8)` (re-confirmed).
- Real-payload publish bound + drove clean: `MessageBus.publishRaw returned Ok(0x3e8)`
  (payload `{"requestId":"sh364","topic":"experience-launch","payload":{}}`, 56 bytes, topic
  "experience-launch", non-empty — the variant SH347 never tried).
- **`[routeb-busrecv-cbentry] SH364 ... cb BODY ENTERED @0x102bd744c` — 0 fire.**
- **`[routeb-busrecv] SH347 ... @0x102bd7474` — 0 fire (unchanged from the empty-payload run).**
- DM-root [0x106a68818] stays 0x0, MH_FLAGS_LOADED/MH_ENGINE_INITIALIZED/MH_APP_READY all false,
  AppBridgeV2 [0x106a705e8]=0x0. EXIT 124 (stable idle loop), 0 SIGSEGV/SIGABRT.
- Route-B live-DM structural gate UNCHANGED (no DM manufactured).

Full log: /tmp/sh364-recv-real.txt (runbook capture). Not committed (gitignored).

## Conclusion (do-not-over-claim)
The receive cb body entry is measured at 0 fires for BOTH the empty-payload (SH347) and the
real-payload (SH364) publish: publishRaw drives clean (Ok 0x3e8) but does NOT dispatch into the
experience-launch cb body headlessly — the publish-side dispatch is registration/live-DM-gated,
matching the standing Route-B wall. The SH364 cb-entry guard pins the ambiguity SH347 left open
to the publish-side dispatch, not the DM-holder read. It does NOT manufacture a DataModel;
consistent with every SEP-17 receive measured behind the live-DM gate.

## Verify
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (incl. 4 SH347/SH364 busrecv
  tests). Recon-v3 deliverables green at HEAD.
- Repro: `bash runs/capture_sh364_busrecv_real.sh /tmp/sh364-real.txt`.
- Commit: local `dev` only (operator pushes).

Single-agent, default-inert (guards env-gated, rung opt-in), no production path edited.