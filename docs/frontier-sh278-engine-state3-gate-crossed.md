# SH278 — cross the SH277 initEngine_ state gate: the engine's own receive sets state=3

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · opt-in `--v2boot-session-engine3` (default-inert) · workspace green

## What this is

SH277 pinned WHY a fabricated manager mono-tails: `initEngine_`'s state-dispatch (0x2bd1cf0)
reads the state word `[this+16]` and only state==3 (`bl 0x2bd1d68` settings serializer) / ==5 /
==9 enter a body; a fabricated/zeroed manager (state 0) falls through to the benign tail
(`mov x0,x19,#0x14; b pthread_mutex_unlock`). SH277 named the exact gate: "the settings path
needs the real session to set `[this+16]==3`, not re-feed inputs."

This cycle finds that the engine itself provides that transition: the SH276 engine-settings
receive (0x2bd1c38) body — `ldrb w8,[this,#649]; mov w9,#1; strb w9,[this,#648]; cbz w8,skip;
mov w8,#3; str w8,[this,#16]` at 0x2bd1cac/0x2bd1cb4/0x2bd1cb8/0x2bd1cbc/0x2bd1cc0 — sets
state to 3 **when `[this+649]!=0`**. Seeding that ONE byte on the fabricated manager makes the
ENGINE's own receive do the state→3 transition (a real session-state path, not a direct seed of
`[this+16]`), then driving the initEngine_ dispatch entry 0x2bd1cf0 takes its ==3 branch.

## What landed

`elfjit.rs` opt-in `--v2boot-session-engine3` (post-ladder rung, single ladder thread, SH55/64
serialized; default-inert):

1. sets version word `[0x10683d8f8]=6`,
2. allocates a leaked zeroed 0x800 manager, seeds `[this+649]=1` (the one byte the receive's
   state→3 transition keys on),
3. drives the engine-settings receive `0x102bd1c38`, then reads back `[this+648]`/`[this+16]`,
4. if state==3, drives the `initEngine_` state-dispatch entry `0x102bd1cf0` with that manager.

Hermetic `sh278` (real-image pins): receive's `ldrb w8,[this,#649]` (0x2bd1cac), `strb w1,
[this,#0x288]` latch (0x2bd1cb4), `cbz skip` (0x2bd1cb8), `mov w8,#3` (0x2bd1cbc), `str
w8,[this,#16]` (0x2bd1cc0); initEngine_ dispatch entry prologue (0x2bd1cf0), state-word ldr
(0x2bd1d08), cmp #3 (0x2bd1d1c), b.ne benign tail (0x2bd1d20), `bl 0x102bd1d68` serializer
(0x2bd1d28), serializer prologue (0x2bd1d68). skip-if-absent.

## Measured (real libroblox.so, SH269 full seed set, 3-run batch runs/capture_sh278_engine_state3.sh)

**Deterministic 3/3:** `EngSettingsReceived returned Ok(0x0): [this+648]=1 [this+16](state)=3`.
The engine's own receive transitions the fabricated manager's state to 3 (SH277's exact gate,
crossed by the engine's own code path). Then driving `initEngine_` dispatch 0x2bd1cf0 takes its
**==3 branch** (the mono-tail `pthread_mutex_unlock` benign return is no longer taken — the
process now SIGSEGVs inside the settings-serializer 0x2bd1d68 world-build instead of returning
clean), lr=0x102b53a78 (mutex-lock ret, inside the serializer's own `[this+0x14]` lock), through
a host-thunk deref.

## Honest framing

The SH277 state gate is CROSSED: state==3 is now reached via the engine's own receive, and the
==3 dispatch branch (settings-serializer entry) is entered — first session-state-driven entry
into the settings path, converting SH277's "a fabricated manager can never reach a settings
body" into a measured crossing at the exact pinned gate. It still does NOT manufacture a DM: the
serializer body 0x2bd1d68 immediately derefs live manager fields the zeroed 0x800 buffer doesn't
carry → SIGSEGV (the standing Route-B live-object wall, now reached one gate further, from the
settings path). MH_* stay false, DM-root 0; SH174 capture-latch (arm at a real
`make_shared<DataModel>`) remains the single forward hook.

## Verify

`cargo test -p arm64jit --example elfjit sh278` = 1 passed with real-image pins; examples
**111/0** (was 110); `cargo build --workspace` + `cargo test --workspace` EXIT 0. Repro:
`runs/capture_sh278_engine_state3.sh`. recon-v3 render plane stays verified green at HEAD.