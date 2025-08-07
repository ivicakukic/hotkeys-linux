use std::path::PathBuf;


#[derive(Debug, Clone)]
pub struct Resources {
    config_paths: Vec<PathBuf>
}

impl Resources {

    pub fn new(config_paths: Vec<PathBuf>) -> Self {
        Resources { config_paths }
    }

    pub fn file(&self, file_name: &str) -> Option<PathBuf> {
        for path in &self.config_paths {
            let file_path = path.join(file_name);
            if file_path.exists() {
                return Some(file_path);
            }
        }
        None
    }

    pub fn icon(&self, icon_file: &str) -> Option<PathBuf> {
        let icon_file = format!("icons/{}", icon_file);
        self.file(&icon_file)
    }

    pub fn log_toml(&self) -> Option<PathBuf> {
        self.file(env!("RESOURCE_LOG_FILE"))
    }

    pub fn settings_json(&self) -> Option<PathBuf> {
        self.file(env!("RESOURCE_SETTINGS_FILE"))
    }

    pub fn data_json(&self) -> PathBuf {
        self.config_paths[0].join(env!("RESOURCE_DATA_FILE"))
    }

}
