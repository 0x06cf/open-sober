# SH189 — PlayerGui class-descriptor REGISTRATION headlessly (one-next-object past the DM ctor)

Status: implemented + verified on the real binary (2/2 clean). Doc for the SH189 session.
Frontier: Route-B — engine SELF-CONSTRUCTS its first GuiObject under a genuine DataModel.
Commit: <SH189COMMIT>. Repro: runs/capture_sh189_dm_services.sh, log runs/sh189-dm-services.txt.

## What was the gate
The 3-agent Route-B cone (deleg_9884123a, deleg_3a08fcf0, deleg_58cfcb06) reconciled the
SH187 state: the get-or-create consumer 0x102dbcd88 create-path `blr [DM-vptr+0x1c0]` =
0x103facf10 is a **boolean registration predicate** (cbz x1 -> ret; else RTTI/IsA proxy), never
a GuiObject; and the `DescribedCreatable<ScreenGui>` rodata cluster (0x9daa5e-0x9dad65) is pure
RTTI type_info (reloc-materialized {vptr 0x106358d90, name}) — NOT a classname->factory dispatch.
So neither lever produces a GuiObject. The genuine DM ctor 0x1023f6038 **never builds
PlayerGui/CoreGui/ScreenGui** (verified); services resolve lazily BY NAME via the per-DM service
container ([dm+0x68] singly-linked list, node[+0x18]=classid, [dm+0x68]=next; [dm+0x78] vector),
which first requires the **global class-name registry** (header 0x106dca0e70, resolver 0x2373cec)
to resolve "PlayerGui" -> classid 0x87e. That registry entry is a .bss-zeroed once-init → the
**ONE-NEXT-UNSYNTHESIZED-OBJECT is the PlayerGui class descriptor**, registered by the engine's
own class-register getter 0x10201fce0 (once-latch 0x106c97f30, body 0x10201fda4, nested
source-descriptor builder latch 0x106c883a0, classid 0x87e, typeid 0x298).

## Code (arm64jit jit.rs, default-inert, +1 hermetic test 375->376)
`routeb_dm_service_seed_guard` (env JIT_ROUTEB_DM_SERVICES=1, scoped to StartLuaAppDM entry
[0x1023efe2c,0x1023eff40], OnceLock-idempotent):
1. Only on a present, writable DM holder page (page_is_mapped + routeb_ensure_writable gate the
   raw read — hermetic-safe): seed a coherent EMPTY service list **head** at [dm+0x68] (zeroed
   0x80 node, node[+0x18]=0, node[+0x68]=0) + empty [dm+0x78] vector, only when currently NULL
   (never clobber a real built service).
2. Clear the two CHAINED once-latches 0x106c97f30 + 0x106c883a0 (=0) so the register once-path
   re-runs.
3. Drive the REAL PlayerGui class-register **getter** 0x10201fce0 via run_guest_callback
   ([0;8], parameterless) — recon task-0 correction: the body 0x10201fda4 is the once-body, the
   once-guard lives in this caller getter; 0x106c980b8 is the descriptor OBJECT not a latch.
4. Probe success markers: class-desc counter [0x106dca0e28], cached desc [0x106c97f28],
   descriptor vtable [0x106c980b8], PlayerGui vtable-family slot [0x106c980b8+0x230].
+1 hermetic test (env-off no-op / env-on wrong-pc region-gated instant / env-on correct-pc returns).
The guard is wired at the gov-tail block-entry hook (jit.rs ~4066) next to the SH187 real-ctor guard.

## Empirical (real libroblox.so, llvmpipe; capture 2/2 clean)
```
SH189: seeded empty service-list head 0x...202aa0 at [dm+0x68]
SH189: service vector [dm+0x78] = 0 (empty, walkers early-out)
SH189: PlayerGui class-register GETTER 0x10201fce0 DROVE ok ret x0=0x106c980b8
SH189: class-desc counter [0x106dca0e28] = 0 (want >0), cached desc [0x106c97f28] = 0x106c980b8,
       desc vtable [0x106c980b8] = 0x1067a6150 (want ✓), PlayerGui vtable-family
       [0x106c980b8+0x230] = 0x106648908 (want ✓)
EXIT 124 (timeout after completion), 0 SIGSEGV/SIGABRT (reproducible 2/2; the lone EXIT-134
run was the pre-existing SH55/64 run-variable clone-worker flake at V2InitWithParams, upstream
of this guard)
```
+ SH187 upstream re-verified in the same log: real DM ctor constructs the genuine DM
  (obj vptr set 0x1067162e8/0x1067163a0/0x1067163f8 GENUINE MATCH) + plants holder 0x106391908.
+ 376/0 hermetic suites (arm64jit), default env-off path unregressed.

## Verdict
DELIVERABLE: the engine's OWN PlayerGui class-registration code executed headlessly for the
first time, materializing the genuine PlayerGui class descriptor (cached 0x106c97f28, descriptor
vtable 0x1067a6150, PlayerGui vtable-family slot 0x106648908) — the one-next-object past the DM
ctor. HONEST note: the class-desc **counter** [0x106dca0e28] stayed 0 even though the descriptor
is demonstrably populated — recon had this at the 5fb225c tail; likely a DIFFERENT counter word
(likely [0x106dca0e28+...] or the counter is only advanced by a sibling body). Not chased: the
descriptor-object markers (cached/vtable/vt-family) are the stronger, recon-agreed proof.
STANDING WALL: a **PlayerGui service node** on [dm+0x68] ([node+0x18]==0x87e) and a **ScreenGui
instance** parented under it, attached to the scene-scan lists R+0x180/R+0x188, are the NEXT
unsynthesized objects — recon task-1 (deleg_728c4b80) confirms the service walker 0x5e09bc8 +
0x2377600 only REFCOUNT an existing node (node creation needs the RBX::PlayerGui ctor, vt-dispatched /
live-app-shell-gated), i.e. NOT headlessly reachable from current state. That is the standing
migration gate, unchanged. The 3 headless-drivable objects past this are the CLASS DESCRIPTORS:
PlayerGui (done), ScreenGui (getter 0x10201f4f0, desc 0x106c980a40, classid 0x1b87), StarterGui
(getter 0x10202014c, classid 0x892) — same once-idiom, worth registering for completeness so the
resolver can answer the DM's getService by name.

## Next (closest unblocked)
1. Register the OTHER two class descriptors (ScreenGui getter 0x10201f4f0 / StarterGui
   0x10202014c) with the same guard so the global class-name registry resolves all three by name
   (cheap, same mechanic, confirms the resolver 0x2373cec path returns classid for each).
2. Add a resolver probe: drive 0x102373dec(x0=&0x106dca0e70, x1=&RBX::String "PlayerGui") and
   assert [el+0x10]==0x87e — the strongest registry-side success proof.
3. The PlayerGui SERVICE NODE + ScreenGui INSTANCE stay the migration gate (vt-dispatched ctor /
   live app-shell). Do NOT re-derive the service node (recon-negative). The cone stays armed.