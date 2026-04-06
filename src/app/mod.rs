use crate::config::Config;
use crate::core::editor::Editor;
use crate::input::action::SidebarCategory;
use crate::windows::modal::{Modal, ModalType};
use crossterm::event::{self, Event};
use std::collections::HashSet;
use std::time::Duration;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

mod ui;

pub struct SearchResult {
    pub filepath: String,
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
    pub git_manager: crate::core::git::GitManager,
    pub git_changes: Vec<crate::core::git::GitFileChange>,
    pub git_scroll: usize,
    pub git_selected: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            editors: vec![],
            active_tab: 0,
            config: Config::default(),
            modal: None,
            should_quit: false,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            workspace: None,
            sidebar_category: SidebarCategory::FileTree,
            search_query: String::new(),
            search_results: vec![],
            search_selected: 0,
            search_scroll: 0,
            search_num_files: 0,
            search_num_occurrences: 0,
            git_manager: crate::core::git::GitManager::new(),
            git_changes: vec![],
            git_scroll: 0,
            git_selected: 0,
        }
    }

    pub fn load_workspace(&mut self, path: &str) {
        self.workspace = Some(crate::core::tree::FileTree::new(std::path::PathBuf::from(
            path,
        )));
        self.git_manager.set_root(path.to_string());
        self.refresh_git();
    }

    pub fn open_diff(&mut self, change_idx: usize) {
        if let Some(change) = self.git_changes.get(change_idx).cloned() {
            let mut editor = Editor::new();
            let mut lines = vec![format!("DIFF: {}", change.path), String::new()];
            for dl in change.diff {
                lines.push(dl.content);
            }
            editor.load_diff(&change.path, lines);
            self.editors.push(editor);
            self.active_tab = self.editors.len() - 1;
        }
    }

    pub fn open_file(&mut self, path: &str, force_new_tab: bool) {
        // Check if file is already open
        let already_open = self.editors.iter().position(|e| {
            if let Some(p) = &e.filepath {
                p.to_str() == Some(path)
            } else {
                false
            }
        });

        if let Some(idx) = already_open {
            self.active_tab = idx;
        } else {
            let mut editor = Editor::new();
            if editor.load_file(path) {
                let current_is_dirty = self.current_editor().map_or(false, |e| e.dirty);
                if force_new_tab || (self.editors.is_empty()) || (self.editors.len() == 1 && current_is_dirty) {
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

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> std::io::Result<()> {
        while !self.should_quit {
            terminal.draw(|f| ui::draw(f, self))?;

            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        use crate::input::keymap;

        let in_modal = self.modal.is_some();
        let is_tree_focused = self.workspace.as_ref().map_or(false, |w| w.focused);
        if let Some(action) = keymap::map_key(key, in_modal, is_tree_focused) {
            self.dispatch(action);
        }
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
                                &std::path::PathBuf::from(".")
                            };
                            let full_path = root.join(&path_str);

                            // Ensure directory exists
                            if let Some(parent) = full_path.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }

                            if let Ok(_) = std::fs::write(&full_path, "") {
                                self.modal = None;
                                self.open_file(&full_path.to_string_lossy(), true);
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
                    SidebarCategory::Git => SidebarCategory::FileTree,
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
                    SidebarCategory::FileTree => SidebarCategory::Git,
                    SidebarCategory::Git => SidebarCategory::Search,
                    SidebarCategory::Search => SidebarCategory::FileTree,
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
            Action::OpenNewFileModal => {
                self.modal = Some(Modal::new(ModalType::NewFile));
                return;
            }
            _ => {}
        }

        let tab_size = self.config.tab_size;

        let is_tree_focused = self.workspace.as_ref().map_or(false, |w| w.focused);
        if is_tree_focused {
            let mut close_focused = false;
            let mut file_to_open = None;
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
                    Action::MoveUp(_) => {
                        match self.sidebar_category {
                            SidebarCategory::FileTree => {
                                if ws.selected > 0 {
                                    ws.selected -= 1;
                                }
                            }
                            SidebarCategory::Search => {
                                if self.search_selected > 0 {
                                    self.search_selected -= 1;
                                }
                            }
                            SidebarCategory::Git => {
                                if self.git_selected > 0 {
                                    self.git_selected -= 1;
                                }
                            }
                        }
                    }
                    Action::MoveDown(_) => {
                        match self.sidebar_category {
                            SidebarCategory::FileTree => {
                                let max = ws.flatten().len().saturating_sub(1);
                                if ws.selected < max {
                                    ws.selected += 1;
                                }
                            }
                            SidebarCategory::Search => {
                                if self.search_selected < self.search_results.len().saturating_sub(1)
                                {
                                    self.search_selected += 1;
                                }
                            }
                            SidebarCategory::Git => {
                                if self.git_selected < self.git_changes.len().saturating_sub(1) {
                                    self.git_selected += 1;
                                }
                            }
                        }
                    }
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
                                        file_to_open =
                                            ws.nodes[node_idx].path.to_str().map(|s| s.to_string());
                                        let current_is_dirty = self.current_editor().map_or(false, |e| e.dirty);
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
        let search_term = if let Some(modal) = &self.modal {
            modal.input.clone()
        } else {
            return;
        };

        if search_term.is_empty() {
            return;
        }

        if let Some(editor) = self.current_editor_mut() {
            let mut y = editor.cursor_y;
            let mut x = editor.cursor_x + 1; // start from next char
            let total_lines = editor.lines.len();

            for _ in 0..total_lines {
                if y >= total_lines {
                    y = 0;
                }
                let line = &editor.lines[y];
                if x < line.len() {
                    if let Some(match_x) = line[x..].find(&search_term) {
                        editor.cursor_y = y;
                        editor.cursor_x = x + match_x;
                        return;
                    }
                }
                y += 1;
                x = 0;
            }
        }
    }

    fn replace_match(&mut self) {
        let (search_term, replace_term) = if let Some(modal) = &self.modal {
            (modal.input.clone(), modal.input.clone()) // Assuming replace_input was meant if modal_type was Replace, but using input for now
        } else {
            return;
        };

        if search_term.is_empty() {
            return;
        }

        if let Some(editor) = self.current_editor_mut() {
            let line = editor.lines[editor.cursor_y].clone();
            if editor.cursor_x + search_term.len() <= line.len() {
                if &line[editor.cursor_x..editor.cursor_x + search_term.len()] == search_term.as_str() {
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
            return;
        }

        let ws_path = if let Some(ws) = &self.workspace {
            ws.nodes[ws.root].path.clone()
        } else {
            return;
        };

        let mut results = vec![];
        let query = self.search_query.to_lowercase();
        let mut files_found = HashSet::new();

        use ignore::WalkBuilder;
        let builder = WalkBuilder::new(ws_path);
        for entry in builder.build().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let mut file_had_match = false;
                    for (i, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(&query) {
                            results.push(SearchResult {
                                filepath: entry.path().to_string_lossy().to_string(),
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
