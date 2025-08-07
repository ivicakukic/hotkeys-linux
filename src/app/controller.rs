/// Application controller for HotKeys Linux
/// Handles board detection, board navigation and action execution coordination

use crate::core::{Action, ActionList, Board, ModifierState, DataRepository, Resources};
use crate::process;
use crate::executor;
use crate::windows::layout::{Size, WindowLayout, WindowStyle};
use crate::windows::board::BoardWindow;

use super::config::{AppSettings, LayoutSettings, Profile, BoardConfig};
use super::board_factory::BoardFactory;
use super::json_repository::JsonRepository;

use anyhow::Result;

use gtk4::prelude::*;
use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::RefCell;

pub struct HotKeysApp {
    settings: AppSettings,
    factory: BoardFactory,
    profile: String,
    resources: Resources,
    repository: Arc<Mutex<dyn DataRepository>>,
}

impl HotKeysApp {
    pub fn new(resources: Resources, profile: Option<String>, settings: AppSettings) -> Result<Self> {
        log::info!("Initializing HotKeys application");

        let profile = profile.unwrap_or_else(|| "default".to_string());
        log::info!("Using profile: {}", profile);

        // Initialize DataRepository
        let repo_path = resources.data_json().to_str().unwrap().to_string();
        let repository = Arc::new(Mutex::new(JsonRepository::new(repo_path)?));
        log::info!("Initialized DataRepository");

        let factory = BoardFactory::new(settings.clone())
            .with_repository(repository.clone(), profile.clone());

        Ok(Self { settings, factory, profile, resources, repository })
    }

    /// Main application loop - handles board navigation and action execution
    pub fn run(&mut self) -> Result<()> {
        log::info!("Starting HotKeys application main loop");

        let initial_board_config = self.detect_initial_board()?;
        let mut board = self.factory.create_board(&initial_board_config)?;

        log::info!("Starting with board: {}", board.title());
        let mut timeout = self.settings.timeout();

        // Spawn uinput device creation in a new thread asynchronously
        std::thread::spawn(|| {
            use crate::input::api;
            std::thread::sleep(std::time::Duration::from_millis(300));
            log::info!("Pre-initializing uinput device in background");
            let _ignore = api::init_global_device();
        });

        loop {
            // Show board and wait for user selection
            let selection = self.show_dialog(board.as_ref(), timeout)?;

            match selection {
                Some((pad_id, modifier_state)) => {
                    log::info!("User selected pad {} with modifiers: {}", pad_id, modifier_state.to_string());

                    // Determine which pad source to use based on modifier state
                    let pad = board.pads(Some(modifier_state)).get_or_default((pad_id - 1) as usize);

                    // Execute actions
                    self.execute_actions(pad.actions)?;

                    // Handle potential board navigation
                    if let Some(board_name) = pad.board {
                        if let Some(new_board_config) = self.find_board_config(&board_name) {
                            log::info!("Navigating to board: {}", new_board_config.name);
                            board = self.factory.create_board(&new_board_config)?;
                            timeout = 0; // Any navigation deactivates auto-close
                            continue; // Show new board
                        }
                    }
                    // If no board navigation, exit app
                    break;
                },
                None => {
                    break; // User cancelled (Escape/timeout)
                }
            }
        }

        log::info!("HotKeys application main loop completed");
        Ok(())
    }

    /// Show board dialog and wait for user selection
    fn show_dialog(&self, board: &dyn Board, timeout: u64) -> Result<Option<(u8, ModifierState)>> {
        log::info!("Showing board: {}", board.title());

        // Create GTK application for this board instance
        let app = gtk4::Application::builder()
            .application_id("com.github.ivicakukic.hotkeys")
            .build();

        // Create shared state for result communication
        let result: Rc<RefCell<Option<(u8, ModifierState)>>> = Rc::new(RefCell::new(None));

        // Clone data for use inside connect_activate
        let board_clone = board.clone_box();
        let settings_feedback = self.settings.feedback();
        let layout = self.settings.layout()
            .clone()
            .map(WindowLayout::from)
            .unwrap_or_else(WindowLayout::default);
        let resources = self.resources.clone();
        let result_clone = result.clone();

        app.connect_activate(move |app| {
            match BoardWindow::show_with_app(app, board_clone.as_ref(), timeout, settings_feedback, layout.clone(), resources.clone(), result_clone.clone()) {
                Ok(()) => {
                    log::info!("Board window setup completed");
                },
                Err(e) => {
                    log::error!("Failed to show board: {}", e);
                }
            }
        });

        // Run the application
        let empty_args: Vec<String> = vec![];
        app.run_with_args(&empty_args);

        let final_result = result.borrow().clone();

        Ok(final_result)
    }

    /// Execute actions
    fn execute_actions(&mut self, actions: Vec<Action>) -> Result<()> {
        if !actions.is_empty() {
            log::info!("Processing {} actions", actions.len());
            let keyboard_layout = self.settings.get_keyboard_layout();
            let delay = self.settings.delay();

            let (background_actions, main_actions) = actions.split();

            let keyboard_layout_clone = keyboard_layout.clone();
            let repository_clone = self.repository.clone();
            let profile_clone = self.profile.clone();
            let join_handle = std::thread::spawn(move || {
                // Giving the desktop manager enough time to return focus to the target application
                if background_actions.is_delayed() {
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                }
                executor::execute_actions(
                    &background_actions,
                    &keyboard_layout_clone,
                    Some(repository_clone),
                    Some(&profile_clone),
                ).map_err(|e| format!("Failed to execute background actions: {}", e))
            });

            let result = join_handle.join();
            match result {
                Ok(result) => {
                    match result {
                        Err(e) => return Err(anyhow::anyhow!("Failed to execute background actions: {}", e)),
                        _ => {}
                    }
                },
                Err(e) => return Err(anyhow::anyhow!("Thread panicked: {:?}", e)),
            }

            // Execute main thread actions
            return executor::execute_actions(
                &main_actions,
                &keyboard_layout,
                Some(self.repository.clone()),
                Some(&self.profile),
            );
        }
        Ok(())
    }

    fn detect_initial_board(&self) -> Result<BoardConfig> {
        let profile = self.settings.get_profile(&self.profile)?;
        let profile_boards = self.get_profile_board_configs(profile);

        let xprop_boards: Vec<&BoardConfig> = profile_boards.iter()
            .filter(|b| b.detection.is_xprop()).copied().collect();
        let ps_boards: Vec<&BoardConfig> = profile_boards.iter()
            .filter(|b| b.detection.is_ps()).copied().collect();
        let default_board = self.find_board_config(&profile.default)
            .ok_or_else(|| anyhow::anyhow!("Default board '{}' not found", profile.default))?;

        if !xprop_boards.is_empty() {
            if process::is_x11_available() {
                match process::get_active_process_info() {
                    Ok(process_info) => {
                        log::info!("Active process: {} (PID: {})", process_info.name, process_info.pid);
                        if let Some(board) = xprop_boards.iter().find(|board| {
                            board.detection.matches(&process_info.name)
                        }) {
                            return Ok((**board).clone());
                        }
                    },
                    Err(e) => {
                        log::warn!("Could not detect active process: {}", e);
                    }
                }
            } else {
                log::warn!("X11 not available, process detection disabled");
            };
        }

        if !ps_boards.is_empty() {
            if let Some(process_board) = self.find_board_among_running_processes(&ps_boards, &default_board) {
                log::info!("Found board based on running processes: {}", process_board.name);
                return Ok(process_board);
            }
        }

        Ok(default_board)
    }

    fn get_profile_board_configs(&self, profile: &Profile) -> Vec<&BoardConfig> {
        self.settings.board_configs.iter()
            .filter(|b| profile.boards.contains(&b.name))
            .collect()
    }

    fn find_board_config(&self, board_name: &str) -> Option<BoardConfig> {
        self.settings.board_configs.iter()
            .find(|b| b.name == board_name)
            .cloned()
    }

    /// Examine running processes and try to find a matching board
    fn find_board_among_running_processes(&self, ps_boards: &[&BoardConfig], default_board: &BoardConfig) -> Option<BoardConfig> {
        // Get all running processes
        let process_names: Vec<String> = match crate::process::get_all_processes() {
            Ok(processes) => processes.iter().map(|p| p.name.clone()).collect(),
            _ => return None,
        };

        // Find boards from this profile that have matching running processes
        let matching_boards: Vec<BoardConfig> = ps_boards.iter()
            .filter(|board| process_names.iter()
                .any(|name| board.detection.matches(name)))
            .map(|&board| board.clone())
            .collect();

        match matching_boards.len() {
            0 => {
                log::debug!("No running processes match any boards in current profile");
                None
            },
            1 => {
                let board = &matching_boards[0];
                log::debug!("Single match found: using board '{}'", board.name);
                Some(board.clone())
            },
            _ => {
                // Multiple matches: prefer default if it's among them, otherwise pick first
                let default_name = &default_board.name;

                if let Some(default_board) = matching_boards.iter().find(|board| board.name == *default_name) {
                    log::debug!("Multiple matches found, using profile default board '{}'", default_board.name);
                    Some(default_board.clone())
                } else {
                    let first_board = &matching_boards[0];
                    log::debug!("Multiple matches found, default not among them, using first match '{}'", first_board.name);
                    Some(first_board.clone())
                }
            }
        }
    }
}

// Mapping between LayoutSettings and WindowLayout
impl From<LayoutSettings> for WindowLayout {
    fn from(layout: LayoutSettings) -> Self {
        WindowLayout {
            size: Size {
                width: layout.width as f64,
                height: layout.height as f64,
            },
            style: WindowStyle::from_string(&layout.window_style),
        }
    }
}