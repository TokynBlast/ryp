use serde_json::{json, Value};
use indexmap::{IndexMap};

pub type Config = IndexMap<String, Value>;

pub fn default() -> Config {
    let default_json = json!({
        "Tab Size": 4,
        "Auto Save": false,
        "Time To Auto Save": 30_000,
        "Tab BG Color": "#333333",
        "Active Tab BG Color": "#2E7D32",
        "Highlighting Theme": "base16-ocean.dark",
        "Search": "Ctrl+F",
        "Help": "Ctrl+K",
        "Sidebar Toggle": "Ctrl+B",
        "Previous Sidebar Tab": "Ctrl+A",
        "Next Sidebar Tab": "Ctrl+D",
        "Refresh Git": "Ctrl+G",
        "Close Tab": "Ctrl+A",
        "Quit": "Ctrl+W",
        "New File": "Ctrl+N",
        "Save File": "Ctrl+S",
        "Open Terminal": "Ctrl+T",
        "Open Debug Console": "Ctrl+E",
    });

    // Convert the JSON into an object for IndexMap
    default_json.as_object()
        .unwrap()
        .clone()
        .into_iter()
        .collect()
}
