# Open Sober — Agent Handoff
## SH322 (Sep 18, 2026, hermes-worker): CROSS the SH273 lifecycle-notifier live-object wall that
## the SH320/321 MAIN-path binder-dispatch reaches — engine settings-init now advances past it.
## First forward on STATUS candidate (a). The do-init DONE-path MAIN dispatch (0x206df4 -> vt+0x30
## -> br x1, SH320 flips the thread-match gate) climbs into nativePostClientSettingsLoadedInitialization3
## (caller 0x2270024 -> 0x2270050 bl 0x21f3748) and SH321 measured it faults fault=0x50 at the SH273
## shared lifecycle-notify body 0x21f3748 (`ldr x8,[x1]` @0x21f376c + `ldrb w8,[x8,#80]` @0x21f3770)
## because arg0's first word [x1]==0 (host-heap controller not-yet-constructed). SH322 (opt-in
## JIT_ROUTEB_LIFECYCLE_EARLYRET=1, jit.rs block-entry guard at exactly 0x1021f3748) seeds the caller
## pair [x1] = leaked obj with byte[+80].bit1=1 so the NEXT insn `tbnz w8,#1,0x21f3870` @0x21f3774
## jumps STRAIGHT to the epilogue canary-check+ret (benign no-op) — bypassing the whole registry-build
## body. A/B (real so, ab_sh322_lifecycle_wall_cross.sh): BASELINE EXIT 139 SIGSEGV @0x1021f3748
## (fault=0x50); FORWARD guard fires 4x, that wall is GONE, first SIGSEGV advances to guestpc=0x1021f5078
## (fault=0x0, EXIT 134). New terminal 0x1021f5078 reads global std::string [0x106ed7a18] (adrp 6ed7000
## + #2584; ldrb [x8] faults fault=0x0) — the .bss cell immediately below the SH248d cookie-jar globals
## [0x106ed7a20/0x28], seedable (SH323 natural next). +hermetic sh322 (jit.rs unit: env/pc-gated, seeds
## [x1] pair when 0, obj byte[+80].bit1 set, leaves non-zero [x1] + NULL pair untouched) + sh322_..._pinned
## elfjit real-image guard (tbnz 0x1021f3774=0x370807e8, early-ret 0x1021f3870=0xf94002c8 + 0x1021f3890=
## 0xd65f03c0, caller 0x102_270024=0xa901a3e9). Verify: sh322 1 passed; elfjit examples 147/0 (146+sh322);
## arm64jit lib 407/0; cargo test --workspace EXIT 0; cargo build EXIT 0; elfjit.rs held 25 B under the
## 1MB hook (condensed SH-prose comments, facts/addresses preserved). HONEST: does NOT manufacture a DM
## (DM-root [0x106a68818]=0, MH_* false); the engine-settings-init body advances past the SH273 wall but
## dies one fencepost later at the SH248d-class .bss std::string region; Route-B live-DM gate UNCHANGED;
## SH174 capture-latch stays the single forward hook. Doc docs/frontier-sh322-lifecycle-wall-crossed.md.
## Single-agent, default-inert.