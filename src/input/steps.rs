/// Linux input steps for HotKeys
/// Provides input step abstractions that execute keyboard actions

use std::time::Duration;
use anyhow::Result;
use crate::input::api;

/// Single key press/release event
#[derive(Debug, PartialEq, Clone)]
pub struct KeyInput {
    pub vk_code: u16,
    pub key_down: bool
}

/// Batch of key events for efficient text input
#[derive(Debug, Clone)]
pub struct KeyInputs {
    pub inputs: Vec<KeyInput>
}

/// Pause/delay between actions
#[derive(Debug, PartialEq, Clone)]
pub struct NoInput {
    pub pause: u16
}

/// Trait for all input actions
pub trait InputStep {
    fn play(&self) -> Result<()>;

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any;
}

impl InputStep for NoInput {
    fn play(&self) -> Result<()> {
        if self.pause > 0 {
            std::thread::sleep(Duration::from_millis(self.pause as u64));
            log::trace!(target: "input_step", "Paused for {}ms", self.pause);
        }
        Ok(())
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl InputStep for KeyInput {
    fn play(&self) -> Result<()> {
        let api_input = map_api_input(self);
        api::send_input(api_input)?;
        log::trace!(target: "input_step", "Sent key input: {:?}", self);
        Ok(())
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl InputStep for KeyInputs {
    fn play(&self) -> Result<()> {
        let api_inputs: Vec<api::KeyboardInput> = self.inputs.iter()
            .map(|input| map_api_input(input))
            .collect();
        api::send_inputs(api_inputs)?;
        log::trace!(target: "input_step", "Sent {} key inputs", self.inputs.len());
        Ok(())
    }

    #[cfg(test)]
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Convert internal KeyInput to API KeyboardInput
pub fn map_api_input(input: &KeyInput) -> api::KeyboardInput {
    api::KeyboardInput {
        vk_code: input.vk_code,
        key_down: input.key_down
    }
}