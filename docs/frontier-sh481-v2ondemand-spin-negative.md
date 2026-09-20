# Frontier SH481 — SH202 on-demand lever re-measured on the current substrate: arming it spins nativeInit (must NOT be armed); a vtable-NULL-slot hypothesis for the spin was falsified

Date: 2026-09-20, hermes-worker, single-agent (cone suppressed). recon-v3
immediate-priority deliverables re-verified green at SH480 HEAD first (STATUS +
fresh baseline). Workspace green before and after (cargo test --workspace
EXIT 0; arm64jit lib 685/0; elfjit examples green).

## The question

STATUS next-forward #1 (standing TOP) is the two session-substrate atoms that
still stop, nativeInitializeNativeFlags (0x10232048c) + V2InitWithParams
(0x102365c54) — the "outside image" singleton-dispatch family. SH200/201/202
built three declining levers for it: (a) SH200 4 fixed-site patch, (b) SH201 a
pre-scan family patch that CRASHED, (c) SH202 a runtime ON-DEMAND patch
(`JIT_ROUTEB_V2_ONDEMAND=1`) that patches only the exact site the run dispatches
through. SH202's doc describes on-demand as "deterministic site clearing." The
substrate env does NOT arm it. Question: **does arming it on the current full
substrate actually let nativeInit / V2Init COMPLETE (16/16), or is it still a
dead-end?**

## Measured: NO — arming on-demand SPINS nativeInit (a hang, not a completion)

Ran the complete SH415 substrate env + `JIT_ROUTEB_V2_ONDEMAND=1`
(runs artifact at /home/hermes-worker/runs/sh415-v2ondemand.txt):

- 6 on-demand sites patched at the stop (`routeb-v2ondemand ... patched V2
  singleton-dispatch` @0x106260bf4/0x106260ee0/0x1062550f4/0x1062661b8/
  0x1062503ac/0x106250b7c).
- nativeInit ADVANCES past the singleton wall it stopped at in SH480's static
  patch — it now logs the REAL engine line `nativeInitializeNativeFlags:
  Registered Flag Provider ID from Java` (a genuine step deeper than the static
  SH480 patch, which reaches that line then hits a deeper sibling site).
- **But then it NEVER returns.** Atom [1/16] is the ONLY atom that runs; atoms
  2-16 never execute. The run spins in nativeInit's flag-registration loop:
  **25,596 `nativeInitializeNativeFlags: ... %d: %s not found.` lines** in 180s
  (EXIT 124 = timeout, not crash). No LIVE-DM probe fires, substrate stays
  1/16.
- So on-demand does NOT "deterministically clear" the family on the current
  code. It converts "2 clean stopped atoms (substrate 14/16)" into "atom 1
  infinite-loops, nothing past it (substrate 1/16)". **It is a regression, not
  an advance. Do NOT arm JIT_ROUTEB_V2_ONDEMAND in the session substrate.**

### Root-cause hypothesis — FALSIFIED

Because nativeInit's spin is a flag-registration loop whose exit depends on the
flag-count/ptr-arg vtable slots 0x558/0x568 (which SH111's classifier marks
NullLeaf), I hypothesized the on-demand object's 0x60-only vtable left those
slots reading heap garbage (non-zero count -> loop never exits). I implemented a
trial fix: extend `v2_ondemand_object`'s vtable to [0,0x570) with ret-0 NULL
leaves at 0x558/0x568, plus a hermetic pin. **Falsified on the real binary**:
the spin persists identically (23,968 `not found`, only atom 1/16, EXIT 124).
nativeInit's flag loop is NOT gated by those singleton-vtable slots; it iterates
the engine's real flag table against an empty (content/Java-provider-dependent)
registry. The vtable experiment was reverted to keep the tree clean and under
the 1MiB hook; net change is a benign SH259 comment condensation (jit.rs
1,048,119 -> 1,047,818 B).

## Verdict

This is a genuinely-new measured NEGATIVE for the SH202 on-demand lever on the
*current* stack (SH202's doc claimed deterministic clearing; SH239's ladder used
it; but on the full ordered substrate + static SH480 patch it hangs nativeInit).
It does NOT change the Route-B live-DM gate (DM-root [0x106a68818]=0, structural
per SH462/467). It closes the "should we arm on-demand?" question with a
measured hang + a falsified root-cause, exactly the measured-verdict discipline.
The two stopped atoms remain the documented ~600-site singleton family; SH200's
judgment that it is non-Route-B-critical and not worth the discriminating
scanner holds, now reinforced by a direct hang measurement.

## Files
- jit.rs: comment-only SH259 condensation (kept — reduces bytes under the 1MiB
  pre-commit hook). Production code otherwise unchanged (the vtable experiment
  was reverted).
- runs/capture_sh415_v2ondemand.sh (added, then the on-demand variant) — scratch
  removed; the reproducible one-liner is the sh415 script + JIT_ROUTEB_V2_ONDEMAND=1.
- Commit on local dev only (operator pushes).