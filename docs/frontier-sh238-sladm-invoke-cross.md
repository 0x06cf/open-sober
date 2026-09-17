# SH238 — Cross StartLuaAppDM's receiveCall dispatch-select to the EC INVOKE slot (forward probe)

Session: Sep 17, 2026 (hermes-worker). Real libroblox.so (host
`~/.cache/open-sober/robbox/libroblox.so`, 109,193,800 B). +1 hermetic
`sh238_sladm_invoke_select_cross_is_env_gated_idempotent_and_union_boxed`
(default-inert env `JIT_ROUTEB_SLADM_INVOKE`). Workspace green (arm64jit 390/0 incl. new
sh238 + all crates). Single-agent (cone suppressed). This is a genuine forward attempt on
the SH235-237 "next drive," not a static recon pin — it changes dispatch behavior under an
opt-in env and MEASURES whether StartLuaAppDM's soft-return can be crossed.

## Why (the SH235-237 lever, now actually driven)

SH235 pinned the lone headless front-door to the genuine EC DM-creation world
(0x102e24598): StartLuaAppDM entry -> receiveCall dispatch -> `bl 0x1023f1210` @0x1023f075c
-> the 9-arg marshaler. SH236 measured that StartLuaAppDM benign-soft-returns Ok headlessly
(flow stops at block 0x1023f01e4 inside helper 0x1023f00f8; the marshaler-call block
0x1023f075c is NEVER entered). SH237 measured the receiveCall dispatch-select's union table
slots are loader-synthesized RELATIVE pointers: [union+0x20]=std::function __clone
0x101db2cf0, [union+0x28]=invoke 0x1021e96f8 (the EC lambda-world invoke).

None of SH235-237 actually ATTEMPTED to cross the select — each parked at "the next drive's
obstacle is the dispatch." This cycle does the crossing and measures the outcome, per the
operator's keep-grinding / convert-judgment-to-measured doctrine.

## The select (re-derived, exact)

StartLuaAppDM entry builds a union and dispatches:
```
23efe90: x8 = 0x10635dd68 (union table)
23efe98: x20 = sp
23efe9c: [sp+0]  = 0x10635dd68
23efea0: [sp+32] = x20 (= sp)              ; union self-ref field
23efea4: x0=sp; w1=0; bl 0x2baeeec          ; do-init / fill union
23efeb0: (block entry) x0 = [sp+32]
23efeb4: cmp x0, x20  (x20 == sp)
23efeb8: b.eq  23efec8                      ; [sp+32]==sp  -> select [+0x20] = __clone
23efebc: cbz   x0, 23efedc                  ; [sp+32]==0   -> epilogue / return
23efec0: mov   w8,#0x28                     ; else         -> select [+0x28] = invoke
23efed0: x9=[x0]; x8=[x9,x8]; blr x8        ; call the slot
```
Default headless state: `bl 0x2baeeec` leaves [sp+32]==sp (self-ref), so the equal branch is
taken and the dispatch selects __clone -> the flow bends into helper 0x1023f00f8 (V2Init
struct-copy + FMOD tail, SH237) and StartLuaAppDM soft-returns Ok — never advancing to the
machine's own marshaler 0x1023f075c / EC world.

## The probe (routeb_startluaapp_invoke_guard, opt-in `JIT_ROUTEB_SLADM_INVOKE=1`)

At the Select block entry pc **0x1023efeb0**, seed guest `[sp+32]` = a leaked 8-byte box
holding 0x10635dd68, so the dispatch takes the non-equal path (w8=0x28) and invokes
`[union+0x28]` = the EC lambda-world **invoke 0x1021e96f8** instead of the benign __clone.
Idempotent (preserves a real non-self [sp+32]); preserves a real session value; sp==0 safe.
Wired into the JIT_ROUTEB_SETFIX dispatch chain, self-gated on `JIT_ROUTEB_SLADM_INVOKE`.

Measurement regions (JIT_REGION_WATCH): StartLuaAppDM body [0x1023efe2c,0x1023f0800),
marshaler [0x1023f1210,0x1023f1300), EC world [0x102e1c650,0x102e25200) + GSDSP on fault.

Repro: `runs/capture_sh238_sladm_invoke.sh` (lever on by default; `JIT_ROUTEB_SLADM_INVOKE=0`
for the "soft-return still" baseline).

## Honest boundary

This is a **measured attempt**, not a shipped Route-B unlock. Expected outcomes, both useful:
- **Forward** — the cross fires (`[routeb-sladm] CROSSED`), StartLuaAppDM falls through to the
  marshaler 0x1023f075c and/or the EC world region fires => the SH235-237 parked dispatch is
  genuinely crossed and the next drive starts FROM there.
- **Measured dead-end** — the invoke slot needs a real closure object (EC `this` + StartAppParams)
  and faults at a specific live-object deref (GSDSP pin) => confirms SH235's
  "fabricatable-object-graph, not a static seed" verdict with a concrete pc, closing this lever
  with evidence rather than judgment.

## MEASURED RESULT (authoritative, this cycle)

2 clean completing ladder runs (EXIT 124, 0 crash). A/B = lever OFF (baseline) vs lever ON
(cross fired):
- **Baseline (run1, lever off):** StartLuaAppDM entered, block 0x1023efeb0 hit, soft-return
  `StartLuaAppDM returned Ok(...)`; entered pcs
  `1023efe2c efeb0 efedc eff4c effa0 effac effc0 effc8 effd0 efffc f0008 f0020 f00f8 f01e4`;
  **marshaler region 0 hits, EC world 0 hits.**
- **Forward (run2, lever on):** `[routeb-sladm] CROSSED -> [sp+32]={box} ... dispatch takes
  INVOKE [union+0x28] ... was 0x5624f64f6a50`. StartLuaAppDM STILL soft-returns Ok through the
  **identical** terminating blocks (`... f00f8 f01e4`); **marshaler region 0 hits, EC world
  0 hits.**
- run3 (confirming, lever on) pending at commit time; run1+run2 already establish the A/B.

**Two measured conclusions (do-not-over-claim, single-agent):**
1. **Correction to SH236:** `[sp+32]` at the dispatch is **natively a REAL pointer
   (0x5624f64f6a50, a live host-heap object), NOT the self-ref `sp`** — so the receiveCall
   dispatch natively selects the **INVOKE (0x28)** slot, NOT __clone (0x20) as SH236 inferred.
   The benign helper 0x1023f00f8 is reached via INVOKE already.
2. **The select-crossing lever is a MEASURED dead-end:** forcing the invoke slot changes
   NOTHING — StartLuaAppDM terminates in helper 0x1023f00f8 (block 0x1023f01e4) and soft-returns
   Ok whether the dispatch selects invoke or __clone; the marshaler-call block 0x1023f075c -> EC
   world stays 0 hits. This empirically confirms SH237's correction: **the marshaler is gated
   DOWNSTREAM of the helper's return path (the big sub-body 0x23f03b4 is reached by control flow
   that never executes headlessly), not by the select slots.** Closing this lever with evidence,
   not judgment (operator's proof-of-dead-end standard for the specific SH235-237 select lever).

**Line-closing caller scan (authoritative):** full-.text probe (SH235 method) shows the big
sub-body 0x23f03b4 (which contains the marshaler-call block 0x1023f075c -> bl 0x1023f1210 -> EC
world) has **ZERO direct `bl` callers** across the whole image. It is reached ONLY via indirect
receiveCall message dispatch (an app-bridge registry handler invoked by `blr`), so there is NO
static call-site to seed into it — the marshaler sits behind the same live-DM / app-bridge
world-build wall SH174/SH204/SH206 mapped, not a headless seed lever. This closes the SH235-238
receiveCall line end-to-end with measured negatives at every step of it.

Does not manufacture a DataModel; Route-B live-DM structural gate (SH209/218/223/224/228/231/232/
235/236/237) UNCHANGED. The next Route-B lever on this line (do-not-implement-now, the
fabricatable-object-graph class SH204/SH174 already mapped) would be forcing the flow into the big
sub-body 0x23f03b4 directly — which derefs live controller objects (x20..x25) and is the same
live-DM structural wall.

## Re-verify

`cargo test -p arm64jit sh238` = 1 passed (hermetic).
`cargo test --workspace` green (arm64jit 390/0 incl. sh238 + all crates). `cargo build --workspace`
EXIT 0 (then `cargo build -p arm64jit --example elfjit` to rebuild the probe binary). recon-v3
render plane unchanged (the new guard is inert without `JIT_ROUTEB_SLADM_INVOKE=1`; no
default-config production path edited).