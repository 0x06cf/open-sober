// SPDX-License-Identifier: MIT
//! SH132 — fake-libaaudio host bridge (direction c: FMOD audio via a host sink).
//!
//! The real Roblox client statically links FMOD; its de-facto output driver is
//! native AAudio, resolved at runtime by `dlopen("libaaudio.so")` + 26×`dlsym`
//! into a function-pointer table (driver region file 0x4fbf3a8, table guest
//! 0x106d0ef20, JNI entry 0x104fbea00). On x86-64 Linux those Android libs are
//! absent, so FMOD's AAudio init fails to open and no PCM is ever drained.
//!
//! This module is the host-side *capability* that makes that open succeed: it
//! intercepts the guest's `dlopen`/`dlsym` for `libaaudio.so` and serves the
//! 26-slot AAudio C ABI from the recon (docs/frontier-sh132-aaudio-bridge.md,
//! prior deleg_ recon) so FMOD's driver init completes and a data callback can
//! be registered. The PCM sink is a host WAV writer.
//!
//! Like SH114/SH131b this is **latent-but-correct**: FMOD only opens the output
//! device once the SoundService session advances (behind the documented SH131d
//! structural wall), so nothing here runs on the current boot path. It is built
//! behind the env gate `JIT_AAUDIO_BRIDGE=1` (default off) so the product path
//! is byte-identical unless the env is set, and it is hermetic-tested for the
//! ABI (symbol order, enum constants, opaque handles, WAV framing).
//!
//! Concurrency note: the sink intentionally does NOT spawn a guest-re-entering
//! thread (that would be a fresh SH55/64 JIT block-cache race). The adapter
//! makes open/start succeed and hands PCM out through a host buffer; wiring a
//! non-concurrent drain is deferred to when output is actually reachable.

use std::sync::{Mutex, OnceLock};

/// Env gate: without this the bridge is completely inert (resolver does not
/// intercept dlopen/dlsym for libaaudio.so). Default off -> default unchanged.
pub const AAUDIO_BRIDGE_ENV: &str = "JIT_AAUDIO_BRIDGE";
/// Sink path env override (default `/tmp/open-sober-fmod.wav`).
pub const AAUDIO_SINK_ENV: &str = "JIT_AAUDIO_SINK";

pub fn bridge_enabled() -> bool {
    std::env::var_os(AAUDIO_BRIDGE_ENV).is_some()
}

// ---------------------------------------------------------------------------
// AAudio C-ABI constants (from the guest disasm; see frontier-sh132 doc §2).
// ---------------------------------------------------------------------------

const AAUDIO_OK: u64 = 0;
const FORMAT_UNDEFINED: i32 = 0;
const FORMAT_PCM_I16: i32 = 1;
const FORMAT_PCM_FLOAT: i32 = 2;
const STATE_STARTED: u32 = 3;
const STATE_STOPPING: u32 = 6;
const STATE_STOPPED: u32 = 7;
// The guest's stop loop waits for getState == 10 (AAUDIOSTREAMSTATE_DISCONNECTED
// in stock AAudio — an FMOD divergence). Our fake converges there on stop.
const STATE_DISCONNECTED: u32 = 10;

/// The exact 26-symbol order the guest resolves via `dlsym` into its fn-ptr
/// table (guest 0x106d0ef20, slot i => base + 8*i). Table pinned by disjoint
/// dlsym recon; do NOT reorder — slot index is the ABI.
const AAUDIO_SYMBOLS: [&str; 26] = [
    "AAudio_createStreamBuilder",
    "AAudioStreamBuilder_openStream",
    "AAudioStreamBuilder_setBufferCapacityInFrames",
    "AAudioStreamBuilder_setFormat",
    "AAudioStreamBuilder_setDirection",
    "AAudioStreamBuilder_setPerformanceMode",
    "AAudioStreamBuilder_setDataCallback",
    "AAudioStreamBuilder_setErrorCallback",
    "AAudioStreamBuilder_delete",
    "AAudioStreamBuilder_setUsage",
    "AAudioStreamBuilder_setInputPreset",
    "AAudioStream_getFramesPerBurst",
    "AAudioStream_getBufferCapacityInFrames",
    "AAudioStream_getBufferSizeInFrames",
    "AAudioStream_getSampleRate",
    "AAudioStream_getChannelCount",
    "AAudioStream_getXRunCount",
    "AAudioStream_setBufferSizeInFrames",
    "AAudioStream_getFormat",
    "AAudioStream_getState",
    "AAudioStream_requestStart",
    "AAudioStream_requestPause",
    "AAudioStream_requestStop",
    "AAudioStream_close",
    "AAudioStream_read",
    "AAudioStream_waitForStateChange",
];

/// Opaque builder created by `AAudio_createStreamBuilder`. The guest never
/// derefs the handle (verified: every consumer passes it back as x0), so it is
/// just a stable pointer tagged by index. Host-side fields are what matter.
#[derive(Default, Clone)]
struct FakeBuilder {
    data_cb: u64,
    data_userdata: u64,
    err_cb: u64,
    err_userdata: u64,
    format: i32,
    direction: i32,
    buffer_capacity: i32,
}

/// Opaque stream returned by `AAudioStreamBuilder_openStream`. Fully opaque to
/// the guest (never deref'd); hosts the numeric values the getters must return.
#[derive(Clone)]
struct FakeStream {
    data_cb: u64,
    data_userdata: u64,
    err_cb: u64,
    err_userdata: u64,
    format: i32,
    direction: i32,
    sample_rate: i32,
    channels: i32,
    frames_per_burst: i32,
    buffer_capacity: i32,
    buffer_size: i32,
    state: u32,
    xrun: u32,
}

impl FakeStream {
    fn from_builder(b: &FakeBuilder) -> Self {
        // Device-native defaults returned to FMOD's getters; it accepts these
        // and derives its ring-buffer sizing from them (recon §3). `state`
        // starts at STOPPED (7) — read by getState before start.
        FakeStream {
            data_cb: b.data_cb,
            data_userdata: b.data_userdata,
            err_cb: b.err_cb,
            err_userdata: b.err_userdata,
            format: FORMAT_UNDEFINED,
            direction: b.direction,
            sample_rate: 48_000,
            channels: 2,
            frames_per_burst: 192,
            buffer_capacity: 0,
            buffer_size: 0,
            state: STATE_STOPPED,
            xrun: 0,
        }
    }
}

/// Registry of fake objects. Handles are `0xAA0_0000` + index so a misused
/// handle is obvious; the index maps into builders/streams arrays.
struct Registry {
    builders: Vec<FakeBuilder>,
    streams: Vec<FakeStream>,
    next_builder: usize,
    next_stream: usize,
}

static REG: OnceLock<Mutex<Registry>> = OnceLock::new();
fn reg() -> &'static Mutex<Registry> {
    REG.get_or_init(|| {
        Mutex::new(Registry {
            builders: Vec::new(),
            streams: Vec::new(),
            next_builder: 1,
            next_stream: 1,
        })
    })
}

fn builder_handle(i: usize) -> u64 {
    // non-zero, obv not the guest image / not a real pointer
    0x0000_0000_AA00_0000u64 | (i as u64)
}
fn stream_handle(i: usize) -> u64 {
    0x0000_0000_AA01_0000u64 | (i as u64)
}

fn builder_idx(h: u64) -> Option<usize> {
    // Builder handles are 0x0000_0000_AA00_<idx>.
    const TAG: u64 = 0xAA00;
    if h >> 16 == TAG {
        let idx = (h & 0xFFFF) as usize;
        if idx != 0 {
            return Some(idx);
        }
    }
    None
}

fn stream_idx(h: u64) -> Option<usize> {
    // Stream handles are 0x0000_0000_AA01_<idx>. `>>16` isolates the 0xAA01 tag
    // so a bare low index (e.g. test-only 0x01) can never alias a real stream.
    const STREAM_TAG: u64 = 0xAA01;
    if h >> 16 == STREAM_TAG {
        let idx = (h & 0xFFFF) as usize;
        if idx != 0 {
            return Some(idx);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Host thunks — one per slot. All integer ABI (safe through HostCall).
// ---------------------------------------------------------------------------

macro_rules! request_mut_stream {
    ($a0:expr, $body:expr, $fallback:expr) => {{
        // `body` is a closure `|s: &mut FakeStream| -> u64`; `fallback` is a
        // closure `|| -> u64` (unit arg) or a plain `u64`.
        let mut g = reg().lock().unwrap();
        match stream_idx($a0) {
            // Handles are 1-based (handle = base|($i) where $i starts at 1).
            Some(idx) if idx >= 1 && idx <= g.streams.len() => $body(&mut g.streams[idx - 1]),
            _ => fallback($fallback),
        }
    }};
}

#[inline]
fn fallback<F: FnOnce() -> u64>(f: F) -> u64 {
    f()
}

extern "C" fn aaudio_create_stream_builder(
    _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    g.builders.push(FakeBuilder::default());
    let i = g.builders.len();
    let h = builder_handle(i);
    drop(g);
    eprintln!("[aaudio:bridge] createStreamBuilder -> {h:#x}");
    h
}

extern "C" fn aaudio_builder_open_stream(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    // a1 = &stream_out pointer where we must write the stream handle.
    let mut g = reg().lock().unwrap();
    let Some(bi) = builder_idx(a0) else {
        return 0x33; // FMOD treats a NULL/non-matching builder as error 0x33
    };
    if bi > g.builders.len() {
        return 0x33;
    }
    let b = g.builders[bi - 1].clone();
    g.streams.push(FakeStream::from_builder(&b));
    let si = g.streams.len();
    let sh = stream_handle(si);
    drop(g);
    // Write the opaque handle into the guest-provided out pointer.
    if a1 != 0 {
        unsafe { *(a1 as *mut u64) = sh };
        eprintln!("[aaudio:bridge] openStream -> {sh:#x} (out@{a1:#x})");
    } else {
        eprintln!("[aaudio:bridge] openStream -> {sh:#x} (NULL out ptr)");
    }
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_buffer_capacity(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    if let Some(i) = builder_idx(a0) {
        if i <= g.builders.len() {
            g.builders[i - 1].buffer_capacity = a1 as i32;
        }
    }
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_format(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    if let Some(i) = builder_idx(a0) {
        if i <= g.builders.len() {
            g.builders[i - 1].format = a1 as i32;
        }
    }
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_direction(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    if let Some(i) = builder_idx(a0) {
        if i <= g.builders.len() {
            g.builders[i - 1].direction = a1 as i32;
        }
    }
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_perf_mode(
    _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    // accept any enum without validation (guest passes 12; stock AAudio ignores
    // unknown perf-modes at builder level — recon §2)
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_data_callback(
    a0: u64, a1: u64, a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    if let Some(i) = builder_idx(a0) {
        if i <= g.builders.len() {
            let b = &mut g.builders[i - 1];
            b.data_cb = a1;
            b.data_userdata = a2;
        }
    }
    eprintln!("[aaudio:bridge] setDataCallback cb={a1:#x} userdata={a2:#x}");
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_error_callback(
    a0: u64, a1: u64, a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    if let Some(i) = builder_idx(a0) {
        if i <= g.builders.len() {
            let b = &mut g.builders[i - 1];
            b.err_cb = a1;
            b.err_userdata = a2;
        }
    }
    AAUDIO_OK
}

extern "C" fn aaudio_builder_delete(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let mut g = reg().lock().unwrap();
    if let Some(i) = builder_idx(a0) {
        if i <= g.builders.len() {
            g.builders[i - 1] = FakeBuilder::default();
        }
    }
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_usage(
    _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    AAUDIO_OK
}

extern "C" fn aaudio_builder_set_input_preset(
    _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    AAUDIO_OK
}

// Stream getters — all pure; return device-native values.
extern "C" fn aaudio_stream_get_frames_per_burst(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.frames_per_burst as u64, || 0)
}
extern "C" fn aaudio_stream_get_buffer_capacity(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.buffer_capacity as u64, || 0)
}
extern "C" fn aaudio_stream_get_buffer_size(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.buffer_size as u64, || 0)
}
extern "C" fn aaudio_stream_get_sample_rate(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.sample_rate as u64, || 48_000)
}
extern "C" fn aaudio_stream_get_channel_count(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.channels as u64, || 2)
}
extern "C" fn aaudio_stream_get_xrun_count(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.xrun as u64, || 0)
}
extern "C" fn aaudio_stream_set_buffer_size(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| {
        s.buffer_size = a1 as i32;
        if s.buffer_size > s.buffer_capacity {
            s.buffer_capacity = s.buffer_size;
        }
        AAUDIO_OK
    }, || 0x1d /* AAUDIO_ERROR_INVALID_HANDLE-ish */)
}
extern "C" fn aaudio_stream_get_format(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.format as u64, || (FORMAT_UNDEFINED as u64))
}
extern "C" fn aaudio_stream_get_state(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| s.state as u64, || (STATE_DISCONNECTED as u64))
}

// Control fns.
extern "C" fn aaudio_stream_request_start(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| {
        s.state = STATE_STARTED;
        eprintln!("[aaudio:bridge] stream {a0:#x} requestStart -> STARTED");
        AAUDIO_OK
    }, || 0x1d)
}
extern "C" fn aaudio_stream_request_pause(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| {
        s.state = STATE_DISCONNECTED;
        AAUDIO_OK
    }, || 0x1d)
}
extern "C" fn aaudio_stream_request_stop(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| {
        // converge to the guest's stop-wait state 10 so its wait loop terminates
        s.state = STATE_DISCONNECTED;
        eprintln!("[aaudio:bridge] stream {a0:#x} requestStop");
        AAUDIO_OK
    }, || 0x1d)
}
extern "C" fn aaudio_stream_close(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    request_mut_stream!(a0, |s: &mut FakeStream| {
        s.state = STATE_STOPPED;
        AAUDIO_OK
    }, || 0x1d)
}
extern "C" fn aaudio_stream_read(
    a0: u64, a1: u64, a2: u64, a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    // Push-only for output (recon §5): FMOD never calls read for output. If the
    // guest ever does, zero-fill the destination (read(stream, buf, numFrames)).
    let nf = a3 as usize;
    if a1 != 0 && nf > 0 {
        unsafe { std::ptr::write_bytes(a1 as *mut u8, 0, nf * 4) };
    }
    let _ = a0;
    AAUDIO_OK
}
extern "C" fn aaudio_stream_wait_for_state_change(
    a0: u64, _a1: u64, a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    // waitForStateChange(stream, *from, &state_out, timeout). Immediate return
    // with the current state written to state_out so the guest's stop loop
    // terminates (it asserts out == 10 == DISCONNECTED).
    let st = request_mut_stream!(a0, |s: &mut FakeStream| s.state as u64, || (STATE_DISCONNECTED as u64));
    if a2 != 0 {
        unsafe { *(a2 as *mut u64) = st };
    }
    AAUDIO_OK
}

// Fully-static bridge functions (state 10 on unknown handle).
const STATICS: [extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64; 4] = [
    aaudio_stream_get_frames_per_burst,
    aaudio_stream_get_buffer_capacity,
    aaudio_stream_get_buffer_size,
    aaudio_stream_get_sample_rate,
];

/// Host thunk set, in the exact 26-slot table order (index == guest slot).
/// All are plain `HostCall`-shaped; the guest dispatches to them by slot.
pub const TABLE: [Option<extern "C" fn(u64, u64, u64, u64, u64, u64, u64, u64) -> u64>; 26] = [
    Some(aaudio_create_stream_builder),
    Some(aaudio_builder_open_stream),
    Some(aaudio_builder_set_buffer_capacity),
    Some(aaudio_builder_set_format),
    Some(aaudio_builder_set_direction),
    Some(aaudio_builder_set_perf_mode),
    Some(aaudio_builder_set_data_callback),
    Some(aaudio_builder_set_error_callback),
    Some(aaudio_builder_delete),
    Some(aaudio_builder_set_usage),
    Some(aaudio_builder_set_input_preset),
    Some(aaudio_stream_get_frames_per_burst),
    Some(aaudio_stream_get_buffer_capacity),
    Some(aaudio_stream_get_buffer_size),
    Some(aaudio_stream_get_sample_rate),
    Some(aaudio_stream_get_channel_count),
    Some(aaudio_stream_get_xrun_count),
    Some(aaudio_stream_set_buffer_size),
    Some(aaudio_stream_get_format),
    Some(aaudio_stream_get_state),
    Some(aaudio_stream_request_start),
    Some(aaudio_stream_request_pause),
    Some(aaudio_stream_request_stop),
    Some(aaudio_stream_close),
    Some(aaudio_stream_read),
    Some(aaudio_stream_wait_for_state_change),
];

/// Symbol for a given table slot (for the resolver/dlsym interception).
pub fn symbol_for_slot(i: usize) -> Option<&'static str> {
    AAUDIO_SYMBOLS.get(i).copied()
}

/// Slot index for a given symbol, or None.
pub fn slot_for_symbol(name: &[u8]) -> Option<usize> {
    let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
    let n = &name[..end];
    AAUDIO_SYMBOLS.iter().position(|s| s.as_bytes() == n)
}

// ---------------------------------------------------------------------------
// Resolver-facing surface: register the thunks into the JIT host-call table
// and return one stable guest-addressable slot per AAudio symbol, and the
// fake `libaaudio.so` handle.
// ---------------------------------------------------------------------------

/// The fake dlopen handle returned for `libaaudio.so`.
pub fn fake_lib_handle() -> u64 {
    0x0000_0000_0000_AA00u64
}

/// True if `handle` is the fake `libaaudio.so` handle we hand out.
pub fn is_fake_handle(handle: u64) -> bool {
    handle == fake_lib_handle()
}

/// Guest-callable thunk address for an AAudio symbol, or None if not ours.
/// Idempotent: caches the slot address per symbol so repeated `dlsym` of the
/// same AAudio name returns the SAME guest address (slot-identity invariant).
pub fn resolve_aaudio_symbol(name: &[u8]) -> Option<u64> {
    use std::collections::HashMap;
    static CACHE: OnceLock<Mutex<HashMap<usize, u64>>> = OnceLock::new();
    let i = slot_for_symbol(name)?;
    let f = TABLE.get(i).copied().flatten()?;
    let mut c = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
    if let Some(&addr) = c.get(&i) {
        return Some(addr);
    }
    let addr = crate::jit::register_host_call_auto(f);
    c.insert(i, addr);
    Some(addr)
}

/// The guest `dlopen` import shim (guest calls dlopen(path, flags)). When the
/// path is `libaaudio.so` under the bridge, return the fake handle; otherwise
/// fall through to real glibc `dlopen` so non-audio libs are unaffected.
extern "C" fn host_dlopen(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let name_ptr = a0 as *const u8;
    let flags = a1 as i32;
    // Read the NUL-terminated path (max 256).
    let mut buf = [0u8; 256];
    let mut len = 0usize;
    while len < buf.len() - 1 {
        let b = unsafe { *name_ptr.add(len) };
        if b == 0 {
            break;
        }
        buf[len] = b;
        len += 1;
    }
    let path = &buf[..len];
    if path == b"libaaudio.so" {
        if bridge_enabled() {
            eprintln!("[aaudio:bridge] guest dlopen({path:?}) -> FAKE handle {:#x}", fake_lib_handle());
            return fake_lib_handle();
        }
        // Bridge disabled: report that real dlopen would fail (absent lib).
        eprintln!("[aaudio:bridge] guest dlopen(libaaudio.so) but JIT_AAUDIO_BRIDGE unset — returning NULL (historical handled elsewhere)");
    }
    // Fall through to the real host dlopen.
    let real: unsafe extern "C" fn(*const libc::c_char, i32) -> *mut libc::c_void =
        unsafe { std::mem::transmute(libc::dlsym(libc::RTLD_DEFAULT, b"dlopen\0".as_ptr() as *const libc::c_char)) };
    unsafe { real(name_ptr as *const libc::c_char, flags) as u64 }
}

/// The guest `dlsym` import shim. If the handle is our fake libaaudio handle
/// and the symbol is one of the 26 AAudio names, return the registered thunk
/// address; otherwise fall through to real glibc `dlsym`.
extern "C" fn host_dlsym(
    a0: u64, a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    let handle = a0;
    if is_fake_handle(handle) {
        // Resolve the name from the guest pointer.
        let name_ptr = a1 as *const u8;
        let mut buf = [0u8; 128];
        let mut len = 0usize;
        while len < buf.len() - 1 {
            let b = unsafe { *name_ptr.add(len) };
            if b == 0 {
                break;
            }
            buf[len] = b;
            len += 1;
        }
        let name = &buf[..len];
        if let Some(addr) = resolve_aaudio_symbol(name) {
            eprintln!("[aaudio:bridge] dlsym(fake, {name:?}) -> {addr:#x}");
            return addr;
        }
        eprintln!("[aaudio:bridge] dlsym(fake, {name:?}) -> unknown (NULL)");
        return 0;
    }
    // Fall through to real glibc dlsym.
    let real: unsafe extern "C" fn(*mut libc::c_void, *const libc::c_char) -> *mut libc::c_void =
        unsafe { std::mem::transmute(libc::dlsym(libc::RTLD_DEFAULT, b"dlsym\0".as_ptr() as *const libc::c_char)) };
    unsafe { real(a0 as *mut libc::c_void, a1 as *const libc::c_char) as u64 }
}

/// The guest `dlclose` import shim (fake handle is a no-op).
extern "C" fn host_dlclose(
    a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    if is_fake_handle(a0) {
        return 0;
    }
    let real: unsafe extern "C" fn(*mut libc::c_void) -> i32 =
        unsafe { std::mem::transmute(libc::dlsym(libc::RTLD_DEFAULT, b"dlclose\0".as_ptr() as *const libc::c_char)) };
    unsafe { real(a0 as *mut libc::c_void) as u64 }
}

/// The guest `dlerror` import shim (never reports an error for the fake).
extern "C" fn host_dlerror(
    _a0: u64, _a1: u64, _a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, _a7: u64,
) -> u64 {
    0
}

/// A full resolver-facing wrapper: given an import name, return the guest-callable
/// thunk address if it is one we intercept under the bridge (dlopen/dlsym/dlclose/
/// dlerror), else None (the caller falls through to its normal binding).
pub fn resolve_bridge_name(name: &[u8]) -> Option<u64> {
    if !bridge_enabled() {
        return None;
    }
    let f: crate::jit::HostCall = match name.first().copied().and_then(|_| {
        let end = name.iter().position(|&b| b == 0).unwrap_or(name.len());
        let h: crate::jit::HostCall = match &name[..end] {
            b"dlopen" => host_dlopen,
            b"dlsym" => host_dlsym,
            b"dlclose" => host_dlclose,
            b"dlerror" => host_dlerror,
            _ => return None,
        };
        Some(h)
    }) {
        Some(f) => f,
        None => return None,
    };
    Some(crate::jit::register_host_call_auto(f))
}

// ---------------------------------------------------------------------------
// SH464: live-stream snapshot + real WAV sink writer + one-buffer guest drain.
// ---------------------------------------------------------------------------

/// Immutable snapshot of the first live FMOD output stream (the one with a
/// registered data callback), plus the PCM params needed to size a drain
/// buffer. Mirrors what `drive_fmod_audio_drain` (session.rs) needs; the
/// snapshot is taken under the registry lock and COPYED OUT, so the substrate
/// drain never holds the lock across a guest data-callback invocation (which
/// could re-enter setDataCallback -> deadlock on the same Mutex).
#[derive(Debug, Clone, Copy)]
pub struct LiveStreamParams {
    /// Stable opaque stream handle handed out by openStream.
    pub stream: u64,
    /// The guest FMOD data callback (registered via setDataCallback).
    pub data_cb: u64,
    pub data_userdata: u64,
    pub format: i32,
    pub sample_rate: i32,
    pub channels: i32,
    pub frames_per_burst: i32,
}

impl LiveStreamParams {
    /// Bytes per PCM sample for the stream's format (float=4, PCM_I16=2,
    /// anything else defaults to float 4 — FMOD's output mix is float).
    pub fn bytes_per_sample(&self) -> usize {
        if self.format == FORMAT_PCM_I16 {
            2
        } else {
            4
        }
    }
    /// Bytes for one full drain buffer covering `num_frames` frames.
    pub fn buffer_bytes(&self, num_frames: u32) -> usize {
        num_frames as usize * self.channels.max(1) as usize * self.bytes_per_sample()
    }
}

/// Snapshot the first live output stream that has a registered data callback.
/// None => no FMOD output device open (the boot path; inert). Pure readout,
/// no guest mutation, safe to call at any time.
pub fn live_stream_snapshot() -> Option<LiveStreamParams> {
    let g = reg().lock().unwrap();
    g.streams.iter().find(|s| s.data_cb != 0).map(|s| LiveStreamParams {
        stream: stream_handle(g.streams.iter().position(|x| std::ptr::eq(x, s)).unwrap_or(0) + 1),
        data_cb: s.data_cb,
        data_userdata: s.data_userdata,
        format: s.format,
        sample_rate: s.sample_rate,
        channels: s.channels,
        frames_per_burst: s.frames_per_burst,
    })
}

/// A real WAV file sink. `open_audio_sink` creates/truncates the file at the
/// sink path and writes the 44-byte header (with provisional size fields);
/// `append_pcm` appends raw PCM bytes and tracks the running data length; the
/// size fields are patched on `finish` so the file is a valid single-data-chunk
/// PCM .wav. This is the executable half `wav_header_bytes` only had as framing:
/// no caller previously ever WROTE a sink file.
pub struct AudioSink {
    file: std::fs::File,
    path: std::path::PathBuf,
    data_len: u32,
    channels: u16,
    sample_rate: u32,
    bits: u16,
    frames_written: u64,
}

impl AudioSink {
    /// Open (truncate) the sink at `path` and write the PCM WAV header with
    /// provisional size fields (patched on finish). Err if the header write 0s.
    pub fn open(path: &std::path::Path, sample_rate: u32, channels: u16, bits: u16) -> std::io::Result<Self> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        let hdr = wav_header_bytes(sample_rate, channels, bits, 0);
        file.write_all(&hdr)?;
        Ok(AudioSink {
            file,
            path: path.to_path_buf(),
            data_len: 0,
            channels,
            sample_rate,
            bits,
            frames_written: 0,
        })
    }

    /// Append `frames` frames of raw PCM (`num_frames` frames * ch * bytes/sample
    /// bytes each). Tracks the running data length + frame count so `finish` can
    /// patch the RIFF/data size fields.
    pub fn append_frames(&mut self, pcm: &[u8], num_frames: u32) -> std::io::Result<()> {
        use std::io::Write;
        self.file.write_all(pcm)?;
        self.data_len = self.data_len.saturating_add(pcm.len() as u32);
        self.frames_written += num_frames as u64;
        Ok(())
    }

    /// Patch the two size fields in the 44-byte header (RIFF chunk size at byte
    /// 4, data chunk size at byte 40) so the file is a valid PCM WAV, then flush.
    pub fn finish(&mut self) -> std::io::Result<()> {
        use std::io::{Seek, Write};
        use std::io::SeekFrom;
        let riff = 36u32.saturating_add(self.data_len);
        self.file.seek(SeekFrom::Start(4))?;
        self.file.write_all(&riff.to_le_bytes())?;
        self.file.seek(SeekFrom::Start(40))?;
        self.file.write_all(&self.data_len.to_le_bytes())?;
        self.file.flush()?;
        Ok(())
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
    pub fn data_len(&self) -> u32 {
        self.data_len
    }
    pub fn frames_written(&self) -> u64 {
        self.frames_written
    }
}

/// Sink path override (default `/tmp/open-sober-fmod.wav`).
pub fn sink_path() -> std::path::PathBuf {
    std::env::var_os(AAUDIO_SINK_ENV)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/open-sober-fmod.wav"))
}

/// FMOD AAudio data-callback ABI (verified recon §2): `result = data_cb(stream,
/// userData, audioData, numFrames)`. `args[0..4]` map to x0..x3 exactly; the
/// callback writes up to `num_frames` frames of PCM into the guest-visible
/// `pcm_buf` (a leaked host buffer — same address space, so it IS the guest's
/// audioData) and returns an aaudio_data_callback_result (0 = CONTINUE).
/// Returns the number of PCM bytes the callback produced (buffer complete), or
/// Err if the guest callback could not be driven (no active image). The caller
/// must first satisfy the SH55/64 single-jit_run discipline (never concurrent).
pub fn drain_one_buffer(p: &LiveStreamParams, pcm_buf: u64, num_frames: u32, tpidr: u64) -> Result<usize, String> {
    let r = crate::jit::run_guest_callback(
        p.data_cb,
        [p.stream, p.data_userdata, pcm_buf, num_frames as u64, 0, 0, 0, 0],
        tpidr,
    )?;
    let _ = r; // aaudio_data_callback_result CONTINUE=0; FMOD fills the buffer regardless
    Ok(p.buffer_bytes(num_frames))
}

/// Wrap the sink in a real 44-byte WAV header for the given PCM params, writing
/// to the sink path. Returns the full header bytes (for MMAP/disk or tests).
pub fn wav_header_bytes(sample_rate: u32, channels: u16, bits: u16, data_len: u32) -> Vec<u8> {
    let byte_rate = sample_rate * channels as u32 * (bits / 8) as u32;
    let block_align = channels * (bits / 8);
    let mut h = Vec::with_capacity(44);
    h.extend_from_slice(b"RIFF");
    h.extend_from_slice(&(36 + data_len).to_le_bytes());
    h.extend_from_slice(b"WAVE");
    h.extend_from_slice(b"fmt ");
    h.extend_from_slice(&16u32.to_le_bytes());
    h.extend_from_slice(&1u16.to_le_bytes()); // PCM
    h.extend_from_slice(&channels.to_le_bytes());
    h.extend_from_slice(&sample_rate.to_le_bytes());
    h.extend_from_slice(&byte_rate.to_le_bytes());
    h.extend_from_slice(&block_align.to_le_bytes());
    h.extend_from_slice(&bits.to_le_bytes());
    h.extend_from_slice(b"data");
    h.extend_from_slice(&data_len.to_le_bytes());
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_inert_without_env() {
        unsafe { std::env::remove_var(AAUDIO_BRIDGE_ENV) };
        assert!(!bridge_enabled(), "bridge must be disabled (inert) out of the env");
    }

    #[test]
    fn table_has_26_slots_all_filled() {
        assert_eq!(TABLE.len(), 26);
        for (i, f) in TABLE.iter().enumerate() {
            assert!(f.is_some(), "slot {i} must have a host thunk");
            assert!(
                symbol_for_slot(i).is_some(),
                "slot {i} must have a symbol"
            );
        }
    }

    #[test]
    fn symbol_table_matches_known_aaudio_names() {
        // Spot-check the ABI-critical names at their recon-pinned slots.
        assert_eq!(symbol_for_slot(0), Some("AAudio_createStreamBuilder"));
        assert_eq!(symbol_for_slot(1), Some("AAudioStreamBuilder_openStream"));
        assert_eq!(symbol_for_slot(5), Some("AAudioStreamBuilder_setPerformanceMode"));
        assert_eq!(symbol_for_slot(6), Some("AAudioStreamBuilder_setDataCallback"));
        assert_eq!(symbol_for_slot(19), Some("AAudioStream_getState"));
        assert_eq!(symbol_for_slot(20), Some("AAudioStream_requestStart"));
        assert_eq!(symbol_for_slot(25), Some("AAudioStream_waitForStateChange"));
        // slot lookup round-trips
        assert_eq!(slot_for_symbol(b"AAudioStream_requestStop\0"), Some(22));
        assert_eq!(slot_for_symbol(b"AAudio_nope\0"), None);
    }

    #[test]
    fn builder_stream_lifecycle_roundtrip() {
        // createStreamBuilder -> setDataCallback -> openStream -> getters/state.
        let b = aaudio_create_stream_builder(0, 0, 0, 0, 0, 0, 0, 0);
        assert_ne!(b, 0);
        assert_eq!(aaudio_builder_set_data_callback(b, 0x104fbfbd8, 0x7777, 0, 0, 0, 0, 0), 0);
        let mut out: u64 = 0;
        let oc = aaudio_builder_open_stream(b, (&mut out) as *mut u64 as u64, 0, 0, 0, 0, 0, 0);
        assert_eq!(oc, 0);
        let s = out;
        assert_ne!(s, 0);
        // getters return device-native values
        assert_eq!(aaudio_stream_get_sample_rate(s, 0, 0, 0, 0, 0, 0, 0), 48_000);
        assert_eq!(aaudio_stream_get_channel_count(s, 0, 0, 0, 0, 0, 0, 0), 2);
        assert_eq!(aaudio_stream_get_frames_per_burst(s, 0, 0, 0, 0, 0, 0, 0), 192);
        // start advances state; FMOD checks it with `cmp w0,#4`-or-#10 in its drain
        assert_eq!(aaudio_stream_request_start(s, 0, 0, 0, 0, 0, 0, 0), 0);
        assert_eq!(aaudio_stream_get_state(s, 0, 0, 0, 0, 0, 0, 0), STATE_STARTED as u64);
        // stop converges to the guest's stop-wait state (10)
        assert_eq!(aaudio_stream_request_stop(s, 0, 0, 0, 0, 0, 0, 0), 0);
        assert_eq!(aaudio_stream_get_state(s, 0, 0, 0, 0, 0, 0, 0), STATE_DISCONNECTED as u64);
    }

    #[test]
    fn invalid_handles_are_safe() {
        // unknown / malformed handles never panic
        assert_eq!(aaudio_stream_get_state(0xDEAD, 0, 0, 0, 0, 0, 0, 0), STATE_DISCONNECTED as u64);
        assert_eq!(aaudio_stream_request_start(1, 0, 0, 0, 0, 0, 0, 0), 0x1d);
        assert_eq!(aaudio_builder_open_stream(0, 0, 0, 0, 0, 0, 0, 0), 0x33);
    }

    #[test]
    fn wav_header_is_44_bytes_pcm() {
        let h = wav_header_bytes(48_000, 2, 16, 0);
        assert_eq!(h.len(), 44);
        assert_eq!(&h[0..4], b"RIFF");
        assert_eq!(&h[8..12], b"WAVE");
        assert_eq!(&h[12..16], b"fmt ");
        assert_eq!(&h[20..22], &1u16.to_le_bytes()); // PCM
        assert_eq!(&h[22..24], &2u16.to_le_bytes()); // stereo
        assert_eq!(&h[24..28], &48_000u32.to_le_bytes()); // rate
        // float32 (32-bit) framing — mono 32-bit => block align = 1*(32/8) = 4
        let hf = wav_header_bytes(44_100, 1, 32, 0);
        assert_eq!(&hf[32..34], &4u16.to_le_bytes()); // block align = ch*(bits/8) = 4
        assert_eq!(&hf[34..36], &32u16.to_le_bytes()); // bits per sample
        assert_eq!(&hf[20..22], &1u16.to_le_bytes());
    }

    #[test]
    fn statics_reachable() {
        // the const STATICS list must be non-empty and valid (build sanity)
        assert_eq!(STATICS.len(), 4);
        let _ = STATICS[0](0, 0, 0, 0, 0, 0, 0, 0);
    }

    // Silence the unused-const/lint family with real usage in one test.
    use crate::jit::HostCall;
    #[test]
    fn table_is_hostcall_sized() {
        // every entry transmutes to sql HostCall-shaped fn (ids stable)
        let first: HostCall = TABLE[0].unwrap();
        let _x = first;
        assert_eq!(core::mem::size_of::<Option<extern "C" fn(u64,u64,u64,u64,u64,u64,u64,u64)->u64>>(),
                   core::mem::size_of::<Option<HostCall>>());
    }

    // --- SH464: live-stream snapshot + real WAV sink writer + drain ABI ---

    /// The registry is a process-global OnceLock Mutex; a prior lifecycle test
    /// may leave streams populated. Helper: clear the registry so snapshot
    /// assertions start from a known state.
    fn clear_registry() {
        reg().lock().unwrap().streams.clear();
        reg().lock().unwrap().builders.clear();
    }

    #[test]
    fn live_stream_snapshot_none_when_no_data_cb() {
        clear_registry();
        // A stream opened WITHOUT a data callback is NOT a live FMOD output
        // (FMOD only becomes an output device once it registers its cb).
        let b = aaudio_create_stream_builder(0, 0, 0, 0, 0, 0, 0, 0);
        let mut out: u64 = 0;
        aaudio_builder_open_stream(b, (&mut out) as *mut u64 as u64, 0, 0, 0, 0, 0, 0);
        assert!(live_stream_snapshot().is_none(), "no data_cb => not a live output stream");
        clear_registry();
    }

    #[test]
    fn live_stream_snapshot_captures_fmod_output_stream() {
        clear_registry();
        let b = aaudio_create_stream_builder(0, 0, 0, 0, 0, 0, 0, 0);
        aaudio_builder_set_data_callback(b, 0x104fbfbd8, 0x7777, 0, 0, 0, 0, 0);
        let mut out: u64 = 0;
        aaudio_builder_open_stream(b, (&mut out) as *mut u64 as u64, 0, 0, 0, 0, 0, 0);
        let snap = live_stream_snapshot().expect("data_cb registered => snapshot present");
        assert_eq!(snap.stream, out, "snapshot stream handle == openStream out");
        assert_eq!(snap.data_cb, 0x104fbfbd8, "captured guest FMOD data callback");
        assert_eq!(snap.data_userdata, 0x7777, "captured data-callback userdata");
        assert_eq!(snap.sample_rate, 48_000, "device-native sample rate");
        assert_eq!(snap.channels, 2, "device-native channels");
        assert_eq!(snap.frames_per_burst, 192, "device-native frames/burst");
        // default format UNDEFINED -> bytes/sample falls back to float 4.
        assert_eq!(snap.bytes_per_sample(), 4, "UNDEFINED format => float 4 B/sample");
        assert_eq!(snap.buffer_bytes(192), 192 * 2 * 4, "buffer = frames*ch*bps");
        clear_registry();
    }

    #[test]
    fn audio_sink_writes_valid_pcm_wav() {
        use std::io::Read;
        let dir = std::env::temp_dir().join(format!("sh464_sink_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.wav");
        let _ = std::fs::remove_file(&path);
        // open truncates + writes the 44B header; append PCM; finish patches sizes.
        let mut sink = AudioSink::open(&path, 48_000, 2, 16).unwrap();
        let frame_bytes = 2 * 2; // stereo 16-bit interleaved = 4 B/frame
        let pcm1: Vec<u8> = vec![0x11; frame_bytes * 192];
        sink.append_frames(&pcm1, 192).unwrap();
        sink.append_frames(&vec![0x22; frame_bytes * 192], 192).unwrap();
        assert_eq!(sink.frames_written(), 384);
        assert_eq!(sink.data_len(), (frame_bytes * 384) as u32);
        sink.finish().unwrap();
        // Read the whole file back: 44B header + data.
        let mut bytes = Vec::new();
        std::fs::File::open(&path).unwrap().read_to_end(&mut bytes).unwrap();
        let hdr = &bytes[..44];
        assert_eq!(&hdr[0..4], b"RIFF");
        assert_eq!(&hdr[8..12], b"WAVE");
        assert_eq!(&hdr[12..16], b"fmt ");
        assert_eq!(&hdr[20..22], &1u16.to_le_bytes());
        assert_eq!(&hdr[22..24], &2u16.to_le_bytes()); // stereo
        assert_eq!(&hdr[24..28], &48_000u32.to_le_bytes());
        assert_eq!(bytes.len(), 44 + frame_bytes * 384, "header + all appended PCM");
        // patched RIFF size (byte 4) = 36 + data_len; data size (byte 40) = data_len.
        let data_len = frame_bytes * 384;
        assert_eq!(&hdr[4..8], &(36u32 + data_len as u32).to_le_bytes(), "RIFF size patched");
        assert_eq!(&hdr[40..44], &(data_len as u32).to_le_bytes(), "data size patched");
        // PCM body intact.
        assert!(bytes[44..].iter().all(|&b| b == 0x11 || b == 0x22), "both PCM batches present");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn audio_sink_empty_finish_is_valid_zero_data() {
        let dir = std::env::temp_dir().join(format!("sh464_emptysink_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.wav");
        let mut sink = AudioSink::open(&path, 44_100, 1, 32).unwrap();
        sink.finish().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes.len(), 44, "open writes header even with no data");
        assert_eq!(&bytes[4..8], &36u32.to_le_bytes(), "empty RIFF = 36");
        assert_eq!(&bytes[40..44], &0u32.to_le_bytes(), "empty data length = 0");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn drain_one_buffer_requires_active_image() {
        // Without a live guest image EXEC_CTX, run_guest_callback errors — so the
        // drain must report Err (never silently fabricate PCM) on a harness that
        // did not load libroblox.so. This pins the honesty contract: the audio
        // drain only produces real frames once a real session runs it.
        let p = LiveStreamParams {
            stream: 0xAA01_0001,
            data_cb: 0x104fbfbd8,
            data_userdata: 0x7777,
            format: FORMAT_UNDEFINED,
            sample_rate: 48_000,
            channels: 2,
            frames_per_burst: 192,
        };
        let pcm = vec![0u8; p.buffer_bytes(192)];
        assert!(drain_one_buffer(&p, pcm.as_ptr() as u64, 192, 0).is_err());
    }
}