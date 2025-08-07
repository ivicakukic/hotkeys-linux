use serde::{Deserialize, Serialize};

/// Runtime pad structure with optional styling
#[derive(Debug, Clone, Default)]
pub struct Pad {
    pub header: String,
    pub text: String,
    pub icon: String,
    pub actions: Vec<super::Action>,
    pub board: Option<String>,
    pub color_scheme: Option<ColorScheme>,
    pub text_style: Option<TextStyle>,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ColorScheme {
    pub name: String,
    pub opacity: f64,
    pub background: String,
    pub foreground1: String, // lines
    pub foreground2: String, // text
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let mut hex = hex.to_lowercase();
        if hex.starts_with("0x") { hex = hex[2..].to_string(); }
        if hex.starts_with("#") { hex = hex[1..].to_string(); }

        match hex.len() {
            6 => Some(Self {
                r: u8::from_str_radix(&hex[0..2], 16).unwrap_or_default(),
                g: u8::from_str_radix(&hex[2..4], 16).unwrap_or_default(),
                b: u8::from_str_radix(&hex[4..6], 16).unwrap_or_default(),
            }),
            _ => None
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn from_hex_or(hex: &str, optb: &str) -> Option<Self> {
        Self::from_hex(hex).or(Self::from_hex(optb))
    }

    /// Convert to normalized RGB values for Cairo (0.0-1.0 range)
    pub fn to_rgb(&self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    pub fn inverted(&self) -> Color {
        Color {
            r: 255 - self.r,
            g: 255 - self.g,
            b: 255 - self.b,
        }
    }
}

impl ColorScheme {
    pub fn background(&self) -> Color {
        self.to_color(&self.background, "#00007f")
    }

    pub fn foreground1(&self) -> Color {
        self.to_color(&self.foreground1, "#5454a9")
    }

    pub fn foreground2(&self) -> Color {
        self.to_color(&self.foreground2, "#dbdbec")
    }

    fn to_color(&self, value: &String, default: &str) -> Color {
        Color::from_hex_or(value.as_str(), default).unwrap()
    }

    pub fn inverted(&self) -> ColorScheme {
        ColorScheme {
            name: format!("Inverted{}", self.name),
            opacity: self.opacity,
            background: self.background().inverted().to_hex(),
            foreground1: self.foreground1().inverted().to_hex(),
            foreground2: self.foreground2().inverted().to_hex(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextStyle {
    pub name: String,
    pub header_font: String, // e.g. "Impact Bold 24"
    pub pad_header_font: String, // e.g. "Consolas 14"
    pub pad_text_font: String, // e.g. "Arial Bold 16"
    pub pad_id_font: String, // e.g. "Impact Bold 16"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModifierState {
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default, rename = "super")]
    pub super_key: bool,
}

impl std::fmt::Display for ModifierState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if self.ctrl { parts.push("Ctrl"); }
        if self.shift { parts.push("Shift"); }
        if self.alt { parts.push("Alt"); }
        if self.super_key { parts.push("Super"); }
        write!(f, "{}", parts.join("+"))
    }
}

impl ModifierState {
    pub fn is_none(&self) -> bool {
        !self.ctrl && !self.shift && !self.alt && !self.super_key
    }
}

