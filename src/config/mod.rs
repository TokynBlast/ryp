use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub tab_size: usize,
    pub theme: ThemeConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub tab_bg: String,
    pub active_tab_bg: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_size: 4,
            theme: ThemeConfig {
                tab_bg: "#333333".to_string(),        // Dark grey
                active_tab_bg: "#2E7D32".to_string(), // Green
            },
        }
    }
}
