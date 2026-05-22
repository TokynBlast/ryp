use serde_json::{json, Value};
use indexmap::IndexMap;

pub type Config = IndexMap<String, Value>;

pub fn default() -> Config {
    let default_json = json!({
        // The comments are the explanations to use on hover
        "Tab Size": 4, // Number of spaces to insert when pressing tab
        "Auto Save": false, // Whether to autosave
        "Time To Auto Save": 30_000, // How long between autosaves
        "Tab BG Color": "#A9A9A9", // The color of the tabs
        "Tab FG Color": "#FFFFFF", // Color of tab foreground
        "Active Tab FG Color": "#FFFFFF", // Color of the text of the current tab
        "Active Tab BG Color": "#EE7101", // Color of tab currently in
        "Inactive Sidebar BG Color": "#808080", // Color of sidebar when not active
        "Active Sidebar BG Color": "#EE7101", // Color of sidebar BG when active
        "Inactive Sidebar Text Color": "#FFFFFF", // Color of sidebar FG when inactive
        "Active Sidebar Text Color": "#EE7101", // Color of sidebar FG when active
        "Selected Text Font Color": "#000000", // Color of selected text
        "Highlighted Text BG": "#0084FF", // Color of selected text BG
        "Editor BG Color": "#323232", // Background color of entire editor
        "Highlighting Theme": "base16-ocean.dark", // Highlighting theme for languages
        "Search": "Ctrl+F", // Shortcut to search
        "Search Highlight Selected Color BG": "#FFFF00", // BG of current chosen item when searching (modal, !sidebar)
        "Search Highlight Unselected Color BG": "#00FFFF", // BG of any item not chosen when searching (modal, !sidebar)
        "Search Highlight Selected Font Color": "#000000", // Font color of selected search item
        "Search Highlight Unselected Font Color": "#FFFFFF", // Font color of unselected items when searching
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
        "Debug Character": "λ", // Character put next to plugin print outputs
        "Debug Console": true, // Whether the debug console "exists" or not
    });

    // Convert the JSON into an object for IndexMap
    default_json.as_object()
        .unwrap()
        .clone()
        .into_iter()
        .collect()
}
