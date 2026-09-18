# Frontier SH329 — app-start after-fork two-arm closure (governor-flag fork) pinned; recon-v3 re-verified green at HEAD

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh329_appstart_afterfork_two_arm_closure_pinned`
(elfjit real-image guard, 6 byte-pins, 1 passed). elfjit examples 153/0 (152 + sh329); arm64jit lib
408/0; workspace green; elfjit.rs held 38 B under the 1MB pre-commit hook (comment-prose condensed,
facts/addresses preserved; sh314 comment minimally condensed). Route-B live-DM gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). Recon-v3 re-verified green at this HEAD (see §5).

## 1. Why this cycle

SH328 crossed fn 0x25f52b4 (x20-params wall) and left the app-start continuation terminal at
`guestpc=0x1025f501c` fault=0x0 — `ldr x8,[x0]` @0x25f5050 where x0=[AppStarted+0x408]=0 (a real
AppStarted heap obj's live member; SESSION-CTOR class). SH328's probe documented the DEFAULT arm only.
This cycle answers whether the **0x25f503c two-arm fork** offers a seed-side bypass.

## 2. The fork (real libroblox.so, robbox)

```
25f502c: ldrb w8,[x9,#3488]      ; x9=adrp 0x6a64000 -> governor-flag byte [0x6a64da0]
25f503c: cbz w8, 0x25f504c       ; flag CLEAR -> arm-1
25f5040: mov x0,x19
25f5044: bl 0x2ea3a84            ; arm-2: nativePreloadFlagOverrides(x19)
25f5048: b 0x25f5050
25f504c: ldr x0,[x19,#1032]      ; arm-1: x0 = [AppStarted+0x408]
25f5050: ldr x8,[x0]             ; BOTH arms converge here -> fault if x0==0
25f5058: ldr x8,[x8,#136]; blr x8  ; vt[+136] dispatch on the loaded object
```

## 3. Measured (A/B, real so, full SH328 seed set, 2 independent runs + SH328's own)

- **Arm-1 (default, govflag CLEAR)** — SH328's documented terminal: `ldr x0,[x19,#1032]` =
  [AppStarted+0x408] = 0 -> `ldr x8,[x0]` @0x25f5050 faults. (SH328 probe + reconfirmed this cycle.)
- **Arm-2 (govflag SET)**: `JIT_ROUTEB_APPSART_GOVFLAG=1` seeds governor-flag byte [0x106a64da0].bit0
  (the 0x25f503c discriminator) -> continuation takes `bl 0x2ea3a84` (nativePreloadFlagOverrides,
  x0=x19). That function ALSO returns x0=0 headlessly (its preload-overrides live-object [0x106a64d98]
  is unconstructed, SH269-class) -> `b 0x25f5050` -> SAME fault `guestpc=0x1025f501c` fault=0x0.

**Closure:** NO value of the governor-flag byte yields a non-null object at 0x25f5050. The AppStarted
live-member gate at +0x408 (arm-1) and the nativePreloadFlagOverrides return (arm-2) are BOTH null
headlessly. The app-start continuation therefore cannot advance past 0x25f5050 by toggling this fork —
it is SESSION-CTOR class (object built only by a real Activity-session ctor), consistent with SH251/SH317/
SH324 closures. No new seed added.

## 4. Verify

- `cargo test -p arm64jit --example elfjit -- sh329` = 1 passed (6 pins: governor-flag ldrb 0x1025f502c=
  0x39768128, fork cbz 0x1025f503c=0x34000088, arm-2 bl nativePreloadFlagOverrides 0x1025f5044=0x9422ba90,
  arm-1 ldr [AppStarted+0x408] 0x1025f504c=0xf9420660, both-arms ldr 0x1025f5050=0xf9400008, vt[+136] ldr
  0x1025f5058=0xf9404508).
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0 (arm64jit lib 408/0; elfjit examples
  153/0); elfjit.rs = 1048538 B < 1048576 hook.
- govflag A/B repro: `--v2boot-session` + `JIT_ROUTEB_APPSART_GOVFLAG=1` (arm-2) and default (arm-1)
  both first_sigsegv guestpc=0x1025f501c fault=0x0. Log kept locally (gitignored).

## 5. Recon-v3 re-verified green at this HEAD

`runs/capture_taskv4_frame.sh` (SH60) at HEAD: **24** task-driven frames; present #21..#23 swap Ok(0x1);
395 node pops; **0** json abort; dispatch ~#3285000; EXIT 124 (stable idle). Matches SH326/SH287 markers.

## 6. Honest

No make_shared<DataModel> (DM-root 0, MH_* false); Route-B live-DM structural gate UNCHANGED. The
AppStarted+0x408 / nativePreloadFlagOverrides objects are fork-independent nulls = SESSION-CTOR class;
per the SEP-17 SESSION-CTOR PRIMARY LEVER the forward remains the REAL Activity/AppBridge session drive
so the upstream ctor RUNS and builds these members (not another static seed). SH174 capture-latch stays
the single forward hook. sh314's do-not-re-tread closure comment was lightly condensed (statement
unchanged) to hold elfjit.rs under the 1MB hook.