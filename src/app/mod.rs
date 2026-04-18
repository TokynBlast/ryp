use crate::config::Config;
use crate::core::editor::Editor;
use crate::input::action::SidebarCategory;
use crate::windows::modal::{Modal, ModalType};
use crate::plugin::action::PluginAction;
use crossterm::event::{self, Event};
use hashbrown::HashSet;
use std::time::Duration;
use syntect::highlighting::ThemeSet;
use syntect::highlighting::Style;
use syntect::parsing::SyntaxSet;
use std::path::Path;
use std::path::PathBuf;
use parking_lot::Mutex;
use triomphe::Arc;
use std::thread;
use aho_corasick::AhoCorasick;
use serde_json::{json, Value};

mod ui;

pub struct SearchResult {
    pub filepath: PathBuf,
    pub line_number: usize,
    pub content: String,
}

pub struct App {
    pub editors: Vec<Editor>,
    pub active_tab: usize,
    pub config: Config,
    pub modal: Option<Modal>,
    pub should_quit: bool,
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub workspace: Option<crate::core::tree::FileTree>,
    pub sidebar_category: SidebarCategory,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    pub search_scroll: usize,
    pub search_num_files: usize,
    pub search_num_occurrences: usize,
    pub search_advanced: Vec<String>,
    pub git_manager: crate::core::git::GitManager,
    pub git_changes: Vec<crate::core::git::GitFileChange>,
    pub git_scroll: usize,
    pub git_selected: usize,
    pub settings_selected: usize,
    pub settings_scroll: usize,
    pub terminal: crate::core::terminal::Terminal,
    pub terminal_visible: bool,
    pub debug_console_visible: bool,
    pub dirty: bool,
    pub rx: crossbeam::channel::Receiver<PluginAction>,
    pub whitespace_cache: Arc<Mutex<Vec<usize>>>,
    pub highlight_cache: Arc<Mutex<Vec<Vec<(Style, String)>>>>,
    pub host_terminal_height: u16,
    pub debug_logs: Vec<String>,
}

impl App {
    pub fn new(rx: crossbeam::channel::Receiver<PluginAction>) -> Self {
        Self {
            editors: vec![],                                                    // All editors open
            active_tab: 0,                                                      // Current active tab
            config: crate::config::default(),                                   // Current configuration
            modal: None,                                                        // Selection windows (confirm leave, new file, etc.)
            should_quit: false,                                                 // Whether Ryp should quit or not
            syntax_set: SyntaxSet::load_defaults_newlines(),                    // ???
            theme_set: ThemeSet::load_defaults(),                               // The current theme
            workspace: None,                                                    // Filetree, and other stuff
            sidebar_category: SidebarCategory::FileTree,                        // Current sidebar piece
            search_query: String::new(),                                        // Actual query to look for
            search_results: vec![],                                             // All results of search
            search_selected: 0,                                                 // Selected result in searches
            search_scroll: 0,                                                   // Y index on scroll
            search_num_files: 0,                                                // Number of files searched for query
            search_num_occurrences: 0,                                          // Times a query has occured
            search_advanced: vec![],                                            // Advanced features, like *.mi, etc.
            git_manager: crate::core::git::GitManager::new(),                   // manager for git
            git_changes: vec![],                                                // Changes in Git
            git_scroll: 0,                                                      // Y index on git tab scroll
            git_selected: 0,                                                    // Git diff file selected
            settings_selected: 0,                                               // Setting selected
            settings_scroll: 0,                                                 // Scroll on settings
            terminal: crate::core::terminal::Terminal::new(PathBuf::from(".")),// The terminal; Defaults to current path
            terminal_visible: false,                                            // Sets whether the terminal is currently visible or not
            debug_console_visible: false,                                       // Whether plugin debug console is visible or not
            dirty: true,                                                        // Whether there have been changes or not to the file(s)
            rx,                                                                 // Crossbeam send and receive
            whitespace_cache: Arc::new(Mutex::new(Vec::new())),                 // Cache for where whitespace is, used in searching (performance increase)
            highlight_cache: Arc::new(Mutex::new(Vec::new())),                 // Cache for highlighting (performance increase)
            host_terminal_height: 0,
            debug_logs: vec![],
        }
    }

    pub fn load_workspace(&mut self, path: &Path) {
      let path = &path.canonicalize().unwrap_or(path.to_path_buf());

      self.workspace = Some(crate::core::tree::FileTree::new(
        path.to_path_buf()
      ));
      self.git_manager.set_root(path);
      self.refresh_git();
      let _ = self
          .terminal
          .tx
          .send(format!("cd {}\n", path.display()).as_bytes().to_vec());
    }

    pub fn open_diff(&mut self, change_idx: usize) {
        if let Some(change) = self.git_changes.get(change_idx).cloned() {
            let mut editor = Editor::new();
            let mut lines = vec![format!("DIFF: {}", change.path), String::new()];
            for dl in change.diff {
                lines.push(dl.content);
            }
            editor.load_diff(Path::new(&change.path), lines);
            self.editors.push(editor);
            self.active_tab = self.editors.len() - 1;
        }
    }

    // This is just an example, and meant to show it works, since actual logic will be much more complex...
    pub fn change_settings(&mut self) {
        if let Some((_, val)) = self.config.get_index_mut(self.settings_selected) {
            match val {
                //TODO: Make it a box, that is on or off, like an HTML checkbox
                Value::Bool(b) => *val = Value::Bool(!*b),
                // TODO: Make it a continuous typing input,
                //       and escape on enter press, or esc.
                //
                //TODO: Also allow for up and down arrow movement
                //      to change and affect the number
                //      (Left and right should move cursor)
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        *val = serde_json::json!(i + 1);
                    }
                },
                // TODO: Make it a continuous typing input,
                //       and escape on enter press, or esc
                Value::String(s) => {
                    *val = Value::String(s.to_string() + "a")
                }

                _ => return,
            }
        }
    }

    pub fn open_file(&mut self, path: &Path, force_new_tab: bool) {
        // Check if file is already open
        let already_open = self.editors.iter().position(|e| {
            if let Some(p) = &e.filepath {
                p == path
            } else {
                false
            }
        });

        if self.workspace.is_none() {
          let workspace = path.parent().unwrap_or(Path::new("."));
          self.load_workspace(workspace);
        }

        if let Some(idx) = already_open {
            self.active_tab = idx;
        } else {
            let mut editor = Editor::new();
            if editor.load_file(path) {
                let theme_name = self.config.get("theme")
                    .and_then(|v| v.get("Highlighting Theme"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("base16-ocean.dark");

                let theme = &self.theme_set.themes[theme_name];

                editor.rebuild_highlight_cache(&self.syntax_set, theme);

                let current_is_dirty = self.current_editor().map_or(false, |e| e.dirty);
                if force_new_tab
                    || (self.editors.is_empty())
                    || (self.editors.len() == 1 && current_is_dirty)
                {
                    self.editors.push(editor);
                    self.active_tab = self.editors.len() - 1;
                } else if self.editors.len() == 1
                    && self.editors[0].lines.len() == 1
                    && self.editors[0].lines[0].is_empty()
                    && !self.editors[0].dirty
                {
                    self.editors[0] = editor;
                } else {
                    self.editors[self.active_tab] = editor;
                }
            }
        }
    }

    pub fn current_editor_mut(&mut self) -> Option<&mut Editor> {
        if self.editors.is_empty() {
            None
        } else {
            Some(&mut self.editors[self.active_tab])
        }
    }

    pub fn current_editor(&self) -> Option<&Editor> {
        if self.editors.is_empty() {
            None
        } else {
            Some(&self.editors[self.active_tab])
        }
    }

    pub fn run(&mut self, term: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        while !self.should_quit {
            self.host_terminal_height = term.size().unwrap().height;
            while let Ok(action) = self.rx.try_recv() {
                self.dirty = true; // Mark dirty because state changed
                match action {
                    PluginAction::MakeSetting { name, value } => {
                        self.config.insert(name, json!(value));
                    }

                    PluginAction::InsertText { text, x, y } => {
                        todo!("Implement InsertText\nUse self.active_tab in `src/app/mod.rs`");
                    }

                    PluginAction::GetSettingValue { name, responder } => {
                        let val = self.config.get(&name).cloned().unwrap_or(serde_json::Value::Null);

                        let mut lock = responder.value.lock();
                        *lock = Some(val);
                        responder.signal.notify_one(); // Wake up Lua!
                    }

                    PluginAction::DebugLog { message } => {
                        self.debug_logs.push(message);
                    }

                    PluginAction::SetSetting { name, value } => {
                        self.config.insert(name, json!(value));
                    }
                }
            }

            let had_update = self.terminal.update();
            if had_update {
                self.dirty = true;
            }

            if self.dirty {
                // do the cache spawn first, completely separately
                {
                    let cache = Arc::clone(&self.whitespace_cache);
                    let lines: Vec<String> = self.current_editor()
                        .map(|e| e.lines.clone())
                        .unwrap_or_default();
                    // TODO: Make this a crossbeam, rather than a thread
                    thread::spawn(move || {
                        let result: Vec<usize> = lines.iter()
                            .enumerate()
                            .filter(|(_, line)| line.chars().any(|c| c == ' ' || c == '\t'))
                            .map(|(i, _)| i)
                            .collect();
                        let mut cache = cache.lock();
                        *cache = result;
                    });
                } // borrow of self ends here

              term.draw(|f| ui::draw(f, self))?;
              self.dirty = false;
          }

            // Once typing, we assume more typing will occur, so we drop blocking
            let timeout = if self.dirty {
                Duration::from_millis(0)
            } else {
                Duration::from_millis(100)
            };

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        use crate::input::action::Action;
        use crate::input::keymap;
        use crossterm::event::{KeyCode, KeyModifiers};

        if self.terminal_visible {
            // Check for toggle keys or ESC
            let is_ctrl_t =
                key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL);
            let is_f5 = key.code == KeyCode::F(5);
            let is_esc = key.code == KeyCode::Esc;

            if is_ctrl_t || is_f5 || is_esc {
                self.dispatch(Action::ToggleTerminal);
                return;
            }

            self.dispatch(Action::TerminalInput(key));
            return;
        }

        let in_modal = self.modal.is_some();
        let is_tree_focused = self.workspace.as_ref().map_or(false, |w| w.focused);
        if let Some(action) = keymap::map_key(key, in_modal, is_tree_focused) {
            self.dispatch(action);
        }
        self.dirty = true;
    }

    pub fn dispatch(&mut self, action: crate::input::action::Action) {
        use crate::input::action::Action;
        use crate::windows::modal::ModalType;

        // Route Modal actions
        if let Some(ref mut modal) = self.modal {
            match action {
                Action::CloseModal => {
                    self.modal = None;
                    return;
                }
                Action::ModalInsert(c) => {
                    modal.insert_char(c);
                    if modal.modal_type == ModalType::NewFile {
                        self.validate_new_file();
                    }
                }
                Action::ModalDelete => {
                    modal.pop_char();
                    if modal.modal_type == ModalType::NewFile {
                        self.validate_new_file();
                    }
                }
                Action::ModalTab => {
                    if modal.modal_type == ModalType::Replace {
                        modal.toggle_focus();
                    } else if modal.modal_type == ModalType::QuitPrompt {
                        modal.active_button = (modal.active_button + 1) % 3;
                    } else if modal.modal_type == ModalType::ConfirmExit {
                        modal.active_button = (modal.active_button + 1) % 2;
                    } else {
                        self.find_next_match();
                    }
                }
                Action::ModalUp | Action::ModalLeft => {
                    if modal.modal_type == ModalType::QuitPrompt
                        || modal.modal_type == ModalType::ConfirmExit
                    {
                        if modal.active_button > 0 {
                            modal.active_button -= 1;
                        }
                    }
                }
                Action::ModalDown | Action::ModalRight => {
                    if modal.modal_type == ModalType::QuitPrompt {
                        if modal.active_button < 2 {
                            modal.active_button += 1;
                        }
                    } else if modal.modal_type == ModalType::ConfirmExit {
                        if modal.active_button < 1 {
                            modal.active_button += 1;
                        }
                    }
                }
                Action::ModalConfirm => {
                    if modal.modal_type == ModalType::Search {
                        self.find_next_match();
                    } else if modal.modal_type == ModalType::Replace {
                        self.replace_match();
                        self.find_next_match();
                    } else if modal.modal_type == ModalType::ReplaceAll {
                        // TODO: Implement loop stopping
                        loop {
                            self.replace_match();
                            self.find_next_match();
                        }
                    } else if modal.modal_type == ModalType::QuitPrompt {
                        match modal.active_button {
                            0 => self.should_quit = true, // Discard
                            1 => self.modal = None,       // Cancel
                            2 => {
                                // Save
                                let mut saved = false;
                                if let Some(editor) = self.current_editor_mut() {
                                    if let Some(path) = &editor.filepath {
                                        let content = editor.lines.join("\n");
                                        if std::fs::write(path, content).is_ok() {
                                            editor.dirty = false;
                                            saved = true;
                                        }
                                    }
                                }
                                if saved {
                                    self.should_quit = true;
                                } else {
                                    self.modal = None;
                                }
                            }
                            _ => {}
                        }
                    } else if modal.modal_type == ModalType::NewFile {
                        let path_str = modal.input.clone();
                        if !path_str.is_empty() && modal.error_message.is_none() {
                            let root = if let Some(ws) = &self.workspace {
                                &ws.nodes[ws.root].path
                            } else {
                                &PathBuf::from(".")
                            };
                            let full_path = root.join(&path_str);

                            // Ensure directory exists
                            if let Some(parent) = full_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }

                            if let Ok(_) = std::fs::write(&full_path, "") {
                                self.modal = None;
                                self.open_file(&full_path, true);
                            }
                        }
                    } else if modal.modal_type == ModalType::ConfirmExit {
                        match modal.active_button {
                            0 => self.modal = None,
                            1 => self.should_quit = true,
                            _ => {}
                        }
                    }
                }
                _ => {} // Other actions ignored in modal
            }
            return;
        }

        match action {
            Action::SwitchSidebarCategory(category) => {
                self.sidebar_category = category;
                if let Some(ws) = &mut self.workspace {
                    ws.focused = true;
                    ws.visible = true;
                }
                return;
            }
            Action::NextSidebarCategory => {
                self.sidebar_category = match self.sidebar_category {
                    SidebarCategory::FileTree => SidebarCategory::Search,
                    SidebarCategory::Search => SidebarCategory::Git,
                    SidebarCategory::Git => SidebarCategory::Settings,
                    SidebarCategory::Settings => SidebarCategory::FileTree,
                };
                if self.sidebar_category == SidebarCategory::Git {
                    self.refresh_git();
                }
                if let Some(ws) = &mut self.workspace {
                    ws.focused = true;
                    ws.visible = true;
                }
                return;
            }
            Action::PrevSidebarCategory => {
                self.sidebar_category = match self.sidebar_category {
                    SidebarCategory::FileTree => SidebarCategory::Settings,
                    SidebarCategory::Git => SidebarCategory::Search,
                    SidebarCategory::Search => SidebarCategory::FileTree,
                    SidebarCategory::Settings => SidebarCategory::Git,
                };
                if self.sidebar_category == SidebarCategory::Git {
                    self.refresh_git();
                }
                if let Some(ws) = &mut self.workspace {
                    ws.focused = true;
                    ws.visible = true;
                }
                return;
            }
            Action::FocusFile(path, line) => {
                self.open_file(&path, false);
                if let Some(line_num) = line {
                    if let Some(editor) = self.current_editor_mut() {
                        editor.cursor_y = line_num.saturating_sub(1);
                        editor.cursor_x = 0;
                    }
                }
                if let Some(ws) = &mut self.workspace {
                    ws.focused = false;
                }
                return;
            }
            Action::SearchFiles(query) => {
                self.search_query = query;
                self.perform_search();
                return;
            }
            Action::RefreshGit => {
                self.refresh_git();
                return;
            }
            Action::OpenDiff(idx) => {
                self.open_diff(idx);
                return;
            }
            Action::ChangeSettings => {
                self.change_settings();
                return;
            }
            Action::OpenNewFileModal => {
                self.modal = Some(Modal::new(ModalType::NewFile));
                return;
            }
            Action::ToggleTerminal => {
                self.terminal_visible = !self.terminal_visible;
                self.dirty = true;
                return;
            }
            Action::TerminalInput(key) => {
                self.terminal.handle_key(key);
                return;
            }
            Action::ToggleDebugConsole => {
              self.debug_console_visible = !self.debug_console_visible;
              if self.debug_console_visible {
                self.dirty = true;
              }
            }
            _ => {}
        }

        let tab_size = self.config.get("Tab Size").and_then(|v| v.as_u64()).unwrap_or(4);

        let is_tree_focused = self.workspace.as_ref().map_or(false, |w| w.focused);
        if is_tree_focused {
            let mut close_focused = false;
            let mut file_to_open: Option<PathBuf> = None;
            let mut open_in_new_tab = false;

            {
                let ws = self.workspace.as_mut().unwrap();
                match action {
                    Action::InsertChar(c) => {
                        if self.sidebar_category == SidebarCategory::Search {
                            self.search_query.push(c);
                            self.perform_search();
                        }
                    }
                    Action::DeleteChar => {
                        if self.sidebar_category == SidebarCategory::Search {
                            self.search_query.pop();
                            self.perform_search();
                        }
                    }
                    Action::MoveUp(_) => match self.sidebar_category {
                        SidebarCategory::FileTree => {
                            ws.selected = ws.selected.saturating_sub(1);
                        }
                        SidebarCategory::Search => {
                            self.search_selected =
                                self.search_selected.saturating_sub(1);
                        }
                        SidebarCategory::Git => {
                            self.git_selected =
                                self.git_selected.saturating_sub(1);
                        }
                        SidebarCategory::Settings => {
                            if self.settings_selected > 0 {
                                self.settings_selected -= 1;

                                // If the selection goes above the visible area, scroll up
                                if self.settings_selected < self.settings_scroll {
                                   self.settings_scroll = self.settings_selected;
                                }
                            }
                        },
                    },
                    Action::MoveDown(_) => match self.sidebar_category {
                        SidebarCategory::FileTree => {
                            let max = ws.flatten().len().saturating_sub(1);
                            if ws.selected < max {
                                ws.selected += 1;
                            }
                        }
                        SidebarCategory::Search => {
                            if self.search_selected < self.search_results.len().saturating_sub(1) {
                                self.search_selected += 1;
                            }
                        }
                        SidebarCategory::Git => {
                            if self.git_selected < self.git_changes.len().saturating_sub(1) {
                                self.git_selected += 1;
                            }
                        }
                        SidebarCategory::Settings => {
                            if self.settings_selected < self.config.len().saturating_sub(1) {
                                self.settings_selected += 1;

                                // If the selection goes below the visible area, scroll down
                                // TODO: Implement visible area, 3 is just a stable size for when really small...
                                let visible_height = self.host_terminal_height;
                                if self.settings_selected >= self.settings_scroll + visible_height as usize {
                                    self.settings_scroll += 1;
                                }
                            }
                        },
                    },
                    Action::InsertNewline | Action::ModalConfirmForceNewTab => {
                        match self.sidebar_category {
                            SidebarCategory::FileTree => {
                                let force_new = action == Action::ModalConfirmForceNewTab;
                                let flat = ws.flatten();
                                if ws.selected < flat.len() {
                                    let node_idx = flat[ws.selected].0;
                                    if ws.nodes[node_idx].is_dir {
                                        if !force_new {
                                            ws.toggle(node_idx);
                                        }
                                    } else {
                                      file_to_open = Some(ws.nodes[node_idx].path.clone());
                                        let current_is_dirty =
                                            self.current_editor().map_or(false, |e| e.dirty);
                                        open_in_new_tab = force_new || current_is_dirty;
                                        close_focused = true;
                                    }
                                }
                            }
                            SidebarCategory::Search => {
                                if self.search_selected < self.search_results.len() {
                                    let result = &self.search_results[self.search_selected];
                                    let path = result.filepath.clone();
                                    let line = result.line_number;
                                    self.dispatch(Action::FocusFile(path, Some(line)));
                                    return;
                                }
                            }
                            SidebarCategory::Git => {
                                if self.git_selected < self.git_changes.len() {
                                    self.dispatch(Action::OpenDiff(self.git_selected));
                                    return;
                                }
                            }
                            SidebarCategory::Settings => {
                                if self.settings_selected < self.config.len() {
                                    self.dispatch(Action::ChangeSettings);
                                }
                            },
                        }
                    }
                    Action::SwitchFocus => {
                        close_focused = true;
                    }
                    Action::ToggleSidebar => {
                        ws.visible = false;
                        close_focused = true;
                    }
                    Action::Quit => self.should_quit = true,
                    _ => {}
                }
            }

            if let Some(path_str) = file_to_open {
                self.open_file(&path_str, open_in_new_tab);
            }

            if close_focused {
                if let Some(ws) = &mut self.workspace {
                    ws.focused = false;
                }
            }
            return;
        }

        // Actions that need access to the editors list (to avoid borrow conflicts)
        match action {
            Action::NextTab => {
                if !self.editors.is_empty() {
                    self.active_tab = (self.active_tab + 1) % self.editors.len();
                }
                return;
            }
            Action::PrevTab => {
                if !self.editors.is_empty() {
                    if self.active_tab == 0 {
                        self.active_tab = self.editors.len() - 1;
                    } else {
                        self.active_tab -= 1;
                    }
                }
                return;
            }
            Action::CloseTab => {
                if self.editors.is_empty() {
                    self.should_quit = true;
                    return;
                }

                let current_dirty = self.editors[self.active_tab].dirty;
                let is_new = self.editors[self.active_tab].filepath.is_none();

                if current_dirty && !is_new {
                    // For now, reuse QuitPrompt but it leads to quitting the app.
                    // In a future refactor we might want to distinguish.
                    // But the user's primary request is about the non-dirty last tab.
                    self.modal = Some(crate::windows::modal::Modal::new(ModalType::QuitPrompt));
                } else {
                    self.editors.remove(self.active_tab);
                    if self.active_tab >= self.editors.len() {
                        self.active_tab = self.editors.len().saturating_sub(1);
                    }
                }
                return;
            }
            Action::Quit => {
                if self.editors.is_empty() {
                    self.should_quit = true;
                    return;
                }

                let mut any_dirty = false;
                for e in &self.editors {
                    if e.dirty {
                        any_dirty = true;
                        break;
                    }
                }

                if any_dirty {
                    if self.editors.len() > 1 {
                        self.modal = Some(crate::windows::modal::Modal::new(ModalType::ConfirmExit));
                    } else {
                        self.modal = Some(crate::windows::modal::Modal::new(ModalType::QuitPrompt));
                    }
                } else {
                    self.should_quit = true;
                }
                return;
            }
            _ => {}
        }

        // Route Global & Editor actions
        match action {
            Action::SwitchFocus => {
                if let Some(ws) = &mut self.workspace {
                    if ws.visible {
                        ws.focused = true;
                    }
                }
                return;
            }
            Action::ToggleSidebar => {
                if let Some(ws) = &mut self.workspace {
                    ws.visible = !ws.visible;
                    if ws.visible {
                        ws.focused = true;
                    } else {
                        ws.focused = false;
                    }
                }
                return;
            }
            Action::OpenSearch => {
                self.modal = Some(Modal::new(ModalType::Search));
                return;
            }
            Action::OpenReplace => {
                self.modal = Some(Modal::new(ModalType::Replace));
                return;
            }
            Action::OpenHelp => {
                self.modal = Some(Modal::new(ModalType::Help));
                return;
            }
            Action::Save => {
                self.save_current_file();
                return;
            }
            _ => {}
        }

        if let Some(editor) = self.current_editor_mut() {
            match action {
                Action::MoveUp(shift) => editor.move_up(shift),
                Action::MoveDown(shift) => editor.move_down(shift),
                Action::MoveLeft(shift) => editor.move_left(shift),
                Action::MoveRight(shift) => editor.move_right(shift),

                Action::PageUp(shift) => {
                    for _ in 0..40 {
                        editor.move_up(shift);
                    }
                }
                Action::PageDown(shift) => {
                    for _ in 0..40 {
                        editor.move_down(shift);
                    }
                }

                Action::InsertChar(c) => editor.insert_char(c),
                Action::InsertNewline => editor.insert_newline(),
                Action::DeleteChar => editor.delete_char(),
                Action::Tab => {
                    for _ in 0..tab_size {
                        editor.insert_char(' ');
                    }
                }
                _ => {}
            }
        }
    }

    fn find_next_match(&mut self) {
        // Extract and clone the search term to drop the immutable borrow of self
        let search_term = match &self.modal {
            Some(modal) if !modal.input.is_empty() => modal.input.clone(),
            _ => return,
        };

        if let Some(editor) = self.current_editor_mut() {
            let total_lines = editor.lines.len();
            let start_y = editor.cursor_y;
            let start_x = editor.cursor_x + 1;

            for i in 0..total_lines {
                let y = (start_y + i) % total_lines;
                let line = &editor.lines[y];
                let x_offset = if i == 0 { start_x } else { 0 };

                if x_offset < line.len() {
                    if let Some(match_x) = line[x_offset..].find(&search_term) {
                        editor.cursor_y = y;
                        editor.cursor_x = x_offset + match_x;
                        return;
                    }
                }
            }
        }
    }

    fn replace_match(&mut self) {
        let (search_term, replace_term) = if let Some(modal) = &self.modal {
            (modal.input.clone(), modal.replace_input.clone())
        } else {
            return;
        };

        if search_term.is_empty() {
            return;
        }

        if let Some(editor) = self.current_editor_mut() {
            let line = editor.lines[editor.cursor_y].clone();
            if editor.cursor_x + search_term.len() <= line.len() {
                if &line[editor.cursor_x..editor.cursor_x + search_term.len()]
                    == search_term.as_str()
                {
                    let mut new_line = line[..editor.cursor_x].to_string();
                    new_line.push_str(&replace_term);
                    new_line.push_str(&line[editor.cursor_x + search_term.len()..]);
                    editor.lines[editor.cursor_y] = new_line;
                }
            }
        }
    }

    fn refresh_git(&mut self) {
        self.git_changes = self.git_manager.get_changes();
    }

    fn save_current_file(&mut self) {
        if let Some(editor) = self.current_editor_mut() {
            if let Some(path) = &editor.filepath {
                let content = editor.lines.join("\n");
                if std::fs::write(path, content).is_ok() {
                    editor.dirty = false;
                }
            }
        }
    }

    fn validate_new_file(&mut self) {
        if let Some(modal) = &mut self.modal {
            if modal.modal_type == ModalType::NewFile {
                let path_str = &modal.input;
                if path_str.is_empty() {
                    modal.error_message = None;
                    return;
                }
                let root = if let Some(ws) = &self.workspace {
                    &ws.nodes[ws.root].path
                } else {
                    &std::path::PathBuf::from(".")
                };
                let full_path = root.join(path_str);
                if full_path.exists() {
                    modal.error_message = Some("File already exists!".to_string());
                } else {
                    modal.error_message = None;
                }
            }
        }
    }

    fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            self.search_num_files = 0;
            self.search_num_occurrences = 0;
            self.search_advanced.clear();
            return;
        }

        let ws_path = if let Some(ws) = &self.workspace {
            ws.nodes[ws.root].path.clone()
        } else {
            return;
        };

        let mut results = vec![];
        let mut files_found = HashSet::new();

        use ignore::WalkBuilder;
        let builder = WalkBuilder::new(ws_path);
        let ac = AhoCorasick::new([&self.search_query]).unwrap();

        for entry in builder.build().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let mut file_had_match = false;
                    for (i, line) in content.lines().enumerate() {
                        if ac.is_match(line) {
                            results.push(SearchResult {
                                filepath: entry.path().to_path_buf(),
                                line_number: i + 1,
                                content: line.trim().to_string(),
                            });
                            file_had_match = true;
                        }
                    }
                    if file_had_match {
                        files_found.insert(entry.path().to_path_buf());
                    }
                }
            }
        }

        self.search_num_occurrences = results.len();
        self.search_num_files = files_found.len();
        self.search_results = results;
        self.search_selected = 0;
        self.search_scroll = 0;
    }
}
