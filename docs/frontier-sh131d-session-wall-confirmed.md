# SH131d — Session-advance structural wall re-confirmed (deleg_d6a8999d)

## Status (honest, re-confirmed from a fresh disassembly angle)
The engine's StartLuaAppDM do-init / SendAppEventOnAppReady can NOT be made to
construct the app-data-model / Lua app-shell / DataModel / first GuiObject by any
harness-runnable seed, .text patch, or env toggle on this headless VPS. This is
structural, not a missing seed.

## Evidence (disasm by deleg_d6a8999d)
- **No soft-return bail remains.** StartLuaAppDM (0x1023efe2c) with the SH109
  version-gate [0x10683d350] low-byte==6 takes the main path, builds a stack
  app-bridge object (vtable file 0x635dd68 populated at runtime), and calls the
  pipe 0x2baeeec → (sync-gate [0x10683d010]=-1, SH126) → do-init 0x2206c40.
- **do-init 0x2206c40** consumes the once-guard [0x106a68410].bit0 (SH122/125
  seed it), reads flags-latch [0x106a683e8].bit0 (SH125), and hits the thread
  gate 0x2206db8 (`pthread_self` vs [0x106863a68]; SH122 seeds to self → b.ne
  NOT taken → the MAIN path `br [stack_obj vtable+0x30]` = the actual
  Lua/app-shell constructor).
- **Ascent already the deepest it can go:** SH122 hit json serialization +
  telemetry clamps; SH123 cleared the 0x10217582c set-find and reached
  0x102208xx; the exit(232) deep in the do-init is intercepted (SH124). The
  stopping point is engine-internal (GameActivity/FMOD), i.e. the inbound JNI
  app-shell (ScriptContext/DataModel Lua) the SH129/JNI-arena wall cannot supply
  headlessly.
- **Ok(0x3e8)=1000 is the engine's OWN app-bridge status constant** ("invocation
  handled, no session node built"), not a JIT soft-return sentinel. It appears
  because jit_run returns x[0] after the guest's final ret (pc==0). So the rungs
  aren't "bailing" — they run and the engine reports handled-without-building.

## Consequence
- The harness `R+0x180/0x188` scene-list (SH68/renderscene) and the SH115/126
  vtable materializations are harness-fed fakes, not engine self-built nodes.
- The 24-frame task-plane (SH130/131) is the seeded type-4 producer + thunk,
  not a session-driven producer.
- Real engine-self-constructed screens (login/home/in-game) and the cookie-jar /
  .ROBLESECURITY login-persist contract (SH129) remain the standing structural
  wall, reachable only if/when the Lua app-shell + a fully-wired Java Activity
  can be provided — out of reach of the current JIT-layer seeding.

## Going forward
Do NOT re-introduce seeds/patches targeting the session-advance directly; the
harness-accessible gates are all removed and it is confirmed structural. The
productive directions are (a) hardening whatever the real client touches on its
reachable boot path (syscalls/threading/JNI/GLES as they surface), (b) the
harness-authored render/emitter frame plane as a correctness-test of the
engine's own GLES path, and (c) audio (FMOD) via the host bridge. A GPU host may
change feasibility only for performance, not this structural gap.