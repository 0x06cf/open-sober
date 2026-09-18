# Frontier SH342 — measured: the SETWORLDBUILD world-build continuation is unreachable from every driven rung (gate never fires) + V2_ONDEMAND full-ladder crash site changed

## Session
Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Measurement-only (region-watch,
no production seed added). Workspace green at HEAD (SH341): build ok, tests 411/56/30/22/26/...,
0 failures; recon-v3 frame plane re-verified green this cycle (real mesh, 0 crashes, EXIT 124).

## Question
STATUS candidate #2 (SH341-refined) put the LSM pool-pop 0x626b6d0 wrapper's caller forward, and the
SEP-17 SESSION-CTOR lever named the do-init->nativeAppBridgeAppStart__ continuation. The SH198-surface
lever (`JIT_ROUTEB_SETWORLDBUILD`, seeds [0x106a70568]=1 so the V2InitWithParams rung falls through its
gate block at 0x102368100 to `bl 0x102ea3b14` = nativeAppBridgeAppStart__ world-build) is the concrete
gate for that continuation. This cycle measures whether any driven rung actually reaches the world-build
fn (0x102ea3b14 calls operator-new(0x18) + ctor 0x2eacce4 + nativeAppBridgeAppStart__ 0x2365960).

## MEASURED (real libroblox.so, SH307-forward deep reach)
Two ladders, both with JIT_ROUTEB_SETWORLDBUILD=1 + region-watch on the world-build body
[0x102ea3b14,0x102ea3bf0) and nativeAppBridgeAppStart__ [0x102338510,0x102338700):

- **Send-appevent reach** (--v2boot-send-appevent, the SH310-completing path): `routeb-worldbuild` gate
  fired **0 times** — the SETWORLDBUILD seed at gate block 0x102368100 NEVER lands, because the
  V2InitWithParams rung benign-soft-returns `Ok(0x3e8)` from entry 0x102365c54 WITHOUT executing its
  gate block. World-build fn 0 hits, nativeAppBridgeAppStart__ 0 hits. (Ditto with JIT_ROUTEB_V2_ONDEMAND=1.)
- **Full ladder** (--v2boot-skip-appstart, WITHOUT send-appevent) WITH JIT_ROUTEB_V2_ONDEMAND=1:
  dies at a NEW crash site `guestpc=0x1026d63c0 fault=0x0` (SIGABRT after) — never reaches the
  world-build gate either. 0 world-build / 0 appstart hits.

## Interpretation (do-not-over-claim)
- The SH198-surface world-build gate's PREMISE is broken, not its seed: the gate block 0x102368100 is
  itself never reached from the driven V2InitWithParams entry (which short-returns Ok(0x3e8) on the
  send-appevent path). Seeding [0x106a70568]=1 at gate-block entry cannot help while the gate block
  does not execute. This is the same V2Init early-soft-return wall SH202 documented (~365-site
  singleton-dispatch family crashes/bounces V2Init before the world-build gate); V2_ONDEMAND does not
  cross it on this run — the run dies at StartLuaAppDM's nested-frame live-object site instead.
- The new full-ladder crash site 0x1026d63c0 is the fn prologue (`stp x29,x30,[sp,#-48]!`) of a
  StartLuaAppDM nested frame reached via `bl 0x2b9ec9c` at 0x26d63bc, whose body `ldr x0,[x0]`
  derefs a null `this` (fault=0x0) — the SH324-class session-ctor live-object wall, NOT a seedable gate.
- Net on this path: **no seedable lever reaches the nativeAppBridgeAppStart__ world-build continuation.**
  It is gated behind the live-DM/session-ctor wall (SH324/SH340). SH342 answers STATUS candidate #2's
  open question: the world-build gate is unreachable, not merely un-triggered. No production edit;
  DM-root 0, MH_* false, Route-B live-DM gate UNCHANGED. SH174 capture-latch stays the single forward hook.

## Verify / files
- Recon-v3 frame plane re-verified at HEAD this cycle (capture_taskv4_frame_mesh.sh: real mesh via
  engine wrapper, 0 SIGSEGV/SIGABRT, EXIT 124).
- Probe: runs/capture_sh342_worldbuild.sh (send-appevent + SETWORLDBUILD + V2_ONDEMAND, region-watch
  on world-build/appstart bands). Captures gitignored per CLAUDE.md.
- No production code changed; workspace green.
- Commit: local `dev` only (operator pushes). Doc + probe only.