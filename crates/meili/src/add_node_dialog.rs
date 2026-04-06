//! State for “Add node” from the context menu and the Shell-hosted [`Input`](gpui_component::input::Input)
//! (gpui-component widgets must render in the Shell, not in Ferrum `Plugin::render`).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::pick_state;

static OPEN: AtomicBool = AtomicBool::new(false);
/// Cleared on next Shell render so the label field resets when opening.
static CLEAR_INPUT: AtomicBool = AtomicBool::new(false);
/// Right-click world position for placing the new node (`None` after take or close).
static PENDING_WORLD: Mutex<Option<(f32, f32)>> = Mutex::new(None);

pub fn open_at(world: gpui::Point<gpui::Pixels>) {
    pick_state::pending_set(None);
    OPEN.store(true, Ordering::SeqCst);
    CLEAR_INPUT.store(true, Ordering::SeqCst);
    let x: f32 = world.x.into();
    let y: f32 = world.y.into();
    if let Ok(mut g) = PENDING_WORLD.lock() {
        *g = Some((x, y));
    }
}

pub fn close() {
    OPEN.store(false, Ordering::SeqCst);
    if let Ok(mut g) = PENDING_WORLD.lock() {
        *g = None;
    }
}

pub fn is_open() -> bool {
    OPEN.load(Ordering::SeqCst)
}

pub fn take_should_clear_input() -> bool {
    CLEAR_INPUT.swap(false, Ordering::SeqCst)
}

/// Consumes the stored world position (call when confirming add). Returns `None` if missing.
pub fn take_pending_world() -> Option<(f32, f32)> {
    PENDING_WORLD.lock().ok().and_then(|mut g| g.take())
}
