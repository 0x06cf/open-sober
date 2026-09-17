# Frontier SH261 — app-start drain surface advances past the LSM wall (measured, live-object, pinned)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. Workspace green, no default-config production path edited
(pure regression-pin + measurement, default-inert).

## Why (genuinely new, not a re-tread)

SH260 (d323ef6/f5d187e) parked the LocalStorageManager insert-leaf
(SIGSEGV 0x101db1d04) as the persistence/data-store detour and returned to
Route B. The operator directive forbids treating any "wall" as a reason to
stop; it demands the closest unblocked thing be advanced. At THIS HEAD the
exact SH259 repro (full SH248c-f + SH259 seed set, real libroblox.so) was
re-run: the app-start body now walks DEEP past the LSM line into its own
self-drive message-loop drain. That surface was never measured before —
SH260's terminal (LSM wall) is NOT where control stops in all runs.

## Measured result

Region-watch on the deep app-start window [0x10233a000,0x102350000) fires
100+ distinct pcs (run-to-run 148..248) INCLUDING the app-start drain
0x233bc88..0x233bcf0:

```
0x233bc88 mov x0,x20
0x233bc8c bl  0x21dae90   ; the drain once-check / spin
0x233bcb4 bl  0x21daef8   ; registry/once builder (sub sp,#0x30, then map-insert op_new(0x18) @0x1db07c)
0x233bcb8 adrp x8,6a70000; ldr x0,[x8,#400]  ; read [[0x6a70c90]]
0x233bcc0 cbz x0 -> 0x233bcf8
0x233bcc4 ldr x8,[x0]; ldr x8,[x8,#48]; blr x8  ; LIVE heap-object virtual dispatch vt+48
0x233bce0 ldr x8,[x19,#16]; subs; ... (loop counter)
```

### Terminal is RUN-VARIABLE live-object class

Run-to-run the deep app-start path ends in one of three, all the SH174/SH204
live-object gate:

1. **LSM insert-leaf SIGSEGV 0x101db1d04** (dominant; SH260 already measured
   + parked as the persistence detour).
2. **`std::bad_function_call`** thrown inside the drain path — an EMPTY
   `std::function` target is invoked. The empty function object is a LIVE
   host-heap app-start object, NOT a fixed .bss global: there is no constant
   address to hand-seed, so this is the fabricatable-object-graph/live-this
   class, not a seedable cell.
3. An early-return SIGSEGV at engine-init 0x1021748a4 (the app-hang-monitor /
   engine-init object chain).

The drain terminates by virtual-dispatching vt+48 on a host-heap object
(0x233bcc4..0x233bccc) and by invoking a live heap std::function — both
runtime-resolved, both the same structural gate.

## Decision / honest (do-not-over-claim)

- The app-start body demonstrably advances PAST the LSM line into its own
  self-drive drain — a real, newly-measured surface (SH260's parked terminal
  is not the only endpoint).
- It STILL terminates at live-object construction (empty std::function /
  heap vt dispatch) — this is the SAME Route-B structural gate, at a deeper
  point. No fixed-.bss seed lever exists in the new surface.
- Does NOT manufacture a DataModel. Does NOT lift the Route-B live-DM gate.
- SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>)
  remains the single forward hook.

## Artefacts

- `sh261_appstart_drain_surface_advances_past_lsm_wall_pinned`
  (crates/arm64jit/examples/elfjit.rs, real-image guard family, skip-if-absent):
  10 byte-pins — the drain block (0x233bc88/8c/b4/b8/c0/c4/ccc/e0),
  the registry-builder entry (0x21daef8), and the map-insert op_new site
  (0x21db07c) — plus window/4-alignment. Drift fails loudly.
- Verify: `cargo test -p arm64jit --example elfjit sh261` = 1 passed;
  `cargo test --workspace` exit 0 (398 arm64jit + all crates).
- Repro: runs/capture_sh259_settings_once.sh (fresh measurement; this cycle).
- recon-v3 plane re-verified green at THIS HEAD: runs/g-recon.txt = 24
  task-driven frames `swap Ok(0x1)`, 196 node pops, 0 json abort, 0 crash,
  EXIT 124.

## Route-B standing (unchanged)

Route-B live-DM = structural gate. SH174 capture-latch stays the single
forward hook. recon-v3 deliverables (type4 self-driven frames + json zero-fix)
stay shipped + verified.