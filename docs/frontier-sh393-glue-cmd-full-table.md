# Frontier SH393 — FULL app-command dispatcher table drive on the SH366 entry (15/20 safe-command map, cmd 1/13/15/17/18 measured-unsafe); a per-command safety classification of the engine's own process_cmd

Date: Sep 21, 2026, hermes-worker. Single-agent (cone suppressed). New opt-in rung
`--v2boot-glue-cmd-full` -> `drive_glue_process_cmd_full` (jit.rs) + real-image hermetic
`sh393_glue_cmd_full_table_bound_pinned` (arm64jit lib 446) + capture
`runs/capture_sh393_glue_full.sh`. Workspace green (cargo test --workspace EXIT 0; arm64jit lib
446/0; jit.rs 1,046,770 B <1MiB hook; elfjit.rs 1,048,492 B <1MiB hook — the elfjit example was
4 bytes over the hook mid-cycle and was pulled back under by condensing two SH-prose comments,
facts/addresses preserved).

## Why this cycle

Recon-v3 immediate-priority deliverables re-verified green at this exact HEAD first
(self-driven frame + JSON fix + session-gated producer inertness):
- `capture_taskv4_frame.sh` (SH391 guard armed): confirmed-green attempt 1, **24 real task-driven
  frames `present #N swap Ok(0x1)`**, 196 node pops, 0 json abort, 0 SIGSEGV/ABRT, EXIT 124.
- `capture_sh304_session_producer.sh`: session-gated producer correctly INERT (0 GATED, 0
  fabricated frames, lone present #0 = render-plane warmup, EXIT 124).
- JIT_JSON_ZERO_FIX present at 0x102355d40.

The genuinely-open, non-re-tread forward on the operator's SESSION-CTOR lever: SH366 first
entered the engine's real app-command dispatcher `process_cmd` (0x102bcd6e4) and SH368 drove a
verified-safe {6,8,11} subset. The operator's directive is to "drive the engine's REAL
Activity-session init state machine" — a real Activity consumes the FULL command queue in
lifecycle order. Nobody had ever driven the complete jump table or classified all 20 cases.
SH393 does both.

## The drive (default-inert JIT_ROUTEB_* env reuse, opt-in --v2boot-glue-cmd-full)

Reuses the confirmed-green SH366/368 entry: fabricated shared app/inner/win, version-gate
0x10683d8b0 kept 0, once-guard OFF, each command on its OWN fresh CpuState jit_run, session
observables read back after every command (marker [inner+9], once-guard, AppBridgeV2 0x106a705e8,
surface XID 0x10683d348, flags-loaded 0x10672739d4, DM class-registry 0x106dca0e70).

## MEASURED + disasm-classified per-command safety map (real libroblox.so)

process_cmd dispatches cmd in [1..20] via a 16-bit rel offset table (0x69408a) to case
0x2bcd730 + rel*4. Every case begins `adrp x8,683d000; ldr x0,[x8,#2224]` (version gate
0x683d8d0) then `and w8,w0,#0xff; cmp w8,#0x6; b.cc <target>`. The `b.cc` TARGET decides safety at
version-gate byte0<6, and SH393 measured + pinned it:

- **SAFE (15): drive cleanly.**
  - b.cc->shared epilogue 0x2bcdbf0 / map direct to epilogue: cmds **3, 5, 6, 9, 10, 14, 16, 19, 20**.
  - b.cc->glue/telemetry-only body that returns without a live-object deref: cmds **2, 4, 7, 8, 11
    (INIT_WINDOW)** + 12. MEASURED: all 15 `process_cmd returned Ok(0x0)`, cmd 11 sets the
    INIT_WINDOW marker [inner+9]=1 and returns the win pointer, cmd 19/20 return Ok.
- **UNSAFE (5) — measured the runtime refuses them headlessly:**
  - **cmd 1** (case 0x2bcd9fc): PRE-GATE live-object deref `ldr x8,[x20,#24]; ldr x9,[x8,#56]`
    before ANY version-gate check — measured SIGSEGV fault=0x38 on the zeroed fabricated app
    (SH366/367 live-object class: needs a real Activity-built window).
  - **cmd 13** (0x2bcdc14): gate-checks then b.cc->body 0x2bcdc60 which `ldr x8,[x20,#24]` @0x2bcdc74
    and `stp x0,x22,[x8,#32]` — measured SIGSEGV fault=0x20 (guestpc 0x102bcdc74). This PROVES the
    gate-check alone does NOT imply safety; the b.cc target does.
  - **cmd 15/17/18** (0x2bcdce8/0x2bcda80/0x2bcdc84): gate-checked but b.cc->live-object-deref
    bodies ([x20+24]/[x20+8] chains) — same measured-unsafe class as cmd 13.

The drive drives the 15 safe commands in a lifecycle-first order
{6,8,11,2,3,4,5,7,9,10,12,14,16,19,20} and EXCLUDES the 5 unsafe (recording the exclusion in the
header). **MEASURED (confirm:1 on attempt 4; attempts 1-3 drove all 15 Ok + `SH393 done` then hit
the documented SH353-class run-variable persistence-lane wall AFTERWARD, unrelated to the drive):**
```
[elfjit:glue-full] cmd 11: process_cmd returned Ok(...)   marker[inner+9]=1  (INIT_WINDOW)
[elfjit:glue-full] SH393 done: 15/15 commands returned Ok, 0 stopped;
   AppBridgeV2 selftransition 0x0->0; surface XID 0x200000->2097152; flags=0x0 dmreg=0x0
```
EXIT 124, 0 SIGSEGV/ABRT in the drive region, cmd-11 marker fires, once-guard stays OFF. The
session observables re-confirm SH366/368's reading with the FULL table: driving the engine's own
command dispatcher does NOT self-transition AppBridgeV2 ([0x106a705e8] 0x0->0) or the surface XID
(0x200000 — those move only when a live session/do-init builds the DM world; DM-root
[0x106a68818]=0, MH_* false unchanged).

## What is genuinely new + honest

- First FULL 20-entry classification of the engine's own process_cmd jump table by MEASUREMENT
  (not just inference): 15 safe / 5 unsafe, with the exact case addresses and the b.cc-target
  safety rule. This is the substantive SESSION-CTOR advance — it turns SH368's 3-command subset
  into a complete, per-command, re-verifiable safe map of the real command queue.
- The cmd-1/13 rule (gate-check alone ≠ safety; the b.cc target does) is a genuine disasm+machine
  fact that corrects any "gate-checked => safe" assumption.
- Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0).
  SH174 capture-latch stays the single forward observer.

## Do-not-re-tread (unchanged closures, all still stand)

LSM skips (SH349/350/358/373), EC reader-gate (SH355/356/374), 0x258b5d8/SetInitParams
(SH362/375), window-attach real (SH367), ALooper (SH365), governor gates full-ladder (SH379),
-9 string (SH380), map-header (SH248h), once-lambda store (SH381), SH267 node-cell (SH385),
setDataModelToCurrent (SH388), LSM-manufactured wiring (SH385). SH393 ADDS: do NOT drive cmd
1/13/15/17/18 expecting a headless return — they are measured to deref unconstructed live-object
data past/before the gate (SH366/367 class).

## Files

- crates/arm64jit/src/jit.rs: +`drive_glue_process_cmd_full` (opt-in --v2boot-glue-cmd-full,
  default-inert, ZERO change to the SH366/367/368 clean drive); +hermetic
  `sh393_glue_cmd_full_table_bound_pinned` (arm64jit lib 445->446).
- crates/arm64jit/examples/elfjit.rs: +rung wiring (--v2boot-glue-cmd-full) + 2 condensed SH-prose
  comments to hold elfjit.rs under the 1MiB hook (facts/addresses preserved).
- runs/capture_sh393_glue_full.sh (repro; log runs/sh393-glue-full.txt outside repo).
- Commit: local dev only (operator pushes).

## Verify

- `cargo test -p arm64jit --lib sh393` = 1 passed.
- `cargo test --workspace` EXIT 0 (arm64jit lib 446/0).
- Real-binary confirmed-green: runs/capture_sh393_glue_full.sh -> RESULT=confirm:1 (15/15 Ok,
  cmd-11 marker, EXIT 124). Recon-v3 deliverables re-verified green at this HEAD (24 frames swap
  Ok(0x1), producer inert, JSON fix present).