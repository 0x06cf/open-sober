# SH217 — READY-TO-FIRE correction of the SH161b governor-tail mode-2 seed window

Status: implemented + hermetic-verified at HEAD. Single-agent (Route-B cone suppressed).
Real binary: /home/hermes-worker/.cache/open-sober/robbox/libroblox.so (guest = file_vaddr +
0x100000000). Workspace green (cargo test --workspace EXIT 0).

## The defect (measured, not assumed)

SH161b (recon deleg_61f88e9a) seeds impl[+0x2b8]=2 so that fn 0x1023c12c0 — reached from the
governor-tail epilogue `mov x0,x19; mov w1,#0x2; bl 0x1023c12c0` — takes its benign `b.eq` mode-2
no-op (stack-canary + ret) instead of falling into its fault-prone transition body (reads
[0x6a70700] version-state, dispatches 0x23c14dc/0x23c1504/0x23c1574, and conditionally enters the
FMOD-AAudio 0x626b6d0 path whose run-variable crash site is guest 0x106240d8c = SH212 crash A).

SH161b's guard window was `if pc != 0x102e9fe04 { return }` — a STRICT single-pc assertion that the
call-site block is entered at 0x102e9fe04. A region-watch of the completing --v2boot ladder
(JIT_REGION_WATCH=0x1023c12c0-0x1023c1500,0x102e9fe04-0x102e9fe40,0x102e9fcc4-0x102e9fdc8) shows
this premise is WRONG:

- The governor tail is translated as ONE block whose entry point is 0x102e9fcc4.
- fn 0x1023c12c0 IS re-reached on the completing ladder (entry 0x1023c12c0 + transition body
  0x1023c1384/0x1023c13ac/0x1023c13c0/0x1023c13c8 all hit, and the b.eq canary-ret terminal
  0x1023c1434 hit) — the tail block-drop lift (elfjit.rs block_cache_drop_region
  [0x102e9fcb0,0x102ea3b40], SH160/SH161) WORKS.
- The bl ret-landing 0x102e9fe0c is a block entry; 0x102e9fe04 itself is NEVER a block entry.
- Therefore `routeb_tail_eq_guard` fired ZERO times during the run, leaving the transition body
  (and its conditional FMOD-AAudio dispatch) live on every completing ladder.

## The fix

Changed `routeb_tail_eq_guard`'s window from `pc == 0x102e9fe04` to the tail-region entry window
[0x102e9fcc4, 0x102e9fdc8] — the SAME window routeb_tail_dispatch_guard uses, i.e. the operator's
named "exact SH159c/routeb_tail_dispatch_guard pattern". The seed now lands at the real tail-block
entry, BEFORE the `bl 0x1023c12c0` executes, so fn 0x1023c12c0 takes its mode-2 no-op early-return
and the whole transition body (incl. the SH212 crash-A FMOD-AAudio dispatch) is bypassed.

Unchanged semantics: idempotent (only writes impl[+0x2b8] when ==0, preserves a real nonzero
session value), reads x19 as the impl base exactly like routeb_tail_dispatch_guard, default-inert
(no env-gate change — it fires under JIT_ROUTEB_SETFIX as before). Pure window correction, no ABI
or seed-value change.

Hermetic `sh161b_routeb_tail_eq_guard_seeds_impl_2b8_and_leaves_real` updated to the corrected
window: 0x102e9fc00 (out-of-window) does not seed; 0x102e9fcc4 (in-window real tail entry) seeds 2;
0x102e9fdb0 preserves a live nonzero 7; impl==0 doesn't deref. No weakening — assertions tightened
to the measured entry point.

## Verify

- cargo test --workspace: arm64jit lib green, sh161b hermetic green.
- Rerun completing ladder w/ region-watch: fn 0x1023c12c0 entered + b.eq terminal 0x1023c1434 hit,
  0 crash, EXIT 124; now with the SH161b `impl[+0x2b8] seeded =2` log line present (previously zero).

## Standing (do-not-re-tread, unchanged)

Route-B live-DM = structural gate (SH209 do-not-re-tread). recon-v3 render plane green at HEAD
(capture_taskv4_frame.sh: 24 task-driven frames, 197 node pops, no json abort, 0 crash, EXIT 124).
This cycle is a readiness run: it makes the installed SH161b lever actually fire (the operator's
POST-SH161 "NEXT GATE" implementation), hardening the ladder against the SH212 FMOD-AAudio crash-A
class when the transition body would otherwise execute. It does NOT manufacture a live DM.