# SH165 — the governor-tail dispatch now ENTERS real engine-init (DM-force)

Date: Sep 15, 2026, hermes-worker. Workspace green (541/0). Commits: <SH165COMMIT>.

## What shipped
Two real code changes (both default-inert, opt-in):

### 1. `fallocate` aarch64 nr 47 (permanent persistence gap — closed)
The aarch64 guest emits `fallocate` as syscall nr **47** (asm-generic), NOT the x86-64
285. The old wire `285 => SYS_fallocate` never fired under an ARM guest -> a real
`posix_fallocate` (SQLite prealloc) fell through to the catch-all -ENOSYS. Now `47 | 285`
both route to SYS_fallocate. The fsmap_persist integration test now exercises 47 (the
guest number) + keeps the 285 legacy check. (deleg_... task-2, P1 persistence gap.)
+ updated test asserts (the 285 sub-assert's offset now exceeds the file size, so it
actually grows).

### 2. `routeb_dm_force_guard` — governor tail enters REAL engine-init (Route-B forward)
Recon task-0 (authoritative) pinned the real first-login DM engine-init:
- **fnB = guest 0x102bd1b98** (ELF 0x2bd1b98). It has NO benign-tail and NO
  [this+0x10] state dispatch — it unconditionally derefs `[this+0x40]->[+0x18]->[+0x10]`
  then `bl 0x102bd8ce8`. A zeroed shell SIGSEGVs at `ldr x8,[x8,#0x18]` (guest
  0x102bd1c08). The 'Engine settings is null' abort string is DEAD code (0 ADRP refs).
- The governor-tail dispatch (impl[+0x408] then `ldr x8,[x0]; ldr x8,[x8,#48]; blr x8`)
  calls vt[+0x30] with x0=impl. SH161/SH164 kept that slot a **leaf no-op**
  (0x106a72000 inert DISPATCH) -> the governor tail resolved benignly but NEVER entered
  the DM engine-init (SH164 region-watch: 0 hits).
- **FIX `routeb_dm_force_guard` (jit.rs, gated JIT_ROUTEB_DMFORCE=1):** at any governor-tail
  block entry, substitute impl[+0x408] with a fabricated NativeDataModelManager shell
  (`routeb_dm_force_shell`) whose vt[+0x30]=0x102bd1b98 and whose settings chain is
  pre-seeded: [shell+0x40]=P1, [P1+0x18]=P2, [P2+0x10]=P3(=arg to 0x102bd8ce8), and
  [shell+0x18]=valid. Recon (task-0) also showed the 0x102bd8ce8 pipeline reads
  [arg+0x18] as a C-string forwarded to manager vt slots +0xf8/+0x108/+0x1f0, so the
  shell ALSO seeds a real NUL-terminated `"{}"` feature-flag payload into [P3+0x18].
  Idempotent; placed BEFORE routeb_tail_dispatch_guard so DMFORCE overrides the inert
  seed only when set. Default-inert (env off -> only the SETFIX inert DISPATCH runs,
  exactly as before).

**VERIFIED (real libroblox.so, llvmpipe, opt-in JIT_ROUTEB_DMFORCE=1 + the SH161 ladder
env):**
- `[routeb-dmforce] ... fabricated NativeDataModelManager instance 0x7f09... (vt[+0x30]=
  0x102bd1b98 engine-init, settings chain seeded) at pc=0x102e9fcc4 -> real engine-init
  entered` fires.
- **`[region-watch] entered region 0x102bd1a30-0x102bd1d08 at guest pc=0x102bd1b98`**
  (1 hit) — the REAL engine-init fnB now executes, where SH164 measured **0 hits**.
- **EXIT 124 (stable idle), 0 SIGSEGV/SIGABRT, ladder done, SendAppEventOnAppReady Ok.**
- The `dmtrace vt=0` line is the honest read-guard: the shell is a host-heap addr
  (0x7f...) outside the guest image domain, so the capture's store-domain guard zeros
  vt — the shell itself is correctly present in impl[+0x408].

This is the Router-B wall MOVED: the governor tail now dispatches into real
NativeDataModelManager::initEngine_ code instead of a leaf no-op.

## The recon cone (next gate, authoritative)
2 subagents this cycle (leader cone):
- **deleg_62a86bcd task-0:** 0x102bd8ce8 reads [arg+0x18] as a C-string (fwd to manager vt
  +0xf8/+0x108 via the 0x130-frame FFI 0x102bd8dac->vt +0x1f0 = continueAfterFlagsLoaded_),
  then 0x102bd9058 (vt +0x720 check / app-shell init). With a zeroed arg it faults at
  0x102bd8d1c ([arg+0x18]); with the seeded `"{}"` string it advances into the manager vt
  dispatch. **The manager singleton** comes from getter 0x102174c04 -> guest holder
  **0x102727550**, which JNI_OnLoad (boot entry 0x102173ff4) already stlr'd with a host
  JavaVM* -> the getter would return a host JNINativeInterface* and blr vt[+0xf8] into raw
  host JNI. Forward: re-seed 0x102727550 with a fabricated manager M (all-leaf vtable:
  +0x30=write-leaf `str x0,[x1]; mov w0,#0; ret`, +0xf8/+0x108/+0x1f0=leaf, all else 0 so
  vt[+0x720]==0 -> benign soft-return; M+0=vt, M+8=0). RE-SEED IS SCOPED: this global is
  JNI-critical, so seed it ONLY on the fnB entry hook (pc 0x102bd1b98 / the DMFORCE tail
  window), never a blanket clobber — restore or leave the JavaVM* path intact for non-DM
  JNI. vt+0xf8 = the network feature-flag fetch; NO fabricated continueAfterFlagsLoaded_
  string needed (leaves ignore the arg). Do NOT set the manager vt[+0x30] to 0x102bd1b98
  (would recurse); it must be the write-leaf.
- **deleg_42b6f9d6 task-1 (session producer handoff spec):** current wiring lines pinned
  (RENDERCTX elfjit.rs:201, LADDER_DONE elfjit.rs:220/7006/7753, TASKFRAME_HALT
  elfjit.rs:351, type4_frame_thunk elfjit.rs:376-409 installed at elfjit.rs:8089/8098,
  MH_APP_READY jni.rs:742/767-770, scene-node render_scene_base elfjit.rs:447-501,
  walker ABI elfjit.rs:2383-2385). The future self-constructing producer gate (when
  MH_APP_READY ^ LADDER_DONE ^ RENDERCTX live): repoint [0x106829ea8]=0, set TASKFRAME_HALT,
  hand deque to live drain (vtable 0x106829f00 -> handler 0x10285371c), epoch bump
  [Q]+=0x1_0000_0000, FUTEX_WAKE(0x8a) on [Q]+4. Latent behind MH_APP_READY as before.

## Honest residual
- The SH165 shell reaches the engine-init entry but the pipeline then hits the manager
  vtable (getter returns host JavaVM*). NEXT implementable = the scoped 0x102727550
  manager re-seed (recon above) so fnB's bl 0x102bd8ce8 advances one real engine-init
  step instead of heading toward a host-JNI dispatch.
- DataModel/self-constructed UI still behind the content/DataModel wall (unchanged);
  R1 synthetic CoreScript confirmed NO-GO (dead string, 0 refs), R2 UniversalApp.rbxm
  URL stage is GO latently downstream of a live DM.

## Repro
- `runs/sh165-dmforce.txt` (DMFORCE fired, ladder done, EXIT 124)
- `runs/sh165-dmforce-region.txt` (fnB region-watch 1 hit @0x102bd1b98, EXIT 124)
- hermetic: `sh164_dm_force_shell_is_coherent_and_env_gated` (jit.rs)

## Next
1. Implement the SCOPED manager re-seed at guest 0x102727550 on the fnB entry hook:
   fabricated all-leaf-vtable manager M (+0x30 write-leaf, +0xf8/+0x108/+0x1f0 leaf,
   vt[+0x720]==0) so 0x102bd8ce8 -> getter -> vt+0xf8 -> +0x108 -> FFI +0x1f0 ->
   vt+0x720 soft-returns cleanly. Then re-run the real ladder and observe whether the
   engine-init pipeline benign-completes (vs its next fault).
2. Dispatch the next Route-B cone on the app-shell ctor 0x102207b50 path (the actual
   app-shell builder, reached only via the AppStart JNI nativeAppBridgeAppStart at
   0x102338ff8, per deleg_42b6f9d6).