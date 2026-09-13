# SH82 — nativeGameGlobalInit (rung 1) UNPARKED: the park is a thread-dispatch
# main-thread-id gate, not a latch plain-write; seed [0x106863a68] = self

## Result (wall ADVANCE: rung 1 no longer parks on the real binary)

Root-caused the Route-B rung-1 park that has stalled the `--v2boot` ladder since
SH54/55: `nativeGameGlobalInit` (guest 0x102206404) is NOT self-blocking — its
`GameGlobalInitImpl` dispatches to the real TaskScheduler construction but ONLY
on the thread the engine stores as "main" at `.data` cell **0x106863a68**
(file 0x6863a68). On the harness's detached ladder thread (a non-main thread) it
parks forever in a completion spin. Seeding that cell with the ladder thread's
own `pthread_self()` makes the rung take the returning match path — **rung 1
now runs its real do-init** and faults FURTHER at the next gate (guest 0x1021daf78,
a downstream null-deref) instead of parking (exit 124). Same one-gate-per-SH
stepwise pattern as SH80->SH81.

**SH82b follow-on:** seeding the intern/hash-table bucket-array word of the
JNICall singleton (+0x30 = table base, +0x38 = capacity 0x400) advances the
ladder one MORE gate — the do-init's hash insert now faults FURTHER at guest
0x1029f3f7c (a `blr x8` through an invalid per-entry callback ptr at [x19+16]),
proving the do-init is executing deep real registration work. Product path
(no --v2boot) unregressed (exit 124, persist 45B byte-exact, present #0 swap
Ok(0x1), 0 crash).

Recon: read-only subagent deleg_f89a40ab (disasm of the same binary) + my own
disasm verification + A/B on the real binary.

## Corrected mechanism (disasm-verified, not the earlier recon claim)

File 0x2206db8 (guest 0x102206db8), the `GameGlobalInitImpl` thread-dispatch:
- 0x2206de4 `ldr x20,[x8,#2664]` (x8=adrp 0x6863000) loads the engine's stored
  "main thread id" from cell **0x6863a68** (guest 0x106863a68, `.data` rw-).
- 0x2206de8 `bl pthread_self` (x0 = current thread).
- 0x2206dec `cmp x0,x20`.
- 0x2206df0 `b.ne 0x2206e28` (0x540001c1, imm19 -> target offset 0x38):
  - **taken** (x0 != x20, i.e. the harness's detached ladder thread) -> 0x2206e28
    runs the real inline do-init chain (operator-new, bl 2206f04/2207118/2207578)
    which constructs the TaskScheduler and ends in the **0x2207648 completion
    spin**: `ldrb [x19+1]; tbnz #0 -> done; else bl 22076f0 (->0x2850520 scheduler
    pump) ; b 0x2207648`. The headless main thread never processes the posted
    job, so the flag never sets and the rung parks FOREVER (pre-fix: ladder
    drives nativeGameGlobalInit, never prints "after ...", exit 124).
  - **not-taken** (x0 == x20, thread is the stored main) -> 0x2206df4 tail-calls
    `[thiz+32`]+vtable+48` and GlobalInit is DONE (returns).

The earlier recon doc (recon-routeB-globaltinit-unblock.md) said the park is a
nanosleep poll gated by latch byte [0x72739d4]; SH80/81 chased that latch. SH82
shows the TRUE first gate is the thread-identity dispatch: [0x72739d4] is an
OUTPUT of the do-init (set at file 0x22474e8 inside it), never reachable while
the do-init only ever runs the park/no-return path on the detached thread.

## SH82 A/B (real libroblox.so, --v2boot ladder) — what works and what regresses

Approach A — **force the b.ne via a .text NOP (REGRESSION, REVERTED).** Patching
guest 0x102206df0 `b.ne` -> `nop` makes the dispatch fall through to the
not-taken path (0x2206df4) on EVERY thread INCLUDING the main thread's own boot
call -> the main-thread StartApp serialization re-routed incorrectly -> 6/6 runs
json-crash (exit 139; baseline no-patch is 8/8 clean exit 124). A global .text
branch rewrite can't distinguish "the ladder thread" from "the main thread's own
call", so it is the WRONG lever. Reverted.

Approach B — **seed the stored-main-id cell [0x106863a68] = the ladder thread's**
**own pthread_self (LANDED, ADVANCES).** Since the dispatch tests
`pthread_self() == [0x6863a68]`, writing the ladder thread's own id into the
cell before driving nativeGameGlobalInit makes that one call take the match path
(returns) with NO .text write and NO effect on the main thread's stored id
(restored to its original value immediately after the rung). Measured on the
real binary: rung 1 no longer parks -> it executes the do-init and faults
FURTHER at guestpc 0x1021daf78 (null deref, fault=0x0) — the next gate.

**Honest scope:** this unpacks the FIRST Route-B gate (the park) and proves the
rung-1 do-init is now reached, but it does NOT yet complete the ladder (rung 1
now faults at the next unsynthesized downstream object, 0x21daf78). The type-4
producer vector [0x106829ea8] stays 0 (later rungs 2-6 not yet reached). Full
nativeGameGlobalInit completion + a returned rung-2 warm-up is the next gate.

## New (hermetic) regression

`routeb_globalinit_thread_dispatch_main_id_cell_and_seed` pins:
- the main-id cell guest addr 0x106863a68 (== file 0x6863a68 + 0x100000000),
- the `ldr x20,[x8,#2664]` (0xf9453514) + `b.ne` (0x540001c1) encodings,
- the b.ne imm19 -> park-target offset (0x38 = 0x2206e28 - 0x2206df0),
- the seed write target == the main-id cell.

Workspace **511/0** (+1), example tests 22 pass (unchanged).

## Repro

`runs/capture_v2boot_sh82.sh` (add `--v2boot` -> expect the `seeded main-thread-id`
line + the rung-1 run faulting at guestpc 0x1021daf78 instead of parking).

## Next (ranked)

Rung 1 (nativeGameGlobalInit) is now unfolded and its real do-init runs deep
registration work; each seeded gate exposes the next. Current fault: guest
0x1029f3f7c — a `blr x8` through an invalid per-entry callback ptr at
[x19+16] of a hash table the do-init's registration path fills. (a) disassemble
the hash-table builder to find what [x19+16] should point at (a real per-entry
fn — likely a haddr/has-empty or deleter) and seed the table object with a
benign leaf (mirror routeb_singleton_leaf) so the insert completes; (b) once
nativeGameGlobalInit fully returns, drive rung 2 nativeUpdateAdapterInit
(0x10221c3ec) and check whether rungs 2-6 install the type-4 producer vector
[0x106829ea8]; (c) wire the NativeHelper gameActivity_* callbacks into the
RegisterNatives registry (recon step 2) so StartLuaAppDM can build real
GuiObjects once reached. Standing structural wall otherwise unchanged.