# Frontier SH344c — CORRECTION to SH344b: the app-shell ctor band [0x102207b50..0x102209000] is NOT a stable negative on the SH343-deepened full ladder (3/3 runs fire ~78 pcs, terminal at the SH285 reader wall); and the DMCONT continuation's serialize body trips into the LSM lane (`bl 0x1d9d8b0`) and faults at 0x101db1b08 BEFORE reaching the app-start `bl` — so SH245 lever #2 (seed F+0x18) is not the effective next gate

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Measurement-only
(no production code edited). Workspace green (cargo test --workspace exit 0, 594 passed).

## Why this cycle
SH344b (committed 22:36) reported the app-shell ctor band [0x102207b50..0x102209000]
as a "stable negative (0 hits)" on the SH343 full ladder, and named SH165-fwd
"Next" = the +0x1f0 flag-completion re-router "deeper toward a real engine-constructed
app-shell". Two things needed checking before re-deriving a seed from that framing:
(a) was the 0-hit readout a stable negative or a divergent single sample, and
(b) does the DMCONT continuation actually reach the post-app-start F+0x18 deref
(0x2bd2080) that SH245 lever #2 would unblock, or does it fault earlier.

## Measured (real libroblox.so, SH343 full-ladder env + KEYFIX, 3 runs each)

### 1. The app-shell band is RUN-VARIABLE POSITIVE, not 0-hit stable
3/3 runs (repro runs/sh344c_appshell_ab.sh) fire the app-shell ctor band ~78 distinct
pcs over [0x102207b50..0x102208eac]; every run terminates at the SH285 reader/pop
live-object wall 0x101db1b08 (fault 0xffffffffffffffff) — NOT at the activity-lifecycle
route SH344b reported. A driver-level A/B mid-cycle also showed the 1/3-activity-lifecycle
divergence (guestpc 0x10284cf5c) on one sample: i.e. the band is BOTH fireable deep AND
sample-divergent — SH344b's "0 hits stable negative" was the divergence arm, not the
stable class. The band entry 0x102207b50 is the recon-sh165fwd `__cxa_guard` one-time-init
(adrp 0x6a640d70 / AcqRel guard read / tbz) and the deep pcs 0x208e88/0x208eac are a
destructor-style container loop following a `bl 0x208e84` — the FastLog/exception-log
warmers, NOT a DM/UI builder. So even when it fires, it warms logging; nobody constructs
GuiObjects or a DataModel from this band. This CORRECTS SH344b and SH340's framing: the
band fires on the full ladder too, but remains the recon-sh165fwd warmer class.

### 2. The DMCONT continuation faults at SH285 BEFORE reaching app-start — F+0x18 is not the gate
Region-watch on the continuation body [0x102bd1d68..0x102bd2040] + post-app-start tail
[0x102bd2050..0x102bd2160] (3 runs, runs/sh344c_req*tail.txt):
- Body runs end-to-end: 0x102bd1d68 -> serialize (1dfc..1f5c) -> app-name guard re-seed
  0x102bd1f64 -> 0x102bd2014 (last body hit). ~24 pcs, the whole flags-serialize.
- Post-app-start tail [0x2050..0x2160]: **0 hits** — the continuation NEVER reaches the
  `bl 0x2bd2058` (app-start call, -9015652). Instead, after the serialized app-name/str2
  fields it dives into `bl 0x1d9d8b0` (initStorageManagerNative, the LSM lane) and the run
  terminates at guestpc=0x101db1b08 (SH285 reader/pop live-object wall, fault ff..ff).
- Consequence: SH245 lever #2 (seed a valid controller at F+0x18 so the 0x2bd2080
  `*(*(F+0x18)+16)` post-app-start deref survives) is NOT the effective next gate — the
  continuation dies one hop EARLIER in LocalStorageManager init. SH285's verdict stands:
  0x101db1b08 is a live-object-class wall (cause-not-symptom), do NOT repair-seed it.

## Honest conclusion
- The app-shell band firing (78 pcs) on the full ladder is a real forward-observation but
  it is the FastLog warmer (recon-sh165fwd), not self-constructed UI; no DM-root, MH_* false.
- The DMCONT continuation does NOT complete to app-start; it caps at the SH285 LSM reader
  wall one hop before the bl app-start. So the "flag-completion re-router deeper toward
  app-shell" (frontier-sh344 Next) is gated by the SH285 wall, and SH245 lever #2 (F+0x18)
  would not change that (it targets the wrong, deeper fencepost).
- Route-B live-DM structural gate UNCHANGED. SH174 capture-latch stays the single forward
  hook. Persistence lane (SH343) remains committed and green.

## Files
- Repro: runs/sh344c_appshell_ab.sh (3-run A/B, app-shell band + terminal per run).
- (Live captures sh344c-req*.txt gitignored.)
- No production code changed; elfjit.rs unchanged (1,048,479 B). Workspace green.

## Next (honest, unchanged despite the correction)
The forward is still the SEP-17 SESSION-CTOR cause-level drive (real Activity/AppBridge
session so the upstream ctor constructs the DM world for real). The SH285 reader wall is
NOT seedable; the only lever is cause-level session construction or the un-driven
messageBus experience-launch / dataModel-bindings receive entries that SH264 named as the
next candidates on the session-ctor line. The DMCONT line is measured-complete through its
serialize body and capped at SH285 — no product of the +0x1f0 flag-completion slot changes
that.