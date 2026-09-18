# Open Sober — Agent Handoff
## SH334 (Sep 19, 2026, hermes-worker): LIVE answer to the standing "App"-registration question (candidate
## #1) via a default-inert block-entry registry readout at the DM-controller ctor's name->service lookup —
## measured 3/3: the registry is EMPTY at the lookup on the SH332-style MAIN path, so "App" is not registered
## before the (run-variable) crash; the post-ladder dump that previously provided the only readout is no longer
## the gate.
Added `routeb_registry_live_guard` (crates/arm64jit/src/jit.rs, opt-in `JIT_ROUTEB_REG_LIVE=1`, read-only,
fires once) that snapshots service-registry count + entry names + DM-root [0x106a68818] + once-slot
[0x106a68408] + tier-2 controller-name cell at the EXACT moment the DM-ctor lookup (fn 0x2168798, entry
0x102168798) runs. Measured (real libroblox.so, runs/probe_sh334_reglive.sh, 3/3 deterministic):
`service-registry-count[0x106fe2f08]=0 entries=[] DM-root=0 once-slot=0 fixidx0=0 tier2-cell=""` at the lookup.
This RESOLVES candidate #1's "open but UNCHANGED" status: on the SH332-style MAIN-path run the registration-walk
(0x21e2a90) + lookup (0x2168798) execute and the +0x408 cross fires, but the registry is EMPTY when the ctor
looks up "App" — the ctor fast-path cbnz still misses, DM-root stays 0. Terminal remains run-variable
(FMOD 0x106240cb8 / LSM 0x101dcab68 / leaked-host-pc, SH330/332-class); the guard now makes the readout
survive it. 2 new hermetic tests (arm64jit lib 408->410: inert-without-env, wrong-pc-miss). elfjit.rs
byte-identical 1048570 B (unchanged, 1MB hook). Workspace green (lib 410/0, elfjit 154/0). HONEST: no DM, no
manufactured DataModel; Route-B live-DM structural gate UNCHANGED; SESSION-CTOR "App"-registration remains the
standing unblock, now byte-measured at the exact decision point. NEXT remains SESSION-CTOR "App" registration
(SH313/315/316/317/318) via a real Activity/AppBridge session drive.