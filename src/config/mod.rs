use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub num_of_item: usize,
    pub tab_size: usize,
    pub theme: ThemeConfig,
    pub auto_save: bool,
    pub auto_save_timer: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub tab_bg: String,
    pub active_tab_bg: String,
    pub highlight_theme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            num_of_item: 6,
            tab_size: 4,
            theme: ThemeConfig {
                tab_bg: "#333333".to_string(),                 // Dark grey
                active_tab_bg: "#2E7D32".to_string(),          // Green
                highlight_theme: "base16-ocean.dark".to_string(),// Easy to read
            },
            auto_save: true,
            auto_save_timer: 30_000, // every 5 minutes, miliseconds
        }
    }
}
