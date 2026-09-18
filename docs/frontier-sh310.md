# Frontier SH310 — SendAppEventOnAppReady fabricated-'Home'-jstring ABI CONFIRMED (candidate (b) closed)

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed).

## What was measured (this cycle)

Closed the open step-2 real-jstring ABI question (STATUS candidate (b)): DOES the
fabricated `Home` jstring in x5 ACTUALLY resolve through SendAppEventOnAppReady
(0x102bb463c) and drive it to build + dispatch its real app-event? Disassembled the
FULL body from fresh objdump and measured the decisive observable on real
libroblox.so with the SH307-forward env (the env where SendAppEventOnAppReady
previously `returned Ok(0x107273d50)`).

### The fresh disasm (authoritative, file-offset == guest-0x100000000)

SendAppEventOnAppReady entry 0x102bb463c. ABI placement:
- `0x2bb4660 mov x19, x5` — the EVENT jstring is the 5th arg (x5), as the step-2
  correction predicted (SH155 had wrongly used x2).
- Four `bl 0x21e1fec` jstring->RBX-string helpers convert x2->[sp+0x48], x3->[sp+0x30],
  x4->[sp+0x18], x5->[sp] (0x2bb4684/0x2bb4694/0x2bb46a4/0x2bb46b4). The [sp] slot is
  the one discriminated.
- `0x2bb46b8..0x2bb4708`: SSO length decode (csel x9) then: len==12 -> 0x2bb4778
  ("DataModel.." 8-byte compare, w19='lsl#1'), len==5 -> 0x2bb4738 (5-char "Games"/etc,
  w19=3), len==4 -> 0x2bb46e4 which builds w10 = 0x656d6f48 = "Home" LE and
  `cmp w9,w10; b.eq 0x2bb47c4` (movz w19,#4), else "Chat" 0x2bb47cc (w19=1), else
  `0x2bb47bc mov w19,wzr`. So 'Home' resolves w19=4.
- Version gate `0x2bb47d8..0x2bb47ec`: reads [0x10683d350], low byte >=6 && byte1 >=5
  else `b.cc 0x2bb4824` (skips the build). Passed -> `0x2bb48b0 bl operator_new(0x58)`
  app-event obj, event vtable 0x635e068+0x68 stored at [obj], the 4 converted strings
  copied in, `0x2bb4948 str w19,[x20,#80]` (the event-type discriminator), then
  `0x2bb4958 bl 0x2baeeec` (the app-bridge pipe -> do-init 0x2206c40), cleanup,
  canary-check, ret.

### The decisive measurement

The discriminator + version-gate are MID-BLOCK (SH301 class: direct `b`/`b.eq` targets
inside a straight-line compiled block are NOT block entries -> region-watch is blind to
them; my first SH310 band on [0x2bb47bc,0x2bb4830) returned 0 hits for exactly this
reason — an observation artifact, NOT absence of execution). But the app-event pipe
`bl 0x2baeeec` @0x2bb4958 IS a guest `bl` target -> opens a block entry, and it is
REACHED ONLY past the discriminator + version-gate + 0x58 alloc. With --v2boot-skip-appstart
this rung is the only caller of 0x2baeeec in this env, so a hit is unambiguous.

SH307-forward env + `JIT_REGION_WATCH=0x102baeeec-0x102baf000,0x102206c40-0x102209000`,
3 runs (runs/sh310b-pipe-*.txt, runs/sh310c-pipe-*.txt):
- **run 2 (clean, EXIT 124): SendAppEventOnAppReady returned Ok; pipe 0x102baeeec FIRED;
  do-init 0x102206c40 entered and ran the ENTIRE app-shell-ctor body** (block entries
  0x102206c40 -> 0x102207b50 ctor -> 0x102208e88, past the FMOD tail 0x102208ebc).
- runs 1/3 (EXIT 134): pipe + do-init entered too, then aborted inside do-init — the
  run-variable live-object wall (SH248b logging-perturbation class from watching 2 bands).

### Verdict — candidate (b) CLOSED

The fabricated `Home` jstring in x5 GENUINELY round-trips through the real body: helper
0x21e1fec converts it into the SSO string at [sp], the discriminator resolves the 'Home'
w19=4 branch (movz w19,#4 reached), the version gate passes, the 0x58 app-event is
allocated and populated, and it is dispatched through the app-bridge pipe -> do-init ->
app-shell ctor. The observed `w19-event=0x0` in 46 prior logs was the SH308 measurement
artifact (x19 is callee-saved, so the post-jit_run read reflects the restored value, not
the discriminator). The step-2 letter's "a fabricated jstring DOES resolve" correction is
now CONFIRMED as measured, and the onAppReady rung's full build+dispatch path is
authoritative byte-pinned. This de-risks the ladder: SendAppEventOnAppReady's 'Home'
event genuinely reaches the engine's app-event level, and its do-init/app-shell-ctor
traversal (SH308/309 reach) is re-confirmed via the pipe block entry.

## Honest (do-not-over-claim)

- Does NOT manufacture a DM: DM-root [0x106a68818]=0, MH_* all false. Route-B live-DM
  structural gate UNCHANGED. SH174 capture-latch stays the single forward hook.
- The run is run-variable past the pipe (do-init deep construction hits the live-object
  wall), so this anchors the ABI + the app-event build+dispatch, NOT a new fixed forward
  execution distance beyond do-init's app-shell-ctor body (which SH308 already measured).
- The version gate [0x10683d350] value is whatever JIT_DRIVE_LIFECYCLE/settings seeds it;
  it passed headlessly (the build+pipe fired), so no extra seed was needed this env.

## Verify / files

- New hermetic `sh310_sendappevent_home_build_dispatch_closed` (elfjit.rs real-image
  guard): pins the x5 ABI (0x2bb4660=0xaa0503f3), jstr->RBX helper call (0x2bb46b4=
  0x97d8b64e), 'Home' compare (0x2bb4704=0x6b0a013f), discriminator target (0x2bb47c4=
  0x52800093), version gate (0x2bb47d8=0x12001c08), discriminator store (0x2bb4948=
  0xb9005288), app-event pipe bl (0x2bb4958=0x97ffe965). Skip-if-absent.
- elfjit.rs held under the 1MB pre-commit hook (condensed SH81/177/245/264/285/295/307+
  render-doc prose, all addresses/constants preserved).
- Repro: runs/capture_sh310b_pipe.sh (2-band watch, run-variable), runs/capture_sh310c_pipe_clean.sh
  (pipe-only). Raw runs/sh310b-pipe-*.txt / runs/sh310c-pipe-*.txt (not committed).
- Commit: local `dev` only (operator pushes).