# Open Sober — Agent Handoff
## SH326 (Sep 18, 2026, hermes-worker): re-verified the two recon-v3 deliverables (type4_frame_thunk
## self-driven frames + json-zero-fix) are ALREADY landed + green at HEAD (no new code needed), and
## pinned the standing 0x1025f5300 gate's SEP-17 forward contract. Verified: 24 task-driven frames,
## present #19..#23 swap Ok(0x1), 197 node pops, 0 json abort, EXIT 124. The gate 0x1025f5300 is
## deterministic 3/3 (fault=0x140); cross = [appstart_obj+24]!=NULL. Extended sh325 hermetic guard with
## 2 new real-image pins (0x1025f52ec=bl-nativeAppBridgeAppStart-0x233bf20, 0x1025f52f0=ldr x19,[sp,#328]).
## MEASURED PREMISE REVERSAL (SH326c): the AppStarted+24 construction write (0x2e89150 str x0,[x19,#24],
## inside factory 0x2e890c4, reached via bl 233bf20 once-path guard [0x106a6f490]) FIRES headlessly —
## JIT_DUMP_PC=0x102e89150 shows x0=0x5588d0f085e0 (real heap obj) x19=0x106a6f468 written to [0x106a6f480].
## The SH324-325 "cross unconstructible headlessly / only real-Java builds it" premise is OVERTURNED.
## Gate still faults because construction is NONDETERMINISTIC: 0x2e890c4's branch `bics xzr,x8,x0; b.eq
## 0x2e89108` + `lsr x8,x0,#62; cmp #2; b.ne 0x2e89118` gates construction on a producer-counter tag from
## bl 2baaac0 (tag==2 -> unconstructed return; else -> construction loop). So construction happens only
## when that counter returns a non-tag-2 value at the right time — same run-variable class as the
## SetInitParams drain. NEXT: force the construction branch deterministically (gate 2e890e4/2e890f4 so it
## always falls through to the 2e89118 construction loop) OR drive 2e890c4 synchronously + re-read
## [0x106a6f480] after bl 233bf20 returns. elfjit examples 150/0, arm64jit lib 408/0, workspace green,
## elfjit.rs under 1MB hook. HONEST: no DM (DM-root 0, MH_* false); Route-B live-DM gate UNCHANGED.