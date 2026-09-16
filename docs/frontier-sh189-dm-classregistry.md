# SH189 — PlayerGui class-descriptor REGISTRATION headlessly (one-next-object past the DM ctor)

Status: implemented + verified on the real binary (2/2 clean). Doc for the SH189 session.
Frontier: Route-B — engine SELF-CONSTRUCTS its first GuiObject under a genuine DataModel.
Commit: dfa68e0 (SH189 PlayerGui) / 4513237 (SH189b ScreenGui). Repro: runs/capture_sh189_dm_services.sh, log runs/sh189-dm-services.txt.

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

## SH190 ERRATUM (same session, empirical — do NOT re-tread): forcing the derived PlayerGui vptr did NOT land.
After SH190 observed the instance-base construction, I tried the obvious next lever: patch the PlayerGui
ctor's sub-init 0x255d2f4 tail `b 0x23768e8` (at 0x255d334, opcode 0x17f8656d) to `ret` (0xd65f03c0) so the
derive body returns to 0x255d204 and writes the PlayerGui-class vptr 0x106648950 at 0x255d21c. EMPIRICAL:
the patch FIRED ("PATCHED sub-init tail ... -> RET", 0 crashes, EXIT 124) but the post-drive object STILL
holds vptr 0x106796dc0 (instance-ctor 0x2374310). Root-cause: the derive body's 0x255d21c write is NOT
reached even with the tail ret — the post-drive object is re-initialized by getOrCreate's instance-ctor
write path (order: PlayerGui derive write attempted but the object's settled vptr is the instance-base one).
The block at 0x10255d204 is ALSO not a fresh block-entry, so a block-entry probe there never fires — do NOT
reuse that as a reachability signal. VERDICT: getOrCreate constructs at the instance-base layer; the full
PlayerGui-class vptr layer is order-dependent and NOT forceable by this single tail patch. The honest next
gate for a REAL full PlayerGui is the class-keyed create path with the derive-body executing (needs the
app-shell/live-DM vt-dispatched ctor), NOT this micro-branch. Kept default-inert observation (SH190) as the
deliverable; the tail-RET patch was REVERTED (cruft). Do NOT re-tread 0x255d334/0x10255d204.

## SH190 addendum (same session, committed): INSTANCE OBJECT OBSERVED — the ctor-entry capture confirms a real, vtable'd engine object constructs headlessly.
The SH189c residual ("instance allocated but not yet OBSERVED") is CLOSED. The ret/out-buffer walk
was a RED HERRING: the pair-consumer's `ret x0` points into a string ("Invalid da...") and the
out-buffer stays {0,0} (shared_ptr attach skipped). The authoritative observation is the ctor-entry
capture: `routeb_dm_instance_ctor_capture` fires at the PlayerGui ctor block entry 0x10255d1dc
(x0==the op-new'd object) inside the nested jit_run, snapshots x0 + its vptr, and the guard reads
the object's post-drive layout. EMPIRICAL (real libroblox.so, 3/3, EXIT 124, 0 crash):
```
[routeb-dmins] ctor-entry pc=0x10255d1dc obj=0x7f..599950 vptr-at-entry=0x0
[routeb-dmins] PlayerGui pair-consumer 0x10255d0e4 DROVE ok ... ctor-obj=0x7f..599950
  entry-vptr=0x0 post-vptr=0x106796dc0 => obj vptr=0x106796dc0
  obj [+0x0]=0x106796dc0 [+0x8]=0 [+0x10]=0 [+0x18]=0x106dc0c58 [+0x20]=0
       [+0x28]=0 [+0x30]=0x100000000 [+0x38]=0
  engine constructed a real INSTANCE-BASE object (vptr 0x106796dc0 = instance-ctor 0x2374310)
```
0x106796dc0 is in-image (filevma 0x6796dc0) and equals the instance-ctor 0x2374310's
relocated vtable (`adrp x8,0x6796000; add #0xdc0; str x8,[x19]` @0x2374368) — the FIRST genuinely
OBSERVED engine self-constructed instance object on Route B. The derived PlayerGui-class vptr
0x106648950 is NOT yet applied headlessly: the ctor's sub-init chain (bl 0x255d2f4 -> getter
0x201fce0 -> tail 0x23768e8) mounts the instance BASE; the derived vptr write at 0x255d21c
(`adrp x8,0x6648000; add #0x950`) doesn't land under the shared_ptr-skip path. NEXT (closest
unblocked): drive the derived layer so [obj+0] becomes 0x106648950 — i.e. force the ctor body past
the instance-base mount to its own vptr write (inspect why bl 0x255d2f4 returns to a different
vptr-set path, or NOP/force the branch so 0x255d21c executes) — then a REAL PlayerGui instances
self-constructs, clearing the derived-class layer before service-node attach ([dm+0x68]) + scene
scan (R+0x180/0x188). CODE: default-inert (JIT_ROUTEB_DM_INSTANCE), +wiring routeb_dm_instance_ctor_capture
in the block-entry dispatch + entry-vptr snapshot in the guard. Workspace green.

## SH189c addendum (same session, committed): REAL PLAYERGUI INSTANCE CONSTRUCTION reaches completion headlessly — the migration gate MOVED. Core creator ServiceProvider::getOrCreate 0x102373458 (class-manager resolve at 0x23736dc -> operator-new 0x1d96768 -> blr ctor-functor 0x255d1b4 -> insert). Pair-consumers 0x10255d0e4 (PlayerGui) / 0x10247a88c (ScreenGui). Real NON-virtual ctors 0x255d1dc (PlayerGui vptr 0x106648950) / 0x247a984 (ScreenGui vptr 0x106649ce0); instance-ctor 0x2374310 (vptr 0x106796dc0). Key gates: (a) creator's current-DM global guest 0x107333948 (`adrp x8,0x7333000; add #0x948; ldar x0,[x8]` @0x23737d4-0x23737dc -> bl 0x21daef8), DIFFERENT from the loop's *0x106391908; (b) x23 = instance-ctor's incoming x1 = the owner sourced from the pair-consumer's x0 (`str x0,[sp,16]; add x5,sp,#0x10` -> functor `ldr x1,[x1]`) -> so run the consumer with x0=dm.
CODE: `run_guest_callback_x8` (adds x8 out-reg; args array only covers x0-7) + `routeb_dm_instance_guard` (env JIT_ROUTEB_DM_INSTANCE, StartLuaAppDM-scoped, OnceLock): plant *(0x107333948)=dm, drive pair-consumer 0x10255d0e4 with x0=dm + x8=&out. +1 hermetic test (377/0).
EMPIRICAL (real binary, capture_sh189c_instance.sh, 2/2 EXIT 124, 0 crash): `planted DM ... into creator current-DM global 0x107333948` + `PlayerGui pair-consumer 0x10255d0e4 DROVE ok ret x0=0x7f9ffa567361` — the REAL PlayerGui instance ctor chain now EXECUTES TO COMPLETION headlessly (previously the x23=0 null-deref EXIT 134; passing x0=dm fixed the owner). HONEST: out={0x0,0x0}, obj vptr=0 — the drive returns Ok but does NOT yet surface the constructed instance through the out-buffer (its shared-ptr out path isn't populated on this return branch), so the instance object isn't yet OBSERVED; the ctor chain completing is the boundary reached. The lone EXIT-139/nativeInitialize crash was the SH55/64 clone-worker flake (before this guard). Strand: the instance is allocated (ret x0 host ptr) but we don't yet read it back — next: walk the operator-new'd object for the PlayerGui vptr 0x106648950 to CONFIRM self-construction.
STANDING: this is the furthest Route-B instance-construction has reached headlessly. The PlayerGui/ScreenGui service NODE on [dm+0x68] + scene-scan attach (R+0x180/0x188) remain the outermost gate, but they now sit BEHIND a reachable, completing ctor chain rather than an unreachable vt-dispatched factory. Do NOT conflate with the migration gate — the instance ctor is now demonstrably driveable; only the observed-instance + scene-attach layers remain.

## SH189b addendum (same session, committed): SCREENGUI CLASS-DESCRIPTOR REGISTRATION too.
Corrected ABI (recon deleg_5c489b38): the ScreenGui register BODY 0x10201f4f0 null-derefs
headless (guestpc 0x101db7e38 `str x0,[x22,#8]` at the class-member builder, source=0) — it must
be driven via its CALLER GETTER **0x10201f42c** (parameterless; latch 0x106c980a28 + nested
source-builder guard 0x106c96868), exactly the PlayerGui pattern. StarterGui getter 0x102020120
is NOT all-zero-safe (forwards caller x0->source into the register helper; needs a real source).
VERIFIED (real binary, 2/2 clean EXIT 124, 0 crash): ScreenGui getter 0x10201f42c DROVE ok
ret x0=0x106c98a40 — **recon's desc addr 0x106c980a40 is 0x2000 LOW; the real desc object is the
RETURNED addr 0x106c98a40**, whose vtable-family slot [0x106c98a40+0x230]=**0x106649c98** (exact
recon-expected ScreenGui value) and desc vtable [0x106c98a40]=0x1067a6150 (shared
DescribedCreatable descriptor vtable, same base for PlayerGui+ScreenGui). Both PlayerGui (0x87e)
and ScreenGui (0x1b87) class descriptors are now registered headlessly in the engine's own
global class-name registry. StarterGui left un-driven (ABI needs a real source descriptor; its
source-builder not isolated — documented follow-up, low ROI while it's a descriptor-only gain).