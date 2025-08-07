/// Handler for modifier keys (Ctrl, Shift, Alt, Super)

use crate::core::ModifierState;
use gtk4::gdk;

pub struct ModifierHandler {
    state: ModifierState,
}

impl ModifierHandler {
    /// Create new handler with given initial state
    pub fn new(initial_state: ModifierState) -> Self {
        Self {
            state: initial_state,
        }
    }


    /// Get current state (for comparison or external use)
    pub fn state(&self) -> &ModifierState {
        &self.state
    }

    /// Handle key press event
    /// Returns true if this was a modifier key we handle, false otherwise
    pub fn handle_key_press(&mut self, keyval: gdk::Key) -> bool {
        match keyval {
            gdk::Key::Control_L | gdk::Key::Control_R => {
                self.state.ctrl = true;
                true
            },
            gdk::Key::Shift_L | gdk::Key::Shift_R => {
                self.state.shift = true;
                true
            },
            gdk::Key::Alt_L | gdk::Key::Alt_R => {
                self.state.alt = true;
                true
            },
            gdk::Key::Super_L | gdk::Key::Super_R => {
                self.state.super_key = true;
                true
            },
            _ => false,
        }
    }

    /// Handle key release event
    /// Returns true if this was a modifier key we handle, false otherwise
    pub fn handle_key_release(&mut self, keyval: gdk::Key) -> bool {
        match keyval {
            gdk::Key::Control_L | gdk::Key::Control_R => {
                self.state.ctrl = false;
                true
            },
            gdk::Key::Shift_L | gdk::Key::Shift_R => {
                self.state.shift = false;
                true
            },
            gdk::Key::Alt_L | gdk::Key::Alt_R => {
                self.state.alt = false;
                true
            },
            gdk::Key::Super_L | gdk::Key::Super_R => {
                self.state.super_key = false;
                true
            },
            _ => false,
        }
    }
}
