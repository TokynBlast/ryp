use serde_json::{json, Value};
use indexmap::IndexMap;

pub type Config = IndexMap<String, Value>;

pub fn default() -> Config {
    let default_json = json!({
        // The comments are the explanations to use on hover
        "Tab Size": 4, // Number of spaces to insert when pressing tab
        "Auto Save": false, // Whether to autosave
        "Time To Auto Save": 30_000, // How long between autosaves
        "Tab BG Color": "#333333", // The color of the tabs (default is #333333)
        "Active Tab BG Color": "#2E7D32", // Color of tab currently in (default is #2E7D32)
        "Highlighting Theme": "base16-ocean.dark", // Highlighting theme for languages
        "Search": "Ctrl+F", // Shortcut to search
        "Help": "Ctrl+K", // Shortcut to open help modal
        "Sidebar Toggle": "Ctrl+B", // Shortcut for opening or closing sidebar
        "Previous Sidebar Tab": "Ctrl+A", // Shortcut to go to the previous tab
        "Next Sidebar Tab": "Ctrl+D", // Shortcut to go to the next tab
        "Refresh Git": "Ctrl+G", // Shortcut to refresh git view
        "Close Tab": "Ctrl+A", // Shortcut to close the current tab
        "Quit": "Ctrl+W", // Shortcut to quit Ryp
        "New File": "Ctrl+N", // Shortcut to open a new file
        "Save File": "Ctrl+S", // Shortcut to save current open file
        "Open Terminal": "Ctrl+T", // Shortcut to open the builtin terminal
        "Open Debug Console": "Ctrl+E", // Shortcut to open the plugin debug console
    });

    // Convert the JSON into an object for IndexMap
    default_json.as_object()
        .unwrap()
        .clone()
        .into_iter()
        .collect()
}
