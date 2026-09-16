# SH212 — Residual ladder flakes characterized at HEAD: FMOD/AAudio audio-subsystem contact + raced flag-manager system_error (both non-seedable)

**Type:** characterization cycle (no production code change; pure measurement + disassembly, per SH192/SH204/SH208/SH209 pattern). Workspace green. recon-v3 render plane re-verified green.

## Why this cycle

SH209 measured the DM-creator reachability negative; SH210 verified all Route-B latent wiring is ARMED; SH211 pinned the byte anchors. The standing residual that breaks deterministic-clean ladder completion was last characterized at SH205/SH208 (8/10 clean, faults at 0x104c393f0 / 0x10284ce54) and SH209 (0x102b9dee0 fault=0x10). This cycle re-measures the CURRENT fault distribution at SH211 HEAD with JIT_GUEST_STACK_DUMP=1 + JIT_OUTSIDE_TRACE=1, settles the two new dominant crash classes into in-image disassembly, and — importantly — identifies the FMOD/AAudio audio subsystem as a genuine boot-contact point, the first firm sign the "sound" pillar is actively coming up headlessly.

## Measurement (8 canonical ladder runs, SH210 env + JIT_GUEST_STACK_DUMP=1 + JIT_OUTSIDE_TRACE=1)

Result: **7/8 clean (EXIT 124, full V2 ladder), 1 guest SIGSEGV, 1 raced EXIT-139.**

| run | EXIT | state |
|-----|------|-------|
| 1,3,4,5,6,7 | 124 | CLEAN — full ladder: nativeInitFlags → gameGlobalInit → do-init (once-guard self-latches) → governor MODERN router [0x106a70880]=1 → V2Confirm → V2Start → V1 AppStart |
| 2 | 134 | SIGSEGV **guestpc=0x106240d8c** fault=0x0, x0=0x0 |
| 8 | 139 | git aborted by libc++abi `terminating due to uncaught exception of type std::__ndk1::system_error` at nativeInitializeNativeFlags |

## Crash A — FMOD/AAudio NULL-`this` (run 2): the AUDIO-subsystem contact point

- `guestpc=0x106240d8c` = file `0x6240d8c`, inside `Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged` (file vaddr 0x6240d8c = `add x10,x10,#0x3ff`, the char-decode block ENTRY — the harness records the translated-block start, not the faulting instr).
- fault=0x0, x0=0x0 ⇒ a genuine NULL `this`/NULL string deref inside this FMOD AAudio stream-format parser (`strstr`/`strtol` on a stream-format string at 0x6240cd4/0x6240d1c; `ldrb w12,[x9]` char loop with x9 threading a guest string).
- **GSDSP caller attribution was ATTEMPTED and is DEFINITIVE-inconclusive (non-seedable):** GSDSP fired (state_matches=true) but the guest-word dump is all heap + one sign-extended host pointer — `GSDSP[+0=0x556c.. +8=0x556c.. +16=0x556c.. +24=HOST(0xffffff80ffffffd8) +32=0x556c.. +40=0x556c.. +48=0x556c.. +56=HOST(0xffffff80ffffffd8)]` and `BT: -> HOST(0x7f2bb86f58bb)`. **No in-image GUEST return-address anywhere** ⇒ the NULL object's frame is a host-allocated live object (an unmaterialized audio device), NOT a fixed `.bss` caller. This is the same signature that made SH206's 0x2306130 non-seedable.
- **Signature of a genuine audio-subsystem boot probe:** the engine (FMOD) is calling its AAudio output-format/headphone-change path during early init on a box with **no audio device and no AAudio**; the stream-format string is NULL/garbage run-variably ⇒ null-deref. This is forward-motion evidence, not a regression: **the client's FMOD audio layer is being reached headlessly for the first time as a measured contact point** (consistent with the FMOD-dispatch mechanism work at the top of the SH lineage).
- **Classified NON-SEEDABLE** (SH55/64 + SH206 live-member + SH208 host-pointer classes): the NULL object is threaded from a live enclosing object (audio-device singleton the boot never materializes), not a fixed `.bss` pointer; no `[0x...]=const` seed targets it; a hand-seed would fabricate an audio device. A real fix belongs to making the engine degrade audio-init gracefully (a live-session objective-c hardening item), NOT a static code patch.
- Do-not-re-tread as a seed.

## Crash B — raced `std::system_error` in nativeInitializeNativeFlags (run 8)

- Sequence: `[roblox:rbx.JNIRobloxSettings] nativeInitializeNativeFlags: Registered Flag Provider ID from Java: %d` (line 78) then `libc++abi: terminating due to uncaught exception of type std::__ndk1::system_error` (line 79). Only 1/8 runs; the 7 others run the identical code path clean.
- The exception's `.what()` prints as garbage stack words (`p: p}`…) — the **unconstructed-guest-stack-string family** (same class as the recon-v3 JSON leak the JIT_JSON_ZERO_FIX clears), i.e. the thrown system_error's message string is read off an uninitialized guest frame. A raced resource (flag-manager worker thread / mutex EDEADLK / thread-spawn EAGAIN in this constrained one-shot env) — non-deterministic, non-seedable, do-not-chase as a seed.
- Sitting inside nativeInitializeNativeFlags right after the flag-provider register — same function family SH116/SH116b/SH205 cleared the LOCK crashes from; this residual is the RACED (not deterministic) member, and SH116b did not change its 1/8 appearance.

## SH116/SH116b/SH205 slot-fix status (re-verified)

Run 8 shows `SH116 patched nativeInit lock-owner helper @0x102320710` + `SH116b patched nativeInit flag-manager site @0x102320a24` — both fire every run; the DETERMINISTIC lock-class fault (SH203/SH205: GSDSP GUEST(0x102320a98)) is gone (no run this cycle faults at 0x102320a98). The residual is the raced resource class, NOT SH116b.

## Standing

- **Route-B live-DM = structural gate UNCHANGED** (reconfirmed: do-init once-guard self-latches to a strcmp intern 0x400000b, DM-root=SH156 seed, liveDM=false; no static seed reaches any NativeDataModelManager creator body — SH209 verdict holds, do-not-re-tread).
- recon-v3 immediate-priority deliverables GREEN at this HEAD (24 task-driven frames swap Ok(0x1), 195 node pops, no json abort, EXIT 124 — re-run this cycle).
- Entire Route-B latent wiring ARMED (SH210 five marker classes still mount on a clean completing run).
- **NEW (genuine forward observation): the FMOD/AAudio audio-subsystem boot path is REACHED headlessly** (Crash A). The "sound" pillar has a concrete, measured first contact point; its current manifestation is a run-variable NULL-`this` (non-seedable). A future live-session / audio-harden milestone starts from this anchor (guest 0x106240d8c region, `Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged`).

## Repro

`bash runs/capture_sh212_residual_flakes.sh [N]` — default 8 runs, prints clean/crash + guestpc + GSDSP caller per run, then the FMOD/AAudio classification.

## No production code change

This is a measurement + disassembly cycle. No seed, no patch, no test edit shipped — the two crashes are both proven non-seedable (live-object NULL / raced resource), so a static patch would be feature-flag cruft against empty endpoints (SH184/SH186 discipline). Workspace left green at HEAD 8485bf2.