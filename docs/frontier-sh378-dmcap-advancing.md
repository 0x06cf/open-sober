# Frontier SH378 — SH174 DM-allocation capture latch (CAPTURE-ONLY, no delegate) stays silent
# on the furthest-advancing env (SH377 crossing+GOVFLAG+PRELOAD+PACK_SKIP, SendAppEventOnAppReady
# to full RETURN): 0 validated make_shared<DataModel>, terminal drains into the closed LSM pool-pop
# lane 0x101d9a528 — the single forward hook still does not fire at the farthest reach

Session: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One new probe
runs/capture_sh378_dmcap_advancing.sh + this doc. No production path edited (probe uses ONLY
existing default-inert guards: SH373 crossing + SH269 GOVFLAG + SH307 PRELOAD_VALUECELL + SH349/350
skips + SH174 JIT_DM_ALLOC_CAPTURE). Workspace green (cargo test --workspace EXIT 0, 436/0 arm64jit).

## The genuinely-new intersection measured this cycle

SH344 measured the SH174 DM-allocation capture on the SH343 env with JIT_DM_ALLOC_CAPTURE_DELEGATE=1
— the delegating hook whose host-side run_guest_callback dispatch is disruptive on the deep full
ladder and died with std::bad_function_call (EXIT 139) BEFORE a clean readback of whether
make_shared<DataModel> ever fires. So nobody has ever taken a clean capture readback on the
furthest-forward write.

This cycle runs the SH377 ADVANCING env (the only one that drives SendAppEventOnAppReady to full
RETURN past governor + preload + pack) with the CAPTURE-ONLY safe latch (JIT_DM_ALLOC_CAPTURE=1,
NO DELEGATE — the guard seeds only when no hook is installed). Result (real libroblox.so):

- `SendAppEventOnAppReady returned Ok(0x107273d50)` — the farthest the send-appevent path has ever
  gone IS reached under the capture probe (~10 attempts spooled to the advancing env; the run then
  continues past it).
- **`grep -c "[validated]"` = 0** AND the capture trail is never installed (no `routed ... capture
  trail` / `FIRST call#` allocation-traffic lines: the only `bytes=` hit is the SH339 w19 jstring
  readback, NOT an allocation). => The operator-new wrapper OP_NEW_WRAPPER itself is never entered
  with a hook-installable state on this env, and no validated in-image-vtable DataModel allocation
  ever occurs.
- Terminal: `guestpc=0x101d9a528 fault=0x0` (EXIT 134 SIGABRT after SIGSEGV) — the LSM pool-pop
  write site, the SAME measured-closed SH350/SH341 persistence family the loop has crossed into and
  closed repeatedly. The run does not reach the governor/app-shell-ctor path (0 hits) nor Lua.

## Interpretation (do-not-over-claim)

This is map-completion at the most-forward reach, not a crossing:
- The single SH174 forward hook (the DM-allocation capture-trail) does NOT fire even on the env that
  returns SendAppEventOnAppReady cleanly. The furthest possible headless advance still drains into
  the closed persistence lane before any make_shared<DataModel>.
- It also closes the one gap in the SH344 record (clean capture readback): with the disruptive
  DELEGATE absent, the capture-ONLY latch is byte-inert on this env too — the trail never even
  installs, meaning the JIT never reaches OP_NEW_WRAPPER in an installable state on the advancing
  write. Confirms "no DM allocates" independent of the SH344 delegate artifact.
- Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_FLAGS_LOADED/APP_READY
  false, AppBridgeV2[0x106a705e8]=0x0). No DataModel manufactured. SH174 capture-latch stays the
  single forward observer.

## Do-not-re-tread (unchanged)

- Do NOT re-drive LSM sub-call skips deeper (SH349/350/358/373/375/377/378 stand — whack-a-mole
  unbounded).
- Do NOT re-attack the EC reader-gate (SH356/374), 0x258b5d8/SetInitParams (SH362/375), the
  window-attach once-guard (SH367), the ALooper loop (SH365).
- Do NOT re-arm JIT_DM_ALLOC_CAPTURE_DELEGATE for a "cleaner" readback — SH344 already proved it
  disrupts the deep ladder; the capture-ONLY readback here is the clean artifact.

## Files

- runs/capture_sh378_dmcap_advancing.sh (probe), runs/sh378-dmcap-advancing.txt (live capture,
  gitignored).
- No Rust edited. Commit: local `dev` only (operator pushes).

## Repro

objdump -d `aarch64-linux-gnu-objdump -d --start-address=0x101d9a500 --stop-address=0x101d9a5b0`
(LSM pool-pop write site) to see the closed-lane terminal this env drains into.