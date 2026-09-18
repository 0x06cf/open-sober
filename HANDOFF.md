# Open Sober — Agent Handoff
## SH335 (Sep 19, 2026, hermes-worker): timing-accurate closure on the "App"-registration question
## (candidate #1) via a count-transition reglive guard — measured 3/3 on the bus route that
## "App" is NEVER registered at any DM-ctor lookup; the fast-path matches only "Execute" -> no DM.
Turned the once-per-run `routeb_registry_live_guard` (jit.rs, JIT_ROUTEB_REG_LIVE=1, read-only,
default-inert) into a **count-transition** latch: it now dumps at guest pc 0x102168798 (DM-ctor
name->service lookup entry 0x2168798) whenever the service-registry count [0x106fe2f08] changes,
instead of firing once on the first (empty) lookup. Also page-safe-guarded the tier-2
fixidx/controller-cell reads (any_page_mapped) so the guard can't SIGSEGV on an unmapped page.
Measured (real libroblox.so, runs/probe_sh335_bus_reglive.sh, on the --v2boot-session-bus route,
3/3 deterministic): the lookup fires once per service registration, count 0->12, and at EVERY count
the registry holds ONLY the task-scheduler family — Thread (BG/FG), Spawn (BG/FG), Yield (BG/FG),
Close (BG/FG), Sleep, Sched, UNKNOWN — with "App" NEVER a registry entry (exact-token scan across
all runs: NONE), DM-root [0x106a68818]=0 at every lookup, and once-slot [0x106a68408] transitioning
0x0->0x400000b only at the terminal count=12 lookup (= the fast-path's match on "Execute", not
"App"). This closes SH334's open timing gap (its once-per-run guard only captured the empty first
readout); consistent with SH313/315/316/317/318. 2 hermetic tests kept green, now pass in parallel
(env-race SIGSEGV eliminated by page-safety). Workspace green (lib 410/0, elfjit 154/0, 590 passed
workspace, 0 fail). elfjit.rs byte-identical 1048570 B (unchanged). HONEST: no DM; Route-B live-DM
gate UNCHANGED; NEXT = real session drive that registers the "App" service (SH313/315/316/317/318)
so the DM-ctor fast-path resolves a live controller. +probe_sh335_bus_reglive.sh +frontier doc.