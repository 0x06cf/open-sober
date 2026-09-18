# Frontier SH302 — EC reader-gate seed measured inert; the pre-reader continuation is a single-opaque-block live path

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--v2boot-session-dmfn` + `JIT_ROUTEB_EC_READERGATE=1`), pure closure of
the exact "pre-reader continuation" lever SH301's NEXT GATE names — converted
from "this is the gate" into "seeded + measured; the reader stays unreached."

## tl;dr

SH301 proved SH300's realsession flag at [0x106d31e28] is latched but its READER
at 0x2e246f4 is never reached (neither the real V2Init bl-target 0x1023c5538 nor
the benign 0x1023c1b0c ever fires as a block entry). SH302 identifies the 
*pre-reader continuation* — the mechanical reason — and seeds it, but MEASURES
the reader still does not open. So the SH301 dormant verdict is confirmed at the
newest HEAD with a concrete seed attempt on the named gate (not just a doctrine
re-run), and the pre-reader mechanism is now byte-pinned in a hermetic test.

## SH300/301 recap (authoritative, do-not-re-tread)

EC world 0x102e24598 (genuine DM-creation machine, SH235/298/298b/299) reads the
realsession byte [0x106d31e28] at `0x2e246f4` (`adrp 6d31000; ldrb w8,[x8,#3624];
cbz w8,0x2e2472c`). With flag=1 -> `bl 23c5538` (real
/nativeAppBridgeV2InitWithParams AppBridge-V2 singleton factory) + `bl 23f1654`
(nativeAppBridgeStartLuaAppDM) + a vt[+16] dispatch. Across ~10 configs neither
bl-target ever fires as a block entry -> the reader is unreached; the EC body
soft-returns before it (SH301 block-entry doctrine, single-compiled-block bound).

## SH302: WHICH pre-reader path and why the reader is gated

Fresh disasm of [0x2e24598, 0x2e247dc]:

```
2e24598 stp x29,x30,[sp,#-96]!     ; prologue (x29 = frame base = entry_sp-96)
2e245b0 mov x29, sp
...
2e24694..  app-request build (string assigns + FMOD/AAudio touches, benign)
2e246d8 ldr x0, [x8, #32]          ; x8 = [x29,#104] (caller-frame OBJECT pointer)
2e246dc cbz x0, 0x2e246f4          ; <-- THE GATE: x0==0 -> straight into reader
2e246e0.. ldr x8,[x0]; ldr x8,[x8,#48]  ; else a LIVE vt[+48] dispatch
2e246f0 blr x8                     ;       that CONSUMES control (never returns
2e246f4 <READER>                   ;       to fall through -> reader unreached)
```

So the reader is gated behind a caller-frame OBJECT at `[x29,#104]` (==
`entry_sp + 8` after the prologue) whose `[+0x20]` word decides the `cbz`. If it
is non-NULL-with-nonzero [+0x20], control enters a live-object vt[+48] dispatch
at 0x2e246f0 and never falls through to the reader. That dispatch is the
"pre-reader continuation" SH301 named. SH302 seeds `[x29,#104]` to a leaked
zeroed buffer so `[+0x20]==0` -> the `cbz x0, 0x2e246f4` is TAKEN.

## SH302 fix (default-inert)

`routeb_ec_world_reader_gate_guard` (jit.rs, opt-in `JIT_ROUTEB_EC_READERGATE=1`),
wired after `routeb_ec_world_arg0_vt_guard`. At EC-world entry block
pc=0x102e24598, reads the entry SP (state.x[31]) and, when the caller-frame slot
`[entry_sp+8]` (= the `[x29,#104]` the body reads) is non-NULL, writes it to a
leaked 0x40 zeroed buffer (`[+0x20]==0`). NULL slot is deliberately left NULL (a
NULL would turn 0x2e246d8 `[0+32]` into a SIGSEGV, a NEW fencepost, not this
guard's job). Idempotent, env-gated, default-inert.

## Measured (real libroblox.so, full SH300 env + JIT_ROUTEB_EC_READERGATE=1)

- **readergate fires deterministically** (5/5 completing runs): `[routeb-sh302]
  seeded EC reader-gate [x29+8]@0x...c988 = zeroed buf ... [+0x20]=0 -> cbz
  @0x2e246dc TAKEN`. SH300 flag also fires 5/5.
- **dmfn returned Ok(0x...4b10) 4/5 CLEAN** (run5 = known SH55/64 session-drive
  flake, unrelated: crashes in the SEP-17 session rung before the dmfn line).
- **The reader's bl-targets STILL do not fire** — 0/5 for both the real V2Init
  entry (JIT_DUMP_PC=0x1023c5538 = 0 dumps) and the benign 0x1023c1b0c
  (region-watch 0 hits). Region-watch on the EC body shows ONLY the two block
  entries at 0x102e24598 + 0x102e245f4 (and the shared string-assign helper
  0x102b504e4).
- **Terminal byte-identical to baseline**: SIGSEGV guestpc=0x101db1b08 (LSM
  insert-leaf wall) then SIGABRT, EXIT 134. My seed of [x29,#104]+32 did NOT
  change the run's observable trace.

## VERDICT (do-not-re-tread SH302)

- SH302 is a **measured inert** on the exact SH301 "pre-reader continuation"
  lever: the seed fires and writes the gate slot, but the reader does not open.
  Two consistent explanations: (a) the whole [0x2e245f4..0x2e247dc] tail is ONE
  compiled block, so my seed is applied at block-entry but the block was already
  cached from an earlier translation or the [+0x20] load resolves through a
  different live object mid-block; or (b) the single block's real exit is an
  EARLIER soft-return I have not isolated, upstream of 0x2e246d8. Either way the
  named gate is now SEEDED+MEASURED, not just described.
- Route-B live-DM structural gate UNCHANGED (no make_shared<DataModel>; MH_*
  false, DM-root 0). SH174 capture-latch (arm *(0x106391908) at a real session
  make_shared<DataModel>) stays the single forward hook.
- This is closure + hardening in the SH243/251c/262/272 mold: no prod path runs
  with the guard (env-gated), and the pre-reader mechanism is byte-pinned so a
  future cycle knows the exact lever + that seeding the gate slot does not cross
  it.

## Verify

- `cargo test -p arm64jit --lib -- sh302 sh301` = 2 passed (408 lib tests total).
- `cargo test --workspace` EXIT 0 (all crates green; arm64jit 408/0).
- `cargo build --workspace` EXIT 0.
- Repro: `runs/capture_sh302_readergate.sh`; dumps `runs/sh302*.txt`,
  `runs/sh302b-v2interior-r{1,2,3}.txt` (5/5 firm via sh302c batch).

## NEXT GATE (for the next frontier session)

The reader is not reachable by seeding its gate slot. The remaining forward
lever on this line is to make the single compiled block that spans
[0x2e245f4..0x2e247dc] actually FALL THROUGH to 0x2e246f4 — which requires
understanding the block's real early-exit (an earlier branch/soft-return in
0x2e245f4..0x2e246d8) rather than seeding the reader-gate object alone. That is
a live-object / SESSION-CTOR mechanism within an opaque multi-instruction block,
NOT a value seed — per the standing doctrine, keep single-drive discipline and
do-not-re-tread broad static seeds (SH174/204/248h/256). SH300/301/302 are
closed: do NOT re-seed the realsession flag OR the reader-gate slot to "unlock
the reader"; the gate is the block's internal control flow, not its inputs.