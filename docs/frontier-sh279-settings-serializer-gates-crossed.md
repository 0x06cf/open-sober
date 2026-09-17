# SH279 — cross the settings-serializer world-build gates: the initEngine_ state=3 path SELF-DRIVES into app-start depth

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · extends `--v2boot-session-engine3` (default-inert) · workspace green

## What this is

SH278 crossed the SH277 initEngine_ state gate: a fabricated manager with `[this+649]=1` makes the
ENGINE's own engine-settings receive (0x2bd1c38) set `[this+16]=3`, then driving the initEngine_
dispatch 0x2bd1cf0 takes its ==3 branch into the settings-serializer body. But SH278's terminal was
a deterministic SIGSEGV: the serializer world-build deref'd fields of the zeroed 0x800 manager and
died on `fault=0x300` before reaching its real body.

This cycle root-causes **both** serializer internal walls and crosses them, so the state=3 settings
path now runs the serializer's WHOLE body headlessly for the first time and **self-drives into
deep app-start** — the first time the engine's OWN session-state path (not the fabricated DMCONT
ladder) reaches app-start depth.

## The two crossed gates (both measured, both seedable)

**Gate 1 — the settings-config NULL lock (`fault=0x300`).** The serializer body 0x2bd1d68:
- `ldr x0,[x19,#64]` reads `[this+0x40]` — the manager's settings-config object pointer = 0 on the
  fresh zeroed 0x800 mgr3 → calls 0x2bcdfc4 with `x0=0`.
- 0x2bcdfc4 (the config **lock + field-copy** helper): `add x0,x0,#0x300; bl 0x2b53a68` =
  `pthread_mutex_lock([obj+0x300])` → locks guest addr `0x300` → deterministic SIGSEGV
  (`fault=0x300`, guestpc host-thunk read-shim, register dump reproducible 3/3).

Fix (in the `--v2boot-session-engine3` rung): give `[this+0x40]` a leaked **zeroed 0xc00
settings-config object** so the lock is a valid `PTHREAD_MUTEX_INITIALIZER` and the helper's
field-copy (0x2bce010, x0=stack-AppStarted, x1=obj+0xf0) lands in the buffer.

**Gate 2 — the empty app-name NULL-store (`fault=0x0`, guestpc=0x102bd1fd4).** After Gate 1,
the serializer's own `bl 0x2b504e4` (0x2bd1dfc) overwrites `[this+0x48]` with the app-name, empty
from the zeroed flags-holder → the app-name guard at 0x2bd1f64 (`ldrb w8,[x19,#72]`, `ldr
x9,[x19,#80]`, `csel`/`cbnz`, then **deliberate** `mov x8,xzr; strb 0x61,[x8]` = write 'a' to [0])
faults at 0x2bd1fd4. This is the SH245/SH248c M+0x48 guard class.

Fix (two parts): (a) pre-seed `[this+0x48]` as a LONG "Home" std::string (cap 0x11 bit0=1, [8]=size 5,
[16]=leaked "Home\0" data) so the serializer-assign preserves the long bit; (b) generalize
`routeb_cont_appname_seed_guard` to re-seed the **live `this` (x19)** at block-entry 0x102bd1f64
(`[this+0x50]=5`), so it also covers the engine3 rung's fresh manager (previously it only re-seeded
the DMCONT `routeb_cont_managed_m()`).

## Measured (real libroblox.so, full SH269 seed env, runs/sh279_batch.sh)

**Deterministic 3/3:** `state=3`, SH279 config seed fires, app-name pre-seed fires, app-name guard
re-seed fires, and the terminal moved from SH278's `guestpc=host-thunk fault=0x300` (settings lock)
to **`guestpc=0x101db1d04`** — the standing LSM insert-leaf wall (SH260). The LSM_NODES variant
(SH267) crosses it to `0x101db1b08` (free-list, one deeper). The initEngine_ state=3 serializer now
runs its ENTIRE body (config lock + field-copy + app-name guard + post-guard) and control flows on
into nativeAppBridgeAppStart-depth self-drive.

A/B at this head (runs/sh278-ab-baseline.txt): WITHOUT `--v2boot-session-engine3` the ladder
completes clean (EXIT 124, no crash) — the crossing is attributable to the state=3 dispatch path.

## Honest framing

The SH279 crossing makes the **engine's own session-state settings path** (state→3 via the engine's
receive → initEngine_ ==3 branch → full serializer body) execute headlessly and self-drive into the
same app-start depth the DMCONT manufactured-ladder reaches (SH260/267 LSM wall). This is
cause-not-symptom progress on the SEP-17 SESSION-CTOR primary lever: the settings config-state gate
is no longer a wall. It still does NOT manufacture a live DataModel: DM-root[0x106a68818] stays 0,
MH_* stay false, and the app-start path terminates on the known LSM persistence wall (SEP-15
parked detour) / live-object class beyond. SH174 capture-latch (arm at a real make_shared<DataModel>)
remains the single forward hook.

## Verify

`cargo test -p arm64jit --example elfjit sh279` = 1 passed (real-image pins); examples **112/0**
(was 111); `cargo build --workspace` + `cargo test --workspace` EXIT 0 (578 arm64jit + all crates).
`sh279` pins Gate-1 (`ldr x0,[x19,#64]` 0x102bd1de8, `bl 0x2bcdfc4` 0x102bd1df8, helper `add
x0,#0x300` 0x102bcdfd4, `bl mutex_lock` 0x102bcdfdc, field-copy prologue 0x102bce010) and Gate-2
(app-name guard 0x102bd1f64/0x68/0x78, NULL-store 0x102bd1fd4/0xd8). elfjit.rs trimmed comments to
stay under the 1MB pre-commit hook (1,048,530 B < 1,048,576). Repro runs/sh279_batch.sh +
runs/sh278b-trace.txt + runs/sh278-ab-baseline.txt. Single-agent, default-inert.