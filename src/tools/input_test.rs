// Simple uinput test to debug sending of input events.
// This bypasses all GTK4 code to isolate the problem

use crate::{input::api, app::config::KeyboardLayout};
use anyhow::Result;

pub fn test_direct_uinput(keyboard_layout: KeyboardLayout) -> Result<()> {
    println!("=== Direct uinput Chrome test ===");

    {
        println!("Creating uinput device...");
        let _unused = api::get_global_device()?;
    }

    println!("Waiting 3 seconds for you to focus Chrome...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Send CTRL + T
    use crate::input::script::{for_line, for_shortcut, for_pause, InputScript};


    let script1 = for_shortcut("Ctrl T".to_owned());
    let script2 = for_pause(500);
    let script3 = for_line("https://www.example.com".to_owned(), keyboard_layout.mappings.clone());

    let combined_script = InputScript{
        steps: script1.steps
            .into_iter()
            .chain(script2.steps.into_iter())
            .chain(script3.steps.into_iter())
            .collect()
    };
    combined_script.play()?;

    Ok(())
}
