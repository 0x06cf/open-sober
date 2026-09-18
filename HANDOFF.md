# Open Sober — Agent Handoff
## SH339 (Sep 19, 2026, hermes-worker): measured negative answering SH308's open Step-2 ABI question — the fabricated "Home" jstring does NOT route to event-code 4 (falls to 0).

Single-agent (cone suppressed). New default-inert read-only guard `routeb_appevent_w19_guard`
(jit.rs, opt-in JIT_ROUTEB_APPEVENT_W19) captures SendAppEventOnAppReady's event discriminator
INPUT MID-EXECUTION at its real block boundary 0x102bb46b8 (region-watch-verified), which the
post-return `se.x[19]` read (SH308) could not do (x19 callee-saved/restored → always 0).
MEASURED 4/4 real-libroblox.so runs: [sp].b0=0x0c → libc++ SSO size 6 → discriminator's
`cmp #0xc/#5/#4` falls through to event-code **0**, NOT 4. The fabricated handle bytes at
x19/x5 ARE `486f6d65` ("Home") — source correct — but the RBX string the router reads at
[sp] is not "Home", so the harness's 'Home' event does not reach the router as such.
IMPORTANT refinement: the do-init pipe forward-reach (SH307/308 app-shell ctor, FMOD audio
tail, StartAppWithParams, app-data-model count 0→1) STILL fires with event-code 0 in every run
— it is driven by the SH126 sync-gate, NOT by correct 'Home' routing, so the session-advance
is robust to the discriminator result. Does NOT manufacture a DM (DM-root 0, MH_* false);
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
Source = +guard only (jit.rs, 76 lines), default-inert. Workspace green (build EXIT 0, tests
590/0). +frontier doc +runs/capture_sh339_appevent_w19.sh +captures sh339-w19{b,g,h,i}.txt.
HONEST: a measurement (opens no new gate), not a session advance; it closes the last open
ABI sub-question SH308 carried on the do-init pipe's event side and proves the pipe does not
depend on the fragile fabricated-jstring round-trip.