# Frontier SH322 — cross the SH273 lifecycle-notifier live-object wall reached via the
# SH320/321 MAIN-path binder-dispatch (guard-gated new reach, candidate (a))

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh322` (jit.rs unit test, 1 passed)
+ `sh322_lifecycle_wall_earlyret_canary_pinned` (elfjit real-image guard, 1 passed). Workspace
green; elfjit examples 147/0 (146 + sh322); arm64jit lib 407/0. elfjit.rs held 25 B under the
1MB pre-commit hook (condensed SH-prose comments, facts/addresses preserved). Route-B live-DM
gate UNCHANGED (no DM; DM-root [0x106a68818]=0, MH_* false). SH174 capture-latch single forward hook.

## 1. What SH322 does

SH321 measured that the do-init DONE-path MAIN binder-dispatch (0x206df4 -> vt+0x30 -> br x1,
SH320 flips the thread-match gate) climbs into REAL engine settings-init
(`nativePostClientSettingsLoadedInitialization3`, caller 0x2270024 -> 0x2270050 ->
`bl 0x21f3748` @0x2270060) and dies at the SH273 lifecycle-notifier body 0x21f3748,
`ldrb w8,[x8,#80]` @0x21f3770 fault=0x50 because arg0's first word [x1]==0 (host-heap controller
not-yet-constructed). SH321's conclusion: "the dispatch reaches ENGINE-SETTINGS-INIT, not a DM
ctor" — it's a NEW ROUTE into the SH273 wall (via real engine init, not a fabricated JNI entry).

This cycle executes STATUS candidate (a): CROSS that wall. Disasm of 0x21f3748 shows the fork
is benign-no-op-able: `ldr x8,[x1]` @0x21f376c -> `ldrb w8,[x8,#80]` @0x21f3770 ->
`tbnz w8,#1,0x21f3870` @0x21f3774. If byte[+80].bit1 is SET, control jumps STRAIGHT to
0x21f3870 = the epilogue canary-check + ret — a benign no-op that BYPASSES the whole
registry-build body (which needs the live session controller). That is the SH159c
"mode-2 no-op" pattern, applied to a new site.

## 2. The guard (default-inert)

`routeb_lifecycle_wall_earlyret_guard` (jit.rs, opt-in `JIT_ROUTEB_LIFECYCLE_EARLYRET=1`),
registered in the block-entry guard dispatch after the SH320 guard. Fires at the callee block
entry pc=0x1021f3748 (a real `bl` target -> opens a block entry, SH301 doctrine), reads x1 (the
caller's sp+0x18 pair pointer, i.e. `&[x22]`), and when [x1]==0 writes [x1] = a leaked object
(`routeb_lifecycle_earlyret_obj`, 0x200 zeroed bytes with byte[+80] = 0x02, bit1 set). The next
`ldr x8,[x1]` loads it, `ldrb [x8+80]`=0x02, `tbnz w8,#1` TAKEN -> 0x21f3870 canary-check+ret.
Seeds only the measured NULL headless state; never corrupts a live ref (idempotent, skips any
non-zero [x1]).

## 3. Measured (real libroblox.so, JIT_DRIVE_LIFECYCLE full seed set, runs/ab_sh322_lifecycle_wall_cross.sh)

A/B (both arms use the SH320 MAIN-path reach, JIT_ROUTEB_DONEPATH_MAIN=1):
- BASELINE (+DONEPATH_MAIN only): EXIT 139 (SIGSEGV) at guestpc=0x1021f3748 — the SH273 wall.
- FORWARD (+JIT_ROUTEB_LIFECYCLE_EARLYRET=1): guard fires 4x (`[routeb-sh322] seeded caller pair
  [x1]=... lifecycle early-ret obj ...`), fn 0x21f3748 no longer faults; the SIGSEGV MOVES forward
  to guestpc=0x1021f5078 (fault=0x0) — a REAL fencepost advance past the SH273 wall (EXIT then 134,
  the engine self-aborts after advancing deeper rather than segfaulting at the wall).

The new terminal 0x1021f5078 reads a global std::string [0x106ed7a18] (`adrp x8,6ed7000;
ldr x8,[x8,#2584]` = [0x106ed7a18]; `ldrb w10,[x8]` faults fault=0x0 when [0x106ed7a18]==0).
That cell ((6ed7000+#2584)=6ed7a18) is the slot IMMEDIATELY below the SH248d cookie-jar globals
[0x106ed7a20]/[0x106ed7a28] — the same .bss std::string region. Natural next forward (SH323):
seed [0x106ed7a18] = empty SSO string (routeb_empty_sso_string, the SH248d helper) so the
whitespace-check fn at 0x21f5078 completes too. Then chase the next fencepost on the SAME
engine-settings-init / session-driven line.

## 4. Honest

This is a genuine forward on the SESSION-CTOR MAIN-path line: the SH273 shared lifecycle wall
(which SH321 reached via real engine init) is now crossed by a benign no-op seed — the engine
settings-init body advances past it instead of faulting. It does NOT manufacture a DataModel
(DM-root 0, MH_* false); the new terminal at 0x1021f5078 is itself the SH248d-class .bss
std::string region (seedable). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch
stays the single forward hook.

## 5. Verify

- `cargo test -p arm64jit --lib sh322` = 1 passed (env/pc-gated; seeds [x1] pair when 0; object
  has byte[+80].bit1 set; leaves non-zero [x1] and NULL pair untouched).
- `cargo test -p arm64jit --example elfjit -- sh322` = 1 passed (real-image pins: tbnz
  0x1021f3774=0x370807e8, early-ret ldr 0x1021f3870=0xf94002c8 + ret 0x1021f3890=0xd65f03c0,
  caller stp 0x102_270024=0xa901a3e9).
- `cargo test --workspace` EXIT 0; `cargo build` EXIT 0; elfjit examples 147/0; arm64jit lib 407/0.
- Repro: `runs/ab_sh322_lifecycle_wall_cross.sh` (baseline EXIT 139 @0x1021f3748 vs forward
  seed=4, SIGSEGV advances to 0x1021f5078).

Single-agent, default-inert. No production code path altered.