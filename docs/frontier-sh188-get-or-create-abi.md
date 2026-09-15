# Get-or-Create consumer 0x102dbcd88 — full ABI map + DRIVE SPEC (SH187c recon)

Binary: `~/.cache/open-sober/robbox/libroblox.so` (ELF DYN aarch64, PIE base 0x100000000).
All function addresses below: FILE vaddr = guest − 0x100000000.
Recon method: `aarch64-linux-gnu-objdump -d` + APS2-packed-RELA decode of `.rela.dyn`
(repo's `decode_aps2`, ported to python) to materialize RELATIVE slots.

---

## 1. What the consumer is

`0x102dbcd88` is **`RBX::DataModel::getOrCreateDelegates()`-style / a MemStorage-bind
consumer** — NOT a GuiObject factory. It is `bl`'d from 9 sites:
`24c61e8` (MemStorage_bind), `275bd04` (nativeOnDestroyed), `312bc68`, `455b940`,
`4561fe0`, `45629f0`, `4564a98`, `4a7ae3c`, `4a93d00`. It is the "ensure this DM
slot exists" entry used by app-shell bind/cleanup. The **returned object is not a
GuiObject** — see §4.

## 2. Full ABI (end-to-end disassembly of 0x102dbcd88)

Signature and register map (aarch64 AAPCS):

```
x0  = in  OBJ    : the DM object (loop passes dm_base = obj+0x1f0, the CM/tertiary subobject)
x1..x7 = unused by the create-path (only x0 is consumed; seed code passes all 0)
x8  = the keyed-lookup "threshold" slot convention — OBJ+0xb0..0xc0 is the vector
sp  = 16-byte aligned scratch (desc cells + out-slots)
ret x0 = the found/created slot element value (see §4)
```

Control flow:

```
2dbcd88: frame (sp-0x60); x19 = x0 (OBJ); canary
2dbcdb0: bl 0x2dbcf54        ; "get" worker
2dbcdb4: ldr x20,[x0]        ; x20 = *worker-ret
2dbcdb8: cbz x20 -> 2dbcde4  ; empty -> CREATE path
2dbcdbc: (ret x20)           ; FAST path: return the OBJ slot value
...
2dbcde4: (CREATE) ldr x9,[x19,#0x58]; x0 = x9 & ~7; bl 0x265ea54  ; build 16B struct at sp+0x18
2dbcdf4: ldr x20,[sp,#0x18+8]  ; x20 = the *service/base object* for this DM slot
2dbcdf8-2dbce5c: checks `RBX::String` at [x20+0x70] == "Instance" (8-byte const 0x65636E61 7473496E)
                 -> if match, vtbl[+0x40](x20) fast hook + bl 0x2374d4c
2dbce64: ldrb w8,[0x6dbf000+0x238]  ; GLOBAL GATE byte @ guest 0x106dbf238
2dbce6c: cbz w8 -> 2dbce8c   ; if gate==0, SKIP the DM-vptr create call entirely
2dbce70: ldr x8,[x19]        ; x8 = OBJ vptr           (tertiary 0x1067163f8)
2dbce74: ldr x8,[x8,#448]    ; slot +0x1c0  -> guest 0x103facf10
2dbce78: mov x0,x19          ; x0 = OBJ
2dbce7c: mov x1,x20          ; x1 = the service/base object
2dbce80: blr x8              ; ** THE CREATE-PATH **
2dbce84: tbz w0,#0 -> 2dbcef4  ; bool result bit0 = keep-x20-or-zero
2dbce8c-2dbcea0: stp out; x0=x20,x1=x19,w2=0,x3=&out; bl 0x240c52c  ; *registration ctor*
2dbcea4+: destructor/refcount; bl 0x2dbd018 (key), 0x2411658 (vector addr),
         0x21ebe98 (store), ret element value
```

**Key parameter**: the "key" is the global cell at **guest 0x106a665a8** (a lazily-
init'd singleton index held behind the two once-guards 0x106a665a0/0x106a665b0).
Seeding that cell to 0 makes the consumer index element [0] of the keyed vector.

**Keyed-lookup vector** (resident on OBJ): `OBJ+0xb0` = base data ptr D,
`OBJ+0xb8` = end, 16-byte stride; `OBJ+0xc0` = end-of-storage (cap). Worker
`0x2411658` does `ldp [obj+0xb0],[obj+0xb8]; if key < (end-base)>>4 -> return
base+key*16` else grows. The element the consumer reads/writes is **`*(D)`**.
The loop already seeds `[OBJ+0xb0]=D`, `[OBJ+0xc0]=D+16`.

## 3. The create-path slot `blr [DM-vptr + 0x1c0]`

The create-path uses the **tertiary** DM vptr (`[OBJ+0]` where OBJ = obj+0x1f0, so
vptr = **0x1067163f8**), slot offset **+0x1c0** = vaddr **0x67165b8**.

Decoding `.rela.dyn` RELATIVE at 0x67165b8:
  `0x67165b0 -> guest 0x102377c0c`
  `0x67165b8 -> guest 0x103facf10`   <-- the slot [DM-vptr+0x1c0] target
  `0x67165c0 -> guest 0x101dc7428`

Target **guest 0x103facf10** (file 0x3facf10) is a **16-byte guarded predicate**, NOT an
object factory:

```
3facf10: cbz x1 -> 3facf4c (return 0)
3facf14: frame; x0 = x1 (the service obj); x1=&0x6796850; x2=&0x63ab8e0; x3=-2
3facf38: bl 0x2b83ab4        ; protected RTTI/IsA-style "can-create" / self-register proxy
3facf3c: cmp x0,#0; cset w0,ne ; return BOOL (x1 valid && engine says yes)
3facf4c: mov w0,wzr; ret     ; x1==0 -> returns FALSE
```

Semantics of the returned slot (w0): **it is a boolean yes/no "this service/base
object is compatible to register under this DM slot"**; the consumer tests bit0
(`tbz w0,#0`). It is *not* a pointer and carries no GuiObject.

(For contrast — the *primary* DM vptr 0x1067162e8 slot +0x1c0 = vaddr 0x67164a8 ->
guest 0x1036f0a38 = `ldr x8,[x0,#0x70]; add x0,x8,#8; ret` — is itself only an
accessor returning `*(obj+0x70)+8` (embedded/once-wrapper), also not a factory.)

## 4. What the returned x0 *is* — honest verdict

- The consumer returns the **element value stored in the DM's keyed vector at
  OBJ+0xb0** (i.e. `*(D)`), which the internal registration path parked there.
  In the drive's FAST mode `*(D)=dm_base` so ret = the DM itself; in DISPATCH
  mode `*(D)=0` and the create-path runs.
- The createsvg that CAN run (`0x240c52c` + the `0x2b83ab4` compatibility proxy)
  **registers a service/base object into the DM slot**; the RBX::String it keys on
  is `"Instance"` (the DM's own embedded Instance wrapper at DM+0x70, +8 = the
  instance-slot wrapper). It does **not** allocate a ScreenGui/ScrollingFrame/
  TextLabel, and nothing in the consumer's reach types into a GuiObject class.
- A genuine GuiObject GUI tree additionally requires a **live service container /
  PlayerGui / CoreGui instance on the DM** so a GuiObject can parent. This
  consumer does not create that; it only arms the DM's *Instance* service-slot.

**Verdict (task item 4):** the create-path `blr [DM-vptr+0x1c0]` at 0x2dbce80
fundamentally **cannot** yield a GuiObject — that slot is a boolean
registration-compatibility predicate (guest 0x103facf10), and the only object the
consumer ever materializes is the DM-embedded `"Instance"` wrapper. Chasing a
GuiObject through 0x102dbcd88 is a dead end.

## 5. Single next unblocked thing

To get a REAL GuiObject the main loop must stop at get-or-create and instead arm the
DM's **service container / PlayerGui** in the *same ctor build*. Lowest-risk next
stepping stone, in order:

1. SEED the create-call's own **gate byte** `0x106dbf238 = 1` (already done) so the
   create-path actually fires, then PROBE:
   - `*(dm_base actual DM base + 0x0)` vptr == tertiary 0x1067163f8, and
   - the keyed-vector element `*(D)` after the run: comes back == **0** in
     DISPATCH mode (create-path predicate w0.bit0 == 0 because the service obj
     passed in x1 is a zeroed host buffer with no valid RTTI) — this is the
     testable "nonzero GuiObject" *negative*: DISPATCH ret should be 0, FAST ret
     should be dm_base. Both are engine-run, crash-free assertions.
2. Then (the real unblock): drive the DM ctor CONTINUATION so the build reaches its
   service-container arm. In `0x23f6038` the DM builds subobject at `0x23f6b0c`
   (currently NOP'd at 0x1023f60b8) which writes the embedded-instance vptr family
   `0x6797028` and ctor 0x2374c90 for a `0x169`-tagged service. **Arm PlayerGui by
   removing the SH187 NOP and letting `bl 0x23f6b0c @ 0x1023f60b8` run with a valid
   descriptor chain**, then re-run get-or-create to pick up the armed service.
   Concretely: restore the `0x94000295` at 0x1023f60b8 under a new env
   `JIT_ROUTEB_DM_SERVICES=1`, seed desc[+8]/[+16]/[+24]/[+32] as vptr-valid
   (0x6797028 family) instead of zero buffers.

## 6. Concrete DRIVE SPEC (paste into main loop) — GET-OR-CREATE as far as it goes

```
// === seeds (guest addrs), all under JIT_ROUTEB_DM_REALCTOR_CONSUMER=1 ===
once-guard outer 0x106a665a0 = 1
once-guard inner 0x106a665b0 = 1
key cell         0x106a665a8 = 0        // index 0
create-gate      0x106dbf238 = 1        // (DISPATCH mode) lets blr fire
D  = leak 0x40 zero buffer (16B-aligned)
*(dm_base + 0xb0) = D                    // keyed vector base
*(dm_base + 0xc0) = D + 16               // cap/end-of-storage
*(D) = FAST: dm_base   |  DISPATCH: 0    // element value
del0,del1,del2 = leak 0x40 zero bufs
*(dm_base+0xe8)=del0; *(dm_base+0xf0)=del1; *(dm_base+0xc0)=del2
*(dm_base+0x38c) = 0

// === run_guest_callback ===
let ret = run_guest_callback(0x102dbcd88, [dm_base, 0,0,0,0,0,0], tp);
// FAST   : expect Ok(dm_base) — 0 crashes  (already the historical result)
// DISPATCH: expect Ok(0x0)    -- create-path ownership bool gate, NOT a GuiObject

// === probe (guest mem) to see the DM is alive ===
let dm = *0x106391908;               // current-DM holder
let v0 = *(dm+0x0), v1=*(dm+0x8), v2=*(dm+0x1f0);
assert (v0,v1,v2)==(0x1067162e8,0x1067163a0,0x1067163f8); // genuine DM vptr set
let elem = *(dm+0xb0); let val = *elem;   // == ret in both modes
// There is NO scene-node list R+0x180/0x188 reachable from this consumer:
// that probe belongs to the RENDERER walker (frontier-sh64), not the DM get-or-create.
```

Env for the 200s harness run (capture_sh187_real_ctor.sh style):
`JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1
JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1
JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_DM_REALCTOR_CONSUMER=1
JIT_ROUTEB_DM_REALCTOR_DISPATCH=1` + entry `0x2173ff4 --jni --startapp 0x258b144`.

## 7. Testable artifact checklist
- [ ] DISPATCH drive ret x0 == 0x0 (create-path bool gate, no crash) — proves the
      genuine DM vptr+0x1c0 dispatch ran real relocated code.
- [ ] FAST drive ret x0 == dm_base, holder 0x106391908 genuine vptr set intact.
- [ ] Gate byte 0x106dbf238 readback == 1 before the call.
- [ ] **GuiObject assertion must be moved OFF this consumer** (see §4/§5); the
      next real lever is un-NOP'ing 0x1023f60b8 under JIT_ROUTEB_DM_SERVICES=1 to
      run the DM's service-subobject ctor.