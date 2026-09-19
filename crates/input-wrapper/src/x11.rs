//! X11 event source for input-wrapper (§6): connect to an X server (headless via
//! Xvfb or a real desktop), create a window, translate server mouse/keyboard events
//! into Android touch/key events via [`input::PointerTracker`], and hand them to a
//! caller callback (which the runtime routes to the guest AInputQueue).
//!
//! This module is backend-agnostic on purpose — the translation itself lives in
//! `input.rs` and is fully unit-tested. The `pump` function here only proves the
//! X11 wiring compiles and can pull real events headlessly (Xvfb smoke test).

use std::sync::OnceLock;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ChangeWindowAttributesAux, ConnectionExt, CreateWindowAux, EventMask, WindowClass,
};
use x11rb::rust_connection::RustConnection;

use crate::input::{self, PointerTracker, MotionEvent};

/// The connection that owns the runtime's ANativeWindow X11 window. X11 only
/// allows a client to register input interest (ChangeWindowAttributes/event_mask)
/// on a window from ITS OWN connection; a second connection's attempt is a
/// BadAccess. So the window-creating connection is registered here (by the runtime
/// window layer, which otherwise `Box::leak`s it just to keep the window alive) and
/// [`pump_registered_window`] reuses it to select + drain that window's events.
static OWNER_CONN: OnceLock<RustConnection> = OnceLock::new();

/// Register the connection that created the runtime's desktop X11 window so the
/// input pump can select input on that window (same client = no BadAccess).
/// Called once by the window layer after it opens the window. A leaked clone is
/// what keeps the window alive for the boot; here we also hand the pump a live
/// handle to drain events on it.
pub fn register_window_connection(conn: RustConnection) {
    // Keep ONE owner connection; a fresh one per window replaces the prior. The
    // first wins so a second window-layer registration in the same process keeps
    // the earliest (which is also the one whose XID is registered as ANativeWindow).
    let _ = OWNER_CONN.set(conn);
}

/// Errors surfaced from the X11 layer.
#[derive(Debug)]
pub enum XError {
    Connect(String),
    Create(String),
}

/// Open a connection to the X server given by `display` (e.g. ":99" for Xvfb) and
/// create a mapped 640x480 window. Returns the connection + window id, or an error.
pub fn open_window(display: Option<&str>) -> Result<(RustConnection, u32), XError> {
    open_window_sized(display, 640, 480)
}

/// Like [`open_window`] but with an explicit framebuffer `width`/`height`. The
/// runtime's ANativeWindow layer (GRAPHICS_RECOMMENDATION §5.3) hands the guest
/// a desktop window as its ANativeWindow handle, so the window is sized to the
/// framebuffer `ANativeWindow_getWidth/Height` report (1280x720) — a coherent
/// window whose EGL window surface is buildable by Mesa's x11 platform.
pub fn open_window_sized(
    display: Option<&str>,
    width: u16,
    height: u16,
) -> Result<(RustConnection, u32), XError> {
    let (conn, screen_num) = x11rb::connect(display)
        .map_err(|e| XError::Connect(format!("x11rb connect: {e}")))?;
    let screen = &conn.setup().roots[screen_num];
    let win = conn
        .generate_id()
        .map_err(|e| XError::Create(format!("generate_id: {e}")))?;
    conn.create_window(
        screen.root_depth,
        win,
        screen.root,
        0,
        0,
        width,
        height,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new()
            .background_pixel(screen.white_pixel)
            .event_mask(
                EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::KEY_PRESS
                    | EventMask::KEY_RELEASE,
            ),
    )
    .map_err(|e| XError::Create(format!("create_window: {e}")))?
    .check()
    .map_err(|e| XError::Create(format!("create_window check: {e}")))?;
    conn.map_window(win)
        .map_err(|e| XError::Create(format!("map: {e}")))?
        .check()
        .map_err(|e| XError::Create(format!("map check: {e}")))?;
    Ok((conn, win))
}

/// Drain pending server events once, translating mouse/keyboard through the tracker
/// and invoking `dispatch` for each synthesized Android event. Non-blocking.
pub fn pump(
    conn: &RustConnection,
    tracker: &mut PointerTracker,
    dispatch: &mut dyn FnMut(MotionEvent),
) -> Result<(), XError> {
    use x11rb::protocol::Event;
    loop {
        let Ok(Some(ev)) = conn.poll_for_event() else {
            break;
        };
        match ev {
            Event::ButtonPress(b) | Event::ButtonRelease(b) => {
                let pressed = matches!(ev, Event::ButtonPress(_));
                // Only the primary button drives the touch stream; wheel/context are
                // non-touch and deliberately ignored here (guests read those via keys).
                if b.detail == input::button::PRIMARY {
                    let e = tracker.on_button(b.detail, pressed, b.event_x as f32, b.event_y as f32);
                    dispatch(e);
                }
            }
            Event::MotionNotify(m) => {
                if let Some(e) = tracker.on_motion(m.event_x as f32, m.event_y as f32) {
                    dispatch(e);
                }
            }
            // Keyboard is handled by the caller (needs keysym decode); we expose the
            // keycode mapping in input.rs. Here we merely ignore key events.
            _ => {}
        }
    }
    Ok(())
}

/// Subscribe an EXISTING window (e.g. the guest's registered ANativeWindow XID)
/// to pointer/button/motion events on `display`, then drain any already-queued
/// events once through the tracker, dispatching each translated Android touch
/// event to `dispatch`. Returns the number of native Android events dispatched.
///
/// Unlike [`open_window_sized`] this does NOT create a window — it consumes the
/// window the runtime already created and registered as the guest ANativeWindow,
/// so a host poll loop can deliver real pointer input to a constructed
/// login/home screen. Non-blocking (single drain), safe to poll repeatedly.
///
/// The event interest is selected, not grabbed, so the pointer stays with X and
/// other clients are untouched; the window itself stays alive because the runtime
/// leaks the connection that created it.
pub fn pump_registered_window(
    _display: Option<&str>,
    window: u32,
    tracker: &mut PointerTracker,
    dispatch: &mut dyn FnMut(MotionEvent),
) -> Result<usize, XError> {
    // Use the connection that CREATED the window: X11 only lets a client select
    // input interest (event_mask on ChangeWindowAttributes) on a window it owns.
    // A fresh connection would get BadAccess on Xvfb.
    let conn = match OWNER_CONN.get() {
        Some(c) => c,
        None => {
            return Err(XError::Connect(
                "no owner connection registered for the input pump".into(),
            ));
        }
    };
    // Select (not grab) pointer/button/motion interest for THIS client on the
    // existing window, so its events are delivered to our connection.
    conn.change_window_attributes(
        window,
        &ChangeWindowAttributesAux::new().event_mask(
            EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION,
        ),
    )
    .map_err(|e| XError::Create(format!("change_window_attributes: {e}")))?
    .check()
    .map_err(|e| XError::Create(format!("change_window_attributes check: {e}")))?;
    let mut dispatched = 0usize;
    pump(conn, tracker, &mut |e| {
        dispatch(e);
        dispatched += 1;
    })?;
    Ok(dispatched)
}

/// Block for a short time (used in the smoke test to let a real server run).
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}