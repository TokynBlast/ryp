use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use serde_json::Value;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub tab_size: usize,
    pub theme: ThemeConfig,
    pub auto_save: bool,
    pub auto_save_timer: usize,
    pub extra: BTreeMap<String, Value>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ThemeConfig {
    pub tab_bg: String,
    pub active_tab_bg: String,
    pub highlight_theme: String,
}

impl Config {
    pub fn len(&self) -> usize {
        const BASE_SIZE: usize = 6;              // MUST MATCH
        BASE_SIZE + self.extra.len()
    }
    #[inline(always)]
    pub fn base(&self) -> usize { return 6; }    // MUST MATCH
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_size: 4,
            theme: ThemeConfig {
                tab_bg: "#333333".to_string(),                 // Dark grey
                active_tab_bg: "#2E7D32".to_string(),          // Green
                highlight_theme: "base16-ocean.dark".to_string(),// Easy to read
            },
            auto_save: true,
            auto_save_timer: 30_000, // every 5 minutes, miliseconds
            extra: BTreeMap::new(),
        }
    }
}
