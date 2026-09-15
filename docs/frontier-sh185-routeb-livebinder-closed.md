# SH185 — Route-B closure CONFIRMED at the live-binder level (last theoretical crack =
# migration-gated) + type-4 self-sustaining loop SOUNDNESS audit (pre-hook NULL, 3 flood exits).

Date: Sep 15, 2026, hermes-worker. Workspace green (374/0 + all crates) at HEAD 1940eae.
No production edit — this cycle PERSISTS the 2 fresh cone results that close the last
open Route-B angle and confirm the recon-v3 self-drive loop is safe as-is. Commit <SH185COMMIT>.

## Cone (deleg_67e9118b, 2 READ-ONLY agents, both authoritative)

### 1. DataModel live-binder handlers = MIGRATION-GATE, not seedable (task-0, decisive-positive-closure)
- `dataModelBindings_onGameLoaded` / `dataModelLifeCycle_onAppLuaWillStart` are LIVE JNI→AppBridge
  events, NOT messageBus subscribers. `onGameLoaded` = export
  `Java_com_roblox_engine_jni_NativeGLInterface_nativeAppBridgeV2SendAppEventOnGameLoaded` @
  **0x2bb429c** (0x3a0 bytes): marshals JNI args via GetStringUTFChars (bl 0x21e1fec x3), builds an
  AppEvent, dispatches @ **0x2baeeec** which pulls the AppBridge global (GOT 683d000+8) and calls
  0x2206c40/0x275a0c4. Needs a real JVM JNIEnv + activity lifecycle; NO subscription table to seed.
- Sole genuine messageBus path = 'experience-launch request' (strings @ 0x47de3c/0x595956) with
  cb **0x102bd76e8** (file 0x2bd76e8). Subscribe is registered (MessageBus subscribe export @
  **0x2ba5bb8**) INSIDE the migration-gated `initializeLuaApp_` (log 'listen for experience-launch
  request' @ 0x59594b). Callback payload reads **DM at `[DataModelBindings+16]`** (`0x2bd7704 ldr x0,[x20,#16]; cbz -> 0x2bd7858 cleanup`); if non-null it runs the FULL DM construction (getDataModel,
  AppShell reporter, pass-key/scene, bridge handoffs, vtable-dispatch teardown) — i.e. it DOES drive
  SceneGraph construction, but the DM must ALREADY sit in `[DataModelBindings+16]`, and only the gated
  bootstrap (initEngine_->bootstrap->DM load) produces it. Headless: subscribe never registers
  (inside gated init), and a null/forged [this+16] hits cbz->cleanup, construct nothing.
- VERDICT: MIGRATION-GATE. 'Manufacturing a DM handoff' = manufacturing the DM, which is exactly the
  gated work. This closes the ONE plausible headless crack SH184 left open: no live-binder subscriber
  is reachable, the sole messageBus handler needs a DM only the gated bootstrap makes.

### 2. type-4 self-sustaining frame loop = SAFE AS-IS (task-1, positive audit)
- Engine producer 0x10285682c pre-hook `this->[24]`: `0x2856848 ldr x8,[this,#24]; 0x285685c cbz x8`
  -> NULL = skip veto, push unconditional. **No store to object-field [24]/[32] anywhere in the
  engine/drain region 0x2850000–0x2860000** (only [x29,#24] stack spills + one node-ctor [32] write
  @0x2858af4, not the owner) => pre-hook is NULL zero-init at headless boot => host-built nodes push,
  NOT dropped. (Harness may defensively zero owner->[24] before seeding — cbz-guarded, deterministic —
  but it is already NULL, so no change strictly required.)
- Deque tagged-CAS format confirmed: stored ptr = `(node & ~7)` low 48 bits, ABA tag high 16 bits
  (8-byte aligned, canonical <2^48). Owner this = `frame->[112] & ~0x3f`; this[8]=per-CPU array base;
  slot = base+(cpu&0xf)*0x4a140; TAIL slot +0x18 (flag [node+32]&1==0, SH60 path) writes node->[0]=low48
  (old tail) then CAS(tail,…); HEAD slot +0x10 (flag bit0==1) allocs head-cell via 0x102852c24, links
  node at [cell+8], head-tag CAS. So the SH60-style host node ([node+32]&~1 as x3, [node+40]|=1
  dispatchable, [node+112]=0x106829f00) pushes correctly on the tail path; producer only writes
  node->[0] (intrusive link), so the host node needs a writable +0 word.
- Flood guard reachable from presenter (3 independent exits): drain pop-loop 0x102856e40 fires
  controller lifecycle cbs via `dq->[112]&~3f -> [40]` w4=2 (0x2856f34 pre-pop), w4=3 (0x2856f78
  post-pop), w4=4 (0x2856ffc pre-repush) inside the loop => a HALT/capture gate armed in
  controller->[40] is invoked from the presenter every cycle. Re-push gated on dispatchable [node+40]:
  non-zero -> engine producer re-push (0x2857020); zero -> node freed (0x285704c, no re-queue). Plus
  producer [24] veto. All terminate the drain; pop reads head while re-push writes tail (FIFO) => no
  recursive same-node re-entry, no unbounded growth. The self-sustaining loop either self-sustains
  one-frame-at-a-time or drains to empty. SOUND.

## STANDING
- Route-B live DM = MIGRATION GATE at FOUR stacked levels now (full-lifecycle SH184-1, holder-no-
  consumer SH184-2, live-binder SH185-1, and the SH183 vtable-no-producer). Headless Route-B crack is
  CLOSED at the strongest evidentiary depth the loop has ever reached. Do NOT re-dispatch
  holder-*0x106391908, DMCONT, or live-binder-subscriber cones (their negatives are authoritative).
- recon-v3 self-drive loop verified SOUND as-is (task-1) + re-verified end-to-end (24 frames/196 pops)
  at SH184. No production hardening required (pre-hook already NULL).
- NEXT (honest): the ONLY headless-forward line left is real 3D output of the engine's OWN authored
  scenes (objectives a/specialized-runtime) via the working GLES bridge + type-4 self-drive — i.e.
  Route-A content (scenes the harness drives through the engine's own renderer), which the operator
  said NOT to polish at Route-B's expense but Route-B is now closed headlessly; the live-DM session is
  a migration (GPU-host/real-input) item with the SH174 capture latch as the validated observer.
  Any new Route-B push must first locate a genuine live-DM construction event that is NOT one of the
  four closed gates — else it is strictly migration work.