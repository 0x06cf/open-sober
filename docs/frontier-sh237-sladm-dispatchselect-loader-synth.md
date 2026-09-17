# SH237 — StartLuaAppDM's receiveCall dispatch-SELECT is loader-synthesized .data.rel.ro (NOT session-gated); helper 0x1023f00f8 completes a real V2Init sub-body

Session: Sep 17, 2026 (hermes-worker). Real libroblox.so (host `~/.cache/open-sober/robbox/libroblox.so`,
109,193,800 B). +1 hermetic `sh237_startluaappdm_receivecall_dispatch_slot_union_zero_and_helper_realsub`
(real-image guard family as sh235/236; skip-if-absent). Workspace green (568/0; arm64jit lib 389/0;
elfjit examples 82/0). No production code path edited — pure recon + regression net. Single-agent (cone suppressed).

## Why (extend SH236's measured dispatch-return, close the un-pinned select + the helper's real body)

SH236 measured that StartLuaAppDM (0x1023efe2c) soft-returns headlessly in helper 0x1023f00f8 (last
block-entry 0x1023f01e4) before ever entering the marshaler-call block 0x1023f075c (`bl 0x1023f1210` ->
the sole EC-world front-door, SH235). SH236 pinned WHERE it returns but left two things un-pinned:
(a) the receiveCall DISPATCH-SELECT mechanism (what the entry union reads to pick its handler) and
(b) what helper 0x1023f00f8 actually does. This cycle closes both with byte pins + a loader-decoded
measurement that corrects SH235/236's *class* claim for the select.

## Measured (real libroblox.so; loader's own packed-RELA decode, authoritative)

### (A) The dispatch-select table is LOADER-SYNTHESIZED, not session-gated — corrects SH235/236's class

StartLuaAppDM entry builds a StartApp-params-like union on the stack:
```
23efe90: adrp x8, 635d000;  add x8,x8,#0xd68      ; x8 = 0x10635dd68 (the union table addr)
23efe9c: str  x8, [sp]                            ; [sp+0]  = 0x10635dd68  ("union first word")
23efea0: str  x20,[sp,#32]                        ; [sp+32] = sp            (self-ref)
23efea4: mov  x0,sp; mov w1,wzr; bl 0x2baeeec     ; fill the union (do-init/controller resolve)
...
23efed0: ldr x9,[x0]; ldr x8,[x9,x8]; blr x8      ; select = [0x10635dd68 + 0x20 or 0x28]
```
The select dispatch is `blr [0x10635dd68 + 0x20/+0x28]`. SH235/236 inferred this was a
session-populated / "fabricatable-live-graph" table (a SELECT can't be a static seed). This cycle
MEASURED it against the real image via the loader's own relocation decode (`read_elf_relocations`,
which handles this .so's packed-RELA — `readelf -rW` mis-reads it):

- On-disk raw file bytes in the union-table band [0x635d970,0x635e700) are all-zero. That is what a
  naive `readelf`/file scan sees, which is where SH235/236's "zero/session-populated" reading came from.
- BUT the loader synthesizes R_AARCH64_RELATIVE relocations IN that band at load. The band has many
  RELATIVE relocs; the two select slots specifically:
    - [+0x20] guest 0x10635dd88  -> RELATIVE addend 0x1db2cf0  = **__clone stub** (guest 0x101db2cf0)
    - [+0x28] guest 0x10635dd90  -> RELATIVE addend 0x21e96f8  = **invoke** (guest 0x1021e96f8)
- These are EXACTLY the std::function lambda-world pair SH231 mapped in the ExperienceController
  DM-creation machinery ("shared __clone stub 0x1db2cf0 / invoke 0x21e96f8"). So the union table
  StartLuaAppDM dereferences to pick its receiveCall handler is the EC lambda-world's own function-
  closure slots — already loader-synthesized real pointers, NOT a blank session table.

**Correction (do-not-re-advance SH235/236's class claim for the SELECT):** the receiveCall *select
slots* are NOT "session-gated / not-a-static-seed" — they are relocation-synthesized real code pointers
(the std::function __clone/invoke) and are fully available to a future drive at load time. The real
gate for reaching the marshaler 0x1023f075c is DOWNSTREAM of this select (the helper body, see B), not
the select slots themselves. This narrows where the Route-B effort must aim (and removes a wrong
"dead lever" vector).

### (B) Helper 0x1023f00f8 is NOT a benign-return stub — it COMPLETES a real V2Init sub-body

SH236 described helper 0x1023f00f8 as "reads stack flags [sp+8]/[sp+32], benign-returns". Fresh disasm
corrects that: those two reads are the SSO length/flag bytes of TWO libc++ std::string LOCALS the
helper constructs, and the helper then runs real V2Init machinery before its `ret`:
```
  23f00f8  sub sp,#0x70                       prologue
  23f013c  bl 0x2256510                       string-local init #1 (__sso into [sp+8] region)
  23f01b0  bl 0x2256510                       string-local init #2 (into [sp+32] region)
  23f01e0  bl 0x23c1574                       V2Init struct-copy (same fn the transition body dispatches to)
  23f01f0  bl 0x626b6d0                       FMOD-AAudio headphone-format notify (conditional on the SSO flag)
  ...      ldp/ret                            stack-canary + return
```
So the headless "soft return" is a COMPLETED sub-body that copies V2-init params and fires an audio
notification, not a flag-check stub that stops the flow early. This matches the SH235/236 Route-B
doctrine unchanged in substance (no DataModel is produced either way), but fixes the mechanism label
so a future drive doesn't chase a phantom "empty flag" fix.

## Code (sh237, real-image guard family as sh235/236, skip-if-absent)

Byte-pins: union first-word store 0x1023efe9c=0xf90003e8 (`str x8,[sp]`; x8 = the 0x10635dd68 table),
union self-ref store 0x1023efea0=0xf90013f4, helper string-local init #1 0x1023f013c=0x97f998f5,
string-local init #2 0x1023f01b0=0x97f998d8, V2Init struct-copy `bl` 0x1023f01e0=0x97ff44e5
(resolves to 0x1023c1574), FMOD-AAudio `bl` 0x1023f01f0=0x94f9ed38 (resolves to 0x10626b6d0).
Plus the loader-compared (A): asserts `read_elf_relocations` finds R_AARCH64_RELATIVE relocs in band
[0x635d970,0x635e700) AND that the two select slots 0x635dd88 (+0x20) and 0x635dd90 (+0x28) are among
them; eprintln reports the resolved targets (0x101db2cf0 / 0x1021e96f8). Guest=file+0x100000000
transform + 4-alignment for all six .text sites. No production path edited.

## Verdict (honest, do-not-over-claim)

Does NOT manufacture a DataModel, does NOT lift the Route-B live-DM structural gate (SH209/218/223/
224/228/231/232/235/236 unchanged). It (a) CONVERTS SH235/236's *static* "dispatch select is
session-gated / fabricatable-live-graph, not a static seed" into a MEASURED mechanism that the select
slots are RELATIVE-relocated real code pointers (the EC lambda-world __clone/invoke pair) — correcting
a wrong-dead-lever framing and re-aiming the next drive DOWNSTREAM toward the helper, and (b) corrects
SH236's helper label from "benign flag-check stub" to "completes a real V2Init struct-copy + FMOD
tail". Both keep the standing verdict (live-DM world-build = structural gate) but sharpen where the
load-time synthesis actually sits. Standing forward hooks unchanged (SH174 capture-latch arming at a
real make_shared<DataModel>; recon-v3 deliverables stay shipped + verified).

## Re-verify

`cargo test -p arm64jit --example elfjit sh237` = 1 passed (real-image guard).
`cargo test -p arm64jit --examples` = 82/0 (was 81). `cargo test --workspace` green (568/0).
`cargo build --workspace` EXIT 0. recon-v3 render plane re-verified green at HEAD (24 task-driven
frames swap Ok(0x1), 195 pops, no json abort, 0 crash). Tree clean at end of commit.