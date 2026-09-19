# Frontier SH385 — COMPOSED LSM-crossing NEGATIVE + byte-anchored SH285 reader-mechanism

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start and end (cargo test --workspace EXIT 0; arm64jit lib 439->440; jit.rs
1,014,534 B <1MiB hook).

## Why this cycle
SH384 manufactured a genuine vtable-owning LocalStorageManager object by running its
real ctor, and SH383/384 both named "wire the manufactured manager INTO the SH285
reader/pop lane" as the next step. Before implementing that wiring (which requires
the LSM map's value-string to be coherent — structurally deep), this cycle closes
the LAST never-run-composition loophole: the LSM crossings (KEYFIX + APPEND_SKIP +
PACK_SKIP) plus the full reaching env have never been composed end-to-end while
watching whether the do-init MAIN dispatch body 0x10258b5d8 (SH362's measured
never-executing body) finally runs. SH379 ran append+pack but LACKED keyfix (the
SH341/343 crossing of exactly the pool-pop it terminal'd at); nobody combined all
three.

## The composed measurement (real libroblox.so, full ladder, KEYFIX+APPEND_SKIP+PACK_SKIP)
```
EXIT 134; SIGABRT+SIGSEGV; region hits: 258b(do-init body)=0 2bd1(cont)=0 2dbcc=0
ea3(governor)=0 207b(appshell)=0 1f1d8ac(scriptctx)=0
keyfix fired 1x (poisoned .text KEY 0x101d968e4 -> valid host-heap cell)
terminal: guestpc=0x101d9a528 fault=0x3e9000dfcdb (pool-pop, NON-poisoned key this run)
```
Even with every LSM crossing armed, the ladder STILL drains to the LSM pool-pop
family (0x101d9a528) and the do-init dispatch body 0x10258b5d8 gets ZERO region hits.
This closes the never-composed-crossing loophole: composing all three LSM crossings
does not exit the persistence family nor reach the do-init body. It is consistent
with SH379 (which did not run keyfix) and re-confirms the "measured-returned LSM
family" verdict (SH249/341/343/349/350/358/373/374/375/377/378/379) from the newest,
fullest crossing combination.

## Byte-anchored reader-mechanism refinement (genuinely new)
Disasm of the SH285 fault site one level deeper than any prior record:
- reader 0x1d99e30 (bl'd at 0x1db1ae8 -> x20): `adrp x8,726f000; ldr x8,[x8,#2240]`
  (= lsm_map_global bucket base) -> `ldar x8,[bucket]` -> `ldr x0,[x8, idx<<3]`
  (node) -> `ldr x0,[x0,#40]` = the value x20 that feeds the pool-move.
- append 0x1d9a15c (the SH285 letter): `add x10, x0, x1` @0x1d9a168 computes the
  write base = manager(x0=x19) + value(x1=x20); `strb w11,[x10],#-1` @0x1d9a180 is
  the faulting store. With x20 uninitialized the base overflows to a high faulting
  address (fault=0xff..ff).
- SH267's zeroed per-node cells give a nominal node[+40]=0, BUT SH285's live register
  dump at the terminal showed x20=0xffff80b3c723a4c0 (NOT 0) on the settings-state
  path — i.e. the node the reader actually reaches is NOT the SH267-seeded zeroed
  cell. The exact node/value is path-dependent and set only by a real LSM session
  ctor. This bullets the "which node" ambiguity that any manufactured-manager wiring
  must resolve: it is not the SH267 cell.

## Honest + do-not-re-tread
No DataModel manufactured (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0).
Route-B live-DM structural gate UNCHANGED. SH174 capture-latch stays the single
forward observer. SH385 ADDS: the fully-composed LSM-crossing negative (never-run
before) + the byte-anchored reader/pool-move mechanism + the node-not-sh267-cell
refinement. Do NOT re-compose LSM crossings expecting the do-init body (SH379/385
both drain); do NOT assume the SH267 node cells are what the SH285 reader consumes
(SH385). The LSM persistence lane is now closed as a Route-B avenue with the
strongest (fully-composed + byte-anchored) evidence on record.
Unchanged closures: LSM skips, EC reader-gate (SH355/356/374), 0x258b5d8/SetInitParams
(SH362/375), window-attach real (SH367), ALooper (SH365), governor gates full-ladder
(SH379), -9 string (SH380), map-header repair (SH248h), once-lambda store (SH381).

## Files
- crates/arm64jit/src/jit.rs: +hermetic `sh385_lsm_reader_value_slot_and_poolmove_base_pinned`
  (arm64jit lib 439->440).
- runs/capture_sh385_composed_lsm.sh (repro; log /home/hermes-worker/runs/sh385-composed-lsm.txt outside repo).