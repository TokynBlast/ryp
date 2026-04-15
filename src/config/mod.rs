use serde_json::{json, Value};
use std::collections::BTreeMap;

pub type Config = BTreeMap<String, Value>;

pub fn default() -> Config {
    let mut c = BTreeMap::new();

    c.insert("tab_size".to_string(), json!(4));

    // You can still nest maps inside the Value
    c.insert("theme".to_string(), json!({
        "tab_bg": "#333333",
        "active_tab_bg": "#2E7D32",
        "highlight_theme": "base16-ocean.dark"
    }));

    c.insert("auto_save".to_string(), json!(true));
    c.insert("auto_save_timer".to_string(), json!(30_000));
    c.insert("heyo".to_string(), json!("234"));

    c
}
