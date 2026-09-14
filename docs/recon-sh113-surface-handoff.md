# Recon SH113 — `nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams` drive ABI, the onAppReady dispatch, and the first self-constructed scene-node path

READ-ONLY recon. All guest addresses = file vaddr + `0x100000000`; file offsets given too.
Real binary: `/home/hermes-worker/.cache/open-sober/robbox/libroblox.so` (arm64, 104 MB, branch `dev`).

This spec gives a host harness the **exact register-level ABI** to drive the
surface handoff JNI function, proves the **Surface jobject is irrelevant** (the
JIT's `ANativeWindow_fromSurface` host shim ignores it), and maps the chain from
the stored window pointer to the engine's first `GuiObject` → scene-node
construction. Honest blockers flagged at the end.

---

## 0. TL;DR for the harness

1. **Wiring the window is already done and trivial.** The runtime replaces
   `ANativeWindow_fromSurface` with a host shim
   (`crates/arm64jit/src/shims.rs`, `anativewindow_fromsurface`). The shim
   **ignores both `JNIEnv*` and the surface `jobject`** and returns the XID that
   SH112 wired via `set_anativewindow_xid()`. So a host harness does **not** need
   to fabricate a real Android `Surface` object, does **not** need a
   `native_window` field, does **not** need a "real XID jobject". Pass **any
   non-null 64-bit pointer** as the surface arg.
2. Drive the guest JNI entry `nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams`
   (guest `0x1025f5fec`) as a normal **4-arg JNI call**:
   `x0=JNIEnv*`, `x1=jobject thiz`, `x2=jobject Surface (any non-null)`,
   `x3=jobject platformParams (any readable non-null pointer)`.
   It stores the wired XID into `[0x10683d348]` (refcount `[0x10683d344]`).
3. That function alone **stores the window and advances the app-bridge data
   model, but does NOT itself fire the `gameActivity_onAppReady` JNI callback.**
   To reach the `onAppReady` milestone headlessly, either (a) drive the engine's
   app-event path
   `nativeAppBridgeV2SendAppEventOnAppReady` (guest `0x102bb463c`), or (b) use
   the already-wired forced path: `GetMethodID("gameActivity_onAppReady")` +
   `CallVoidMethod` → sets `MH_APP_READY` (hermetic-pinned in `crates/arm64jit/src/jni.rs`).
4. **Ladder placement:** insert the update-surface call **immediately after
   `V2StartAppWithParams` (guest `0x10258b144`)** so the real window is resident
   at `[0x10683d348]` before any render/EGL-surface path reads it; fire the
   onAppReady callback (a) at the same point in the SAME thread, before the
   present-walker path.
5. **The real blocker is NOT the ABI.** The milestone atoms are set-only; the
   engine only constructs `GuiObject`/scene nodes if `StartLuaAppDM`'s Lua
   session actually advances, which today is parked (see §7).

---

## 1. `nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams` — exact ABI & instructions

Guest `0x1025f5fec` / file `0x25f5fec`, size `0x194` bytes (ends file `0x25f6180`).
Export: `Java_com_roblox_engine_jni_NativeGLInterface_nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams`
(JNI dotted mangle — plain two-object JNI, **no `jstring`**).

### 1.1 Argument contract (drive registers on entry)

| reg | JNI role | alive-to | purpose |
|----|----------|----------|---------|
| `x0` | `JNIEnv*` | whole fn → saved `x21` | ROM JNIEnv* the JIT intercepts |
| `x1` | `jobject` thiz | entry only | NativeGLInterface instance; **not used** after entry |
| `x2` | `jobject z` surface | → `x20` | **any non-null pointer** (host shim ignores it) |
| `x3` | `jobject` platformParams | → `x19` | readable non-null pointer (see §4) |

JNI descriptor (only the 2 object params matter):
`(Landroid/view/Surface;Lcom/roblox/engine/jni/PlatformAppParams;)V`-shaped —
consumer passes two opaque jobjects. There is nothing like the V1
`String,String,Z,String,String,String` pattern here **on the two we must supply**;
this entry is all-object.

### 1.2 First N instructions (decompiled, register roles annotated)

```asm
25f5fec  sub  sp, sp, #0xe0
25f5ff0  stp  x29,x30,[sp,#176]
25f5ff4  stp  x22,x21,[sp,#192]
25f5ff8  stp  x20,x19,[sp,#208]
25f5ffc  add  x29, sp, #0xb0
25f6000  adrp x22, 67d1000        ; x22 = GOT base (canary + PLT/GOT inject)
25f6008  adrp x8,  683d000
25f600c  mov  x19, x3             ; x19 = platformParams jobject          (ARG4)
25f6010  mov  x20, x2             ; x20 = Surface jobject                 (ARG3)
25f6014  mov  x21, x0             ; x21 = JNIEnv*                          (ARG1)
25f6018  ldr  x9, [x22]           ; stack-canary probe
25f601c  stur x9, [x29,#-8]
25f6020  ldr  x8, [x8,#848]       ; [0x683d350] version/level word gate
25f6024  and  w9, w8, #0xff
25f6028  and  x10,x8, #0xfc00
25f602c  cmp  w9, #0x6 ; ccmp ...  ; verbose-log level gate (skip = fine)
25f6064  adrp x2, 40a000; add x2,#0x73  ; x2 = rodata "nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams"
25f606c  mov  x0, x21             ; x0 = env
25f6070  mov  x1, x20             ; x1 = Surface jobject
25f6074  bl   258b374             ; APPLY SURFACE + STORE WINDOW  (guest 0x10258b374)
25f6078  adrp x8, 683d000
25f607c  sub  x0, x29, #0x40      ; x0 = &56-byte params-out struct (stack)
25f6080  mov  x1, x21             ; x1 = env
25f6084  ldr  x20,[x8,#840]       ; x20 = current window [0x10683d348]
25f6088  mov  x2, x19             ; x2 = platformParams jobject
25f608c  bl   258b4c4             ; PARSE PLATFORM PARAMS into x0 struct (guest 0x10258b4c4)
25f6090  ...                      ; pack 64-byte event struct, ctor an object
25f60e0  add  x0, sp, #0x40; mov x19, sp
25f60ec  bl   2baeeec             ; dispatch event into app-bridge (guest 0x102baeeec)
25f611c  ...                      ; stack-canary epilogue -> ret
```

The two callees are the **shared** helpers also used by `V2StartAppWithParams`
(guest `0x10258b144` calls both — see §5). Note: `258b374`/`258b4c4` are marked
with the `+0x230`/`+0x380` offsets of the `V2StartAppWithParams` symbol; they are
plain internal functions.

---

## 2. `0x10258b374` «applySurface + store window» — where the XID lands

Guest `0x10258b374` / file `0x258b374`. Drive-ABI: `x0=JNIEnv*`, `x1=Surface
jobject`, `x2=optional name string`; **returns void** (window stored in .bss).
This is the ONLY site that calls `ANativeWindow_fromSurface`.

```asm
258b374  sub sp,#0x70 ...            ; prologue; x20=x2(save name), x21=canary
258b39c  bl  ANativeWindow_fromSurface@plt   ; x0=env, x1=surface (unchanged)
                                            ; -> JIT host shim anativewindow_fromsurface
                                            ;    ignores both, returns wired XID
258b3a4  mov x19, x0                ; x19 = ANativeWindow* == the wired XID
258b3a8  ldr x8, [x8,#840]          ; x8 = old window   (guest [0x10683d348])
258b3b0  cmp x0,x8 ; b.eq 258b43c   ; same-surface fast path
258b3b8  [0x10683d344] += 1         ; refcount++        (word)
258b3cc  if(old) bl ANativeWindow_release@plt
258b3dc  str x19, [0x10683d348]     ; STORE new window ptr / XID
258b49c  mov sp,.. ; ret
```

### 2.1 Do-not-touch store sites (guest)

| guest addr | width | meaning |
|-----------|-------|---------|
| `0x10683d344` | word refcount | incremented on each apply |
| `0x10683d348` | pointer | **`ANativeWindow*` = the real XID** (EGL window surface consumes this) |
| `0x10683d350` | word | verbose-log level source (reads only) |

A host harness may therefore also **seed `[0x10683d348]` directly** with the XID —
but driving the real function is cleaner (it also does release/refcount + the
`2baeeec` data-model nudge) and needs no fabricated Surface.

---

## 3. The Surface `jobject`: nothing to fabricate (offsets for completeness)

In a stock Android, `ANativeWindow_fromSurface(env, jobject)` extracts the
Surface's internal `native_window` native field (the `Surface.mNativeObject`
long stored by the platform surface-flinger) and wraps it. **This runtime does
not use that path.** The JIT replaces the symbol:

```rust
// crates/arm64jit/src/shims.rs
extern "C" fn anativewindow_fromsurface(_env, _surf, ...) -> u64 {
    let xid = ANATIVE_WINDOW_XID.load(Relaxed);   // set_anativewindow_xid(SH112)
    if xid != 0 { return xid; }
    HOST_THUNK_BASE | 0x2000                       // sentinel only when no XID wired
}
```

Consequences, all verified by reading `shims.rs` + the PLT:
- The surface argument register is **dead**.
- There is **no `native_window` field read anywhere** in the boot path; no
  `GetLongField("native_window")` fires before `eglCreateWindowSurface`.
- The only requirement is that whatever the harness passes as `x2` is a readable
  non-null address (the shim marks `_surf` unused, so even a constant works, but
  keep it non-null to survive any intermediate register tracing).
- `anativewindow_release` is a **no-op**, so the harness pays no lifecycle cost
  when the old window is released at `258b3cc`.

So: **Surface jobject = any non-null 64-bit token.** The real XID is injected
out-of-band through `set_anativewindow_xid()` (SH112), independent of JNI.

---

## 4. `0x10258b4c4` platform-params read — what the third jobject must tolerate

Guest `0x10258b4c4` / file `0x258b4c4` (`parsePlatformParams`). ABI:
`x0=&out struct`, `x1=JNIEnv*`, `x2=platformParams jobject`. It reads three
fields off the platform-params object and writes them into the 56-byte struct:

```asm
258b4e4  strb w8,[x0,#24]; ...        ; init out struct
258b4e8  ldr x8,[x1]                   ; *env -> function table
258b4fc  ldr x8,[x8,#248]              ; slot 31 = GetObjectClass
258b510  mov x0,x1; mov x1,x2; blr x8  ; GetObjectClass(env, platformParams)
258b51c  mov x22, x0                   ; jclass
258b520  x3 = rodata 0x5421d0 "dpiScale"
258b534  bl 2366414(env, jclass, platformParams, "dpiScale")
            ; -> GetFieldID("dpiScale", f) + GetFloatField  -> out+8 (s0)
258b538  x3 = rodata 0x2d208c "currentNow"
258b550  bl 224de88(env, jclass, platformParams, "currentNow")  -> w23
258b558  x3 = rodata 0x398f55 "energyCounter"
258b56c  bl 224de88(env, jclass, platformParams, "energyCounter") -> w0
258b570  stp w23,w0,[out,#12]; ret     ; out+8 float dpiScale, out+12 ints
```

JNI slots used: `GetObjectClass` (248), `GetFieldID` (752/8=94),
`GetFloatField` (816/8=102). All are registered host shims in `jni.rs` that
return stable defaults (GetObjectClass returns a stable non-zero jclass per
jni.rs:861; field getters default 0 for unknown fields). The object pointer is
never dereferenced by the guest for its contents — reads go through JNI shims.

**Minimum:** pass a non-null readable pointer. Best-effort: point it at the real
`PlatformAppParams` jobject if one is reused from `V2InitWithParams`; a sentinel
(host-allocated, non-null) is functionally sufficient because the field getters
are no-op defaults. Flag: if the JIT's `GetFieldID`/`GetFloatField` shims are
updated to deref the object, the sentinel must point at readable memory.

---

## 5. Ordering & the shared helper duplication

`V2StartAppWithParams` (guest `0x10258b144`) itself calls **both** helpers:
- `258b204  bl 258b374`  ← applySurface + store window
- `258b278  bl 258b4c4`  ← parse params
then at `258b2dc  bl 2baeeec` (the same app-bridge event dispatch).

So `UpdateSurfaceAppWithPlatformParams` is effectively a **stripped re-run of the
surface half of `V2StartAppWithParams`** (takes the surface + params directly,
skips the JNI string/telemetry scaffolding of the StartApp path). Driving it is
valid in the same window as, or instead of, the corresponding lines of
`V2StartAppWithParams`.

---

## 6. Reachability: does update-surface alone fire onAppReady?

**No — not directly.** Disassembly shows `UpdateSurfaceAppWithPlatformParams`
only:
1. stores the window at `[0x10683d348]` (§2), and
2. ships a 64-byte event through `0x102baeeec` (`app-bridge data-model advance`,
   `w1=0` → `2baef6c bl 2206c40`, the gameGlobalInit+0x83c sub).

The `gameActivity_onAppReady` JNI callback is a **separate engine→Java dispatch**:
`RBX::NativeHelper_JNI::gameActivity_onAppReady(P12GameActivity, RKNSt6nkd112basic_string…)`
(`__func__` rodata `0x696c25`, method-name `0x595971`) → `CallVoidMethod(env,
nativeHelperObj, GetMethodID("gameActivity_onAppReady"))`. It fires once the
engine's session decides the surface is attached and the app-shell is ready —
not from within the update-surface JNI entry itself.

Two ways the harness actually reaches it headlessly:

**(a) Engine app-event path** — `nativeAppBridgeV2SendAppEventOnAppReady`
(guest `0x102bb463c`, file `0x2bb463c`, size `0x50c`). 6-arg JNI:
`x0=env`, `x1=thiz`, `x2..x5` = four `jstring`s. It string-enum-parses the first
string against `"GameActivity"`/`"Home"`/`"AvatarEditor"`/`"onResume"`/`"onPause"`
4-byte magic constants into an event code `w19`, builds an 88-byte event struct,
and dispatches it (to `2b4cd1c` / the app-event pipe). This is the documented
"parse the app-ready event" entry. To fire it you must supply a **real `jstring`
for `x2`** (the JIT's `GetStringUTFChars`/21e1fec must resolve it) with the
APP_READY event token — the hardest material requirement in this whole surface,
because it is the one place a fabricated string object matters.

**(b) Forced direct path (simplest, already proven)** — the JIT's
`GetMethodID("gameActivity_onAppReady")` + `CallVoidMethod` already sets
`MH_APP_READY` (jni.rs:732-760), and a hermetic test pins it (jni.rs:1749-1794).
This is the recommended harness action for the milestone atom.

### Recommended ladder insertion

```
[rung] nativeGameGlobalInit       (0x102206404, returns Ok — gate 0x72739d4.bit0)
  → nativeAppBridgeV2InitWithParams (0x102365c54)
  → nativeAppBridgeStartLuaAppDM    (0x1023efe2c, returns Ok)
  → nativeAppBridgeV2StartAppWithParams (0x10258b144)   ; internally stores window
  → nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams (0x1025f5fec)  ; <-- HERE, re-store
        x0=env, x1=thiz, x2=non-null, x3=readable params
  → fire onAppReady: forced CallVoidMethod("gameActivity_onAppReady")  ; same thread as the ladder
```

Single jit_run thread (SH55/SH64: the shared block cache SIGABRTs on concurrent
top-level entries). Surface/update must land **before** any render/EGL-window
surface path reads `[0x10683d348]`; onAppReady must fire on the **same thread**
that runs StartLuaAppDM so the session-advance sees the milestone.

---

## 7. First GuiObject → scene-node path (guest addresses)

Renderer scene-list contract (per `docs/frontier-sh63-scene-populated-nodes.md`):
- Render-manager `R`; `R+0x180` = head, `R+0x188` = tail (one-past-end),
  nodes are **0x28**-stride.
- Node layout: `+0x00` unused, `+0x08` **render-obj** (its `vt[+64]` is the
  dims-query the build reads; its `vt[+24]` is the per-item draw the present
  walker blr's → geometry emitter `0x105b35288`), `+0x18` view ptr (W/H at
  `+112/+116`, must be non-NULL), `+0x20` linker tail.
- **Per-node frame-build walker:** `0x105b2eb9c` reads head→tail, builds one real
  `0x98` frame-desc per node, links via `0x5b2d9e0`.
- **Per-node present walker:** `0x105b2ed48` (draws drags immediate-mode GL
  commands; SH63/SH64 note it SIGSEGVs if entered raw from the harness because of
  TLS canary + `nativeOnDestroyed` teardown tail — the parked
  `nativeGameGlobalInit` bl at `0x5b2ee54` is the gate to un-park).
- **Geometry emitter:** `0x105b35288` (primitive-setup `0x105b353d0`), the
  SH25-34/SH68-proven real draw path.

When `StartLuaAppDM`'s Lua session actually runs, `GuiService` builds the login /
home `GuiObject` tree; each `GuiObject` registers a **render object + scene node**
into this scene list (`R+0x180/0x188`). The renderer then walks the nodes and blr's
each `render-obj vt[+24]` → `0x105b35288`, so the **scene-node count is derived
from real instances with no host-authored layout.** That is the target end-state.

Honest reachability today:
- `UpdateSurfaceAppWithPlatformParams` + window store: **automatically safe** —
  the JIT already hands the XID to `eglCreateWindowSurface` and presents
  (SH112 runlog: real rendering clean). This part is **done**.
- `MH_APP_READY` milestone: **reachable** via the forced `CallVoidMethod` path;
  hermetic-pinned.
- **Real `GuiObject`/scene-node construction: NOT yet** — the milestone atoms are
  **set-only, not polled into a session-advance gate** (trigger-map note). The
  engine builds real GuiObjects only when `StartLuaAppDM`'s session advances past
  the SDK-init/login gates, which today parks/soft-returns
  (`nativeInitializeNativeFlags` soft-returns at pc outside image; SH111). That
  is the standing structural wall (route-B step 2 tail), not an ABI issue.

---

## 8. ABI blockers / caveats (be honest)

1. **No `Surface` fabrication needed** — best property of this whole deliverable;
   the host shim makes the surface arg dead. The only real-XID dependency is
   `set_anativewindow_xid()` (SH112), already wired.
2. **platformParams** must be non-null/readable; a sentinel works only while the
   GetFieldID/GetFloatField/GetIntField shims stay default-returning (they are).
   Prefer passing the real `PlatformAppParams` jobject reused from init.
3. **`SendAppEventOnAppReady` (option a) needs a real `jstring` for `x2`** if you
   drive the engine path — that is the one place a fabricated Java string object
   matters. The forced `CallVoidMethod` path (option b) avoids it entirely.
4. **Single-threaded ladder** (JIT block-cache can't take concurrent top-level
   `jit_run`). Keep update-surface + onAppReady on the ladder thread.
5. **Session advance is the wall.** Firing `MH_APP_READY` is necessary but not
   sufficient for GuiObjects; `StartLuaAppDM` must actually advance (login state,
   scheduler/flags gates, non-parking GlobalInit). Until those land, expect the
   milestone atoms to set without a scene-node count.