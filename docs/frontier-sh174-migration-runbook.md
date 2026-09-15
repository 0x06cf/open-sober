# Migration-Harness Runbook — capture a live RBX::DataModel and arm DataModelServices

Status: PLANNED (recon-authoritative, code-grounded). Target host: the GPU-capable /
real-input host the operator migrates to for the REAL app-launch session. On the
headless VPS this runbook cannot fire (the live-DM make_shared is unreachable,
~29 recon angles agree: migration gate). It exists so the capture+arm is a cold-followable
procedure the instant a real session advances.

Author: hermes-worker SHXXX (recon deleg_d5ba0bc8, task-2; READ-ONLY, nothing run).
All guest addrs are file_vaddr + 0x100000000.
Source ground truth: crates/arm64jit/src/jit.rs (`routeb_dm_alloc_capture` /
`routeb_dm_alloc_capture_guard`), docs/frontier-sh167-dm-alloc-capture.md,
docs/frontier-sh169-dm-capture-delegating.md, docs/frontier-sh172-dmservices-arming-corrected.md,
docs/frontier-sh170-ladder-verify-serial-pitfall.md.

## GOAL
Catch the pointer the instant the engine's own inlined make_shared<DataModel> runs
(ExperienceController::join -> submitStartGameTask) during a REAL app-launch session,
validate it, write it into the current-DM holder in-process, and confirm the first
engine-self-constructed GuiObject scene node appears at the render-manager R+0x180/0x188.

## STEP A — ENV MATRIX (host of the real session)
- JIT_DM_ALLOC_CAPTURE=1        — master switch. Without it the guard returns at once
  (jit.rs:1163); every other var is inert. Arms the SH167 latch at pc 0x102a0d9b8.
- JIT_DM_ALLOC_CAPTURE_DELEGATE=1 — delegation opt-in. The real binary ships its OWN
  nonzero ACTIVE allocator hook; plain latch never clobbers a live hook. With DELEGATE=1
  the engine's hook is SAVED into PREV_DM_ALLOC_HOOK and the trail performs the real
  allocation by calling THROUGH the JIT to the engine's own hook
  (run_guest_callback(prev_hook,[a0..a7], current_guest_tp())), so the returned base is
  from the engine's pool (free-path stays valid). Use BOTH together.
- JIT_DRIVE_LIFECYCLE=1          — canonical ladder companion (session can form).
- Canonical ladder companions: JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1.
- DO NOT SET JIT_SERIALIZE_RENDER on this ladder — SH170 pitfall: on a BARE ladder it
  raises WORKER_ADMISSION_GATE, which parks a clone worker the do-init depends on ->
  UNHANDLED guest C++ exception, EXIT 139, every run. Serialization is for the COMBINED
  render+ladder run only.

Command:
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
      --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent

ABI reminder: routeb_dm_alloc_capture ABI is probe-verified (a0=size, a1=call-site-tag
code/rodata addr, a2=flag). Use a0, NEVER max(a0,a1) — a1 can be a huge code/rodata addr
and over-allocating allocs multi-GB garbage (SH167 draft bug).

## STEP B — SEE THE LATCH ARM
stderr must show (one line):
  [routeb-dmalloc] SH167/SH169 routed CRT operator-new ACTIVE hook 0x1067daaf0
      -> capture trail <T> (prev_hook <ENGINE_HOOK>, default_hook <D>) at pc=0x102a0d9b8
      -> all operator-new blr the capture trail (latent until a real session make_shared<DataModel>)
This is the ONLY proof the ACTIVE allocator-hook global 0x1067daaf0 now points at the
host trail and every operator-new routes through it.

## STEP C — WAIT FOR A LIVE DM
On the headless ladder the trail stays SILENT (that silence is itself the proof of the
migration gate). With real display + real input driving the app-launch, the engine's
make_shared<DataModel> runs and the trail logs the gold line:
  [routeb-dmalloc] ... bytes=0x... -> base 0x<B> [validated] (delegate prev_hook 0x<ENGINE>)
Record B. Only a real session's make_shared produces it.

## STEP D — VALIDATE B INDEPENDENTLY
bytes=B; first word at B must be an in-image vtable (>= 0x100000000, within image bounds).
base_ok = read_vt_in_image(B). If true, the object is a REAL vtable'd object (a genuine DM).

## STEP E — ARM IN-PROCESS (post-migration, never a static seed)
Write B into guest 0x106391908 (the current-DM holder returned by the getter
setDataModelToCurrent @ 0x2dbcc10: adrp x0,6391000 / add x0,x0,#0x908 / ret).
- DO NOT write 0x106391918 (std::function inline capture word) — the registry dispatch
  (0x2dbcd18) receives the DM as an INVOCATION ARG (x1 = stack temp, blr x21), not from
  +0x18 (SH172 category-error pushed into a static seed would be wrong).
- MUST run in-process post-migration: the invokable __func vt (0x106358d40) and __f_
  (0x6391900) are loader-runtime-built under packed ANDROID_RELA (all-zero in file);
  hand-clobbering a relocated vtable as a static seed violates the project rule.
- Registry truth-table: current-DM NULL -> benign no-op stubs; != NULL -> real dispatch.
  Letting NULL->real transition happen is the arming event.

## STEP F — ACCEPTANCE PROBE (engine self-construction)
- CMDLINE MUST NOT include --renderscene; host statics RENDERSCENE_BASE/SCENE_NODES must
  read 0 (render_scene_base never ran). Their absence means a nonzero scene list could
  ONLY come from the engine.
- Find the real RenderManager R. Read head=*(R+0x180), tail=*(R+0x188). n=(tail-head)/0x28.
- n>0 with the host statics all 0 = FIRST ENGINE-SELF-CONSTRUCTED GuiObject SCENE NODE.
- Per node: node+0x08 render-obj vt in-image and vt[+64] a live dims-query; node+0x18
  (after construction) = nonzero frame-desc with [+144]==0x1.
- Record log excerpts + B + n to a new docs/frontier-*.md. Keep dumps small; rm from /tmp
  (7.7G tmpfs, never >100MB).

## RUN-ON-GPU-HOST CHECKLIST
[] branch dev, tree unmodified (README-ONLY at recon time).
[] ~/.cache/open-sober/robbox/libroblox.so present.
[] env EXACTLY as STEP A; JIT_SERIALIZE_RENDER ABSENT.
[] real display (EGL/GLES or llvmpipe) AND a real input device — NativeHelper
   gameActivity_* callbacks are engine->Java post-conditions, NOT drivers (jni.rs:731-786);
   nothing polls MH_* into a session gate headlessly. Without real input the session gate
   cannot form.
[] run canonical command; stderr to a log.
[] GOLD-1: arm marker (ACTIVE hook 0x1067daaf0 -> capture trail). Absent -> env not exported.
[] drive app into submitStartGameTask (ExperienceController::join).
[] GOLD-2: -> base 0x<B> [validated]. IF only FIRST lines without [validated], keep
   supplying input; do NOT conclude yet.
[] verify B first word = in-image vtable.
[] arm *(0x106391908)=B in-process; verify read-back == B; refuse 0x106391918.
[] probe R+0x180/0x188: statics 0 + n>0 -> first engine-self-constructed scene node.
[] 0 SIGSEGV/0 SIGABRT, no EXIT 134/139, swap Ok(0x1).
[] save evidence doc; clean /tmp.

## HARD STOPS
- Do not re-tread .bss/.data DM seeds (proven dead, ~28 recon angles).
- Do not run the elfjit harness on a recon host; the main loop owns it.
- The capture latch and the respondent code are default-inert (env off -> byte-identical).