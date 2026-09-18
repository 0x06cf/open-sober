# Open Sober — Agent Handoff
## SH332 (Sep 19, 2026, hermes-worker): NEW combined MAIN-path reach measured on one run — the SH320
## do-init done-path MAIN binder-dispatch + SH322/323 lifecycle/Settings cross + SH330 app-start +0x408
## cross all on a single jit_run, terminating at the FMOD/AAudio live-object wall (0x106240cb8); the
## registration-walk + name->service lookup execute on this path, yet the "App" tier-2 cell is unchanged.
Measured (real libroblox.so, runs/probe_sh332c_mainpath_regok.sh, full SH330 recipe, NO
--v2boot-skip-appstart, single ladder run, 2/2 deterministic in guard+reach): `guard408=2`,
`fork_501c=1`, `fork_5060=1` (app-start +0x408 gate crosses ON the do-init MAIN path, not just the
harness --startapp route SH330 originally documented), `lifecycle_seed=22` (SH322 early-return across
both 0x1021f3748+0x1021f4538 copies), `sso_seed=2` (SH323 settings-SSO cells), `lookup_hits=4`
(name->service lookup 0x1021687cc), `regwalk_hits=2` (service-registration walk fn 0x21e2a90). The run
terminates SIGSEGV at `guestpc=0x106240cb8 fault=0x0` = FMOD Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged
tail (bl 0x1d97414 -> tbnz w0 @0x6240cb4/8), then SIGABRT via leaked host-pc (SH320-class). That is the
SAME run-variable downstream SH330 documented (RUN2=0x106240cb8) — a live-object / direct-JNI this
misconstruction (SH212/213/132: FMOD does not dlopen libaaudio; bridge never engages), NOT a seedable gate.
HONEST: no DM (DM-root [0x106a68818]=0, MH_* false); no new stable gate; the "App" controller-name cell
([0x107027170+i] fixidx -> "Runtime0" invariant, SH317/318) unchanged — the lookup+walk execute but the
app-start continuation dies at the FMOD/AAudio wall before the registry's "App" entry is written. Route-B
live-DM gate UNCHANGED. Recon-v3 re-verified green (24 frames, present #19..#23 swap Ok(0x1), 192-197
pops, 0 json abort, EXIT 124). NEW repro: runs/probe_sh332c_mainpath_regok.sh. No Rust source changed
(after removing 2 methodologically-flawed probe variants — their incompatible --v2boot-skip-appstart
rung set made the +0x408 fork unreachable); elfjit.rs byte-identical 1048570 B. NEXT remains SESSION-CTOR
"App" registration (SH313/315/316/317/318) so the DM-ctor fast-path resolves a live controller; the new
combined-reach confirms the MAIN path reaches+crosses the +0x408 fork before the FMOD wall.