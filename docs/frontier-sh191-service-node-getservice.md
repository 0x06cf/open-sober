# SH191 — PlayerGui as a real service NODE on [dm+0x68], resolved by the engine's OWN getService walker (headless)

Status: implemented + verified on the real binary (EMPTY-map negative). Commit 7073ff4.
Frontier: Route-B — engine SELF-CONSTRUCTS GuiObjects and they become service-resolvable
scene nodes under the genuine DataModel.
Repro: runs/capture_sh191_service_node.sh, log runs/sh191-service-node.txt.

## What was achieved

After SH190e (both PlayerGui 0x106648950 + ScreenGui 0x106649ce0 instances
SELF-CONSTRUCT headlessly), SH191 makes the constructed PlayerGui a REAL SERVICE NODE:

1. **Link**: host-link the engine-authored PlayerGui obj (vptr 0x106648950) as a
   service node on [dm+0x68] — a thin 0x70 node with `[node+0x18]=0x87e` (classid),
   `[node+8]=the captured instance`, `[node+0x68]=next=0`.
2. **Drive**: call the engine's OWN getService walker **0x105e09bc8** with a fabricated
   long-form "PlayerGui" name string and x8=&out.

### EMPIRICAL (real libroblox.so, llvmpipe, EXIT 124, 0 SIGSEGV/SIGABRT)
```
[routeb-dmsvc] SH191: linked PlayerGui service node 0x7fbb8cdeccf0 (classid 0x87e,
              instance 0x7fbb8c590720) at [dm+0x68] (0x7fbb8c033d20)
[routeb-dmsvc] SH191: resolver map 0x106dca0e70 = {0x0,0x0} (EMPTY) register map
              0x106dca0f60 = {0x0,0x0} (EMPTY)
[routeb-dmsvc] SH191: getService walker DROVE ok ret x0=0x0 out={0x0,0x0} (not-found)
```
**PlayerGui is now a linked service node on the genuine DM's service container**, and the
engine's getService walker executes **cleanly headlessly** (0 crash, EXIT 124, reproducible).
The walker returns not-found because name→classid RESOLUTION needs a populated map.

## The gate (honest, precisely characterized)

The engine's getService resolution uses TWO SEPARATE registries:
- **0x106dca0e70** = the class-name → classid RESOLVER MAP (read by 0x2373cec). Element
  stride 0x18 = {key_ptr @+0, key_len @+8, classid @+0x10}. The walker + many consumers
  (0x247bda4, 0x23738e0, 0x237da5c, 0x240f330, 0x25f8fd8, ...) all adrp 0x6dca000 + add
  #0xe70. It is **EMPTY {0,0}** at this boot state.
- **0x106dca0f60** = the register's own map (written by 0x1dc4bc8→0x1dc4c3c). Also
  **EMPTY {0,0}** here.

Even after the SH189 register GETTER (0x10201fce0) drives the DescribedCreatable class
registration (cached desc 0x106c980b8, vtable-family 0x106648908 — the descriptor cache IS
populated), the name→classid resolver map stays empty. **The per-class descriptor register
and the name→classid resolver map are distinct registries; the resolver map is built by a
bulk registrar, not the per-class getter.** Do NOT hand-write the hash-map element (24-byte
stride + open-addressing hash layout must match; docs in frontier-sh189 warn against this).

## Next (the real leap to name-resolution)

Locate + drive the engine's bulk class-name registrar that fills 0x106dca0e70's
{begin,end} vector (candidates: the once-init builder near 0x1dad050-family that wraps the
same call_once idiom — but that one targets a DIFFERENT page 0x6c26000+0xe70; the resolver
map on page 0x6dca000 needs its own writer). If the map is populated, the getService walker
will resolve "PlayerGui"→0x87e and return the linked instance (item vptr 0x106648950) — the
engine genuinely returning its self-constructed PlayerGui as a service API to the caller
headlessly. The scene-attach layer (R+0x180/0x188) sits behind that.

## SH192 addendum (empirical recon-negative, do-not-re-tread)

**The resolver-map registrar is LOCATED but non-constructive standalone.** The bulk
class-name registrar that fills 0x106dca0e70 is **0x2208ae8** (in-image symbol
`nativeGameGlobalInit+0x26e4`) — it IS reached during the headless ladder (region-watch
`JIT_REGION_WATCH=0x102208ae8-0x102208e30` logs entries at 0x102208ae8/0x102208b6c/0x102208cf8
every clean run). Decoded contract: `x0=&{key_ptr,key_len}` (16B), it hashes the key into the
open-addressing table at 0x106dca0e70, inserts {key_ptr,key_len,classid} (stride 0x18,
classid@+0x10), returns `x0=&element.classid` for the caller to fill.

**Attempted (SH192, built + reverted, do NOT rebuild):** extend `routeb_dm_service_resolve_guard`
to `routeb_ensure_writable` the map pages (0x106dca0e70/0x106dca0e90/0x106dca0f60) then
drive 0x102208ae8 with a fabricated {&"PlayerGui",9} key and write 0x87e into the returned
classid slot, then re-drive the getService walker. **Empirical result (3/3):** the registrar
returns Ok with a HOST-heap slot (0x7f..036f90, NOT a guest map element), the resolver map
stays `{0,0} EMPTY`, and the run afterwards aborts with
`libc++abi: terminating ... bad_weak_ptr`. Root cause: 0x106dca0e70 is a *std::unordered_map
object whose construction (bucket array + node allocator + count word at 0x106dca0e88) only
runs inside live nativeGameGlobalInit*; headless global-init invokes the registrar with an
EMPTY SOURCE (the per-class source vector 0x106dca0e90, itself only populated by the same
world-build), so it inserts nothing and never constructs the map. Driving the registrar
standalone on the raw zeroed map object lets it hash against garbage → returns a host slot
and corrupts the shared map → later bad_weak_ptr on the real resolver consumer.

**Conclusion (exactly the SH191 wall, now with the registrar identified):** the name→classid
resolver map 0x106dca0e70 is constructed + populated only by the live class-registry
world-build (same Route-B live-DM wall). Its registrar is headless-REACHABLE but inserts
nothing because the class-name SOURCE (0x106dca0e90) is empty pre-world-build. Do NOT
re-attempt driving 0x2208ae8 standalone; do NOT hand-write the hash element (SH189 warning).
The getService "PlayerGui"→0x87e resolution, and thus the linked-node RETURN from the
walker, sits behind the live class-registry construction. Scene-attach (R+0x180/0x188)
remains behind that. Clean SH191 (link node + walker executes cleanly, not-found) is the
shipped state — re-verified 3/3 EXIT 124, 0 SIGSEGV/0 SIGABRT.