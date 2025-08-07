use anyhow::Result;

#[allow(dead_code)]
/// DataRepository trait for hierarchical data storage
/// Provides read/write access to profile, board, and padset level data
pub trait DataRepository: Send + Sync + std::fmt::Debug {
    /// Get profile-level data
    fn get_profile_data(&self, profile: &str, key: &str) -> Option<String>;

    /// Set profile-level data
    fn set_profile_data(&mut self, profile: &str, key: &str, value: &str) -> Result<()>;

    /// Get board-level data within a profile
    fn get_board_data(&self, profile: &str, board: &str, key: &str) -> Option<String>;

    /// Set board-level data within a profile
    fn set_board_data(&mut self, profile: &str, board: &str, key: &str, value: &str) -> Result<()>;

    /// Get padset-level data within a board
    fn get_padset_data(&self, profile: &str, board: &str, padset: &str, key: &str) -> Option<String>;

    /// Set padset-level data within a board
    fn set_padset_data(&mut self, profile: &str, board: &str, padset: &str, key: &str, value: &str) -> Result<()>;

    /// Persist any pending changes to storage
    fn flush(&mut self) -> Result<()>;
}