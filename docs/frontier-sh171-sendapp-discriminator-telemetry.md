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

## EMPIRICAL residual (open, low priority): fabricated jstring resolves EMPTY at runtime
Probe on the real binary: driving the rung with `new_string_utf_handle(b"Home")` in
x5 produced w19=0x1 AND the event RBX-string at [sp] was SSO len 0 (empty), which
would require GetStringUTFChars (slot 169) to have returned 0 (the `cbz x0 ->
empty` path at 0x21e201c/0x21e204c). Current source wires slot 169 to the identity
shim `jni_get_string_utf_chars` (jni.rs:1034/570) which returns the handle untouched
(non-null). The two agents disagree on why the run shows empty: agent-2 (source
trace) says the wiring is correct in the tree so the empty implies a stale/un-hit
slot; agent-1 (broader disasm) did not isolate it. Since the path is telemetry-only,
resolving this is low value vs the Route-B live-DM wall. Reserved pending a real
session if a genuine jstring payload ever needs to reach that FLog.

## Actionable takeaway
- w19=4 == "Home" (operator premise CORRECT; the earlier SH171 "Home->5" note was a
  doc bug fixed here).
- Still not a Route-B acceptance marker: the continuation is FLog + local event stat.
- Live-DM structural wall unchanged (~19 recon angles). Next real forward remains the
  migration-gate app-launch with JIT_DM_ALLOC_CAPTURE / JIT_DM_ALLOC_CAPTURE_DELEGATE.