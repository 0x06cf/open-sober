# Open-Sober run status (hermes-worker)

Updated this cycle: SH464 — promote the FMOD/AAudio AUDIO axis to a first-class
driven substrate step (closing the SEP-18 "audio/input — each landed + green +
committed" deliverable list: input landed SH418, audio now SH464). Workspace
green (cargo test --workspace EXIT 0; arm64jit lib 669->675 incl. 6 new sh464
tests; cargo build --workspace + --example elfjit OK). recon-v3 self-driven-frame
deliverable unchanged-green (audio-axis-only change; type4-path untouched).
Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, structural at the
write site per SH462/463). @top

## Current state

- `dev` HEAD: SH464. recon-v3 immediate-priority deliverables green at HEAD
  (type4 self-driven frames + json-abort — verified SH461-VERIFY, re-verified
  SH463). Workspace green (arm64jit 675/0; cargo build --workspace OK).
- SH464 new: audio axis as a first-class driven substrate step — the FMOD/AAudio
  drain `drive_fmod_audio_drain` (session.rs) runs FMOD's captured data_cb via
  run_guest_callback, fills a PCM buffer, and writes a REAL WAV sink (`AudioSink`,
  aaudio.rs). CLOSES the SH132 latent gap (the bridge captured the callback but
  no PCM was ever produced). Inert-by-construction on boot (three guards), fires
  the instant a SoundService session opens FMOD's output.
- Route-B live-DM wall measured two levels deep (SH462 store-watch): once-lambda
  world-build runs + writes sentinel 0x400000b into once-slot, NO guest store
  ever reaches DM-root [0x106a68818] — structural at the write site, DM built by
  an upstream session ctor only a real host drive constructs. Session/runtime-
  surface lever (SEP-17/18) unchanged as the operator-aligned next frontier.

## This cycle's advance

- SH464: audio promoted to a first-class driven substrate step (the input-twin
  of SH418), completing the SEP-18 "audio/input — each landed + green +
  committed" deliverable list. `drive_fmod_audio_drain` (session.rs) + the real
  WAV `AudioSink` + `live_stream_snapshot`/`drain_one_buffer` (aaudio.rs), 6 new
  hermetic tests, all green. Audio-axis only; no re-treads, no DM seed,
  not a render-plane visual.

## Honest status

- No DM (DM-root [0x106a68818]=0, no store reaches it headlessly, MH_GAME_LOADED
  false) — Route-B live-DM structural gate UNCHANGED, confirmed at the store
  level. Content (fsmap remap, R1 CoreScript stage, G3 files-dir), lifecycle
  (MH_*), input (ainput bridge + X11 loop), audio (AAudio drain now a first-class
  substrate step), boot-stack, x86 emitter, and the arm64 translator core remain
  latent-but-correct, firing the instant a live DM owns a session.

## Next-forward candidates

1. (standing, TOP — Route B) do-init completeness / live-DM: measured at read +
   write (SH462) sites as structural. Aligned lever is the session-ctor /
   runtime-surface drive (SEP-17/18): build the Android/Java/session compat
   layer so the engine's OWN session constructs the DM world, not seeds.
2. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged standing gate).
3. Do-not-re-tread unchanged: LSM skips/rebuilds, setDataModelToCurrent, EC
   reader, window-attach real, ALooper, governor gates, and the two nativeInit
   'outside image' substrate atoms (documented run-variable pre-existing lane).
4. Do NOT run the SH174 latch without JIT_DM_ALLOC_CAPTURE_DELEGATE=1.
5. No re-treads until Route B advances or a new family is identified from a
   real-run decode gap (SH463 confirmed the family space is genuinely complete).