# SH164c — DMDRIVE vt-shell replay: ANTI-VERDICT (decisive, read-only recon deleg_876cb7b2)

Date: Sep 15, 2026, hermes-worker. Workspace green (540/0). Persists the negative so
future cycles do NOT re-attempt a vt-shell constructor replay.

## Question
Can the harness write a guest-resident shell into impl[+0x408] with
shell_vt[+0x30] = a real DM-creator address so the governor-tail's own `blr x8`
calls the real creator (JIT_ROUTEB_DMDRIVE) and constructs a live RBX::DataModel?

## Verdict: MECHANICALLY TRIVIAL, SUBSTANTIVELY A DEAD END — no live DM results
The vt-shell mechanics are already proven (the SH164 hermetic test fabricates
shell_vt[+0x30]=0x102bd1a38 and wires the tail). Seeding impl[+0x408] with a fake
object whose vtable slot is a literal is easy. But the creator method is STATEFUL
and cannot run on a fake instance; and no loader-resolved creator vtable exists to
repoint.

## Evidence (full packed ANDROID_RELA decode, 568,272 relocs)
1. **Zero relocations of any type resolve into the DM-creator family**
   [0x102bd1a30, 0x102bd1d08). There is NO .data.rel.ro vtable whose +0x30 slot (or
   any slot) points at getFlagsFromEngine_/initEngine_. The nearest code-refs are a
   NativeDataModelManager vtable cluster 0x635fb30..0x6360200 whose methods span
   0x2bd00xx..0x2bd1654 — the creator funcs lie PAST the cluster's last slot (in a
   ~0x3DC gap). The creator funcs are non-virtual / runtime-dispatched, not in a
   relocatable vtable. `shell_vt[+0x30]` MUST be a literal code address
   (initEngine_ 0x102bd1cf0 or getFlagsFromEngine_ 0x102bd1a38).
2. `relocated_value()` is NOT a real helper (no repo hits) — the pattern is inline
   read_unaligned in routeb_tail_dispatch_capture; for this family it reads stale
   zeros.
3. **`initEngine_` dispatch + the null-settings fault (exact offsets):**
   - Prologue: x19=this; first calls 0x2b53a68(this+0x14).
   - Dispatches on **w8 = [this+0x10]** at 0x2bd1d08: w8==3->body 0x2bd1d68
     (settings-serializer); ==5->0x2bd24b4; ==9->0x2bd2668; **else (incl 0) ->
     benign default tail 0x2b53abc**.
   - **A ZEROED SHELL (w8=0) NEVER ENTERS a settings body** — it returns via the
     benign tail. NO DM work at all.
   - Body 0x2bd1d68 derefs globals [0x67d16f0] + [0x683d8f8], then
     `ldr x9,[x19,#0x50]; cbnz x8 -> skip` — if **[this+0x50]==0** it falls through
     to the `'Engine settings is null'` FLog/abort (0x2d2308 / 0x21fd00c).
   - Seeding shell[+0x50]=nonzero dodges ONLY the log; the body then serializes
     members +0x48/+0x60/+0x78/.../+0x240 (7 calls to 0x2b504e4) and the first
     uninit pointer-member deref faults next. No path constructs a DM on a fake.

## Most probable outcome if attempted
- Zeroed shell (w8=0): initEngine_ returns benign tail — NO DM work.
- w8=3 + +0x50 unset: aborts 'Engine settings is null'.
- w8=3 + +0x50 set: faults soon after on serializer member derefs (worst case the
  0x21fd00c FATAL).
NO live RBX::DataModel under any shell variant.

## Conclusion
Skip DMDRIVE as a DM-construction path. It adds nothing beyond (already-proven)
tail->literal dispatch + pinning the +0x10 dispatch field / +0x50 settings field for
a CONTROLLED gate that still aborts. A live DM still requires a harness DYNAMIC
TRACE driving a REAL instance through the governor tail — this remains the standing
Route-B structural wall. Stays: do NOT re-chase [0x10683cf38] no-op, synthetic
CoreScript dead, DataModelPatch feeding (downstream), DMDRIVE vt-shell (this).

## Files
- docs/frontier-sh164c-dmdrive-antiverdict.md (this file)
- Scratch (not committed): /tmp/dmprobe/, /tmp/dmrecon/, /tmp/sh163 (extracted .so).
- No production code changed (recon only). Workspace 540/0.