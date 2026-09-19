# Open Sober — Agent Handoff

## SH349 (Sep 19, 2026, hermes-worker): CROSS the long-standing SH285 terminal — RET the faulty LSM byte-copy sub-call 0x101d9a15c; persistence lane advances one fencepost to a GOT/canary read wall at 0x101d9a708
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_APPEND_SKIP=1`
(`routeb_patch_lsm_append_skip`, elfjit.rs — RETs the `ldr w8,[x2]` byte-copy sub-call that the
SH285 fault lives inside) + `sh349` hermetic (real-image pins: append prologue / SH285 caller bl
0x97ffa192 / fault store 0x381ff54b / natural ret) + runs/capture_sh349_lsm_append_skip.sh.
Workspace green (elfjit example 157/0; arm64jit 416/0).

### The forward this cycle (the stated SH348 next step, now implemented + measured)
SH348 showed the SH285 SIGSEGV (guestpc=0x101db1b08) survives a whole-init leaf-ret because the
caller block is reached by a mid-function direct jump past the entry patch. So the skip must
target the FAILING SUB-CALL itself. `routeb_patch_lsm_append_skip` RETs only the one byte-copy
leaf (0x101d9a15c, pure memcpy, zero observable side effects) -> EVERY path into the fault is
stubbed regardless of how the caller block is reached.

### MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES + both skips, 3/3)
- OLD terminal GONE: no more `SIGSEGV guestpc=0x101db1b08` (every SH260/284/285/3444/348 run died there).
- NEW terminal (3/3): `SIGSEGV guestpc=0x101d9a708 fault=0xffffffffffffffff` — a stack-canon
  name/version-packing helper reading a `.got` slot [0x1067d16f0] as its canary pointer. The
  persistence lane advances one full fencepost past the returned wall.
- DM-root [0x106a68818]=0, MH_* all false. DMCONT continuation still not at app-start 0x2bd2058.

### Conclusion + next
SH349 is a real, measured forward: the SH285 wall family is crossed for the first time. The new
terminal 0x101d9a708 is another SH285-class live-object wall; the register dump REFUTES the
canary/GOT-gap hypothesis (x20=valid patched canary; the fault is the caller's garbage source
pointer x0/x19=0xff..ff). `bl 0x1d9d8b0` has HUNDREDS of call sites across the binary (the
most-called function), so sub-call-whack-a-mole is unbounded, and the DMCONT→app-start path has
NO bypass (the [0x683d920] latch is the "app-start already ran" re-entry gate, set only after
app-start; the two `bl 1d9d8b0` string-build calls are mandatory to reach `bl 2338ef4`). So this
persistence lane is measured-returned; the Session-CTOR live-DM wall stands. Route-B live-DM gate
UNCHANGED; SH174 capture-latch stays the single forward hook. Next forward (non-persistence
Route-B): the dataModel-bindings receive side — onAppLuaWillStart (the sole SEP-17
dataModel-bindings receive never wired; messageBus publish is driveable-clean per SH347).

## SH348 (Sep 19, 2026, hermes-worker): measured negative — leaf-`ret`ing initStorageManagerNative does NOT clear the SH285 terminal (the byte-copy @0x101db1b08 is reachable past its own entry)
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_INIT_SKIP=1`
(`routeb_patch_lsm_init_skip`, elfjit.rs) + `sh348` hermetic (real-image pins: entry prologue
0x101d9d8b0 stp->ret, sh324 caller bl 0x102256608, SH285 terminal 0x101db1b08, app-start bl region
0x102bd2058) + runs/capture_sh348_lsm_init_skip.sh. Workspace green (arm64jit 416/0 + sh348; elfjit
example 156/0). elfjit.rs 1,044,985 B (<1MB hook). recon-v3 frame plane re-verified green (24 real
task frames, 197 node pops, 0 crash).

### The forward this cycle (a cause-level leg, then measured negative)
SH344b measured the app-shell ctor band at 0 hits; every Session-CTOR rung caps at the SH285
persistence-lane live-object wall. SH344/346 said "do not re-drive a repair seed into [obj+0x50]"
(SH248h trap). This cycle tried a DIFFERENT line-cross (SH117/SH93 precedent): leaf-`ret` the whole
initStorageManagerNative so the Session-CTOR continuation is not required to own the live LSM object.

### MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES, 3 runs)
- SH348 patch fires (`a9bd7bfd -> d65f03c0`) but the SH285 SIGSEGV `guestpc=0x101db1b08
  fault=0xffffffffffffffff` PERSISTS 3/3 — even after widening the block-cache drop to the full
  function body [0x101d9d000..0x101dbe000] + caller. The crash reaches the deep byte-copy by a call
  path the entry patch cannot stop (the fault site is inside a sub-path whose gate is past
  initStorageManagerNative's own entry, OR a caller block jumps to the interior directly).
- App-shell band 0x102207b50 "fires" only as the SH344c FastLog warmer (79 hits at the SINGLE entry
  pc + the 0x102208e8c/0x208eac destructor-loop deep pcs), NOT construction. Route-B live-DM gate
  UNCHANGED (DM-root 0, MH_* false).

### Conclusion + next
SH348 is a MEASURED NEGATIVE that closes the "skip initStorageManagerNative to cross SH285"
candidate and REFINES the attribution: a storage-skip must target the failing sub-call that builds
the unconstructed LSM string, not the whole init. Route-B live-DM structural gate UNCHANGED; SH174
capture-latch stays the single forward hook. Repro: runs/capture_sh348_lsm_init_skip.sh.

## SH347 (Sep 19, 2026, hermes-worker): messageBus RECEIVE half measured headlessly for the first time — publishRaw drives clean (Ok 0x3e8) but the cb's DM-holder read never fires; SH185's static-only closure is now a measured result
Single-agent (cone suppressed). Two default-inert additions
(arm64jit/src/jit.rs: `routeb_busrecv_holder_guard` + `drive_messagebus_publish_receive`,
both env/flag-gated) + elfjit opt-in rung `--v2boot-session-pub` + 2 hermetic tests.
Workspace green 596/0. elfjit.rs unchanged in product path (rung opt-in).

### The forward this cycle
SH264 named "messageBus experience-launch receive" as the un-drive honest-next-candidate; SH185
had closed it by STATIC judgment only (elfjit.rs:7565), and SH269/315/316/337 later MEASURED that
the subscribe half runs headlessly — overturning SH185's static premise. Nobody had ever driven
publishRaw -> cb -> [DataModelBindings+16]. Now done, measured.

### Measured (real libroblox.so, capture_sh347_busrecv.sh, EXIT 124 clean):
- MessageBus.subscribe Ok(0x3e8) (re-confirmed).
- **MessageBus.publishRaw Ok(0x3e8)** — the RECEIVE half now EXECUTES headlessly (a genuine
  first), driveable-clean, not a crash.
- The cb DM-holder read (pc 0x102bd7474) NEVER fires => publish does not reach the
  experience-launch construction cb; [DataModelBindings+16] is never read; DM-root 0, MH_* false.
- Conclusion: SH185's closure is now MEASURED (receive driveable-clean, does not reach SceneGraph),
  closing SH264's open candidate. Route-B live-DM structural gate UNCHANGED.

## Next (unchanged, authoritative)
SEP-17 SESSION-CTOR cause-level drive remains the primary forward. Every driven rung (do-init ->
DMCONT +0x1f0 -> app-start factory -> LSM) is measured; the terminal is the SH285 live-object wall
family — cause-not-symptom, not seedable. SH174 capture-latch stays the single forward hook. The
messageBus publish entry being driveable-clean is a small new forward surface (a future
receive-payload-with-real-string probe), not a DM. All research subagents Route-B-scoped; cone
still suppressed.