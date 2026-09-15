# SH189 — PlayerGui / CoreGui / DM service container: where it's built, and the honest drive/seed spec

Status: read-only recon (no harness run). Answers "where does the genuine DM build assemble
PlayerGui / the service container" against the SH187 live-DM state (ret x0 = obj+0x1f0, planted
to `*0x106391908`, genuine vptr set `{0x1067162e8,0x1067163a0,0x1067163f8}`).
Log: runs/sh187-real-ctor.txt (SH187 ctor kernel drove to completion).

---

## 1. What the REAL DM ctor chain does / does NOT do with services

The DM ctor body `0x23f6038` (guest `0x1023f6038`) builds the DM object through
`0x1023f6728` ret in a ***branch-heavy*** way. The formerly-NOP'd subobject call is at
`0x1023f60b8` -> `bl 0x23f6b0c` (guest `0x1023f6b0c`, SH187 leaves it NOP'd).

`0x23f6b0c(this=obj+0x1f0, x1, x2)` — the tertiary MI base subobject ctor — does:
```
23f6b20: x19 = x0 (= obj+0x1f0)
23f6b3c: bl 0x5e18df4            ; base/embedded subobject init (vptr 0x67949f0 + 0x18 field)
23f6b44: adrp x8, 6797000; add x8,x8,#0x28   ; x8 = 0x6797028  (embedded-instance vptr family)
23f6b4c: str x8,[x0],#176        ; [obj+0x1f0]=0x6797028 ; x0 -> obj+0x1f0+0xb0 = obj+0x2a0
23f6b58: mov w1,#0x169           ; tag = 361
23f6b5c: bl 0x2374c90(x0=obj+0x2a0, w1=0x169, x2=sp)   ; BUILD a 361-entry 16-byte map/vector
```
`0x2374c90` (guest `0x102374c90`) is a **vector<std::pair> of 0x169=361 zeroed (ptr,ptr)
entries built from the stack descriptor** — with the call later overwritten by the DM vptr
family at `0x23f60c4/0x23f6148`, per SH187. **This is the DM's internal class/instance index,
NOT PlayerGui.**

**Negatives (verified in disassembly, SH188 already):
- PlayerGui / CoreGui / StarterGui / ScreenGui are NOT created anywhere in `0x23f6038`'s body.
  No `PlayerGui` string xref falls inside the DM ctor; nothing allocates a `0x169`-tagged
  GuiObject there.
- The get-or-create consumer `0x102dbcd88` (SH188) is a boolean registration predicate —
  `blr [DM-vptr+0x1c0]` = `0x3facf10`, a `can-create` compat gate, returning w0.bit0. It never
  yields a GuiObject. Confirmed dead end for "engine self-constructs a ScreenGui".

### Where the service machinery actually lives
Services are resolved lazily by NAME against two structures the DM owns:

**(a) Global class-name registry map** — header at **guest `0x106dca0e70`** (rw segment).
Lookup fn `0x2373cec(x0=&0x106dca0e70, x1=&name)` (guest `0x102373dec`): hash-map probe
(djb2-ish), returns the map element (or 0); **`[element + 0x10]` = the classid**.
`0x2373e94` is a small accessor `[x0+0x8992]` (TLS-ish global). Guards: `0x2373f94`.

**(b) Per-DM service container** — TWO accessor layouts decoded:
- `0x237da38(dm, &name, &out)` (guest `0x10237da38`): resolves name -> classid via (a), then
  walks **`[dm + 0x78]`** as a *vector header* `{begin, end}` of **16-byte elements
  `{service_ptr, ???}`**, comparing `[service_ptr + 0x18] == classid`. On match it stores
  `{service_ptr, [el+8]}` into out and refcounts. This is the CLASS DESCRIPTOR find.
- `0x5e09bc8(dm, &name, &out)` (guest `0x105e09bc8`, called from `0x2d66364`): the
  **service-list walk**. It resolves name->classid via the same `0x2373cec`, then walks a
  **singly-linked list anchored at `[dm + 0x68]`** (offset 104): each node has
  `[node + 0x18] = classid` and `[node + 0x68] = next`; compare `[node+0x18]==classid`.
  Stores `{node, refcount}` into out; on hit calls `0x2377600` (guest `0x102377600`), a
  `getOrCreate`-style out-pair materializer.
  This is the RBX `ServiceProvider::getService-by-class` walk.
- `0x2374d4c` / `0x2373f94` = the registration/push helper family (`bl`'d by both walkers).

**So the DM "service container / PlayerGui" does NOT exist until something resolves
"PlayerGui" through either walker AND the class registry (a) has a "PlayerGui"->classid
entry.** In the SH187 live-DM build the container is empty and the class registry is
unseeded (`0x106dca0e70` is .bss-zeroed until a class-registration once-init runs).

---

## 2. PlayerGui / related class-name registry facts (file vaddr, guest)

| class | name string (file / guest) | register stub (file / guest) | classid (w3) | typeid (w4) | once-guard bucket |
|-------|---------------------------|------------------------------|--------------|-------------|-------------------|
| PlayerGui | `0x595eeb` / `0x10595eeb` | `0x201fda4` / `0x10201fda4` | `0x87e`=2174 | `0x298`=664 | `0x6c97000+f30` |
| CoreGui   | `0x4280d9` / `0x104280d9` | `0x201ee5c` / `0x10201ee5c` | `0x86a`=2154 | `0xe7`=231  | `0x6c8b000+af0` |
| ScreenGui | `0x54807d` / `0x1054807d` | `0x201f4f0` / `0x10201f4f0` | `0x1b87`=7047 | `0x31e`=798 | `0x6c98000+b8` |
| StarterGui| `0x5168ef` / `0x105168ef` | `0x202014c` / `0x10202014c` | `0x892`=2194 | `0x37d`=893 | `0x6c9a000+??` |

Each stub is gated by a **pthread_once-style self-latching guard** using `0x284ce54`
(begin)/`0x284cf5c`(end)/`0x284cfbc`(retry) around a shared `.bss` latch at the listed
guard bucket — the SAME once-idiom as SH156's global-init. The stub writes a class descriptor
into the global registry bucket and registers classid/typeid.

**RBX::PlayerGui typeinfo/vtable:** PlayerGui's class vtable family sits at **`0x6648000`
(file, guest `0x106648000`)** — toggling the vtable is NOT needed; the registry stores the
descriptor and `[element+0x10]=classid` is the lookup key.

---

## 3. DM service-container LAYOUT (concrete)

- **Service list (linked, the actually-instantiating path):** field at **`dm + 0x68`**
  (offset 104) holds the head node. Node layout:
  `[node+0x00]` = classid dispatch vtable (RBX class object)
  `[node+0x18]` = **classid** (playergui=0x87e, coregui=0x86a, screengui=0x1b87)
  `[node+0x68]` = next node (singly-linked)
  `[node+0x08]` = Instance/refcount wrapper (paired out via 0x2377600 / 0x2216de4).
- **Service vector (class-descriptor find):** `[dm + 0x78]` = `{begin,end}` of 16-byte-stride
  `{service_ptr, classid_or_ref}` array; compare `[service_ptr+0x18]==classid`.
- **DM-go-to-seed:** the loop already has obj (ret of ctor) and `*0x106391908 = obj+0x1f0`.
  The DM base for service-ops is the same ret object (both +0x68 and +0x78 are on it).
- **Class registry** to seed before name resolution: map header `0x106dca0e70`.
  Resolver `0x2373cec` reads `[hdr+0]`/`[hdr+8]` = {begin,end} of key array, `[hdr+0x20]`
  = {ptr, end}, 24-byte stride (key blob ptr, len, value) + `[hdr..]`. Empty header currently
  (registry once-init did not run in a surfaced path).

---

## 4. Concrete drive / seed / probe SPEC (paste into main loop) — with HONEST verdict

### Verdict up front: THIS IS A LOOP-BACK, NOT A SINGLE DRIVEABLE STEP.

To get an **engine-constructed ScreenGui parented under a live PlayerGui on the genuine DM**
you must, in order:
1. Make the **global class-name registry** (`0x106dca0e70`) resolve "PlayerGui" -> classid
   0x87e. That means either running the class-registration stub `0x201fda4` (which needs its
   once-latch `0x6c97000+f30` clear + a valid descriptor build) — itself a chained once-init,
   **the next unsynthesized object** — OR planting the registry element directly.
2. Materialize a **PlayerGui node** and link it into `[dm + 0x68]` (set `[node+0x18]=0x87e`,
   `[node+0x68]=0`). The engine's own allocator/ctor to do so is `0x5e09bc8`'s walk + a
   PlayerGui-typed ctor — that ctor is NOT hit by anything the SH187 drive reaches.
3. Materialize a **ScreenGui instance** (real RBX::ScreenGui vtable family) and attach it as
   a child of the PlayerGui — the GuiObject parent-change / SceneGraph insert path (renderer
   walker `R+0x180/R+0x188` node lists from frontier-sh64, not reachable from the DM ctor).

Step 1's class registry is the concrete "one-next-unsynthesized-object". Bare addresses:
`PlayerGui descriptor bucket 0x6c97000+f30` (guard), classid 0x87e, typeid 0x298,
name `0x10595eeb`; registry header `0x106dca0e70`.

### Seeds that are SAFE to lay without a factory (all guest-space, memory-only)
```rust
// dm = *0x106391908  (== ctor ret x0 == obj+0x1f0, vptr set {2e8,3a0,3f8})
// [1] hang a coherent EMPTY service list so 0x5e09bc8 / 0x237da38 do not fault on NULL walk:
let empty_head = leak_zero(0x80);          // zeroed node: [+0x18]=0, [+0x68]=0
*(dm + 0x68) = empty_head;                 // head of service singly-linked list
*(dm + 0x78) = 0;                          // vector {begin,end}=empty (walker early-outs)
// [2] class-registry: only if you will call 0x2373cec — leave 0x106dca0e70 as-is (empty
//     map header -> resolver returns 0 -> walkers return "not found", crash-free).
//     Do NOT hand-write the registry element: its 24-byte stride + bucket layout must match
//     the once-init's real builder (treat as unsynthesized).
// [3] CONSUMER drive to PROBE the walker survives with the empty list:
run_guest_callback(0x10237da38, [dm, &"PlayerGui"SSO, out,0,0,0,0], tp);
//     expectation: Ok(0), out={0,0}, 0 crash  — "PlayerGui not registered" benign path.
```

### PROBE that proves a real GuiObject self-constructed (the testable artifact)
After any real ScreenGui-under-PlayerGui materializes, read back:
```
dm = *0x106391908
head = *(dm + 0x68)                 // service list head
count = walk(node while node != 0)  // linked via [node+0x68]
classid = *(head + 0x18)            // PROBE: == 0x87e  <=> a real PlayerGui sits on the DM
screens = children-under(playergui) // renderer R+0x180/R+0x188 scene-node count (frontier-sh64)
```
**Zero-crash + one walker returning a node whose `[node+0x18]==0x87e` (PlayerGui)** is the
definition of success this recon can stand behind. Anything less (e.g. the `0x106dca0e70`
registry returning 0, or the get-or-create predicate w0.bit0==0) is the honest DISPATCH
negative SH188 already predicted.

---

## 5. Honest scoping (task item 5 verbatim)

- **Single driveable step?** No — it loops back into the ONE-NEXT-UNSYNTHESIZED-OBJECT chain.
- **Next object + address:** the **PlayerGui class descriptor in the global class-name
  registry** — guard bucket `0x6c97000+f30`, registry header `0x106dca0e70`, register stub
  `0x201fda4` (guest `0x10201fda4`). Only after that resolves does `[dm + 0x68]` get a
  PlayerGui node by `0x5e09bc8`/`0x237da38`, and only then can a ScreenGui attach.
- The DM ctor `0x23f6038` and the SH188 get-or-create consumer are both **confirmed dead
  ends for GuiObject production** — the next real lever is the class registry, not the ctor
  continuation (SH188 §5's "restore the NOP at 0x1023f60b8" restores the DM's internal
  361-entry index, still NOT PlayerGui).

---

## 6. Testable artifact checklist for the main loop
- [ ] `*(dm+0x68)` acceptable as a coherent empty head; walker `0x10237da38` returns Ok(0)
       crash-free with the empty list.
- [ ] `0x106dca0e70` readback stays a valid empty map header (resolver 0 returns benign).
- [ ] No SIGSEGV anywhere in the array of empty-seed + consumer drive.
- [ ] Positive test is DEFERRED to the registry once-init (next cycle) — do not fake a
       PlayerGui node; that node must come from real PlayerGui-typed ctor code.