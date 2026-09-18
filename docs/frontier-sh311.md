# Frontier SH311 — do-init ONCE-lambda FIRES; the DM-controller ctor chain returns 0

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed).

## What was measured (this cycle)

Refined WHERE the do-init (0x2206c40) DM-construction line actually stops. SH310 had
shown the SendAppEventOnAppReady 'Home' event builds + dispatches through the app-bridge
pipe into do-init, and do-init's app-shell ctor region runs (0x102206c40 ->
0x102207b50 -> past the FMOD tail), yet DM-root [0x106a68818] reads 0x0 after every run.
The handoff's EXECUTE-DO-INIT-GATES note flagged the once-guard [0x106a68410] ordering as
the suspected cause (pre-setting it stops the once-lambda). This cycle measures which do-init
path actually runs.

### do-init __call_once structure (fresh objdump, guest==file+0x100000000)

- 0x2206c74..0x2206c84: `adrp x8,6a68000; add x8,#0x410; ldarb w9,[x8]` reads once-guard
  byte [0x106a68410]; `tbz w9,#0,0x2206d10` (bit0==0 -> ONCE path).
- DONE path 0x2206c88 (`movi v0.2d,#0`): `ldr x1,[x8,#1032]` = [0x106a68818] = DM-root into
  x1, builds a stack frame, `bl 0x6201bd4` / `bl 0x220671c` / `bl 0x2206db8` / `bl 0x221942c`,
  ret. Only RUNS when the once-guard is already latched.
- ONCE path 0x2206d10: `mov x0,#0x106a68410; bl 0x284ce54` (once-claim), builds the two rodata
  string consts + `bl 0x2173b3c` (DM-controller construction), then **`str x0,[x23,#1032]`
  @0x2206d74` stores the result into [0x106a68818] = DM-ROOT** (x23 = adrp 6a68000), then
  `bl 0x284cf5c` (once-release), `b 0x2206c88` (done path).
- 0x2173b3c is a thin string-dispatch wrapper: `bl strcmp(x0, rodata constant)` (0x2173b50/
  0x2173b64), cset a flag from the result, then **tail `b 0x61e30bc`** (0x2173b90) carrying
  (x0,x1,w2,w3,w4). DM-root receives whatever that chain returns.

### The decisive measurement (deterministic 3/3, runs/sh311-once-*.txt)

SH310-forward env (JIT_ROUTEB_PRELOAD_VALUECELL=1, GOVFLAG, SH115 patch, once-guard cleared + 
main-id re-seeded by the rung before SendAppEvent) + `JIT_REGION_WATCH=0x102206c40-0x102206d90`.
All three runs (EXIT 124 clean, appev=1, identical pcs):
- **once-lambda 0x2206d10 block-entry FIRED (region pcs 0x102206c40, 0x102206d10, 0x102206d70,
  0x102206d84...)** — the ONCE path runs (the rung's once-guard [0x106a68410] clear IS effective;
  it is NOT a JIT-cached-latched-path artifact).
- Therefore the DM-root store `str x0,[x23,#1032]` @0x2206d74 executes, storing the return of
  `bl 0x2173b3c`.
- Yet DM-root [0x106a68818] reads 0x0 after every run => **`bl 0x2173b3c` (string-dispatch ->
  tail 0x61e30bc DM-controller ctor chain) RETURNS 0 headlessly.** The constructor chain fails
  on its first real construction (a live-object/alloc gate inside 0x61e30bc), so the store lands
  a NULL and DM-root stays 0.

## Verdict — the standing gate is now precisely located

The handoff's "once-lambda never runs because the guard is latched" concern is REFUTED for the
send-appevent rung: the once-lambda DOES run (the rung clears + re-seeds main-id correctly),
and the do-init's real action (store the DM controller into DM-root) executes — but the value
stored is NULL because the DM-controller ctor chain 0x2173b3c -> 0x61e30bc returns 0 headlessly.
So the Route-B do-init line is blocked NOT at the once-guard ordering, but **inside the
DM-controller constructor (0x61e30bc and whatever it derefs)**. THIS is the next concrete
forward gate: drive/discover why 0x61e30bc returns NULL (its first live-object/alloc dependency)
so the once-lambda populates a real DM-root — after which do-init's done-path reads a live
controller and the app-shell/EC construction can proceed.

Per the operator SEP-17 SESSION-CTOR directive: do NOT solve this by fabricating a proxy
DM controller into [0x106a68818] (SH251 proved a genuine-but-fake DM doesn't advance app-start's
own map construction); the value is in tracing 0x61e30bc's NULL return on the real construction
path.

## Honest (do-not-over-claim)

- Does NOT manufacture a DM; DM-root [0x106a68818]=0, MH_* false. Route-B live-DM structural
  gate UNCHANGED. SH174 capture-latch stays the single forward hook.
- The finding is deterministic and reproducible (3/3 identical), and it corrects/refines the
  handoff's once-guard-ordering premise for THIS rung: the lambda runs; the constructor returns 0.
- 0x61e30bc itself was not driven in this cycle (it is the leaf that returns into the store);
  its NULL-return cause is the named next gate, not yet resolved.

## Addendum — the DM-controller ctor chain 0x61e30bc anatomy (why it returns 0)

Fresh disasm 0x61e30bc (the tail of `bl 0x2173b3c`): prologue -> `bl 0x216848c` + `bl 0x21686cc`
(inits) -> `adrp x0,6eda000; add x0,#0xe60` (global 0x106eda0e0) + `bl 0x2b53ba0` (get-singleton)
-> `mov x0,x23(name); mov x1,x20(param)` then **`bl 0x2168798` (name->service lookup) ->
`mov x25,x0; cbnz x0,0x61e32cc`** — a non-zero lookup is returned directly. Headlessly the
lookup returns 0, so control falls to the alternative: reads version counter [0x106fe24f8]
(`ldr w8,[x27,#3848]`, x27=6fe2000), `cmp w8,#0xa7e` branch — if version==0xa7e AND
[0x106fe2f90]==0, it builds a real object via 2x `bl 0x61e3aa4` (construct) storing into the
persistent obj [0x106fe2f08+120/128] and returns it; else returns 0 (or a table-slot value
[0x106fe2f00 + w24*8 + 128] in the version-range case). With the once-lambda's w3=0xff
(w24=255) and no populated service registry headlessly, both paths yield 0.
Concrete consequence: the DM-controller ctor returns 0 because `bl 0x2168798` (name->service
lookup with the do-init's rodata name strings) finds nothing populated headlessly, and the
fallback 0xa7e-build is version-gated ([0x106fe24f8] != 0xa7e). A live DM-root therefore needs
driving 0x2168798 to return a valid service object or populating the registry it reads — the
real-session construction path (SEP-17 SESSION-CTOR), not a `.bss` seed. Not re-tread beyond
this map.

## Verify / files

- New hermetic sh311 (elfjit.rs real-image guard, 5 byte-pins): ldarb 0x2206c7c=0x08dffd09,
  tbz 0x2206c84=0x36000469, once-head 0x2206d10=0xd0024300, DM-root store 0x2206d74=0xf90206e0,
  DM-ctor wrapper 0x2173b3c=0xa9bd7bfd. Skip-if-absent.
- elfjit.rs held under the 1MB pre-commit hook (further condensed render-prose; all addresses/
  constants preserved). Examples 136/0 (was 135).
- Repro runs/capture_sh311_once_lambda.sh; raw runs/sh311-once-*.txt (not committed).
- Commit: local `dev` only (operator pushes).