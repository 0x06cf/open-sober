# Frontier SH464 — promote the FMOD/AAudio audio axis to a first-class driven substrate step (the input twin of SH418)

Worker: hermes-worker · workspace green (arm64jit lib 675/0 incl. 6 new sh464
tests; cargo build --workspace + --example elfjit OK).

## Session
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at HEAD (SH463 baseline; this change is audio-axis-only, the
runtime deliverable's type4-frame path is untouched). Route-B live-DM gate
UNCHANGED (structural at the DM-root write site per SH462/463).

## Why this cycle
The SEP-18 "BUILD THE RUNTIME, NOT THE DM" deliverable list explicitly names
"session boot, then screens, then **audio/input — each landed + green +
committed**." Input was PROMOTED to a first-class ordered-substrate step in
SH413/414/417/418 (`drive_host_input_pump/poll/loop`, wired into
`drive_routeb_session_substrate` after the post-bus content-surface step, with
three inert guards + hermetic tests). Audio was NOT — SH132's fake-libAAudio
bridge only captured FMOD's `data_cb` and made `dlopen("libaaudio.so")` +
`dlsym` succeed; NOTHING ever invoked that callback headlessly, so no PCM was
produced and the WAV sink (only `wav_header_bytes` framing) was never written.
That is the concrete, aligned, non-retread gap closed here.

## What
Two crates touched, both audio-axis only:

### crates/arm64jit/src/aaudio.rs (the audio capability is now executable)
- `LiveStreamParams` + `live_stream_snapshot()`: snapshot the first live FMOD
  output stream (the one with a registered `data_cb`) PLUS its PCM params
  (format/sample_rate/channels/frames_per_burst) + `bytes_per_sample()` /
  `buffer_bytes()`. Pure readout; None when no FMOD output open (boot path).
  Snapshot is COPYED OUT under the registry lock so the drain never holds the
  lock across a guest data-callback invocation (which could re-enter
  setDataCallback -> deadlock on the same Mutex).
- `AudioSink`: a REAL WAV file writer. `open` creates/truncates + writes the
  44-byte header; `append_frames` appends raw PCM + tracks data_len/frames;
  `finish` patches the RIFF/data size fields so the file is a valid playable
  single-data-chunk PCM .wav. This is the executable half `wav_header_bytes`
  only had as framing (no caller previously ever WROTE a sink file).
- `sink_path()` (default `/tmp/open-sober-fmod.wav`, eval `JIT_AAUDIO_SINK`).
- `drain_one_buffer()`: FMOD AAudio data-callback ABI (recon §2),
  `result = data_cb(stream, userData, audioData(pcm_buf), numFrames)` via
  `run_guest_callback(data_cb, [stream, userdata, pcm_buf, numFrames, ...])`,
  returning the produced PCM bytes. Holder of the honesty contract: without an
  active guest image it errors (never silently fabricates PCM).
- Env constant `AAUDIO_SINK_ENV` made `pub` (the substrate test sets it).

### crates/arm64jit/src/session.rs (audio promoted to a first-class step)
- `audio_drain_iters()` (env `AUDIO_DRAIN_ITERS`, default 8) — bounded like
  `input_loop_iters()`, so the ordered drive never spins.
- `drive_fmod_audio_drain(iimg, ib, tpidr, boot_sp, iterations)`: the host
  audio drain, wired into `drive_routeb_session_substrate` immediately after
  the SH418 input loop (same post-bus trigger site). Gated on the SAME three
  inert guards as the input pump/loop — (1) JIT_AAUDIO_BRIDGE env, (2) a live
  image, (3) a live FMOD stream snapshot (a stream with a captured data_cb).
  Any trip -> return 0 without touching the guest / writing a sink. When armed
  it opens the WAV sink once, then repeatedly snapshots the stream, runs the
  guest data callback through `run_guest_callback` to fill a PCM buffer (FMOD
  mixes real audio into it), and appends the produced frames to the sink,
  patching the header on finish. Returns total PCM frames drained.
- Must run on the single jit_run ladder thread (SH55/64 — never concurrent).
  Env-gated -> default product path byte-identical.

## Verification
- Workspace green: `cargo build --workspace` + `--example elfjit` OK; `cargo
  test --workspace` EXIT 0. arm64jit lib 675/0 (was 669) — 6 new hermetic tests:
  5 in aaudio.rs (`live_stream_snapshot_none_when_no_data_cb`,
  `live_stream_snapshot_captures_fmod_output_stream`,
  `audio_sink_writes_valid_pcm_wav`, `audio_sink_empty_finish_is_valid_zero_data`,
  `drain_one_buffer_requires_active_image`) + 1 in session.rs
  (`sh464_audio_drain_is_first_class_substrate_step` inert-guard chain, mirroring
  sh418). All parallel-safe (no shared fixed .bss); registry clear of the
  process-global aaudio registry kept to the aaudio tests themselves.
- One transient full-run flake on the DOCUMENTED pre-existing routeb test
  `sh362_dispatch_body_trace_is_read_only_env_pc_gated` (shared 0xdead fixed .bss
  cell clobbered by a parallel sibling -> false 222/57005; the test locks
  ROUTEB_PROC_TEST_LOCK for exactly this). Passes 3/3 in isolation and on the
  clean workspace re-run; my audio additions don't touch that cell/env. Re-verified
  green before commit.

## Honest
- NOT a DM; does NOT construct a DataModel. Route-B live-DM gate UNCHANGED
  (DM-root [0x106a68818]=0, structural per SH462/463). This is the SEP-18
  "audio" deliverable LIST item landed (input landed SH418): the audio axis is
  now a first-class driven substrate step that produces REAL PCM from FMOD's
  captured data callback into a REAL WAV sink the instant a SoundService session
  advances (latent-but-correct, fires on the session at the right time).
- No re-treads. Not a render-plane visual (Route-A art), not a DM seed.

## Files / verify
- crates/arm64jit/src/aaudio.rs + crates/arm64jit/src/session.rs (audio-axis).
- Verify: `cargo test -p arm64jit --lib` 675/0; `cargo build --workspace` +
  `--example elfjit` OK.
- Commit: local `dev` only (operator pushes to origin).