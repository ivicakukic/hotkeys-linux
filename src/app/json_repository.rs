use std::{collections::HashMap, fs, path::Path};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::core::DataRepository;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct PadSetData {
    #[serde(flatten)]
    data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct BoardData {
    #[serde(flatten)]
    data: HashMap<String, String>,
    
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    padsets: HashMap<String, PadSetData>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
struct ProfileData {
    #[serde(flatten)]
    data: HashMap<String, String>,
    
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    boards: HashMap<String, BoardData>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct RepositoryData {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    profiles: HashMap<String, ProfileData>,
}

#[derive(Debug)]
pub struct JsonRepository {
    file_path: String,
    data: RepositoryData,
    dirty: bool,
}

impl JsonRepository {
    pub fn new(file_path: String) -> Result<Self> {
        let data = if Path::new(&file_path).exists() {
            let contents = fs::read_to_string(&file_path)?;
            serde_json::from_str(&contents).unwrap_or_default()
        } else {
            RepositoryData::default()
        };
        
        Ok(Self {
            file_path,
            data,
            dirty: false,
        })
    }
    
    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl DataRepository for JsonRepository {
    fn get_profile_data(&self, profile: &str, key: &str) -> Option<String> {
        self.data.profiles
            .get(profile)?
            .data
            .get(key)
            .cloned()
    }
    
    fn set_profile_data(&mut self, profile: &str, key: &str, value: &str) -> Result<()> {
        let profile_data = self.data.profiles
            .entry(profile.to_string())
            .or_default();
        profile_data.data.insert(key.to_string(), value.to_string());
        self.mark_dirty();
        Ok(())
    }
    
    fn get_board_data(&self, profile: &str, board: &str, key: &str) -> Option<String> {
        self.data.profiles
            .get(profile)?
            .boards
            .get(board)?
            .data
            .get(key)
            .cloned()
    }
    
    fn set_board_data(&mut self, profile: &str, board: &str, key: &str, value: &str) -> Result<()> {
        let profile_data = self.data.profiles
            .entry(profile.to_string())
            .or_default();
        let board_data = profile_data.boards
            .entry(board.to_string())
            .or_default();
        board_data.data.insert(key.to_string(), value.to_string());
        self.mark_dirty();
        Ok(())
    }
    
    fn get_padset_data(&self, profile: &str, board: &str, padset: &str, key: &str) -> Option<String> {
        self.data.profiles
            .get(profile)?
            .boards
            .get(board)?
            .padsets
            .get(padset)?
            .data
            .get(key)
            .cloned()
    }
    
    fn set_padset_data(&mut self, profile: &str, board: &str, padset: &str, key: &str, value: &str) -> Result<()> {
        let profile_data = self.data.profiles
            .entry(profile.to_string())
            .or_default();
        let board_data = profile_data.boards
            .entry(board.to_string())
            .or_default();
        let padset_data = board_data.padsets
            .entry(padset.to_string())
            .or_default();
        padset_data.data.insert(key.to_string(), value.to_string());
        self.mark_dirty();
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        if self.dirty {
            let json = serde_json::to_string_pretty(&self.data)?;
            fs::write(&self.file_path, json)?;
            self.dirty = false;
            log::info!("Repository data saved to {}", self.file_path);
        }
        Ok(())
    }
}