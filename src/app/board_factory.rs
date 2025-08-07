use std::{collections::HashMap, sync::{Arc, Mutex}};
use anyhow::Result;

use crate::core::{Board, PadSet, ColorScheme, TextStyle, DataRepository, Pad};
use crate::components::boards::{StaticBoard, HomeBoard};
use super::config::{AppSettings, BoardConfig, BoardKind, PadConfig};

pub struct BoardFactory {
    settings: AppSettings,
    repository: Option<Arc<Mutex<dyn DataRepository>>>,
    profile: String,
}

impl BoardFactory {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            settings,
            repository: None,
            profile: "default".to_string(),
        }
    }

    pub fn with_repository(mut self, repository: Arc<Mutex<dyn DataRepository>>, profile: String) -> Self {
        self.repository = Some(repository);
        self.profile = profile;
        self
    }

    pub fn create_board(&self, board_config: &BoardConfig) -> Result<Box<dyn Board>> {
        let color_scheme = self.resolve_color_scheme(board_config);
        let text_style = self.resolve_text_style(board_config);

        match board_config.kind {
            BoardKind::Static => Ok(Box::new(self.create_static_board(board_config, color_scheme, text_style)?)),
            BoardKind::Home => {
                let Some(ref repo) = self.repository else {
                    return Err(anyhow::anyhow!("Repository required for Home board"));
                };
                Ok(Box::new(HomeBoard::new(color_scheme, text_style, self.profile.clone(), repo.clone(), self.settings.file_path().to_string())))
            }
        }
    }

    fn create_static_board(
        &self,
        board_config: &BoardConfig,
        color_scheme: ColorScheme,
        text_style: TextStyle,
    ) -> Result<StaticBoard> {
        let base_pads = self.resolve_base_pads(board_config)?;
        let modifier_pads = self.resolve_modifier_pads(board_config)?;

        Ok(StaticBoard::new(
            board_config.title.clone().unwrap_or_else(|| board_config.name.clone()), // if there is no 'title', use 'name' for title instead
            board_config.icon.clone(),
            color_scheme,
            text_style,
            base_pads,
            modifier_pads,
        ))
    }

    fn resolve_color_scheme(&self, board_config: &BoardConfig) -> ColorScheme {
        // configured "default" if present,  else hardcoded default
        let default_scheme = self.settings.get_color_scheme(&ColorScheme::default().name)
                .cloned()
                .unwrap_or_default();

        match &board_config.color_scheme {
            None => default_scheme,
            Some(scheme_name) => self.settings.get_color_scheme(scheme_name)
                .cloned()
                .unwrap_or(default_scheme)
        }
    }

    fn resolve_text_style(&self, board_config: &BoardConfig) -> TextStyle {
        // configured "default" if present,  else hardcoded default
        let default_style = self.settings.get_text_style(&TextStyle::default().name)
                .cloned()
                .unwrap_or_default();

        match &board_config.text_style {
            None => default_style,
            Some(style_name) => self.settings.get_text_style(style_name)
                .cloned()
                .unwrap_or(default_style),
        }
    }

    fn resolve_pad(&self, pad_config: &PadConfig) -> Pad {
        let pad_color_scheme = pad_config.color_scheme
            .as_ref()
            .and_then(|name| self.settings.get_color_scheme(name))
            .cloned();

        let pad_text_style = pad_config.text_style
            .as_ref()
            .and_then(|name| self.settings.get_text_style(name))
            .cloned();

        Pad {
            header: pad_config.header.clone(),
            text: pad_config.text.clone(),
            icon: pad_config.icon.clone(),
            actions: pad_config.actions.clone(),
            board: pad_config.board.clone(),
            color_scheme: pad_color_scheme,
            text_style: pad_text_style,
        }
    }

    fn resolve_base_pads(&self, board_config: &BoardConfig) -> Result<Box<dyn PadSet>> {
        match &board_config.base_pads {
            Some(padset_name) => {
                let padset_config = self.settings.get_padset_config(padset_name)
                    .ok_or_else(|| anyhow::anyhow!("PadSet '{}' not found", padset_name))?;
                let resolved_pads: Vec<Pad> = padset_config.items
                    .iter()
                    .map(|pad_config| self.resolve_pad(pad_config))
                    .collect();
                Ok(Box::new(resolved_pads))
            },
            None => Ok(Box::new(Vec::new())),
        }
    }

    fn resolve_modifier_pads(&self, board_config: &BoardConfig) -> Result<HashMap<String, Box<dyn PadSet>>> {
        let mut modifier_pads = HashMap::new();

        for (modifier, padset_name) in &board_config.modifier_pads {
            let padset_config = self.settings.get_padset_config(padset_name)
                .ok_or_else(|| anyhow::anyhow!("PadSet '{}' not found", padset_name))?;
            let resolved_pads: Vec<Pad> = padset_config.items
                .iter()
                .map(|pad_config| self.resolve_pad(pad_config))
                .collect();
            modifier_pads.insert(modifier.clone(), Box::new(resolved_pads) as Box<dyn PadSet>);
        }

        Ok(modifier_pads)
    }
}