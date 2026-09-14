# Frontier SH132 — fake-libAAudio host bridge (FMOD audio, direction c)

## Status
SH132 (latent-but-correct). Workspace green (530/0, elfjit example 41/0).
Commit: SH132. Env gate `JIT_AAUDIO_BRIDGE=1` (default off, product path byte-identical).

## Why
The real Roblox client statically links FMOD; its de-facto output driver is
native **AAudio**, resolved at runtime by `dlopen("libaaudio.so")` + 26×`dlsym`
into a function-pointer table (driver region file 0x4fbf3a8, table guest
0x106d0ef20, JNI entry 0x104fbea00). On x86-64 Linux those Android libs are
absent, so FMOD's AAudio init fails to open and no PCM is ever drained. This
bridge gives the host the *capability* that makes that open succeed.

## What
New module `crates/arm64jit/src/aaudio.rs`:

1. **26-slot ABI table** (`AAUDIO_SYMBOLS`, `TABLE`) pinned to the exact dlsym
   order the guest resolves into its fn-ptr table. Slot index is the ABI — do
   NOT reorder (disjoint dlsym recon).
2. **Host-state fake objects**: `FakeBuilder`/`FakeStream` behind a
   `Mutex<Registry>`. Handles are tagged opaque u64s (`0xAA00_<idx>` builder,
   `0xAA01_<idx>` stream) so a misused handle is obvious and never aliases a
   real pointer. The guest never derefs the handles (verified).
3. **Device-native defaults** returned by getters so FMOD derives sane ring
   buffers: 48 kHz, stereo, 192 frames/burst, sample rate 48000. `state`
   converges to `AAUDIOSTREAMSTATE_DISCONNECTED (10)` on stop/pause so the
   guest's stop-wait loop terminates (FMOD divergence: stock AAudio would not
   use 10 as the stop-wait state, but the guest asserts out==10).
4. **dlopen/dlsym/dlclose/dlerror intercept** (`host_dlopen` etc.) in the
   resolver: only for `libaaudio.so` under the bridge; everything else falls
   through to real glibc. Idempotent slot registration via
   `resolve_aaudio_symbol` (per-symbol CACHE) so repeated dlsym returns the
   SAME guest address (slot-identity invariant).
5. **WAV sink**: `wav_header_bytes` builds a 44-byte PCM WAV header framing the
   host sink path (default `/tmp/open-sober-fmod.wav`, env `JIT_AAUDIO_SINK`).

## Concurrency
The sink intentionally does NOT spawn a guest-re-entering thread (that would be
a fresh SH55/64 JIT block-cache race). The adapter makes open/start succeed and
hands PCM out through a host buffer; wiring a non-concurrent drain is deferred
to when output is actually reachable.

## Honest scope
Latent-but-correct: FMOD only opens the output device once the SoundService
session advances (behind the documented SH131d structural wall), so nothing here
runs on the current boot path. It is the correct host-side capability on the
audio direction; it stays inert behind the env gate until the session wall is
reached.

## Verification
- `cargo build --workspace` OK.
- `cargo test --workspace` green (530/0). New hermetic aaudio tests: bridge
  inert-without-env, 26-slot table completeness, symbol names at pinned slots,
  builder→stream lifecycle roundtrip (getters/state transitions), invalid-handle
  safety, WAV 44-byte PCM framing (incl. block-align at offset 32),
  HostCall-shape sanity.
- Fixed a WAV test bug: block-align field is at byte offset 32 (bits-per-sample
  at 34); the original test read the wrong offset.

## Next (audio direction)
- When the session advances and FMOD's AAudio output device is actually opened,
  wire a non-concurrent PCM drain from the host buffer to the WAV sink.