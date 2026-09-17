# SH282 — correct SH281's wrong premise: the state=5 reentry continuation locks FIXED global 0x106863aa0, NOT [box+0xaa0]

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · extends `--v2boot-session-engine5` (default-inert) · workspace green (578/0; elfjit examples 115/0)

## What this is

SH281 crossed the state=5 GlobalInit-reentry NULL-deref (`[config+56]`) and reported the
engine5 rung PARKING at continuation 0x2207118's `add x0,#0xaa0; bl mutex_lock`, attributing
the park to `[box+0xaa0]` = "host-heap junk past the callee's 0x40 box", and proposed the
next-step "supply a correctly-sized settings object to the continuation."

That attribution is WRONG, and the proposed next-step is a DEAD END. This cycle pins the real
mechanism and redirects future work, so no future cycle chases "make the box bigger."

## The correction (fresh disasm, real libroblox.so, verified at runtime)

Continuation 0x2207118 (entered by the state=5 callee 0x275a23c's tail-`b`):

- `0x2207138  str x0,[sp]` — saves the incoming box pointer to a stack slot.
- `0x220713c  adrp x0, 6863000; 0x2207140 add x0,x0,#0xaa0` → **x0 = 0x106863aa0** — a FIXED
  global in the 0x6863000 page, NOT `[box+0xaa0]`.
- `0x2207144  bl 0x2b53a68` (pthread_mutex_lock).
- Then enqueues via `0x2d9713c` (x0=0x106863a70 fixed object, x1=sp→box), cond_signal + unlock.

Decisive point: **0x106863aa0 is TRUE zeroed .bss.** `readelf -lW` shows it sits inside the last
RW LOAD (`0x67d67c0` vaddr, memsz `0xb5d47c` → covers up to `0x7333c3c`) but BEHIND its filesz
(`0x53698` → file-backed bytes end at `0x6829e58`). A zeroed resident page = OS static-initializer
`pthread_mutex_t`: `pthread_mutex_lock` returns immediately, it does NOT park, and it has NO
relation to the callee box. SH281's "box must be a real sized settings salient object" is a
false premise — there is no box at +0xaa0 and no live-object park here.

## The real terminal (this cycle's runtime measurement)

Both clean runs (runs/sh281-clean.txt, runs/sh281-bisect.txt) show the engine5 rung's jit_run
reaches the reentry callee and its intermediate `bl 0x2206f04`, but never RETURNS to the
harness loop within the timeout — EXIT 124 (clean timeout park), NOT SIGSEGV. The park is in
the reentry/continuation world-build (the enqueue-construct `0x22071ac` on fixed object
0x106863a70), i.e. still the SH174/SH204 live-object class — but at a FIXED-GLOBAL object, one
class of wall earlier than a manufactured-box deref.

## Do-not-re-tread

- Do NOT "supply a correctly-sized settings box / make operator_new's 0x40 box bigger" — the
  continuation never reads `[box+0xaa0]`; the lock target is the fixed global 0x106863aa0.
- state=9 (0x2bd2668) still routes to the same 0x2bce0d4 config dispatch — same class, not a
  fresh lever.

## Honest framing

Cause-not-symptom on the SEP-17 SESSION-CTOR primary lever; just corrects WHERE the engine5
rung actually parks (fixed-global reentry continuation world-build, not box+0xaa0). NO DM
manufactured (MH_FLAGS_LOADED=false, MH_APP_READY=false, DM-root 0); Route-B live-DM structural
gate UNCHANGED; SH174 capture-latch stays the single forward hook. recon-v3 plane unchanged.

## Verify

`cargo test -p arm64jit --example elfjit sh281` and `sh282` each 1 passed (real-image pins:
continuation adrp/add/bl 0x10220713c/140/144, enqueue mov/bl 0x102207150/154, consumer ldr/bl
0x102d97148/15c, plus `host_addr_of(0x106863aa0)` Some). elfjit examples **115/0** (was 114);
`cargo build --workspace` + `cargo test --workspace` EXIT 0 (578). Repro runs/sh281-clean.txt +
runs/sh281-bisect.txt. Single-agent, default-inert.