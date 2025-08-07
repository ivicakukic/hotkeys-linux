mod process;
mod input;
mod executor;
mod windows;
mod tools;
mod app;
mod core;
mod components;

use anyhow::Result;
use std::{env, path::PathBuf};

fn print_help() {
    println!("");
    println!("Usage: hotkeys [mode] [options]");
    println!("");
    println!("mode: help, gtk, validate-settings, input-test");
    println!("");
    println!("options:");
    println!("  --config_dir <path>: use specified config directory");
    println!("  --profile <name>: use specific profile for board selection");
    println!("");
    println!("Defaults:");
    println!("  mode: gtk");
    println!("  config_dir: automatic resolution (user config -> system resources)");
    println!("");
}

struct Args {
    mode: String,
    config_dir: Option<String>,
    profile: Option<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();

    let mut mode = "gtk".to_string();
    let mut profile: Option<String> = Some("default".to_string());
    let mut config_dir: Option<String> = None;

    let mut i = 1;

    // First argument might be mode (if it's not an option)
    if args.len() > 1 && !args[1].starts_with("--") {
        mode = args[1].clone();
        i = 2;
    }

    // Parse options
    while i < args.len() {
        match args[i].as_str() {
            "--profile" => {
                if i + 1 < args.len() {
                    profile = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("ERROR: --profile requires a value");
                    print_help();
                    std::process::exit(1);
                }
            },
            "--config_dir" => {
                if i + 1 < args.len() {
                    config_dir = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("ERROR: --config_dir requires a value");
                    print_help();
                    std::process::exit(1);
                }
            },
            _ => {
                eprintln!("ERROR: Unknown option: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    if mode == "help" {
        print_help();
        std::process::exit(0);
    } else if mode != "gtk" && mode != "validate-settings" && mode != "input-test" {
        eprintln!("ERROR: Unknown mode: {}", mode);
        print_help();
        std::process::exit(1);
    }

    Args { mode, config_dir, profile }
}




pub fn get_config_resolution_order(config_dir: Option<PathBuf>) -> Vec<PathBuf> {

    fn get_dev_resources_dir() -> Option<PathBuf> {
        // Dynamic resolution instead of using #[cfg(debug_assertions)]
        // This allows using development resources even with release builds

        if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap();

            let dev_resources = exe_dir.join("../../resources");
            if dev_resources.exists() {
                return Some(dev_resources);
            }
        }
        None
    }

    if let Some(dev_resources_dir) = get_dev_resources_dir() {
        // DEVELOPMENT:
        // use only a single directory

        let single_dir = if let Some(config_dir) = config_dir {
            assert!(config_dir.exists());
            config_dir
        } else {
            dev_resources_dir
        };
        return vec![single_dir];
    }

    // PRODUCTION:
    // use both (config_dir OR user config directory) AND (system resources)

    let mut paths = Vec::new();

    let user_config_dir = if let Some(config_dir) = config_dir {
        assert!(config_dir.exists());
        config_dir
    } else {
        dirs::config_dir()
            .map(|d| d.join("hotkeys"))
            .unwrap_or_else(|| PathBuf::from("./config"))
    };

    if user_config_dir.exists() {
        paths.push(user_config_dir);
    }

    // 2. System resources (build-dependent)
    paths.push(PathBuf::from("/usr/share/hotkeys"));

    paths
}




fn run() -> Result<()> {

    // Check for command line arguments
    let args = parse_args();
    let mode = &args.mode;

    let resources = core::Resources::new(get_config_resolution_order(args.config_dir.map(PathBuf::from)));

    log4rs::init_file(resources.log_toml().unwrap(), Default::default())
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    // Load settings once for all modes
    let settings = app::config::load_settings(&resources)
        .map_err(|e| anyhow::anyhow!("Failed to load settings: {}", e))?;

    log::info!("Starting HotKeys");

    // Handle different execution modes
    match mode.as_str() {
        "gtk" => {
            log::info!("Starting GTK4 mode");

            match crate::app::HotKeysApp::new(resources, args.profile.clone(), settings) {
                Ok(mut app) => {
                    if let Err(e) = app.run() {
                        log::error!("HotKeys application failed: {}", e);
                    }
                },
                Err(e) => {
                    log::error!("Failed to create HotKeys application: {}", e);
                }
            }
        },
        "validate-settings" => {
            log::info!("Validation SUCCESSFUL!");
        },
        "input-test" => {
            log::info!("Running input test");
            if let Err(e) = tools::input_test::test_direct_uinput(settings.get_keyboard_layout()) {
                log::error!("Direct uinput test failed: {}", e);
            }
        },
        _ => {
            std::process::exit(1);
        }
    }

    log::info!("Exiting HotKeys");
    Ok(())
}

fn main() {
    let result = run();

    // We do this for nicer HRESULT printing when errors occur.
    if let Err(error) = result {
        eprintln!("Error: {:?}", error);
    }
}