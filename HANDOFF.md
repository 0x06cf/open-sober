# Open Sober — Agent Handoff
## SH323 (Sep 18, 2026, hermes-worker): advance the SH320/321 MAIN-path engine-settings-init line
## TWO more fenceposts + cross a SECOND lifecycle-notify copy — now reaching StartAppWithParams
## internals (0x102256510 live-object wall), five consecutive crossings since SH321.
## JIT_ROUTEB_SETTINGS_SSO_SEED=1 seeds two NULL cookie/string globals with routeb_empty_sso_string:
## CELL_A [0x106ed7a18] at pc 0x1021f5078 (whitespace-check fn 0x21f5078 ldrb [x8] fault=0x0); CELL_B
## [0x106ed7a28] at pc 0x1025f36ac (StartAppWithParams `bl 0x221364c` returns x0=[0x106ed7a28], ldrb
## [x0] @0x1025f370c — fire at the confirmed block entry 0x1025f36ac, NOT the mid-block fault pc).
## Also SH322's early-ret guard now covers the SECOND lifecycle-notify copy 0x1021f4538 (identical
## sub sp,#0x90 / ldr [x1] / ldrb [x8,#80] / tbnz->canary+ret @0x21f4638). A/B (real so,
## ab_sh323_settings_sso_seed.sh, both arms SH320+SH322): BASELINE first SIGSEGV 0x1021f5078;
## FORWARD seed323=2 (both cells), wall gone, terminal advances 0x1021f5078 -> 0x1025f370c ->
## 0x102256510 (fault=0x0, JNI-receive-style fn reading [x0] with x0=0 = live-object class).
## Five consecutive crossings on one settings-init line: 0x1021f3748 -> 0x1021f5078 -> 0x1025f370c
## -> 0x102256510. +hermetic sh323 (env/pc-gated, both cells seed when NULL, live refs untouched)
## + sh322 extended (second lifecycle copy) + sh323_..._pinned elfjit real-image guard. Verify:
## sh322+sh323 2 passed; elfjit examples 148/0 (147+sh323); arm64jit lib 408/0; workspace green;
## build EXIT 0; elfjit.rs 48 B under 1MB hook (condensed SH-prose). HONEST: no DM (DM-root 0,
## MH_* false); new terminal 0x102256510 is the live-object class (JNI-receive entered with NULL
## this, SESSION-CTOR cave); Route-B live-DM gate UNCHANGED; SH174 latch single forward hook.
## Doc docs/frontier-sh323-settings-sso-line-advances.md. Single-agent, default-inert.