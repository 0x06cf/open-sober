# Frontier SH411 — promote the SEP-17 dataModel-bindings live binder into the ordered session substrate as a first-class driven runtime step

Date: 2026-09-19, hermes-worker, single-agent (cone suppressed). recon-v3
immediate-priority deliverables re-verified green at this HEAD first:
`runs/capture_taskv4_frame.sh` attempt 1 = **24 real task-driven frames
`present swap Ok(0x1)`, 195 node pops, 0 json abort, 0 crash**; the JSON
len-clamp (JIT_JSON_ZERO_FIX @0x102355d40) present. Workspace green (arm64jit
lib 458/0 incl. the new sh411; `cargo test --workspace` 638/0 canonical green).
Production code only in session.rs (off the 1MiB hooks); jit.rs/elfjit.rs
untouched.

## Why this cycle

The SEP-18 BUILD-THE-RUNTIME substrate (`session::drive_routeb_session_substrate`,
SH400) drives the 16-atom ordered `ROUTEB_SESSION_SUBSTRATE` and, since SH409/410,
the host-driven NativeHelper lifecycle + login-vs-home gate. But the SEP-17
SESSION-CTOR directive explicitly names the **"dataModel-bindings live binder"**
as a component to drive, and the substrate only ever drove the MessageBus
SUBSCRIBE half (`0x102ba5bb8` atom). The RECEIVE half — `publishRaw 0x102334684`,
whose cb (file 0x2bd7444/0x2bd76e8) reads the DataModelBindings DM holder
`[x0+16] @0x102bd7474` — was reachable ONLY via the opt-in `--v2boot-session-pub`
probe rungs (SH347/SH364), never as a step of the ordered runtime. That is the
closest unblocked, on-path, non-re-tread runtime piece.

## What landed

- `session::drive_data_model_binder(iimg,ib,tpidr,boot_sp,env,thiz)` — a thin
  first-class driver wrapping the existing serialized `jit::drive_messagebus_publish_receive`
  (publishes an experience-launch event through `publishRaw` on the SAME ladder
  thread, SH55/64 single-jit_run discipline).
- Wired into `drive_routeb_session_substrate` immediately after the
  `MessageBus.subscribe` atom (guest 0x102ba5bb8) — the exact order a real host
  subscribes then publishes. Now the receive half is part of the ordered runtime,
  not a probe rung.
- New hermetic `sh411_data_model_binder_is_first_class_substrate_step` —
  compile-pins the drive signature, asserts the MessageBus trigger atom stays in
  the substrate table (so the wiring can't silently disconnect), and brokers the
  SessionHandles the drive builds. No real binary needed.

## MEASURED (real libroblox.so, SH400 capture env + JIT_ROUTEB_BUSRECV, EXIT 124 / 0 crash)

```
[session-drive] [16/16] MessageBus.subscribe(experience-launch) @ guest 0x102ba5bb8
[elfjit:v2boot-pub-real] MessageBus.publishRaw returned Ok(0x3e8)
[session-drive] dataModel-bindings live binder: publishRaw experience-launch -> 0x3e8
[session-drive] substrate complete: 11/16 atoms returned non-zero Ok
```

The publish-RECEIVE half now drives inside the ordered substrate and returns
Ok(0x3e8) cleanly; the run stays stable (EXIT 124, 0 SIGSEGV/SIGABRT).

## Honest

The busrecv cb-entry holder guard (JIT_ROUTEB_BUSRECV @0x102bd7474) did NOT fire
on this run — i.e. publish DISPATCHED cleanly but the experience-launch cb body
did not enter on this env, consistent with SH364 (cb entry is live-DM-gated; the
receive-side DM holder `[DataModelBindings+16]` stays 0 until a completed do-init
populates it). This is NOT a DataModel manufacture — the binder is the runtime's
first-class RECEIVE surface; the holder it reads is still live-DM-side. Route-B
live-DM structural gate UNCHANGED (DM-root 0, MH_GAME_LOADED false).

## Files

- `crates/arm64jit/src/session.rs`: +`drive_data_model_binder` +`drive_native_app_start`
  + wiring + hermetic (off-hook; jit.rs/elfjit.rs untouched).
- Real-binary log: /tmp/sh411-verify.txt, /tmp/sh411b-verify.txt (outside repo).

## SH411b extension (same cycle)

The SEP-17 SESSION-CTOR directive names TWO components that were only reachable
via probe rungs, not the ordered runtime: the **dataModel-bindings live binder**
(above) and **nativeAppBridgeAppStart** (V1, guest 0x102338510, SH336/SH337 ABI).
SH411b promotes the latter as a first-class post-substrate step
(`session::drive_native_app_start`), driven immediately after MessageBus.subscribe
(the post-bus order SH337 proved gives the app-start walk a populated registry).
Kept OUT of the substrate table (the sh399 `abi_slots<=5` hermetic is load-bearing;
V1 needs x2..x7 = abi_slots 7), driven as a peer of the binder instead. The sh411
hermetic now pins both drive signatures.

MEASURED (real libroblox.so, SH400 env, EXIT 124 / 0 crash):
```
[session-drive] nativeAppBridgeAppStart (V1 0x102338510) with populated registry @ entry
[session-drive] nativeAppBridgeAppStart returned Ok(0x3e8); registry=12 DM-root=0x0
```
The V1 app-start walk executes inside the ordered substrate with a populated
registry (12), confirming the SH337 post-bus order composes in the runtime. Honest:
app-start REGISTERS services (registry 0->12 observable), it does NOT construct a
DataModel (DM-root 0). Route-B live-DM structural gate UNCHANGED. Workspace green
(arm64jit lib 458/0, `cargo test --workspace` 638/0). session.rs only.

Do-not-re-tread unchanged: no LSM skips, no map manufacture, no
setDataModelToCurrent, no single-object DM seeds.