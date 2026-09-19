# Frontier SH347 — MEASURE the messageBus experience-launch RECEIVE half headlessly (publishRaw drives clean; the cb DM-holder is never read: SH185's static-only closure is now a measured result, not a judgment)

Status: `dev`, single-agent (cone suppressed). New jit.rs guard `routeb_busrecv_holder_guard`
(opt-in `JIT_ROUTEB_BUSRECV=1`, READ-ONLY, fires once) + new `drive_messagebus_publish_receive`
pub fn + elfjit rung `--v2boot-session-pub`. +2 hermetic tests. Workspace green (arm64jit lib
416/0; elfjit examples/build pass). elfjit.rs 1,049,451 B; jit.rs 905,716 B (both < 1MB hook).
Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* all false).

## Why this cycle (the SEP-17 RECEIVE entry never driven)

The messageBus experience-launch **receive** path (subscribe 0x102ba5bb8 -> publishRaw
0x102334684 -> cb file 0x2bd7444/0x2bd76e8) is the ONE SEP-17 named receive entry that SH185
closed **by STATIC judgment only** (elfjit.rs:7565 literally: "SH185 closed by STATIC judgment
only"). SH185's static premise was "the subscribe never registers (inside gated init)". But
SH269/315/316/337 subsequently **measured** that subscribe DOES execute headlessly (Ok(0x3e8),
service registry 0->12, once-guard latches) — directly overturning SH185's premise. SH264
explicitly left "messageBus experience-launch receive" as the un-drive honest-next-candidate.
Every prior cycle measured the SUBSCRIBE/publish side or the SERVICE-REGISTRY; nobody ever drove
the **publish -> cb -> [DataModelBindings+16] readback** to see what the receive half does
headlessly. This cycle converts SH185's static-only closure into a measured result.

## What landed

1. `routeb_busrecv_holder_guard` (jit.rs, env `JIT_ROUTEB_BUSRECV=1`): READ-ONLY, fires once at
   guest pc 0x102bd7474 — the cb's DM-holder read (`ldr x0,[x0,#16]`, file 0x2bd7474) — and
   snapshots `[DataModelBindings+16]` (x0 base + 16). No guest write. Default-inert.
2. `drive_messagebus_publish_receive` (jit.rs, pub): drives `publishRaw` (0x102334684,
   Java_..._MessageBus_publishRaw) with topic "experience-launch" + empty payload. ABI
   x0=env x1=thiz x2=topic x3=payload; single jit_run (SH55/64-serialized).
3. elfjit rung `--v2boot-session-pub`: after the existing `--v2boot-session-bus` subscribe
   (SH269/315/337 measured Ok(0x3e8)), drives publish, then reads + prints DM-root.
4. +2 hermetic tests (`busrecv_guard_inert_without_env`, `busrecv_guard_fires_at_cb_reads_holder`).

## MEASURED (real libroblox.so, runs/capture_sh347_busrecv.sh, 1 clean bounded run)

```
MessageBus.subscribe returned Ok(0x3e8)          <- subscribe half (already measured, re-confirmed)
[elfjit:v2boot-pub] MessageBus.publishRaw returned Ok(0x3e8)   <- publish RECEIVE half drives CLEAN
[elfjit:v2boot-pub] publishRaw -> 0x3e8
[elfjit:v2boot-pub] post-publish DM-root[0x106a68818]=0x0
after MessageBus.publishRaw: [0x106829ea8] = 0x0
SH122: MH_FLAGS_LOADED=false MH_ENGINE_INITIALIZED=false MH_APP_READY=false once-guard=0x1
SH155: DM-root=0x0 once-slot=0x400000b vt+0x30=0x0 mark_b(liveDM)=false registry-count=12
       app-data-model-count[0x106dca000+0xe88]=0x1
EXIT 124 (stable idle loop), 0 SIGSEGV/SIGABRT
```

Key measured facts (do-not-over-claim):
- **`MessageBus.publishRaw` (the receive entry) now EXECUTES headlessly and returns Ok(0x3e8)**
  — a genuine first (SH185 never ran it; elfjit.rs:7565 confirms it was static-only). The RECEIVE
  half of the SEP-17 messageBus line is driveable-clean, not a crash / not "unreachable".
- **The cb DM-holder guard at 0x102bd7474 NEVER fired** => publishRaw does NOT dispatch to the
  engine's experience-launch cb that reads [DataModelBindings+16]. No cb entry, so no DM-holder
  read, so result is: publish -> bus dispatches benignly (returns Ok) but does not reach the
  dataModel-bindings construction cb on this route. `[DataModelBindings+16]` is never read.
- DM-root stays 0, MH_* stay false, registry stays 12 (task-scheduler family), app-data-model
  counter 0 -> 1 (the send-appevent path already touched it, SH307-class). No live DM.

## Honest conclusion

This is a MEASURED negative that upgrades SH185 from "static judgment" to "measured result":
the messageBus **receive** half (publishRaw) is confirmed driveable-clean but does NOT fire the
experience-launch cb / read the DM holder headlessly — consistent with SH185's (static) closure,
now byte-measured. It also closes SH264's "still un-driven" honest-next-candidate with evidence:
the entry now RUNS, it just doesn't reach SceneGraph construction, matching every other SEP-17
receive (all measured-latent behind the live-DM gate). Route-B live-DM structural gate UNCHANGED;
SH174 capture-latch stays the single forward hook. The publish entry being driveable-clean (not a
crash) is a small new forward surface for a future receive-payload-with-real-string probe, but it
does not manufacture a DataModel.

## Verify
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (arm64jit lib 416/0 incl.
  2 new SH347 tests; elfjit examples pass).
- Repro: `bash runs/capture_sh347_busrecv.sh /tmp/sh347-recv.txt` -> EXIT 124, subscribe Ok(0x3e8),
  publishRaw Ok(0x3e8), no routeb-busrecv line, DM-root 0. Log /tmp/sh347-recv.txt (gitignored).
- elfjit.rs 1,049,451 B, jit.rs 905,716 B — both under the 1MB pre-commit hook.
- Commit: local `dev` only (operator pushes).

Single-agent, default-inert (guard env-gated + opt-in rung), no production path edited.