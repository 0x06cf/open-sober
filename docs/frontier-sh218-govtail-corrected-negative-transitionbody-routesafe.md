# SH218 — Governor-tail CONTINUATION negative at the CORRECTED-SH161b state + SH161b transition-body purpose pinned (bypass proven Route-B-safe) + manufacture/recon-v3 unregressed at HEAD

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed per operator
Sep-15). Date Sep 16 2026. Workspace green (cargo test --workspace EXIT 0; arm64jit
387/0 + all crates, 0 failures). Commit `9c44697`. Repro
runs/capture_sh218_postsh161b_continuation.sh.

## WHY THIS CYCLE

SH217 (c88c7df) corrected the SH161b mode-2 seed window to the REAL governor-tail
block entry `[0x102e9fcc4,0x102e9fdc8]` — fixing a defect where the seed fired ZERO
times, leaving fn 0x1023c12c0's fault-prone transition body LIVE on every completing
ladder. **This is a genuinely new run-state: the transition body is now actually
bypassed (was never truly bypassed in any prior measurement).** All earlier
governor-tail negatives (SH197/204/209: "tail ends at the 0x102ea30dc canary-ret,
no deeper construction body reached") were measured with the seed window broken. This
cycle (a) re-measures the continuation under the corrected state, (b) pins DOWN the
purpose of the transition body SH161b skips (was never documented, only assumed
fault-prone), and (c) re-verifies the two unregression surfaces (manufacture lever +
recon-v3 render plane) at the new HEAD.

## RESULT

### (a) The transition body is V2Init audio/telemetry param-marshalling — NOT a Route-B forward edge (NEW, protective)

Fresh disasm of fn 0x1023c12c0 (inlined in `nativeAppBridgeV2InitWithParams`, file
0x23c12c0) with SH161b's b.eq target 0x23c1434 and its 3 dispatch helpers:

- fn entry: `ldr w8,[x0,#696(=impl[+0x2b8])]; cmp w8,w1; b.eq 0x23c1434` — the mode-2
  early-return. Target 0x23c1434 = pure stack-canary compare + `ret` (benign no-op).
- The bypassed body (w8!=w1): reads version-state global `[0x6a70700]` (`adrp x9,6a70000
  + ldr [x9,#1792]`, byte-splits at &0xff/>>8==0x3), then builds small structs via
  0x23c14dc (a jump-table dispatch on `w1<=4`, reads static table addr 0x6f2118),
  0x23c1504 (build-info marshaller, copies from the same version-global), and
  0x23c1574 (a repeated 0x29c-struct copy-builder: `ldrh [x1,#8]; ldr [x1]; ...`),
  plus a conditional `bl 0x626b6d0` = `Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged`
  (the SH212 crash-A audio site).
- **Verdict:** every dispatch in the transition body is V2Init parameter/telemetry/audio
  setup, gated on the engine build-version global `[0x6a70700]`. NO DataModel
  construction, NO world-build, NO scene/R+0x180/0x188 writer. SH161b's switch to the
  benign mode-2 early-return therefore forfeits ONLY V2Init audio-param + telemetry
  marshalling — the operator's Route-B forward edge (do-init→app-shell→governor→DM) is
  not in this body. The bypass is provably Route-B-safe (previously only assumed).

### (b) Governor-tail continuation = FRESH NEGATIVE under the corrected state (run 2, EXIT 124)

Canonical completing ladder (V2_ONDEMAND + full SH210 env) + region-watch on
[0x102e9fa80,0x102ea4000) + world-build [0x102ea3b14,0x102ea3c50) + DM-creators
[0x102bd1a30,0x102bd1d08)+[0x102bd21d4,0x102bd2600):

```
run 2: EXIT=124 sh161b_seedlines=1 tail_pcs=24
       deepest=0x102ea3084 0x102ea30d0 0x102ea30dc
       worldbuild_pcs=0 dmcreator_pcs=0 crashes=0
[routeb-setfix] SH161b impl[+0x2b8] seeded =2 ... -> fn 0x1023c12c0 b.eq (mode-2 no-op)
                taken, transition body bypassed (was 0x0)
[elfjit:v2boot] SH155 post-StartLuaAppDM: once-guard=0x1 DM-root=<SH156 seed>
                liveDM-image=false once-slot=0x400000b once-live=false
```

**SH161b now genuinely fires** (transition body truly bypassed for the first time) and
**the governor tail still terminates at the 0x102ea30dc canary-ret epilogue lodestone**
(the `ldr x19,[sp,#16]` after `bl 21e9610`) — no advance into the world-build body
0x102ea3b14 (which itself terminates in `nativeAppBridgeAppStart__ 0x2365960`, the
already-closed SH174 no-DM-factory line), no DM-creator pc. Route-B live-DM structural
gate RECONFIRMED at the corrected-SH161b state. Runs 1/3/4 crashed at the known
non-seedable FMOD AAudio direct-JNI site (guestpc 0x106240ca0 = fn 0x6240900 region,
the SH213/SH212 crash-A class, NOT on the SH161b path), run 3 = the run-variable
SH55/64 flake class.

### (c) Manufacture lever + recon-v3 plane UNREGRESSED at HEAD

- Manufacture lever (SH187 real-ctor drive): `REAL DM ctor wrapper 0x1023f5ff8 DROVE ok`
  → `obj vptr set = 0x1067162e8,0x1067163a0,0x1067163f8 GENUINE MATCH`, planted into
  holder 0x106391908, EXIT 124, 0 crashes. The one genuine headless DM-construction
  artifact still fires clean at the new HEAD.
- recon-v3 render plane (runs/capture_taskv4_frame.sh): 24 task-driven frames (`present
  #20..#23 swap Ok(0x1)` distinct colors), 195 node pops, no json abort, 0 crash.

## CONCLUSION / STANDING

Route-B live-DM world-build remains the structural gate, now at the newest
corrected-SH161b state (the transition body is finally truly bypassed and STILL nothing
past the governor-tail terminal is reached). This cycle adds the previously-missing
mechanism-level safety confirmation: the body SH161b skips is V2Init audio/telemetry
marshalling only — the bypass cannot cost a Route-B forward edge. The manufacture lever
and recon-v3 plane are clean at HEAD. No production code change (SH217 already landed;
this cycle verifies + documents). The live-DM front and its one forward hook (SH174
capture-latch arming *(0x106391908) at a real make_shared<DataModel>) are unchanged.

## DO-NOT-RE-TREAD (refined)

- Do NOT re-run the governor-tail region-watch across the [0x102e9fa80,0x102ea4000)
  range expecting a different answer — now measured at THREE completed states (SH197/204/209
  broken-window + SH218 corrected-window). The terminal 0x102ea30dc is stable.
- Do NOT re-crack the transition body as a seed (proven audio/telemetry, Route-B-neutral).
- Disasm the world-build fn 0x102ea3b14 ONCE: reads [x0,#688] (app-shell ctor target),
  op-new(0x18) + ctor 0x2eacccc + `bl nativeAppBridgeAppStart__ 0x2365960` → SH174
  closed line (no DM factory), do-not-seed.

## TREE / VERIFY

- New repro: runs/capture_sh218_postsh161b_continuation.sh (bash, N arg, prints
  sh161b-seedlines / deepest tail pcs / worldbuild+dmcreator hits / verdict).
- cargo test --workspace green (arm64jit 387/0 + all crates, 0 failures).
- recon-v3 + manufacture lever re-verified at HEAD (above), no regression.