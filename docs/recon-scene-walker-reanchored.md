# Recon — Route-B scene-walker RE-ANCHORED (deleg_bc7f04d0, READ-ONLY)

## The present-walker really does use R+0x180 / R+0x188
Prior recon claimed "no literal [R,#0x180]/[R,#0x188] anywhere in .text" — that was
a mis-located walker. The present-walker is **file 0x5b2ed48 (guest 0x105b2ed48)**,
and its node-walk loop at **file 0x5b2eec0 (guest 0x105b2eec0)** is:

```
ldp x20, x22, [x19, #0x180]   ; HEAD = this+0x180, TAIL = this+0x188
cmp x20, x22 ; b.eq  (terminate when head==tail)
loop:
  ldr x0, [x20, #8]           ; x0 = node+0x08 = render-obj
  ldr x8, [x0]                ; vtable
  ldr x8, [x8, #0x18]         ; present-fn slot
  blr x8                      ; present(node+0x08)
  add x20, x20, #0x28         ; next node (CONTIGUOUS 0x28-stride slots, no stored next deref)
  cmp x20, x22 ; b.ne loop
```

So the node struct as ACTUALLY used at the blr:
- **node+0x08** = render-obj (its vtable[0x18] is the presented function) — load-bearing
- **node+0x00** unused
- **next = node+0x28** (intrusive contiguous stride, `add`, not a pointer chase)
- No separate "view ptr" deref in this walker.

After the loop it also presents a fixed object at [this+0x160] via vtable[0x18] and
clears byte [this+0x260].

## Render manager R = the walker's `this`
R's true list head/tail are literally **R+0x180 / R+0x188**. The harness planting
those is CORRECT. Because nodes are a contiguous 0x28-stride buffer, engine
self-built nodes only present when they physically sit in [R+0x180 .. R+0x188].

## Why the harness cannot (yet) prove engine nodes auto-flow here
The walker 0x5b2ed48 is reached ONLY via indirect dispatch: 0 direct bl/b, 0
reloc/vtable qword or i32 points to it in the whole image, and offsets 0x180/0x188
are ubiquitous. No node-append site / R singleton is statically derivable.

## Deterministic present-gating for a PROBE node (Route-A, de-prioritized while Route B open)
To make the walker present a node deterministically (instead of its virtual/gated
path):
- byte [R+0x260] != 0 (the gate; walker clears it at 0x5b2eef4 after presenting)
  AND [R+0x298]==NULL OR [R+0x2a0]==NULL to stay on the deterministic path.
- R+0x180 = P (node buffer), R+0x188 = P+0x28 (single node -> EXACTLY ONE blr)
- P+0x08 = a render-obj whose vtable[0x18] is a benign/observable present stub.

## HONEST bottom line
The scene model is real and the harness's R+0x180/0x188 planting is correct. But
proving the ENGINE self-built nodes flow here needs a runtime trace of the (still
indirect) node-append, which requires the session to construct a live DataModel /
GuiObject — the standing Route-B content wall. A harness-planted probe node would
prove the walker presents real node+0x08 objects but is Route-A render work,
de-prioritized while Route B is open.