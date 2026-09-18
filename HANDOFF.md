# Open Sober — Agent Handoff
## SH324 (Sep 18, 2026, hermes-worker): ANSWER STATUS candidate #1 — the terminal 0x102256510 IS the
## SESSION-CTOR drive point, reached into a StartLuaAppDM continuation body SH156 never mapped;
## pinned on the real image + PROVED the fabricated-object seed is a dead-end on this wall (x8
## out-param ABI). Next work = real session-ctor drive (SEP-17 primary lever), not another seed.
## Summary of the finding:
## - Determinism: the SH320-323 MAIN-path engine-settings-init line stops at guestpc=0x102256510
##   on 3/3 clean runs (EXIT 139/134/134). Not run-variable — a stable wall.
## - The wall is reached inside the REAL StartLuaAppDM body: fn 0x23f00f8 (= StartLuaAppDM+0x2cc,
##   a nested `sub sp,#0x70` tail frame, fall-through after `bl 2b9ec9c`) — FURTHER than SH156's
##   documented trace (SH156 mapped 0x23efe2c..0x23eff40 + the do-init br to 0x1023eff4c / governor
##   dispatch; it did NOT map 0x23f00f8). Six consecutive crossings since SH320.
## - Fn 0x2256510 (faulting) = member method `this->vt[+32]()` with this=[container+40]==NULL;
##   container = x1 arg of fn 0x23f00f8. BOTH call sites read the same field (0x23f012c
##   `ldr x9,[x1,#40]`; 0x23f01a0 `ldr x0,[x21,#40]`). this=0 -> `ldr x9,[x0]` fault=0x0.
## - PROOF-OF-DEAD-END for fabricated-this: after `blr vt[+32]` the fn does `ldr x0,[sp,#8]`
##   (consumes an x8 indirect-result out-param). A host-thunk leaf receives only x0-x7, so it
##   cannot write the x8 out-slot -> fabricated-this drifts into 222a9fc/LocalStorage RN. The ONLY
##   cross is a REAL vt[+32] on a REAL object = the container's +0x40 member built by the upstream
##   session ctor. This is the "specific gate provably unprocessable by JIT seeds" PROOF form.
## - New sh324 hermetic real-image guard (8 anchors: fn 0x2256510 prologue+dispatch, both call
##   sites, post-dispatch out-param read + LocalStorage bl). elfjit examples 149/0.
## Verify: sh324 1 passed; elfjit 149/0; arm64jit lib 408/0; cargo test --workspace EXIT 0.
## Doc docs/frontier-sh324-sessionctor-drive-point.md. HONEST: no DM (DM-root 0, MH_* false);
## Route-B live-DM gate UNCHANGED. Next: do-init/DMCONT continuation + Activity-session drive so
## the container's +0x40 member is constructed for real (SEP-17), NOT fabricated.