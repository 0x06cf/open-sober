# Open Sober — Agent Handoff
## SH337 (Sep 19, 2026, hermes-worker): measured negative — V1 nativeAppBridgeAppStart__ driven
## against a POPULATED registry (count=12) still does NOT register "App"; the "App"-registration
## SESSION-CTOR lever stays the standing Route-B wall.

Single-agent (cone suppressed). New default-inert rung `--v2boot-postbus-v1appstart` (elfjit.rs)
drives SH336's pinned V1 AppStart__ ABI AFTER `MessageBus.subscribe` populates the service registry
0->12 — the genuinely-untested combination (every prior V1 drive ran from count=0 and died at the
empty lookup). MEASURED (real libroblox.so, clean, EXIT 124, 0 crash):
- SH337 bus Ok(0x3e8); SH337 V1 AppStart__ Ok(0x3e8) — V1 AppStart runs clean on 12 entries.
- post-V1: registry STAYS 12 (task-scheduler family byte-identical), "App" NEVER among the entries,
  DM-root [0x106a68818]=0x0, once-guard=0x1 (already latched from the bus).
- => driving V1 AppStart__ on a populated registry does NOT make the "App" service register.
  Consistent with SH313/315/316/317/318/335: "App" + its tier-2 controller-name cell (Runtime0)
  are built only by a deeper live session ctor, not a bare app-start entry.

Re-verified at HEAD (no regression): recon-v3 self-driven frames (24 frames swap Ok(0x1), 0 json
abort, EXIT 124); SH335 reglive closure (registry 0..12 task-scheduler-only, "App" never an entry).
Workspace green (elfjit examples 155/0, arm64jit lib 410/0, ws 0 failed). elfjit.rs 1048470 B under
1MB hook (new rung + entries dump funded by condensing SH-prose comments; no test weakened).
HONEST: no DM (DM-root 0, MH_* false); Route-B live-DM structural gate UNCHANGED; SH337 is a
measured-negative closure on the SESSION-CTOR line, NOT a session advance. NEXT (unchanged): the
real Activity/AppBridge session drive that registers the "App" service — now with V1-AppStart-on-
populated-registry explicitly excluded. +frontier doc +probe runs/probe_sh337_v1appstart_postbus.sh.