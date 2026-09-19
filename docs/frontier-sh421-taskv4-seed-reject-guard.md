# SH421 — taskv4-seed REJECT guard: the type-4 vector recap

Date: Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Workspace
green (cargo test --workspace EXIT 0; arm64jit lib 477 -> 478 with the new
sh421 hermetic; cargo build --example elfjit OK). Production code in
session.rs (off the 1MiB hooks, 62KB) + elfjit.rs (pulled back under the hook
to 1,048,392 B by condensing two SH-prose comments, addresses kept);
jit.rs untouched (1,048,466 B < hook).

## What and why
recon-selfdrive-seed-jsonfix.md §A's "Reject" rules were DOCUMENTED but never
enforced in code. The `--taskv4-seed <guest-hex>` raw branch (elfjit.rs ~8943)
accepted ANY guest address as the type-4 vector handler, including the
engine's own dispatcher 0x10285371c (which re-enters the popped-task deque
infinitely) and the drain pop-loop 0x102856e40 / producer 0x10285682c (which
are re-entrant — self-drive while draining). Seeding those turns a bounded
task-frame run into runaway recursion on the builder thread. This is the
IMMEDIATE-PRIORITY recon-v3 deliverable's own safety rule, rediscovered
unguarded.

## Code
- session.rs: `pub fn taskv4_seed_rejected(addr: u64) -> bool` — pure guard,
  matches the three recursive engine entries. Host-thunk addrs (0x7f..) and
  the engine frame-fn (0x105b32c00) are NOT rejected (the thunk calls the
  frame-fn via run_guest_callback; it is never itself the vector entry).
- elfjit.rs: the raw-hex branch parses `h`, and if `taskv4_seed_rejected(h)`
  refuses it — `eprintln REJECTED ... vector left 0` and seeds 0 (cleared)
  instead of the recursive addr. The `--taskv4-seed frame/session` host-thunk
  and valid `probe` paths are untouched.
- New hermetic `session::tests::taskv4_seed_rejects_recursive_engine_entries`
  (arm64jit lib 477 -> 478): denies the three engine entries, accepts the
  host-thunk base + engine frame-fn.

## MEASURED (real libroblox.so)
Rejected path — full render env, `--taskv4-seed 0x10285371c`:
```
[elfjit:taskv4] REJECTED seed 0x10285371c (engine dispatcher/drain/producer recurses) — vector left 0
[elfjit:taskv4] seeded dispatcher type-4 vector [0x106829ea8] = 0x0 (cleared)
```
Valid path — same env, `--taskv4-seed 0x105b32c00`:
```
[elfjit:taskv4] seeded dispatcher type-4 vector [0x106829ea8] = 0x105b32c00 — a popped task node reaching w4=4 will now call it
```
So a recursive engine entry is now refused (vector left 0) and a valid hex
seed still installs — no regression on the working seed path. Canonical
recon-v3 frame capture re-run green at this HEAD (see STATUS).

## Honest
Not a DM. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_GAME_LOADED false; re-confirmed this cycle by the SH415
do-init probe under the complete substrate: MH_APP_READY/MH_ENGINE_INITIALIZED
latch true, AppBridgeV2 genuine vt 0x1063a3410, DM-root still 0 — the runtime
surface is complete and exercised, but the live DM needs a real session ctor
the JIT cannot reproduce headlessly). recon-v3 deliverables unchanged-green.
No re-treads (taskv4-seed hardening is the deliverable's own unwired safety
rule, not a Route-B cone).

## Files
- docs/frontier-sh421-taskv4-seed-reject-guard.md
- crates/arm64jit/src/session.rs (+taskv4_seed_rejected + hermetic)
- crates/arm64jit/examples/elfjit.rs (reject wiring + 2 comment condenses)