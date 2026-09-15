# SH180 — Route-B static-construction hunt CLOSED at the vtable level + migration-capture harness verified consistent (2-agent fresh cone)

Date: Sep 15, 2026, hermes-worker. Workspace green (550/0 at HEAD 485ee20 SH179).
Docs only — no production code change warranted. Commit <SH180COMMIT>.

## Why this session ran a fresh cone

Per the operator's Sep-15 directive, Route B (engine SELF-constructs its login/home
GuiObjects) was re-attacked with a fresh 2-agent READ-ONLY recon cone after SH179 pinned
the GENUINE DataModel vtable family. The standing "not seedable / structural wall" verdict
was re-examined from the new ground truth. The cone is authoritative and decisive.

## Agent 1 — the static hunt closes at the vtable-constant level (negative, decisive)

Followed SH179's named residual ("follow the object-vptr STORE from one of the three
candidate vtables, e.g. the 0x2b37b6c ctor, to close sizeof + the join"). Result: the
residual is now CLOSED as EMPTY — there is no ctor, no join, no in-image route.

- `0x2b37b6c` (`bl 2a0d9b8`) is inside fn `0x2b37b44`: it allocates **0x1108** then
  `memcpy(dst,src,0x1108)` — a clone/duplicate, never an object-vptr store. Its sibling
  `0x2b37a8c` allocs 0x1108 and calls `0x2b3ed04`, which writes fields at offsets
  64/72/88/108/168/184 only — **offset 0 (the vptr) is never written**. So the 0x1108
  factories never store a vptr at all → not DataModel.
- `0x28f78a0` = growable byte-buffer (0x1000 backing store at `[x19,#48]`);
  `0x2950b3c`/`0x2950b5c` = string-buffer container (two 0x1000 byte buffers). None stores
  a vptr.
- `2a0d9b8`/`2a0da80` are not `operator new` — they are **hookable allocation wrappers**
  (`[0x67daaf0]` hook else internal `0x1d96a40`; `2a0da80` memset's). No vptr anywhere.
- **Decisive static fact:** relative relocs reference ZERO of the candidate vtable
  addresses {0x67162f0, 0x67163a8, 0x6716400}; full-`.text` scan has ZERO `adrp+add` and
  ZERO `movz/movk` producing any of them; no dynamic symbol resolves at them; the only
  adrps to page 0x671000 (13, all in 0x29bd638–0x29be118) resolve to 0x671010–0x6710c4
  (unrelated). The DataModel vptr values are **unreachable from any static reference in the
  binary.**
- The three vtables are REAL populated objects (code-pointer slots 0x57ce740/0x240a8b8/
  0x57d07f8…, all carry T=0x6714e18 at their −0x10 slot) — they simply have **no writer**.

### Verdict
There is **no in-process jit-runable route to a live (typeinfo-correct) RBX::DataModel**:
constructing one requires writing one of the three vptr values into an object's first word,
and no reachable instruction/reloc/GOT/symbol can produce them. Driving `nativeGameGlobalInit`,
`StartLuaAppDM`, the 0x2206c40→0x2206db8 do-init, ExperienceController, or `V2StartAppWithParams`
changes nothing — none materializes the DataModel vptr. The Route-B gate is structural /
migration-gated at the deepest level yet. The operator-requested "re-examination of the
'not seedable' wall via do-build completion or a dynamic DM-ctor trace" is answered
definitively NEGATIVE: there is **no dynamic DM-ctor to trace**, because no reachable
constructor stores the vptr at all. (SH178's veto was attribution-based; SH180's is
reference-based and stronger.)

Residual NOT statically-ruled-out (noted for completeness, contradictions closed): an engine
path that *copies* a vptr from an already-live object during a data-driven clone — but the
seed object's vptr would still have to originate from one of these unreferenced vtables, so
that path cannot produce a live DM either.

## Agent 2 — migration-capture harness verified internally CONSISTENT at HEAD (positive)

The one genuine forward road to a live DM is a real app-launch session on the GPU host
(capture → validate → arm → probe). Confirmations at HEAD (485ee20):

- (1) Capture latch constants/env all correct — `jit.rs:1299/1322/1324` (env),
  1278–1280 (block pc 0x102a0d9b8 / active 0x1067daaf0 / default 0x1067d0840),
  1200–1204 (delegation via run_guest_callback current_guest_tp()), 1228 (budget 256),
  1315–1317 (idempotent latch).
- **KEY CROSS-CHECK PASSES:** `read_vt_in_image` (`jit.rs:1248–1260`) accepts any vptr ≥ base
  (0x100000000) and within image_len (109,193,800 B). The genuine family
  `0x1067162f0 / 0x1067163a8 / 0x106716400` all satisfy it → a genuine captured DM base logs
  `[validated]`. (Predicate does not read vtable content — on-disk cells are zero,
  loader-populated — which is irrelevant to acceptance.)
- (2) Arming target stays **0x106391908** (getter 0x2dbcc10 re-disassembled: `adrp x0,
  6391000 / add x0,x0,#0x908 / ret` → &0x6391908); runbook `frontier-sh174:67/103` uses it and
  refuses the old 0x106391918. SH172 correction holds.
- (3) Acceptance probe `(R+0x188 − R+0x180)/0x28`, walker 0x105b2ed48, render_scene_base==0 —
  consistent (`jit.rs:11501–11558`, `11467–11468`, `11522–11523`; runbook STEP F).

## Conclusion / standing

- Route-B static construction is now CLOSED at the vtable-reference level (strongest, most
  complete negative in the lineage — replaces all prior "no seed" verdicts).
- The migration harness is verified READY and will observe + validate a genuine live DM the
  instant one forms on the GPU host.
- No production code change warranted this cycle (forcing one would be feature-flag cruft).
  Workspace green 550/0.
- Next (unchanged, GPU-host migration): capture with `JIT_DM_ALLOC_CAPTURE=1 [+] DELEGATE=1`,
  arm `*(0x106391908)=B`, probe R+0x180/0x188 for the first engine-self-constructed scene node.