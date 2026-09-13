# SH94 — don't clobber the INSERT-entry x19 walk iterator (kills the 6.6M-iteration registrar spin)

## Result (wall advance: the post-SH93 gameGlobalInit spin is GONE — the registrar loop now terminates)

Read-only recon deleg_3f812009 (disasm-verified) root-caused the post-SH93 state: after
the CEvent barrier NOP, the do-init's post-barrier body runs cleanly to its `ret` at
file 0x2206e9c (CEvent dtor 0x221a720 + canary check, NO 2nd CEvent wait, NO loop). The
6.6M-iteration spin was PRE-BARRIER, in the pb/Otel registration, and it was the SH92
substitute itself:

- SH92 (jit.rs) substituted the registry map for a non-family map/this into BOTH `x0`
  AND `x19` at the INSERT entry (0x1029f3e70).
- But in the registrar loop (file 0x29b37e0: `mov x1,x19; bl 29f3e70; ldr x8,[x19,#16]!;
  cbnz x8,<loop>`), `x19` is the **callee-saved WALK ITERATOR / KEY** — NOT the map.
  INSERT's prologue reloads its map from `x0` (`mov x19,x0` at 0x29f3e98), so substituting
  `x19` is useless to the call. But because `x19` is callee-saved, overwriting it returns
  the corrupted value to the CALLER, whose `[x19+16]` then reads the substitute's +0x10 ==
  SPAN_HASH (nonzero) FOREVER -> the registrar's `cbnz` never terminates -> the 6.6M-
  iteration spin (`SH92 substituted ... 0x7f57b4000d90 ... reg=x19` = the seeded-map cast
  to cursor+0x10).

**Fix (jit.rs):** at the INSERT entry, substitute `x0` unconditionally (when non-family);
substitute `x19` ONLY when `x0` is ALSO non-family (the SH92 .data-table crash case, where
x19 held the sibling real span map). A real family map's x0 is never substituted, so the
registrar's callee-saved x19 iterator is left intact and the loop terminates normally.

**Verified:** the 6.6M x19-substitute flood is GONE (subx0 count is now a sane 49; the
registrar loop terminates). The v2boot run advances past the spin but now hits a
shallower/different crash (guestpc 0x1029b37ec registrar back-edge or 0x1029f4034
INSERT-tail, fault==rip — execution into a heap pointer; run-variable 134/139) — the
registrar loop now actually executes its real body and a deeper fn-ptr dispatch fault
surfaces. Workspace **516/0**. Product path unregressed (exit 124, persist byte-exact, 0
crash). The fix is gated behind JIT_ROUTEB_HASHFIX (--v2boot). Regression
`routeb_hashfix_substitutes_nonfamily_map_at_insert_entry` extended to assert x19 is NOT
clobbered when x0 is a family map.

## Repro

`runs/capture_v2boot_sh82.sh`. Expect only a handful of `SH92 substituted ... reg=x0`
lines (NO 6.6M reg=x19 flood), and the ladder faulting at the NEW shallower dispatch.

## Next (ranked)

1. The remaining 0x1029b37ec/0x1029f4034 crash: the registrar loop now terminates and
   runs its real body; the fault==rip==heap is a deeper fn-ptr/va-dispatch inside the
   (now-executing) registration or the INSERT-tail new-node path. Trace which blr/br now
   jumps into the heap (likely a freshly-allocated per-node or a registration callback
   fn-ptr slot holding host garbage). Determine the slot -> seed to a real in-image fn or
   host-call-bridge (SH81/SH86-style), or interpose.
2. Goal: gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) ->
   rungs 2-6 install type-4 vector [0x106829ea8].
3. Wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.