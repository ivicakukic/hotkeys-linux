use std::{collections::{BTreeMap, HashMap, HashSet}, fs, path::PathBuf};
use serde::{Deserialize, Serialize, Serializer};
use anyhow::Result;
use crate::core::{ActionList, ColorScheme, TextStyle, Resources};

const DEFAULT_SCHEME: &str = "default";
const DEFAULT_KEYBOARD_LAYOUT: &str = "default";
const DEFAULT_OPACITY: f64 = 0.75;
const DEFAULT_BACKGROUND: &str = "#00007f";
const DEFAULT_FOREGROUND1: &str = "#5454a9";
const DEFAULT_FOREGROUND2: &str = "#dbdbec";

const DEFAULT_TEXT_STYLE: &str = "default";
const DEFAULT_FONT_HEADER: &str = "Impact Bold 24";
const DEFAULT_FONT_PAD_TITLE: &str = "Consolas 9";
const DEFAULT_FONT_PAD_DESCRIPTION: &str = "Arial Bold 12";
const DEFAULT_FONT_PAD_ID: &str = "Impact Bold 10";

/// For use with serde's [serialize_with] attribute
fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KeyboardLayout {
    pub name: String,
    #[serde(default)]
    #[serde(serialize_with = "ordered_map")]
    pub mappings: HashMap<String, String>
}

impl Default for KeyboardLayout {
    fn default() -> Self {
        Self {
            name: DEFAULT_KEYBOARD_LAYOUT.to_owned(),
            mappings: HashMap::new(),
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Detection {
    XPROP(String),
    PS(String),
    NONE
}

impl Default for Detection {
    fn default() -> Self {
        Detection::NONE
    }
}

impl Detection {
    pub fn matches(&self, text: &str) -> bool {
        match self {
            Detection::XPROP(prop) => text.to_lowercase().contains(&prop.to_lowercase()),
            Detection::PS(ps) => text.to_lowercase().eq(&ps.to_lowercase()),
            Detection::NONE => false,
        }
    }

    pub fn is_xprop(&self) -> bool {
        matches!(self, Detection::XPROP(_))
    }

    pub fn is_ps(&self) -> bool {
        matches!(self, Detection::PS(_))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum BoardKind {
    Static,
    Home
}

impl Default for BoardKind {
    fn default() -> Self {
        BoardKind::Static
    }
}

impl BoardKind {
    pub fn is_static(&self) -> bool {
        matches!(self, BoardKind::Static)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PadSetKind {
    Static
}

impl Default for PadSetKind {
    fn default() -> Self {
        PadSetKind::Static
    }
}

impl PadSetKind {
    pub fn is_static(&self) -> bool {
        matches!(self, PadSetKind::Static)
    }
}

/// Configuration-level board structure (internal)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BoardConfig {
    #[serde(default, skip_serializing_if = "BoardKind::is_static")]
    pub kind: BoardKind,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    pub name: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_style: Option<String>,

    #[serde(default)]
    pub detection: Detection,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_pads: Option<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty", serialize_with = "ordered_map")]
    pub modifier_pads: HashMap<String, String>
}

/// Configuration-level pad structure (internal)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PadConfig {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub header: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub icon: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<crate::core::Action>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_style: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct PadSetConfig {
    #[serde(default, skip_serializing_if = "PadSetKind::is_static")]
    pub kind: PadSetKind,

    pub name: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<PadConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub name: String,
    pub boards: Vec<String>,
    pub default: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LayoutSettings {
    pub width: i32,
    pub height: i32,
    pub window_style: String, // "Window" | "Taskbar"
}

/// Main application settings structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppSettings {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    includes: Vec<String>,
    timeout: u64,
    feedback: u64,
    delay: u64,
    color_schemes: Vec<ColorScheme>,
    text_styles: Vec<TextStyle>,
    keyboard_layout: String,
    pub keyboard_layouts: Vec<KeyboardLayout>,

    #[serde(rename = "boards")]
    pub board_configs: Vec<BoardConfig>,

    #[serde(rename = "padsets")]
    pub padset_configs: Vec<PadSetConfig>,

    #[serde(default)]
    pub profiles: Vec<Profile>,

    #[serde(skip_serializing_if = "Option::is_none")]
    layout: Option<LayoutSettings>,

    #[serde(default, skip_serializing)]
    file_path: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            name: DEFAULT_SCHEME.to_owned(),
            opacity: DEFAULT_OPACITY,
            background: DEFAULT_BACKGROUND.to_owned(),
            foreground1: DEFAULT_FOREGROUND1.to_owned(),
            foreground2: DEFAULT_FOREGROUND2.to_owned(),
        }
    }
}


impl Default for TextStyle {
    fn default() -> Self {
        Self {
            name: DEFAULT_TEXT_STYLE.to_string(),
            header_font: DEFAULT_FONT_HEADER.to_string(),
            pad_header_font: DEFAULT_FONT_PAD_TITLE.to_string(),
            pad_text_font: DEFAULT_FONT_PAD_DESCRIPTION.to_string(),
            pad_id_font: DEFAULT_FONT_PAD_ID.to_string(),
        }
    }
}

impl AppSettings {
    pub fn timeout(&self) -> u64 { self.timeout }
    pub fn feedback(&self) -> u64 { self.feedback }
    pub fn delay(&self) -> u64 { self.delay }
    pub fn layout(&self) -> &Option<LayoutSettings> { &self.layout }

    pub fn get_color_scheme(&self, name: &str) -> Option<&ColorScheme> {
        self.color_schemes.iter().find(|s| s.name == name)
    }

    pub fn get_text_style(&self, name: &str) -> Option<&TextStyle> {
        self.text_styles.iter().find(|s| s.name == name)
    }

    pub fn get_keyboard_layout(&self) -> KeyboardLayout {
        let layout_name = self.keyboard_layout.clone();

        self.keyboard_layouts.iter()
        .find(|l| l.name == layout_name)
        .cloned()
        .unwrap_or_else(KeyboardLayout::default)
    }

    pub fn get_profile(&self, name: &str) -> Result<&Profile> {
        self.profiles.iter()
            .find(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))
    }

    pub fn get_padset_config(&self, name: &str) -> Option<&PadSetConfig> {
        self.padset_configs.iter().find(|p| p.name == name)
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Validate the entire settings configuration
    pub fn validate(&self, resources: &Resources) -> Result<(), String> {
        if self.board_configs.is_empty() {
            return Err("No boards defined in settings".to_string());
        }

        if self.profiles.is_empty() {
            return Err("No profiles defined in settings".to_string());
        }

        self.validate_color_scheme_references()
            .map_err(|e| format!("Color scheme validation failed: {}", e))?;

        self.validate_text_style_references()
            .map_err(|e| format!("Text style validation failed: {}", e))?;

        self.validate_profile_board_references()
            .map_err(|e| format!("Profile board validation failed: {}", e))?;

        self.validate_pad_references()
            .map_err(|e| format!("Pad reference validation failed: {}", e))?;

        self.validate_cross_board_references()
            .map_err(|e| format!("Cross board validation failed: {}", e))?;

        self.validate_icons_availability(resources)
            .map_err(|e| format!("Icon availability validation failed: {}", e))?;

        self.validate_action_order()
            .map_err(|e| format!("Action order validation failed: {}", e))?;

        Ok(())
    }

    fn validate_color_scheme_references(&self) -> Result<(), String> {
        for board in &self.board_configs {
            if let Some(scheme_name) = &board.color_scheme {
                if self.get_color_scheme(scheme_name).is_none() {
                    return Err(format!("Color scheme '{}' not found in settings", scheme_name));
                }
            }
        }
        Ok(())
    }

    fn validate_text_style_references(&self) -> Result<(), String> {
        for board in &self.board_configs {
            if let Some(text_style) = &board.text_style {
                if self.get_text_style(text_style).is_none() {
                    return Err(format!("Text style '{}' not found in settings", text_style));
                }
            }
        }
        Ok(())
    }

    fn validate_profile_board_references(&self) -> Result<(), String> {
        for profile in &self.profiles {
            for board_name in &profile.boards {
                let found = self.board_configs.iter().any(|b| b.name == *board_name);
                if !found {
                    return Err(format!("Board '{}' not found in settings for profile '{}'", board_name, profile.name));
                }
            }
            let default_found = self.board_configs.iter().any(|b| b.name == profile.default);
            if !default_found {
                return Err(format!("Default board '{}' not found in settings for profile '{}'", profile.default, profile.name));
            }
        }
        Ok(())
    }

    fn validate_pad_references(&self) -> Result<(), String> {
        for board in &self.board_configs {
            if let Some(ref padset_name) = board.base_pads {
                if self.get_padset_config(padset_name).is_none() {
                    return Err(format!("Base pad set '{}' not found for board '{}'", padset_name, board.name));
                }
            }

            for (modifier, padset_name) in &board.modifier_pads {
                if self.get_padset_config(padset_name).is_none() {
                    return Err(format!("Modifier pad set '{}' not found for board '{}' with modifier '{}'", padset_name, board.name, modifier));
                }
            }
        }
        Ok(())
    }

    fn validate_cross_board_references(&self) -> Result<(), String> {
        for padset in &self.padset_configs {
            for pad in &padset.items {
                if let Some(ref board_ref) = pad.board {
                    let found = self.board_configs.iter().any(|b| b.name == *board_ref);
                    if !found {
                        return Err(format!("Invalid board reference '{}' in settings for pad '{:?}'", board_ref, pad));
                    }
                }

                // Validate pad-level color scheme references
                if let Some(ref scheme_name) = pad.color_scheme {
                    if self.get_color_scheme(scheme_name).is_none() {
                        return Err(format!("Color scheme '{}' not found for pad '{:?}'", scheme_name, pad));
                    }
                }

                // Validate pad-level text style references
                if let Some(ref style_name) = pad.text_style {
                    if self.get_text_style(style_name).is_none() {
                        return Err(format!("Text style '{}' not found for pad '{:?}'", style_name, pad));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_icons_availability(&self, resources: &Resources) -> Result<(), String> {
        for board in &self.board_configs {
            if let Some(ref icon) = board.icon {
                let _ = resources.icon(icon)
                        .ok_or_else(|| anyhow::anyhow!("Icon '{}' not found for board '{}'", icon, board.name));
            }
        }
        for padset in &self.padset_configs {
            for pad in &padset.items {
                if !pad.icon.is_empty() {
                    let _ = resources.icon(&pad.icon)
                        .ok_or_else(|| anyhow::anyhow!("Icon '{}' not found for pad '{:?}' in padset '{}'", pad.icon, pad, padset.name));
                }
            }
        }
        Ok(())
    }

    fn validate_action_order(&self) -> Result<(), String> {
        for padset in &self.padset_configs {
            for pad in &padset.items {
                if !pad.actions.is_order_valid() {
                    return Err(format!("Invalid action order in pad '{:?}' of padset '{}'", pad, padset.name));
                }
            }
        }
        Ok(())
    }
}

/// Components structure for loading additional settings files
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct Components {
    #[serde(default)]
    color_schemes: Vec<ColorScheme>,
    #[serde(default)]
    text_styles: Vec<TextStyle>,
    #[serde(default)]
    keyboard_layouts: Vec<KeyboardLayout>,
    #[serde(default, rename = "boards")]
    board_configs: Vec<BoardConfig>,
    #[serde(default, rename = "padsets")]
    padset_configs: Vec<PadSetConfig>,
    #[serde(default)]
    profiles: Vec<Profile>,
}

fn load_components(file_path: &str) -> Result<Components> {
    let text = fs::read_to_string(file_path)?;
    let components = serde_json::from_str::<Components>(&text)?;
    Ok(components)
}

impl AppSettings {
    /// Append all components from a Components instance
    fn append_all(&mut self, components: Components) {
        self.color_schemes.extend(components.color_schemes);
        self.text_styles.extend(components.text_styles);
        self.keyboard_layouts.extend(components.keyboard_layouts);
        self.board_configs.extend(components.board_configs);
        self.padset_configs.extend(components.padset_configs);
        self.profiles.extend(components.profiles);
    }

    /// Validate if no two components of the same type have equal name
    fn validate_unique_names(&self) -> Result<(), String> {
        let mut seen = HashSet::new();
        for scheme in &self.color_schemes {
            if !seen.insert(scheme.name.clone()) {
                return Err(format!("Duplicate 'ColorScheme' name found: {}", scheme.name));
            }
        }

        let mut seen = HashSet::new();
        for style in &self.text_styles {
            if !seen.insert(style.name.clone()) {
                return Err(format!("Duplicate 'TextStyle' name found: {}", style.name));
            }
        }

        let mut seen = HashSet::new();
        for layout in &self.keyboard_layouts {
            if !seen.insert(layout.name.clone()) {
                return Err(format!("Duplicate 'KeyboardLayout' name found: {}", layout.name));
            }
        }

        let mut seen = HashSet::new();
        for board in &self.board_configs {
            if !seen.insert(board.name.clone()) {
                return Err(format!("Duplicate 'Board' name found: {}", board.name));
            }
        }

        let mut seen = HashSet::new();
        for padset in &self.padset_configs {
            if !seen.insert(padset.name.clone()) {
                return Err(format!("Duplicate 'PadSet' name found: {}", padset.name));
            }
        }

        let mut seen = HashSet::new();
        for profile in &self.profiles {
            if !seen.insert(profile.name.clone()) {
                return Err(format!("Duplicate 'Profile' name found: {}", profile.name));
            }
        }

        Ok(())
    }

    fn with_file_path(mut self, file_path: &str) -> Self {
        self.file_path = file_path.to_string();
        self
    }
}

pub fn load_settings(resources: &Resources) -> Result<AppSettings> {
    let settings_path: PathBuf = resources.settings_json().ok_or_else(|| anyhow::anyhow!("Settings file not found"))?;

    if !settings_path.exists() {
        anyhow::bail!("Settings file does not exist: {:?}", settings_path);
    }

    log::info!("Loading settings: {:?}", settings_path);
    let contents = fs::read_to_string(settings_path.clone())?;

    let mut settings: AppSettings = serde_json::from_str::<AppSettings>(&contents)?
        .with_file_path(settings_path.to_str().unwrap());

    // Load includes
    for include in &settings.includes.clone() {
        let include_path = resources.file(include)
            .ok_or_else(|| anyhow::anyhow!("Included settings file not found: {}", include))?;

        log::info!("Loading components: {:?}", include_path);
        let components = load_components(include_path.to_str().unwrap())?;
        settings.append_all(components);

        settings.validate_unique_names()
            .map_err(|e| anyhow::Error::msg(format!("Validation error in included file '{:?}': {}", include_path, e)))?;
    }

    // Validate the entire settings configuration
    settings.validate(&resources)
        .map_err(|e| anyhow::Error::msg(format!("Settings validation failed: {}", e)))?;

    Ok(settings)
}