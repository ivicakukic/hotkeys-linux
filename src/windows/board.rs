/// GTK4-based 3x3 board window for Linux
/// Provides pixel-perfect recreation of Windows HotKeys UI

use crate::core::{Board, ModifierState, Resources};
use super::layout::{WindowLayout, BoardLayout};
use super::renderer;
use super::modifier_handler::ModifierHandler;
use anyhow::Result;
use gdk4::Key;
use gtk4::prelude::*;
use gtk4::{glib, gdk, GestureClick};
use cairo;
use std::rc::Rc;
use std::cell::RefCell;


/// Main 3x3 board window for Linux with GTK4
pub struct BoardWindow {}

impl BoardWindow {
    /// Show board window using the new app.connect_activate approach
    pub fn show_with_app(
        app: &gtk4::Application,
        board: &dyn Board,
        timeout: u64,
        feedback: u64,
        layout: WindowLayout,
        resources: Resources,
        result_receiver: Rc<RefCell<Option<(u8, ModifierState)>>>,
    ) -> Result<()> {
        // Create GTK4 window and associate with application
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title(&format!("HotKeys - {}", board.title()))
            .width_request(600)
            .height_request(450)
            .default_width(layout.size.width as i32)
            .default_height(layout.size.height as i32)
            .decorated(layout.style.has_decorations())
            .resizable(layout.style.has_decorations())
            .build();

        // Set window properties for overlay behavior
        window.set_modal(false);
        window.set_deletable(true);

        // Enable transparency support
        if let Some(display) = gdk::Display::default() {
            if display.is_composited() {
                log::info!("Display supports compositing - enabling transparency");

                // Set CSS for transparent background
                let css_provider = gtk4::CssProvider::new();
                css_provider.load_from_data("window { background-color: rgba(0, 0, 0, 0); }");

                gtk4::style_context_add_provider_for_display(
                    &display,
                    &css_provider,
                    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            } else {
                log::warn!("Display does not support compositing - transparency may not work");
            }
        }

        // Create drawing area for custom rendering
        let drawing_area = gtk4::DrawingArea::new();
        window.set_child(Some(&drawing_area));

        let timeout_ref = Rc::new(RefCell::new(timeout));
        let modifier_state = Rc::new(RefCell::new(ModifierState::default()));

        // Create shared timeout cancellation function
        let cancel_timeout = Self::create_timeout_canceller(timeout_ref.clone(), drawing_area.clone());

        // Setup all the handlers and show the window
        Self::setup_drawing(&drawing_area, board, timeout_ref.clone(), result_receiver.clone(), modifier_state.clone(), resources)?;
        Self::setup_input_handling(&window, &drawing_area, feedback, result_receiver.clone(), modifier_state.clone(), cancel_timeout.clone())?;
        Self::setup_mouse_handling(&drawing_area, cancel_timeout)?;

        // Setup timeout for auto-close (only if timeout > 0)
        if timeout > 0 {
            Self::setup_auto_close_timer(&window, &drawing_area, timeout_ref.clone());
        }

        // Connect unrealize signal - only for debugging purposes for now, to confirm window destruction order
        window.connect_unrealize(move |_widget| {
            log::info!("Window unrealize signal received - window is actually destroyed");
        });

        // Show window
        window.set_visible(true);
        window.present();
        window.grab_focus();

        // Set icon name after window is shown for proper taskbar grouping
        window.set_icon_name(Some("hotkeys"));

        // Force initial draw
        drawing_area.queue_draw();

        Ok(())
    }

    /// Setup Cairo drawing for the board
    fn setup_drawing(
        drawing_area: &gtk4::DrawingArea,
        board: &dyn Board,
        timeout: Rc<RefCell<u64>>,
        selected_pad: Rc<RefCell<Option<(u8, ModifierState)>>>,
        modifier_state: Rc<RefCell<ModifierState>>,
        resources: Resources,
    ) -> Result<()> {
        let cloned_board = board.clone_box();

        drawing_area.set_draw_func(move |_area, ctx, width, height| {
            let (width, height) = (width as f64, height as f64);

            // Clear everything to transparent
            ctx.set_operator(cairo::Operator::Clear);
            ctx.rectangle(0.0, 0.0, width, height);
            ctx.fill().unwrap();

            // Reset operator to normal
            ctx.set_operator(cairo::Operator::Over);

            // Draw background with color scheme background and opacity
            let color_scheme = cloned_board.color_scheme();

            let bg_color = color_scheme.background().to_rgb();
            ctx.set_source_rgba(bg_color.0, bg_color.1, bg_color.2, color_scheme.opacity);
            ctx.paint().unwrap();

            // Create layout for current dimensions
            let board_layout = BoardLayout::new(width, height);

            // Get countdown time if timer is active (timeout > 0)
            let timeout_value = *timeout.borrow();
            let remaining_time = if timeout_value > 0 {
                Some(timeout_value)
            } else {
                None
            };

            // Draw the 3x3 board with optional countdown using the new Board renderer
            let selected_pad_num = selected_pad.borrow().as_ref().map(|(pad, _)| *pad);
            let current_modifiers = modifier_state.borrow().clone();

            // Use the new Board renderer
            renderer::draw_board(ctx, cloned_board.as_ref(), &board_layout, &resources,
                selected_pad_num, remaining_time, &current_modifiers
            );
        });

        Ok(())
    }

    /// Setup keyboard input handling that captures selection result
    fn setup_input_handling(
        window: &gtk4::ApplicationWindow,
        drawing_area: &gtk4::DrawingArea,
        feedback: u64,
        selected_pad: Rc<RefCell<Option<(u8, ModifierState)>>>,
        modifier_state: Rc<RefCell<ModifierState>>,
        cancel_timeout: Rc<dyn Fn()>,
    ) -> Result<()> {
        // Enable key events and make window focusable
        window.set_can_focus(true);
        window.set_focusable(true);

        let key_controller = gtk4::EventControllerKey::new();
        window.add_controller(key_controller.clone());

        // Helper function for modifier handling
        let handle_modifier_event = |handler_fn: fn(&mut ModifierHandler, gdk::Key) -> bool,
                                     keyval: gdk::Key,
                                     modifier_state: &Rc<RefCell<ModifierState>>,
                                     drawing_area: &gtk4::DrawingArea| -> bool {
            let old_state = modifier_state.borrow().clone();
            let mut handler = ModifierHandler::new(old_state.clone());

            if handler_fn(&mut handler, keyval) {
                let new_state = handler.state().clone();
                if new_state != old_state {
                    *modifier_state.borrow_mut() = new_state.clone();
                    drawing_area.queue_draw();
                }
                true
            } else {
                false
            }
        };

        // Clone references for use in closures
        let cancel_timeout_clone = cancel_timeout.clone();
        let window_clone = window.clone();
        let drawing_area_clone = drawing_area.clone();
        let modifier_state_clone = modifier_state.clone();

        // Handle key presses with result capture (no action execution)
        key_controller.connect_key_pressed(move |_controller, keyval, keycode, state| {
            // Cancel timeout on any key press
            cancel_timeout_clone();

            // Handle modifier key presses using helper function
            if handle_modifier_event(ModifierHandler::handle_key_press, keyval, &modifier_state_clone, &drawing_area_clone) {
                return glib::Propagation::Proceed;
            }

            // Extract modifier state from GTK state for number key selection
            let modifier_state = ModifierState {
                ctrl: state.contains(gdk::ModifierType::CONTROL_MASK),
                shift: state.contains(gdk::ModifierType::SHIFT_MASK),
                alt: state.contains(gdk::ModifierType::ALT_MASK),
                super_key: state.contains(gdk::ModifierType::SUPER_MASK),
            };

            match keyval {
                // Numpad keys (preferred)
                gdk::Key::KP_1 | gdk::Key::_1 | gdk::Key::KP_End |
                gdk::Key::KP_2 | gdk::Key::_2 | gdk::Key::KP_Down |
                gdk::Key::KP_3 | gdk::Key::_3 | gdk::Key::KP_Page_Down |
                gdk::Key::KP_4 | gdk::Key::_4 | gdk::Key::KP_Left |
                gdk::Key::KP_5 | gdk::Key::_5 | gdk::Key::KP_Begin |
                gdk::Key::KP_6 | gdk::Key::_6 | gdk::Key::KP_Right |
                gdk::Key::KP_7 | gdk::Key::_7 | gdk::Key::KP_Home |
                gdk::Key::KP_8 | gdk::Key::_8 | gdk::Key::KP_Up |
                gdk::Key::KP_9 | gdk::Key::_9 | gdk::Key::KP_Page_Up => {
                    log::info!("Number pressed: selecting pad {} with modifiers: {}", keyval.pad_id(), modifier_state.to_string());
                    *selected_pad.borrow_mut() = Some((keyval.pad_id(), modifier_state));
                    Self::on_key_selected(window_clone.clone(), feedback, drawing_area_clone.clone())
                },
                gdk::Key::Escape => {
                    log::info!("Escape pressed - cancelling selection");
                    window_clone.close();
                },
                _ => {
                    log::info!("Other key pressed: {:?}, keycode: {:?} - ignoring", keyval, keycode);
                    return glib::Propagation::Proceed; // Ignore other keys
                },
            }
            glib::Propagation::Stop
        });

        // Handle key releases to detect modifier changes
        let modifier_state_clone = modifier_state.clone();
        let drawing_area_clone = drawing_area.clone();
        key_controller.connect_key_released(move |_controller, keyval, _keycode, _state| {
            // Handle modifier key releases using helper function
            handle_modifier_event(ModifierHandler::handle_key_release, keyval, &modifier_state_clone, &drawing_area_clone);
        });

        Ok(())
    }

    /// Setup mouse input handling to cancel timeout on any click
    fn setup_mouse_handling(drawing_area: &gtk4::DrawingArea, cancel_timeout: Rc<dyn Fn()>) -> Result<()> {
        let gesture = GestureClick::new();
        gesture.set_button(0); // Accept all buttons

        gesture.connect_pressed(move |_gesture, _n_press, _x, _y| {
            // Cancel timeout on any mouse click
            cancel_timeout();
        });

        drawing_area.add_controller(gesture);

        Ok(())
    }

    /// Setup auto close timer for the window
    fn setup_auto_close_timer(window: &gtk4::ApplicationWindow, drawing_area: &gtk4::DrawingArea, timeout: Rc<RefCell<u64>>) {
        let drawing_area_for_countdown = drawing_area.clone();
        let window_for_timeout = window.clone();

        // Single timer that decrements timeout every second
        glib::timeout_add_seconds_local(1, move || {
            let mut time_left = timeout.borrow_mut();
            if *time_left > 0 {
                *time_left -= 1;
                drawing_area_for_countdown.queue_draw(); // Trigger redraw to update visual cue

                if *time_left == 0 {
                    log::info!("Board timeout reached - auto-closing");
                    window_for_timeout.close();
                }

                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    /// Create a shared timeout cancellation function
    fn create_timeout_canceller(
        timeout: Rc<RefCell<u64>>,
        drawing_area: gtk4::DrawingArea,
    ) -> Rc<dyn Fn()> {
        Rc::new(move || {
            if *timeout.borrow() > 0 {
                *timeout.borrow_mut() = 0;
                drawing_area.queue_draw(); // Redraw to remove visual cue
            }
        })
    }

    /// Handle key selection and provide visual feedback if configured
    fn on_key_selected(window: gtk4::ApplicationWindow, feedback: u64, drawing_area: gtk4::DrawingArea) {
        if feedback > 0 {
            drawing_area.queue_draw(); // Trigger immediate redraw for visual feedback
            glib::timeout_add_local(std::time::Duration::from_millis(feedback), move || {
                 log::info!("Feedback timer expired - closing window");
                 window.close();
                 glib::ControlFlow::Break
            });
        } else {
            log::info!("No feedback configured - closing window immediately");
            window.close();
        }
    }




}


trait NumberPad {
    /// Convert number to 3x3 pad ID (1-9)
    fn pad_id(self) -> u8;
}

impl NumberPad for Key {
    fn pad_id(self) -> u8 {
        match self {
            Key::KP_1 | Key::_1 | Key::KP_End => 1,
            Key::KP_2 | Key::_2 | Key::KP_Down => 2,
            Key::KP_3 | Key::_3 | Key::KP_Page_Down => 3,
            Key::KP_4 | Key::_4 | Key::KP_Left => 4,
            Key::KP_5 | Key::_5 | Key::KP_Begin => 5,
            Key::KP_6 | Key::_6 | Key::KP_Right => 6,
            Key::KP_7 | Key::_7 | Key::KP_Home => 7,
            Key::KP_8 | Key::_8 | Key::KP_Up => 8,
            Key::KP_9 | Key::_9 | Key::KP_Page_Up => 9,
            _ => 0, // Invalid key for pad selection
        }
    }
}