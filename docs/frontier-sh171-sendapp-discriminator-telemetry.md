# Frontier SH171 — SendAppEventOnAppReady discriminator: w19 pin re-verified; path is telemetry-only

## Status
Recon-only (read-only disasm, two independent agents + direct objdump here).
No production code change. Workspace green (545/0). Canonical ladder stable on this tree.

## CORRECTED w19 mapping (verbatim disasm, this tree)
The 4-char compare at 0x2bb46e4 builds the magic with THREE instructions, not
a single 32-bit load — subagent 1 and the SH171 doc DROPPED the `add w10,w10,#0xe01`:
```
2bb46f0  mov  w10, #0x6147
2bb46f8  movk w10, #0x656d, lsl #16     ; 0x656d6147
2bb4700  add  w10, w10, #0xe01          ; 0x656d6147 + 0xe01 = 0x656d6f48
2bb4704  cmp  w9, w10
2bb4708  b.eq 0x2bb47c4                 ; -> w19=4\n\n0x656d6f48 little-endian = 'H','o','m','e' = "Home"
```
So **"Home" -> w19=4** (via `mov w19,#0x4` @0x2bb47c4). The operator's original
premise ("Home" in x5 -> w19=0x4) is CORRECT. Corrected full map:
- 0x656d6f48 "Home" -> w19=4  (b.eq 0x2bb47c4)
- 0x74616843 "Chat" -> w19=1  (b.eq 0x2bb47cc, `mov w19,#0x1`)
- 0x65726f4d "More" -> w19=5  (csel fallback)
- len 0xc "AvatarEditor" / len 0x5 "Games" -> earlier len branches (w19=2/0/3)

## The pin is STILL telemetry-only, NOT a Route-B gate
Continuation past the discriminator (0x2bb47d0..0x2bb49e4, verified by the deeper
agent): version-gate on 0x683d350/0x683d358 -> `bl 0x21fd00c` FLog
"[FLog::JNIAppBridge] ... eventFeature = {}." -> build a local 0x58 event struct
(w19 at +0x50) -> dispatch 0x2baeeec -> free 4 strings. NO DataModel / governor /
ScriptContext / MessageBus construction. w19=4 vs 1 vs 5 changes only the logged
integer. So this path is NOT a session-advance lever regardless of the w19 value.

## Empirical discrepancy SETTLED (3rd agent + re-check): no jstring-production bug
The earlier "empty RBX-string at [sp]" observation was an INVALID reading: it
probed `boot_sp-0x140` AFTER jit_run returned, i.e. the function's frame had been
unwound and the stack bytes were stale — not the live in-function string. A fresh
read-only trace refutes all four dead-end causes for an empty string:
(1) handle ABI-correct (x5 -> x19 -> helper x1, non-zero str_handle); (2) builder
0x1d9d074 provably emits a len-4 "Home" SSO for a 4-char input (strlen<0x17 ->
len<<1 + memmove); (3) env x0 is build_jni's, whose slot 169 = identity shim that
returned non-zero (JIT bridge stores it into x0, so 0x21e201c `cbz x0` cannot
fire); (4) no other jstring->RBX path. Net: the fabricated-jstring wiring is
CORRECT; any observed empty/w19-1 mismatch points at hostcall dispatch for the
guest-bl-entered nested helper frame (jit.rs:2978-2999), which is only reachable
when the rung actually nested BL's — not a jni.rs or call-site defect. Since the
whole path is telemetry-only, this is not chased further.

## Actionable takeaway
- w19=4 == "Home" (operator premise CORRECT; the earlier SH171 "Home->5" note was a
  doc bug fixed here).
- Still not a Route-B acceptance marker: the continuation is FLog + local event stat.
- Live-DM structural wall unchanged (~19 recon angles). Next real forward remains the
  migration-gate app-launch with JIT_DM_ALLOC_CAPTURE / JIT_DM_ALLOC_CAPTURE_DELEGATE.