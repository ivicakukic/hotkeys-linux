use std::{collections::HashMap, sync::{Arc, Mutex}};
use crate::core::{Board, PadSet, Pad, ColorScheme, TextStyle, ModifierState, Action, DataRepository};

#[derive(Clone)]
pub struct StaticBoard {
    title: String,
    icon: Option<String>,
    color_scheme: ColorScheme,
    text_style: TextStyle,
    base_pads: Box<dyn PadSet>,
    modifier_pads: HashMap<String, Box<dyn PadSet>>,
}

impl StaticBoard {
    pub fn new(
        title: String,
        icon: Option<String>,
        color_scheme: ColorScheme,
        text_style: TextStyle,
        base_pads: Box<dyn PadSet>,
        modifier_pads: HashMap<String, Box<dyn PadSet>>,
    ) -> Self {
        Self {
            title,
            icon,
            color_scheme,
            text_style,
            base_pads,
            modifier_pads,
        }
    }
}

impl Board for StaticBoard {
    fn title(&self) -> &str {
        &self.title
    }

    fn icon(&self) -> Option<&str> {
        self.icon.as_deref()
    }

    fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }

    fn text_style(&self) -> &TextStyle {
        &self.text_style
    }

    fn pads(&self, modifier: Option<ModifierState>) -> Box<dyn PadSet> {
        if let Some(modifier) = modifier {
            if let Some(pads) = self.modifier_pads.get(&modifier.to_string()) {
                return (*pads).clone();
            }
        }
        self.base_pads.clone()
    }

    fn clone_box(&self) -> Box<dyn Board> {
        Box::new(self.clone())
    }
}

#[derive(Clone)]
pub struct HomeBoard {
    color_scheme: ColorScheme,
    text_style: TextStyle,
    #[allow(dead_code)]
    profile: String,
    #[allow(dead_code)]
    repository: Arc<Mutex<dyn DataRepository>>,
    settings_file_path: String,
}

// impl std::fmt::Debug for HomeBoard {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("HomeBoard")
//             .field("color_scheme", &self.color_scheme)
//             .field("text_style", &self.text_style)
//             .field("profile", &self.profile)
//             .field("repository", &"<DataRepository>")
//             .finish()
//     }
// }

impl HomeBoard {
    pub fn new(color_scheme: ColorScheme, text_style: TextStyle, profile: String, repository: Arc<Mutex<dyn DataRepository>>, settings_file_path: String) -> Self {
        Self {
            color_scheme,
            text_style,
            profile,
            repository,
            settings_file_path,
        }
    }
}

impl Board for HomeBoard {
    fn title(&self) -> &str {
        "HotKeys"
    }

    fn icon(&self) -> Option<&str> {
        Some("icon.png")
    }

    fn color_scheme(&self) -> &ColorScheme {
        &self.color_scheme
    }

    fn text_style(&self) -> &TextStyle {
        &self.text_style
    }

    fn pads(&self, modifier: Option<ModifierState>) -> Box<dyn PadSet> {
        // Get last timestamp from repository
        // let last_timestamp = self.repository.lock()
        //     .map(|repo| repo.get_board_data(&self.profile, "home", "last_action_time"))
        //     .unwrap_or(None)
        //     .unwrap_or_else(|| "Never pressed".to_string());

        if let Some(modifier) = modifier {
            if !modifier.is_none() {

                let mut comment_text_style = self.text_style.clone();
                comment_text_style.pad_text_font = comment_text_style.pad_header_font.clone() + " Italic";

                let modifier = modifier.to_string();
                return Box::new(vec![
                    Pad::default(),
                    Pad {
                        header: "Custom pad colors\nand styles".to_string(),
                        text: "Specialized pad sets for different\nmodifier combinations.".to_string(),
                        text_style: Some(comment_text_style),
                        color_scheme: Some(self.color_scheme.inverted()),
                        ..Default::default()
                    },
                    Pad::default(),
                    Pad::default(),
                    Pad {
                        text: format!("😊 {} 😊", modifier),
                        ..Default::default()
                    },
                    Pad::default(),
                    Pad::default(),
                    Pad{
                        header: "Copy defaults to".to_string(),
                        text: "~/.config/hotkeys/".to_string(),
                        actions: vec![
                            Action::Pause(200),
                            Action::Command(". /usr/share/hotkeys/hotkeys-config".to_string()),
                        ],
                        ..Default::default()
                    },
                    Pad::default()
                ]);
            }
        }

        Box::new(vec![
            Pad::default(),
            Pad {
                header: "Press a NumPad key, a modifier key\nor Escape".to_string(),
                ..Default::default()
            },
            Pad::default(),
            Pad {
                text: "Project url".to_string(),
                actions: vec![
                    Action::Pause(200),
                    Action::OpenUrl("https://github.com/ivicakukic/hotkeys-linux".to_string()),
                ],
                ..Default::default()
            },
            Pad::default(),
            Pad {
                text: "Documentation".to_string(),
                actions: vec![
                    Action::Pause(200),
                    Action::OpenUrl("file:///usr/share/doc/hotkeys/README.md".to_string()),
                ],
                ..Default::default()
            },
            Pad::default(),
            Pad {
                text: "Configuration".to_string(),
                actions: vec![
                    Action::Pause(200),
                    Action::OpenUrl(self.settings_file_path.clone()),
                ],
                ..Default::default()
            },
            Pad::default()
            // Pad {
            //     header: "Last Action".to_string(),
            //     icon: "".to_string(),
            //     text: format!("🕐\n{}", last_timestamp),
            //     actions: vec![
            //         Action::CustomHomeAction,
            //     ],
            //     ..Default::default()
            // }
        ])
    }

    fn clone_box(&self) -> Box<dyn Board> {
        Box::new(self.clone())
    }
}