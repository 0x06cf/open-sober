# Frontier SH171 — SendAppEventOnAppReady discriminator: w19 pin DECISIVE (telemetry-only)

## Status
Recon-only (read-only disasm deleg_76cbd070), no production code change.
Workspace green (545/0). Canonical 9-rung ladder stable on this tree.

## The pin (operator ROUTE-B step-2): "confirm w19-event=0x4 for 'Home' in x5"
ANSWER: the w19 discriminator is a **payload-label selector, not a session gate**.
It maps the event's first 4 bytes to an integer; the entire continuation past it is
**FLog telemetry + a local 0x58 event struct only** — it constructs NO DataModel /
governor / ScriptContext / SendAppEventMessageBus. w19=4 vs 1 vs 5 changes nothing
about executed instructions beyond the logged/embedded integer. This is NOT a
Route-B unblock; chasing it is cosmetic.

## Empirical mapping (real binary, file vaddr 0x2bb463c)
Event RBX-string at [sp] (len-12/len-5/len<-4 decode):
- 0x2bb4660 `mov x19,x5` — x5 (event jstring) is the 4th/out-ref -> [sp]
- 0x2bb46b8 discriminator head:
  - len==0xc -> b.eq 0x2bb4778        (e.g. "AvatarEditor", w19=2)
  - len==0x5  -> b.eq 0x2bb4738        (e.g. "Games", w19=0/3)
  - len!=0x4  -> b.ne 0x2bb47bc        (w19 fallback branch)
- len==0x4 falls into the 4-char compare (first 4 bytes):
  - `0x656d6147` = "Game" -> b.eq 0x2bb47c4  => **w19=4**
  - `0x74616843` = "Chat" -> b.eq 0x2bb47cc  => **w19=1**
  - cmp `0x65726f4d` = "More" -> csel mismatch => **w19=5**
- **"Home" (0x656d6f48, len 4) matches NONE** -> w19=5, NOT 4.

So the operator's "Home in x5 -> w19=4" premise was WRONG. Home -> w19=5.
(w19=4 is "Game".) The probe harness run confirms the event string at [sp] was
**empty** (SSO len 0 -> fallback w19=0x1 path) because the fabricated jstring
handle resolves to an empty RBX-string here — the discriminator sees len 0.

## Why w19=1 fired empirically
The harness passes `new_string_utf_handle(b"Home")` as the jstring; the helper
0x21e1fec calls GetStringUTFChars (vtbl slot 169 = identity shim, returns the
handle) then builds the RBX-string at the dest. On the real run the built string
at [sp] was all-zero (len 0), so the discriminator took the `b.ne 0x2bb47bc`
fallback -> w19=1. To actually hit `mov w19,#0x4` the event must be the literal
"Game" (not "Home"). Not worth forcing — see next.

## Continuation after discriminator (0x2bb47d0..0x2bb49e4)
- version gate on globals 0x683d350 / 0x683d358 (NOTE: NOT 0x683d348)
- `bl 0x21fd00c` FLog: "[FLog::JNIAppBridge] ... eventFeature = {}. " (stride
  0x35f000+0x74a) carrying w19 as value
- builds local 0x58 struct (`bl 1d96768` new #0x58, 4 RBX-strings via 1d9d8b0,
  w19 written at [x20,#80] @0x2bb4948), virtual dispatch 0x2baeeec + blr 0x2bb4984,
  frees the 4 strings via 0x626b6d0.
Purely a telemetry/event-stat object. No DM construction. Both w19 branches
converge on the SAME continuation.

## Actionable takeaway
- Do NOT treat `w19-event=0x4` as a Route-B goal or acceptance marker.
- The SendAppEventOnAppReady path is telemetry post-signal; it does not advance
  the DataModel / app-shell / GuiObject session.
- Live-DM structural wall unchanged (recon consensus ~19 angles). Next real
  forward remains the migration-gate app-launch with JIT_DM_ALLOC_CAPTURE / DELEGATE.