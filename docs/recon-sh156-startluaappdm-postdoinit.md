# SH156 — StartLuaAppDM full post-do-init trace: dispatch goes to a REAL engine-boot body (0x1023eff4c), not a fault

Subagent: deleg_8f1f5b2e (read-only recon). All objdump addresses are ELF VAs;
guest = ELF + 0x100000000 (verified: `.text` ELF VA 0x1d95980; StartLuaAppDM ELF 0x23efe2c = guest 0x1023efe2c).

## 1. Full body of StartLuaAppDM (ELF 0x23efe2c .. 0x23eff48 ret)
Command: `aarch64-linux-gnu-objdump -d --start-address=0x23efe2c --stop-address=0x23eff40 libroblox.so`

```
23efe2c sub sp,sp,#0x60 ; stp x29,x30,[sp,#64] ; stp x20,x19,[sp,#80]
23efe40 ldr x19,[x19,#1776]   ; x19 = [0x67d16f0] GC-canary ptr
23efe4c adrp x8,683d000 ; ldr x0,[x8,#848]     ; x0 = [0x683d350] pb-flag word
23efe5c cmp w8,#6 ; ccmp x9,#0,cs ; b.eq 23efe90   ; if low-byte==0x6 skip crashpad
23efe68..23efe8c bl 21fd00c   ; (crashpad/backtrace-init path, skipped on 0x6)
; ---- main path ----
23efe90 adrp x8,635d000 ; add x8,x8,#0xd68     ; x8 = 0x635dd68 (function-ptr table)
23efe98 mov x20,sp
23efe9c str x8,[sp]          ; [sp+0] = 0x635dd68      <- table addr as slot 0
23efea0 str x20,[sp,#32]     ; [sp+32] = sp (stack self-ref, nonzero)
23efea4 mov x0,sp ; mov w1,wzr ; bl 2baeeec     ; <-- The app-bridge pipe
   ; NOTE: StartLuaAppDM does NOT save/interpret return; it only builds the
   ; stack union {slot0=0x635dd68, slots 8/16/24=0, slot32=sp} and calls.
23efeb0 ldr x0,[sp,#32] ; cmp x0,x20
23efeb8 b.eq 23efec8          ; [sp+32]==sp -> table slot +0x20
23efebc cbz x0,23efedc        ; [sp+32]==0 -> ret (soft)
23efec0 mov w8,#0x28          ; else -> table slot +0x28
23efed0 ldr x9,[x0]           ; x9 = *(union+0) = 0x635dd68
23efed4 ldr x8,[x9,x8]        ; x8 = table[0x20 or 0x28]
23efed8 blr x8                ; <-- virtual dispatch through 0x635dd68 table
23efedc..23efef8 ldr x8,[x19]; cmp; ldp; add sp; ret   ; epilogue (stack-check vs canary)
```
TERMINATION: the ONLY returns are the epilogue at 0x23efef8 (normal ret) and
`b 2206e28` soft paths. It does NOT construct Luau VM / DataModel / app-shell
in its own body — it is a thin wrapper that hands a stack union to the pipe.

## 2. What the app-bridge pipe does + where control REALLY goes after do-init
Command: `aarch64-linux-gnu-objdump -d --start-address=0x2baeeec --stop-address=0x2baef50 ...`
```
2baef04 mov w20,w1 ; mov x19,x0
2baef58 ldr x0,[x8,#8]        ; x0 = [0x683d008] (gov "this")
2baef5c tbz w20,#0,2baef6c
2baef6c mov w2,wzr ; bl 2206c40    ; GlobalInit do-init (w1 bit0==0 path)
```
do-init 0x2206c40 tail — `aarch64-linux-gnu-objdump -d --start-address=0x2206c40 --stop-address=0x2206e38`:
```
2206c5c mov x19,x2 ; mov x20,x1      ; x20 = our stack union addr
2206c74 adrp x8,6a68000; add x8,#0x410; ldar w9,[0x6a68410]  ; once-guard
2206c84 tbz w9,#0,2206d10    ; clear -> __call_once (construct path)
2206cdcd bl 2206db8          ; x0=[0x683d008+0/8], x1=x20(union), x2=0  <- MATCH
2206ce0.. . . ret
MATCH 0x2206db8:
2206de4 ldr x20,[x8,#2664]          ; [0x6863a68] = main-thread id (HARNESS SEEDS)
2206de8 bl pthread_self ; cmp ; b.ne 2206e28   ; not-main-thread -> soft return
2206df4 ldr x0,[x19,#32]            ; x19 = x1 = union  -> [union+0x20] = sp (nonzero!)
2206df8 cbz x0,2206ea4              ; (won't trigger; union slot32 = stack ptr)
2206dfc ldr x8,[x0]                 ; x8 = [sp] = 0x635dd68 (union slot0)
2206e00 ldr x1,[x8,#48]             ; x1 = table[+0x30]
2206e24 br x1                       ; <-- BRANCH (not bl) -> 0x23eff4c
```
CONCLUSION: on the harness's seeded main thread the do-init does NOT return to
StartLuaAppDM's 23efeb0 — the match dispatch `br`s directly into guest
**0x1023eff4c**, which sits immediately after StartLuaAppDM's OWN ret
(0x23eff48) in the same TU. StartLuaAppDM's 23efeb0 tail is DEAD on this path.

Vectors needed (all relocated, all REAL code — no static-zero leak):
- table 0x635dd68 slot +0x30 (guest 0x10635dd98) -> 0x23eff4c  [packed-RELA addend, decoded]
- table 0x635dd68 +0x20/+0x28 (DEAD soft path) -> 0x1db2cf0 (ret stub) / 0x21e96f8 (b 0x626b6d0)

## 3. The real post-do-init body: guest 0x1023eff4c (AppBridgeV2 engine boot)
`aarch64-linux-gnu-objdump -d --start-address=0x23eff4c --stop-address=0x23f0100 ...`
```
23eff4c sub sp,#0x180
23eff70 adrp x8,6a64000; ldrb w8,[x8,#3488]  ; flag = [0x6a64da0] (.bss, 0)
23eff78 cbz w8,23eff9c        ; default PATH B
 ; PATH A (flag) : bl 2366694 ; bl 2dae640 ; blr [x0 vt+0x18]
 ; PATH B (default):
23eff9c bl 2367270           ; AppBridgeV2 GetOrCreate singleton (once-guard [0x6a70618], obj [0x6a705e8]) -> x0
23effa8 bl 2366694           ; init stack union sp+0x10
23effac ldr x8,[x19] ; ldr x8,[x8,#24]  ; vt = [0x6a705e8]=0x63a3410; slot+0x18=[0x63a3428]
23effbc blr x8               ; <-- NEXT GATE (guest 0x1023effbc) -> 0x2e9fa84
```
AppBridgeV2 singleton builder 2ea3084 (`bl 23672b8`) stores vtable ptr
0x63a3410 into [0x6a705e8]; guest self-constructs via __call_once if harness
LEAVES once-guard [0x106a70618] clear. Table 0x63a3410 all relocated:
`[0x1063a3428] -> 0x2e9fa84` (real 0x430-frame function).
`0x2e9fa84` (guest 0x102e9fa84) = the real AppBridgeV2 governor startup: reads
x19=[x0,#32], copies many param unions into [x19+...], calls bl 258c6e4
(nativeAppBridgeStartAppWithParams) and bl 23c14dc etc.

## 4. HONEST conclusion
StartLuaAppDM does NOT dead-end before self-constructing session/GuiObjects.
With the harness seeding the main-thread id [0x106863a68] + clear once-guard
[0x106a68410]/[0x106a70618], the do-init match `br`s into a REAL engine-boot
body (0x1023eff4c), which virtual-dispatches through the AppBridgeV2 governor
vtable to 0x102e9fa84. That dispatcher table is fully packed-RELA-resolved
(no zero vtable slot here — differs from the SH111 leak singletons 0x106829a48).
The do-init does NOT return 'handled' to StartLuaAppDM; it enters the actual
app-boot path inside the guest.

## 5. NEXT-SEED / NEXT-GATE recommendation
- NEXT GATE: guest 0x1023effbc `blr x8` -> 0x102e9fa84 (AppBridgeV2 governor
  startup). Reachable only if: (a) main-thread id [0x106863a68]==pthread_self
  (harness seeds), (b) AppBridgeV2 once-guard [0x106a70618] left CLEAR so the
  guest __call_once runs builder 2ea3084 (writes vt 0x63a3410 at [0x106a705e8]),
  (c) flag [0x106a64da0]==0 (default .bss).
- NEXT SEED: NONE required for the dispatch itself (all vtable slots are
  packed-RELA-resolved to real code). Instead, SEED/expect 0x102e9fa84's first
  governor reads: x19=[gov+0x20], and its call `bl 0x10258c6e4`
  (nativeAppBridgeStartAppWithParams) — watch inside 0x2e9fa84 for the first
  unbuilt sub-object/soft-return (next cycle's probe point = retain it).
- Recommendation: keep StartLuaAppDM rung using the SH155 gate-fix (flags-latch
  + main-id, once-guard CLEAR), and treat guest 0x102e9fa84 as the next
  frontier marker (engine-boot), not a fault site.