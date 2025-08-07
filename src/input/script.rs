/// Linux script system for HotKeys
/// Converts user-defined shortcuts and text into input step sequences

use super::{steps::*, keys::{vkey::{self, VK_SHIFT, VK_ENTER}, ckey::{self, CharacterKey}}};
use std::collections::HashMap;
use anyhow::Result;

/// Container for sequences of input steps
pub struct InputScript {
    pub steps: Vec<Box<dyn InputStep>>
}

impl InputScript {
    pub fn play(&self) -> Result<()> {
        for step in &self.steps {
            if let Err(e) = step.play() {
                log::error!("Failed to execute input step: {}", e);
                return Err(e);
            }
        }
        Ok(())
    }
}

/// Token types for shortcut parsing
#[derive(Debug, PartialEq)]
enum Token {
    PLUS,
    CHAR(String),
    QUOTED(String),
    WORD(String),
}

/// A key combination (e.g., Ctrl+Alt+A)
struct KeyCombination<'a> {
    keys: Vec<vkey::VirtualKey<'a>>,
}

impl<'a> Default for KeyCombination<'a> {
    fn default() -> Self {
        Self { keys: Default::default() }
    }
}

use Token::*;

/// Tokenize a shortcut string into components
/// Examples:
/// - "Ctrl A" -> [WORD("ctrl"), CHAR("a")]
/// - "Ctrl + Shift" -> [WORD("ctrl"), PLUS, WORD("shift")]
/// - "'+''" -> [QUOTED("+")]
fn scan(text: &str) -> Vec<Token> {
    let txt = text.to_owned()
                .replace("'+'", "_PLUS_")
                .replace("+", " + ")
                .replace("_PLUS_", "'+'");

    let splits: Vec<&str> = txt.split(" ").collect();
    let tokens: Vec<Token> = splits
        .iter()
        .filter(|val| { "".ne(**val) })
        .map(|val| {
            let low = val.to_lowercase();
            let len = low.len();
            let chars = low.chars().collect::<Vec<char>>();
            let is_quoted = (len == 3) && (chars[0] == '\'') && (chars[2] == '\'');
            let is_letter = len == 1;

            let letter = if is_letter { chars[0] }
                else if is_quoted { chars[1] }
                else { ' ' };

            let is_plus = is_letter && letter == '+';

            if is_quoted { QUOTED(letter.to_string()) }
            else if is_plus { PLUS }
            else if is_letter { CHAR(letter.to_string()) }
            else { WORD(low) }
        })
        .collect();
    tokens
}

/// Parse tokens into key combinations
/// "Ctrl K + Ctrl B" -> [KeyCombination(Ctrl+K), KeyCombination(Ctrl+B)]
fn parse<'a>(text: &'a str) -> Vec<KeyCombination<'a>> {
    scan(text.to_lowercase().as_str())
    .into_iter()
    .fold(Vec::new(), |mut acc, token| {
        match token {
            CHAR(text) | QUOTED(text) | WORD(text) => {
                if acc.is_empty() {
                    acc.push(KeyCombination::default());
                }
                if let Ok(vkey) = vkey::find_vkey(&text) {
                    acc.last_mut().unwrap().keys.push(vkey.clone());
                }
            },
            PLUS => acc.push(KeyCombination::default())
        }
        acc
    })
}

/// Create input script for shortcut sequence
/// "Ctrl Shift A" -> Press Ctrl, Press Shift, Press A, Release A, Release Shift, Release Ctrl
pub fn for_shortcut(text: String) -> InputScript {
    log::trace!("Shortcut: {}", text);

    let mut steps = vec![];
    for cmb in parse(text.as_str()) {
        // Press all keys in order
        steps.append(&mut cmb.keys.iter().map(
            |key| map_virtual_key(key.vkey, true)).collect());
        // Release all keys in reverse order (LIFO)
        steps.append(&mut cmb.keys.iter().rev().map(
            |key| map_virtual_key(key.vkey, false)).collect());
    }

    InputScript { steps }
}

/// Create input script for pause/delay
pub fn for_pause(pause: u16) -> InputScript {
    log::trace!("Pause: {}ms", pause);
    InputScript { steps: vec![
        Box::new(NoInput { pause })
    ] }
}

/// Create input script for text input
pub fn for_text(text: String, keyboard_layout_mapping: HashMap<String, String>) -> InputScript {
    log::trace!("Text: {}", text);
    for_text_or_line(text, false, keyboard_layout_mapping)
}

/// Create input script for text input with newline
pub fn for_line(text: String, keyboard_layout_mapping: HashMap<String, String>) -> InputScript {
    log::trace!("Line: {}", text);
    for_text_or_line(text, true, keyboard_layout_mapping)
}

/// Internal function for text/line input
fn for_text_or_line(text: String, new_line: bool, keyboard_layout_mapping: HashMap<String, String>) -> InputScript {
    let ckey = ckey::with_layout(keyboard_layout_mapping);

    InputScript { steps : vec![
        Box::new(KeyInputs{
            inputs : text.chars()
                    .filter_map(|ch| ckey.find_ckey(ch))
                    .chain(new_line.then_some(CharacterKey::new(VK_ENTER.clone())))
                    .flat_map(|ck| map_character_key(ck))
                    .collect()
        })
    ] }
}

/// Map virtual key to input step
fn map_virtual_key(vk_code: u16, key_down: bool) -> Box<dyn InputStep> {
    Box::new(KeyInput { vk_code, key_down })
}

/// Map character key to sequence of key inputs (with shift handling)
fn map_character_key(ck: CharacterKey) -> Vec<KeyInput> {
    vec![
        ck.shift.then_some(KeyInput {vk_code: VK_SHIFT.vkey, key_down: true}),
        Some(KeyInput {vk_code: ck.vkey.vkey, key_down: true}),
        Some(KeyInput {vk_code: ck.vkey.vkey, key_down: false}),
        ck.shift.then_some(KeyInput {vk_code: VK_SHIFT.vkey, key_down: false}),
    ]
    .into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::keys::{VK_A, VK_ALT, VK_CTRL, VK_SHIFT};
    use anyhow::anyhow;


    #[test]
    fn test_shortcut_behavior() {
        let script = for_shortcut("Ctrl A".to_string());

        // Test that it creates the right number of steps (press + release for each key)
        assert_eq!(script.steps.len(), 4); // Ctrl down, A down, A up, Ctrl up

        // Test that calling play() doesn't panic (behavior test)
        // Note: In a real test environment, you might mock the input API
        // For now, we just ensure the structure is sound
        assert!(script.steps.len() > 0);
    }

    #[test]
    fn test_text_behavior() {
        let script = for_text("ab".to_string(), HashMap::new());

        // Should create one KeyInputs step with multiple inputs
        assert_eq!(script.steps.len(), 1);

        // The step should be a KeyInputs variant
        let key_inputs = script.steps[0].as_any().downcast_ref::<KeyInputs>().unwrap();

        // Should have 4 inputs: a_down, a_up, b_down, b_up
        assert_eq!(key_inputs.inputs.len(), 4);
    }

    #[test]
    fn test_pause_behavior() {
        let script = for_pause(100);

        // Should create exactly one NoInput step
        assert_eq!(script.steps.len(), 1);

        // Verify it's a NoInput with correct pause value
        let no_input = script.steps[0].as_any().downcast_ref::<NoInput>().unwrap();
        assert_eq!(no_input.pause, 100);
    }

    #[test]
    fn test_scan_basic_tokens() {
        let tokens = scan("ctrl a");
        assert_eq!(tokens.len(), 2);

        match &tokens[0] {
            Token::WORD(w) => assert_eq!(w, "ctrl"),
            _ => panic!("Expected WORD token"),
        }

        match &tokens[1] {
            Token::CHAR(c) => assert_eq!(c, "a"),
            _ => panic!("Expected CHAR token"),
        }
    }

    #[test]
    fn test_scan_plus_token() {
        let tokens = scan("ctrl + shift");
        assert_eq!(tokens.len(), 3);

        match &tokens[0] {
            Token::WORD(w) => assert_eq!(w, "ctrl"),
            _ => panic!("Expected WORD token"),
        }

        match &tokens[1] {
            Token::PLUS => {},
            _ => panic!("Expected PLUS token"),
        }

        match &tokens[2] {
            Token::WORD(w) => assert_eq!(w, "shift"),
            _ => panic!("Expected WORD token"),
        }
    }

    #[test]
    fn test_scan_quoted_plus() {
        let tokens = scan("'+'");
        assert_eq!(tokens.len(), 1);

        match &tokens[0] {
            Token::QUOTED(q) => assert_eq!(q, "+"),
            _ => panic!("Expected QUOTED token"),
        }
    }

    #[test]
    fn test_parse_single_combination() {
        let combinations = parse("ctrl a");
        assert_eq!(combinations.len(), 1);
        assert_eq!(combinations[0].keys.len(), 2);
        assert_eq!(combinations[0].keys[0].title, "ctrl");
        assert_eq!(combinations[0].keys[1].title, "a");
    }

    #[test]
    fn test_parse_chord_combination() {
        let combinations = parse("ctrl k + ctrl b");
        assert_eq!(combinations.len(), 2);

        // First combination: Ctrl+K
        assert_eq!(combinations[0].keys.len(), 2);
        assert_eq!(combinations[0].keys[0].title, "ctrl");
        assert_eq!(combinations[0].keys[1].title, "k");

        // Second combination: Ctrl+B
        assert_eq!(combinations[1].keys.len(), 2);
        assert_eq!(combinations[1].keys[0].title, "ctrl");
        assert_eq!(combinations[1].keys[1].title, "b");
    }

    #[test]
    fn test_parse_combination_with_enter() {

        let combinations = parse("ctrl alt enter");
        assert_eq!(combinations.len(), 1);
        assert_eq!(combinations[0].keys.len(), 3);
        assert_eq!(combinations[0].keys[0].title, "ctrl");
        assert_eq!(combinations[0].keys[1].title, "alt");
        assert_eq!(combinations[0].keys[2].title, "enter");

        let res = for_shortcut("ctrl alt enter".to_string());
        assert_eq!(res.steps.len(), 6); // Ctrl down, Alt down, Enter down, Enter up, Alt up, Ctrl up
        assert_eq!(res.steps[0].as_any().downcast_ref::<KeyInput>().unwrap().vk_code, VK_CTRL.vkey);
        assert_eq!(res.steps[1].as_any().downcast_ref::<KeyInput>().unwrap().vk_code, VK_ALT.vkey);
        assert_eq!(res.steps[2].as_any().downcast_ref::<KeyInput>().unwrap().vk_code, VK_ENTER.vkey);
        assert_eq!(res.steps[3].as_any().downcast_ref::<KeyInput>().unwrap().vk_code, VK_ENTER.vkey);
        assert_eq!(res.steps[4].as_any().downcast_ref::<KeyInput>().unwrap().vk_code, VK_ALT.vkey);
        assert_eq!(res.steps[5].as_any().downcast_ref::<KeyInput>().unwrap().vk_code, VK_CTRL.vkey);

        let input = map_api_input(res.steps[2].as_any().downcast_ref::<KeyInput>().unwrap());
        let linux_key = crate::input::keys::ALL_KEYS.iter()
                    .find(|vk| vk.vkey == input.vk_code)
                    .copied()
                    .map(|vk| vk.linux_key)
                    .ok_or_else(|| anyhow!("Unknown key code: {}", input.vk_code)).expect("dddd");

        assert_eq!(linux_key, 28); // Enter key in Linux
    }

    #[test]
    fn test_map_character_key_no_shift() {
        let ckey = CharacterKey::new(VK_A.clone());
        let inputs = map_character_key(ckey);

        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0], KeyInput { vk_code: VK_A.vkey, key_down: true });
        assert_eq!(inputs[1], KeyInput { vk_code: VK_A.vkey, key_down: false });
    }

    #[test]
    fn test_map_character_key_with_shift() {
        let ckey = CharacterKey::new_sh(VK_A.clone());
        let inputs = map_character_key(ckey);

        assert_eq!(inputs.len(), 4);
        assert_eq!(inputs[0], KeyInput { vk_code: VK_SHIFT.vkey, key_down: true });
        assert_eq!(inputs[1], KeyInput { vk_code: VK_A.vkey, key_down: true });
        assert_eq!(inputs[2], KeyInput { vk_code: VK_A.vkey, key_down: false });
        assert_eq!(inputs[3], KeyInput { vk_code: VK_SHIFT.vkey, key_down: false });
    }
}