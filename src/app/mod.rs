use crate::config::Config;
use crate::core::editor::Editor;
use crate::input::action::SidebarCategory;
use crate::windows::modal::{Modal, ModalType};
use crate::plugin::action::PluginAction;
use crossterm::event::{self, Event};
use hashbrown::HashSet;
use std::collections::VecDeque;
use std::time::Duration;
use syntect::{parsing::SyntaxSet, highlighting::ThemeSet};
use std::path::{Path, PathBuf};
use parking_lot::{Mutex, RwLock};
use triomphe::Arc;
use aho_corasick::AhoCorasick;
use serde_json::{json, Value};
use compact_str::CompactString;
use rayon::{self, prelude::*};
use crate::core::{tree, git, terminal};

mod ui;

pub struct SearchResult {
    pub filepath: PathBuf,
    pub line_number: usize,
    pub content: CompactString,
}

pub struct App {
    pub editors: Vec<Editor>,                                                    // All open editors
    pub active_tab: usize,                                                       // Current active tab
    pub config: Config,                                                          // Configuration of editor(s) and plugin(s)
    pub modal: Option<Modal>,                                                    // Selection windows (confirm leave, new file, etc.)
    // TODO: Remove this, and return later
    pub should_quit: bool,                                                       // Whether Ryp should quit or not
    pub syntax_set: SyntaxSet,                                                   // Syntax set for languages
    pub theme_set: ThemeSet,                                                     // Highlighting colors
    pub workspace: Option<tree::FileTree>,                                       // Sidebar things
    pub sidebar_category: SidebarCategory,                                       // Current sidebar piece open
    pub search_query: CompactString,                                             // What to look for
    pub search_results: Vec<SearchResult>,                                       // Results of a serarch
    pub search_selected: usize,                                                  // Selected search result
    pub search_scroll: usize,                                                    // Scroll amount on search results
    pub search_num_files: usize,                                                 // Number of files with contents found
    pub search_num_occurrences: usize,                                           // Number of times a query found
    pub search_advanced: Vec<CompactString>,                                     // Advanced search input (*.f, /dev/, etc.)
    pub git_manager: git::GitManager,                                            // Git
    pub git_changes: Vec<git::GitFileChange>,                                    // Every change Git found
    pub git_scroll: usize,                                                       // Scroll on Git
    pub git_selected: usize,                                                     // File selected in Git view
    pub settings_selected: usize,                                                // Selected setting
    pub settings_scroll: usize,                                                  // Settings scroll
    pub terminal: terminal::Terminal,                                            // Builtin terminal
    pub terminal_visible: bool,                                                  // If the terminal is visible
    pub debug_console_visible: bool,                                             // If the debug console is visible
    pub dirty: bool,                                                             // If the terminal needs to be updated
    pub plugin_rx: crossbeam_channel::Receiver<PluginAction>,                    // Lua plugin reciever
    pub whitespace_cache: Arc<RwLock<Vec<usize>>>,                               // Where whitespace is in the editor
    pub host_terminal_height: u16,                                               // True height of terminal we're running in
    pub host_terminal_width: u16,                                                // True height of terminal we're running in
    pub debug_logs: VecDeque<CompactString>,                                     // Lua plugin print function routed here
    pub os: CompactString,                                                       // String of what the OS is (not OsString)
    pub key_pressed: Mutex<Option<CompactString>>,                               // Which key was pressed
    pub focused: bool,                                                           // Whether the terminal is focused or not
}

impl App {
    pub fn new( plugin_rx: crossbeam_channel::Receiver<PluginAction>) -> Self {
        Self {
            editors: vec![],
            active_tab: 0,
            config: crate::config::default(),
            modal: None,
            should_quit: false,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            workspace: None,
            sidebar_category: SidebarCategory::FileTree,
            search_query: CompactString::default(),
            search_results: vec![],
            search_selected: 0,
            search_scroll: 0,
            search_num_files: 0,
            search_num_occurrences: 0,
            search_advanced: vec![],
            git_manager: git::GitManager::new(),
            git_changes: vec![],
            git_scroll: 0,
            git_selected: 0,
            settings_selected: 0,
            settings_scroll: 0,
            terminal: terminal::Terminal::new(PathBuf::from(".")),
            terminal_visible: false,
            debug_console_visible: false,
            dirty: true,
            plugin_rx,
            whitespace_cache: Arc::new(RwLock::new(Vec::new())),
            host_terminal_height: 0,
            host_terminal_width: 0,
            debug_logs: VecDeque::with_capacity(40),
            os: CompactString::const_new(
                if cfg!(target_os = "windows") {
                    "Windows "
                } else if cfg!(target_os = "macos"){
                    "MacOS "
                } else if cfg!(target_os = "ios") {
                    "iOS "
                } else if cfg!(target_os = "tvos") {
                    "TVOS "
                } else if cfg!(target_os = "visionos") {
                    "VisionOS "
                } else if cfg!(target_os = "linux") {
                    use os_info::Type::*;
                    match os_info::get().os_type() {
                        Pop => "!Pop_OS ",
                        Arch => "Arch Linux 󰣇",
                        Fedora => "Fedora ",
                        Gentoo => "Gentoo ",
                        Redhat
                        | RedHatEnterprise => "Redhat ",
                        AlmaLinux => "AlmaLinux ",
                        AOSC => "AOSC ",
                        Artix => "Artix ",
                        CentOS => "CentOS ",
                        Cygwin => "Cygwin ",
                        Debian => "Debian ",
                        Elementary => "ElementaryOS ",
                        EndeavourOS => "EndeavourOS ",
                        Garuda => "Garuda ",
                        Illumos => "Illumos ",
                        Kali => "Kali Linux ",
                        Manjaro => "Manjaro ",
                        Mint => "Linux Mint 󰣭",
                        NixOS => "NixOS ",
                        Nobara => "Nobara ",
                        Raspbian => "Raspbian ",
                        RockyLinux => "RockyLinux ",
                        openSUSE => "openSUSE ",
                        SUSE => "SUSE ",
                        Solus => "Solus ",
                        Ubuntu => "Ubuntu 󰕈",
                        Void => "Void Linux ",
                        Zorin => "Zorin ",
                        _ => "Linux ",
                    }
                } else if cfg!(target_os="android") {
                    "Android "
                } else if cfg!(any(target_arch="wasm32", target_arch="wasm64")) {
                    "WebAssembly "
                } else if cfg!(target_os = "freebsd") {
                    "FreeBSD "
                } else if cfg!(target_os = "openbsd") {
                    "OpenBSD "
                } else if cfg!(target_os = "netbsd") {
                    "NetBSD"
                } else if cfg!(target_os = "dragonfly") {
                    "DragonFly BSD"
                } else if cfg!(target_os = "haiku") {
                    "Haiku"
                } else if cfg!(target_os = "solaris") {
                    "Solaris "
                } else if cfg!(target_os = "fuchsia") {
                    "Fuchsia"
                } else if cfg!(target_os = "emscripten") {
                    "Web 󰖟"
                } else if cfg!(target_os = "horizon") {
                    // This could technicallt be a 3DS, or a Switch
                    // But, the character for the Nintendo Logo is
                    // too small: 
                    "Nintendo 󰟡"
                } else if cfg!(target_os = "illumos") {
                    "Illumos"
                } else if cfg!(target_os = "nto") {
                    // This is usually an OS for medical equipment/cars...
                    "QNX Neutrino"
                } else if cfg!(target_os = "vita") {
                    "PlayStation Vita"
                } else if cfg!(target_os = "redox") {
                    // OS written in the same language this text editor is!
                    "Redox OS"
                } else if cfg!(target_os = "vxworks") {
                    "Wind River VxWorks"
                } else if cfg!(target_os = "espidf") {
                    "ESP Board"
                // When GNU Herd gets keyboard support, we can uncomment it.
                // } else if cfg!(target_os = "hurd") {
                //     CompactString::const_new("GNU Herd ")
                } else if cfg!(target_os = "uefi") {
                    // Although this would be insane if it were happening,
                    // it would be pretty cool!
                    // And for that, I've decided to have some fun with these,
                    // and be literal or add fitting icons!
                    if cfg!(target_arch = "x86_64") {
                        "x86_64 UEFI 󰻠"
                    } else if cfg!(target_arch = "x86") {
                        "x86 UEFI 󰻟"
                    } else if cfg!(any(target_arch="arm", target_arch="aarch64", target_arch="arm64ec")) {
                        "ARM UEFI 󰿗"
                    } else if cfg!(target_arch = "avr") {
                        "AVR UEFI "
                    } else if cfg!(target_arch = "bpf") {
                        "BPF UEFI"
                    } else if cfg!(target_arch = "csky") {
                        "C-SKY UEFI "
                    } else if cfg!(target_arch = "hexagon") {
                        "Qualcom Hexago UEFI "
                    } else if cfg!(target_arch = "loongarch64") {
                        "LoongArch64 UEFI 󰻠"
                    } else if cfg!(target_arch = "m68k") {
                        "Motorola 68000 UEFI "
                    } else if cfg!(any(
                        target_arch="mips",
                        target_arch="mips32r6",
                        target_arch="mips64",
                        target_arch="mips64r6",
                        target_arch="msp430",
                        ))
                    {
                        "MIPS UEFI "
                    } else if cfg!(target_arch = "nvptx64") {
                        "NVIDIA PTX UEFI"
                    } else if cfg!(any(target_arch="powerpc", target_arch="powerpc64")) {
                        "PowerPC UEFI ⏻"
                    } else if cfg!(any(target_arch="riscv32", target_arch="riscv64")) {
                        // The  may be misleading, and may at some point need to be changed
                        "Risc-V UEFI "
                    } else if cfg!(target_arch="s390x") {
                        "IBM Z UEFI"
                    } else if cfg!(any(target_arch="sparc", target_arch="sparc64")) {
                        "Sparc UEFI "
                    } else if cfg!(target_arch = "xtensa") {
                        "Xtensa"
                    } else {
                        // Would be unreachable!(), but we can't garuntee that in the future
                        // there won't be more arch' that Rust supports, so we just use "UEFI"
                        // as a backup value for that case
                        "UEFI"
                    }
                } else {
                    "Unknown ?"
                }
            ),
            key_pressed: Mutex::new(None),
            focused: true,
        }
    }

    pub fn load_workspace(&mut self, path: &Path) {
        let path = &path.canonicalize().unwrap_or(path.to_path_buf());

        self.workspace = Some(tree::FileTree::new(
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
            let mut lines  = vec![CompactString::from(format!("DIFF: {}", change.path)), CompactString::default()];
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
                Value::Bool(b) => *b = !*b,
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
            while let Ok(action) = self.plugin_rx.try_recv() {
                match action {
                    PluginAction::MakeSetting { name, value } => {
                        self.config.insert(name, json!(value));
                    }
                    PluginAction::InsertStrAtCursor { txt } => {
                        if let Some(editor) = self.current_editor_mut() {
                            editor.lines[editor.cursor_x].insert_str(editor.cursor_y, &txt);
                        }
                    }
                    PluginAction::GetSettingValue { name, responder } => {
                        let val = self.config.get(&name).cloned().unwrap_or(serde_json::Value::Null);

                        let mut lock = responder.value.lock();
                        *lock = Some(val);
                        responder.signal.notify_one();
                    }
                    PluginAction::DebugLog { message } => {
                        self.debug_logs.push_back(message);
                        if self.debug_console_visible {
                            self.dirty = true;
                        }
                    }
                    PluginAction::SetSetting { name, value } => {
                        self.config.insert(name, json!(value));
                    }
                    PluginAction::GetKeyPress { responder } => {
                        // Since this is something dependent on the user, if they don't have Ryp focused,
                        // they can't type, so it's faster to give a value instea of get a value then
                        // give the value back to the plugin
                        let mut lock = responder.string.lock();

                        if self.focused {
                            *lock = self.key_pressed.lock().take();
                        } else {
                            *lock = None;
                        }

                        responder.signal.notify_one();
                    }
                    PluginAction::GetStrAt { from, to, responder } => {
                        if let Some(editor) = self.current_editor() {
                            let val: CompactString =
                                if editor.lines[from[1]].len() <= from[1]
                                && editor.lines[to[1]].len() <= to[1]
                            {
                                editor.lines[from[0]].to_string().chars().skip(from[1]).take(to[1] - from[1]).collect::<String>().into()
                            } else {
                                CompactString::default()
                            };
                            let mut lock = responder.string.lock();
                            *lock = Some(val);
                            responder.signal.notify_one();
                        }
                    }
                    PluginAction::GetCursorX { responder } => {
                        let mut lock = responder.number.lock();
                        *lock = self.current_editor().map(|editor| editor.cursor_x);
                        responder.signal.notify_one();
                    }
                    PluginAction::GetCursorY { responder } => {
                        let mut lock = responder.number.lock();
                        *lock = self.current_editor().map(|editor| editor.cursor_y);
                        responder.signal.notify_one();
                    }
                    PluginAction::SetCursorPos { pos } => {
                        if let Some(editor) = self.current_editor_mut() {
                            editor.cursor_x = pos[0];
                            editor.cursor_y = pos[1];
                            self.dirty = true;
                        }
                    }
                    PluginAction::GetCursorPos { responder } => {
                        if let Some(editor) = self.current_editor() {
                            let val = Some(vec![editor.cursor_x, editor.cursor_y]);
                            let mut lock = responder.numbers.lock();
                            *lock = val;
                            responder.signal.notify_one();
                        }
                    }
                    PluginAction::SetCursorX { x} => {
                        if let Some(editor) = self.current_editor_mut() {
                            editor.cursor_x = x;
                            self.dirty = true;
                        }
                    }
                    PluginAction::SetCursorY { y } => {
                        if let Some(editor) = self.current_editor_mut() {
                            editor.cursor_y = y;
                            self.dirty = true;
                        }
                    }
                    PluginAction::GetLine { line, responder } => {
                        if let Some(editor) = self.current_editor() {
                            let val = if line <= editor.lines.len() {
                                editor.lines[line].clone()
                            } else {
                                CompactString::default()
                            };
                            let mut lock = responder.string.lock();
                            *lock = Some(val);
                            responder.signal.notify_one();
                        }
                    }
                    PluginAction::SetLine { line, contents } => {
                        if let Some(editor) = self.current_editor_mut() {
                            if line <= editor.lines.len() {
                                editor.lines[line] = contents;
                                editor.dirty = true;
                            }
                            self.dirty = true;
                        }
                    }
                    PluginAction::SetStrAt { from, to, txt } => {
                        if let Some(editor) = self.current_editor_mut() {
                            let (start_x, start_y) = (from[0], from[1]);
                            let (end_x, end_y) = (to[0], to[1]);

                            // Basic bounds check to prevent panics
                            if start_y < editor.lines.len() && end_y < editor.lines.len() && start_y < end_y {

                                // Calculate byte offsets for the start and end of the selection
                                // We use char_indices to remain UTF-8 safe
                                let start_byte = editor.lines[start_y].char_indices().nth(start_x).map(|(i, _)| i);
                                let end_byte = editor.lines[end_y].char_indices().nth(end_x).map(|(i, c)| i + c.len_utf8());

                                if let (Some(s_idx), Some(e_idx)) = (start_byte, end_byte) {
                                    // Get the last line (incase only some was changed)
                                    let line_suffix = editor.lines[end_y].split_off(e_idx);

                                    // Modify the first line incase not all of it was changed
                                    let first_line = &mut editor.lines[start_y];
                                    first_line.truncate(s_idx);
                                    first_line.push_str(&txt);
                                    first_line.push_str(&line_suffix);

                                    // Add in everything else
                                    editor.lines.drain((start_y + 1)..=end_y);
                                }
                            }
                        }
                    }
                    PluginAction::InsertStrAt { pos, txt } => {
                        if let Some(editor) = self.current_editor_mut() {
                            if pos[0] <= editor.lines.len() {
                                editor.lines[pos[0]].insert_str(pos[1], &txt);
                            }
                        }
                    }
                    PluginAction::GetCharAtCursor { responder } => {
                        if let Some(editor) = self.current_editor() {
                            let val = Some(editor.lines[editor.cursor_y].as_bytes()[editor.cursor_x] as char);
                            let mut lock = responder.c.lock();
                            *lock = val;
                            responder.signal.notify_one();
                        }
                    }
                    PluginAction::GetCharAt { pos, responder } => {
                        if let Some(editor) = self.current_editor() {
                            let val: Option<char> = if editor.lines[pos[1]].len() <= pos[0] {
                                Some(editor.lines[pos[1]].as_bytes()[pos[0]] as char)
                            } else {
                                None
                            };
                            let mut lock = responder.c.lock();
                            *lock = val;
                            responder.signal.notify_one();
                        }
                    }
                    PluginAction::InsertCharAt { pos, c } => {
                        if let Some(editor) = self.current_editor_mut() {
                            if let Some(line) = editor.lines.get_mut(pos[1]) {
                                line.insert(pos[0], c);
                            }
                        }
                    }
                    PluginAction::InsertCharAtCursor { c } => {
                        if let Some(editor) = self.current_editor_mut() {
                            if let Some(line) = editor.lines.get_mut(editor.cursor_y) {
                                line.insert(editor.cursor_x, c);
                            }
                        }
                    }
                    PluginAction::SetCharAt { pos, c } => {
                        if let Some(editor) = self.current_editor_mut() {
                            if let Some(line) = editor.lines.get_mut(pos[1]) {
                                // Find the byte range of the character at the given visual index
                                let target_char = line.char_indices().nth(pos[0]);

                                if let Some((idx, old_char)) = target_char {
                                    // CompactString supports replace_range like a normal String
                                    let end_idx = idx + old_char.len_utf8();
                                    line.replace_range(idx..end_idx, &c.to_string());
                                }
                            }
                        }
                    }
                    PluginAction::SetCharAtCursor { c } => {
                        if let Some(editor) = self.current_editor_mut() {
                            if let Some(line) = editor.lines.get_mut(editor.cursor_y) {
                                let target_char = line.char_indices().nth(editor.cursor_x);

                                if let Some((idx, old_char)) = target_char {
                                    let end_idx = idx + old_char.len_utf8();
                                    line.replace_range(idx..end_idx, &c.to_string());
                                }
                            }
                        }
                    }
                }
            }

            if self.focused {
                self.dirty = if !self.dirty {
                    self.terminal_visible && self.terminal.update()
                } else {
                    term.autoresize().is_ok()
                };

                if self.dirty {
                    // Since dirty is now only triggered on changes, including height,
                    // we set it here to give the most accurate info, with anything that
                    // might access it in the future :)
                    let height_size = term.size().unwrap();
                    (self.host_terminal_height, self.host_terminal_width) = (height_size.height, height_size.width);

                    term.draw(|f| ui::draw(f, self))?;
                    self.dirty = false;
                }
            }

            if crossterm::event::poll(Duration::from_millis(10))? {
                let event = event::read()?;
                match event {
                    Event::Key(key) => self.handle_key(key),
                    Event::FocusLost => self.focused = false,
                    Event::FocusGained => self.focused = true,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        use crate::input::{action::Action, keymap};
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
        // TODO: There might be a simpler way to do this
        self.key_pressed = Mutex::new(Some(CompactString::from(key.code.to_string())));
        self.dirty = true;
    }

    pub fn close_current_editor(&mut self) {
        self.editors.remove(self.active_tab);
        if self.active_tab >= self.editors.len() {
            self.active_tab = self.editors.len().saturating_sub(1);
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
                    match modal.modal_type {
                        ModalType::NewFile => self.validate_new_file(),
                        _ => {},
                    }
                }
                Action::ModalDelete => {
                    modal.pop_char();
                    match modal.modal_type {
                        ModalType::NewFile => self.validate_new_file(),
                        _ => {},
                    }
                }
                Action::ModalTab => match modal.modal_type {
                    ModalType::Replace => modal.toggle_focus(),
                    ModalType::QuitPrompt => {
                        modal.active_button += 1;
                        if modal.active_button >= 3 { modal.active_button = 0; }
                    }
                    ModalType::ConfirmExit => {
                        modal.active_button += 1;
                        if modal.active_button >= 2 { modal.active_button = 0; }
                    }
                    ModalType::Search => self.find_next_match(),
                    _ => {}
                },
                Action::ModalLeft => {
                    match modal.modal_type {
                        ModalType::ConfirmExit => {
                            modal.active_button =
                                modal.active_button.saturating_sub(1);
                        }
                        ModalType::Search => {
                            modal.active_button =
                                modal.active_button.saturating_sub(1);
                        }
                        _ => {}
                    }
                }
                Action::ModalRight => {
                    match modal.modal_type {
                        ModalType::Search => {
                          // Active button for the search modal is just the cursor position
                          modal.active_button += 1;
                        }
                        ModalType::ConfirmExit => {
                            if modal.active_button < 1 {
                                modal.active_button += 1;
                            }
                        }
                        _ => {}
                    }
                }
                Action::ModalUp => {
                    match modal.modal_type {
                        ModalType::QuitPrompt | ModalType::CloseTabPrompt => {
                            modal.active_button =
                                modal.active_button.saturating_sub(1);
                        }
                        _ => {}
                    }
                }
                Action::ModalDown => {
                    match modal.modal_type {
                        ModalType::QuitPrompt | ModalType::CloseTabPrompt => {
                            if modal.active_button < 2 {
                                modal.active_button += 1;
                            }
                        }
                        _ => {}
                    }
                }
                Action::ModalConfirm => {
                    match modal.modal_type {
                        ModalType::Search => {
                            self.find_next_match();
                        }
                        ModalType::Replace => {
                            self.replace_match();
                            self.find_next_match();
                        }
                        ModalType::ReplaceAll => {
                            // TODO: Implement better loop stopping
                            if let Some(modal) = self.modal.take() {
                                while self.search_num_occurrences != 0 {
                                    self.replace_match();
                                    self.find_next_match();
                                }
                                self.modal = Some(modal);
                            }
                        }
                        ModalType::QuitPrompt => {
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
                        }
                        ModalType::NewFile => {
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
                        }
                        ModalType::ConfirmExit => {
                            match modal.active_button {
                                0 => self.modal = None,
                                1 => self.should_quit = true,
                                _ => {}
                            }
                        }
                        ModalType::CloseTabPrompt => {
                            match modal.active_button {
                                0 => {
                                    // Discard
                                    self.modal = None;
                                    self.close_current_editor();
                                }
                                1 => {
                                    // Cancel
                                    self.modal = None;
                                }
                                2 => {
                                    // Save
                                    self.modal = None;
                                    self.save_current_file();
                                    self.close_current_editor();
                                }
                                _ => {}
                            }
                        }
                        _ => {} // Other actions ignored in modal
                    }
                }
                _ => {}
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
                // TODO: Turn this into a VecDeque so that it is easier to expand later,
                //       and it becomes a pointer move instead of an if branch
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
                // TODO: Turn this into a VecDeque so that it is easier to expand later,
                //       and it becomes a pointer move instead of an if branch
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
                self.search_query = query.into();
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
                // TODO: Make these expandable
                // TODO: Implement move left and right for single file search
                match action {
                    Action::InsertChar(c) => {
                        if self.sidebar_category == SidebarCategory::Search {
                            self.search_query.push(c);
                            self.perform_search();
                        }
                    }
                    Action::BackSpace => {
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
                            self.settings_selected = self.settings_selected.saturating_sub(1)
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
                    self.active_tab += 1;
                    if self.active_tab >= self.editors.len() {
                        self.active_tab = 0;
                    }
                }
                return;
            }
            Action::PrevTab => {
                if !self.editors.is_empty() {
                    if self.active_tab == 0 {
                        // There's no need for a saturating sub, as we can very safely
                        // assume that because a usize must be >= 0, and we check if
                        // it is 0, there's no need to ever check that it might go below
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
                    self.modal = Some(crate::windows::modal::Modal::new(ModalType::CloseTabPrompt));
                } else {
                    self.close_current_editor();
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
                if self.modal.as_ref().map_or(false, |m| m.modal_type == ModalType::Help) {
                    self.modal = None;
                    self.dirty = true;
                } else {
                    self.modal = Some(Modal::new(ModalType::Help));
                }
                return;
            }
            Action::Save => {
                self.save_current_file();
                return;
            }
            _ => {}
        }

        let height = self.host_terminal_height.clone();

        if let Some(editor) = self.current_editor_mut() {
            match action {
                Action::MoveUp(shift) => editor.move_up(shift),
                Action::MoveDown(shift) => editor.move_down(shift),
                Action::MoveLeft(shift, ctrl) => editor.move_left(shift, ctrl),
                Action::MoveRight(shift, ctrl) => editor.move_right(shift, ctrl),

                Action::Copy() => editor.copy(),
                Action::Cut() => editor.cut(),
                Action::Paste() => editor.paste(),

                Action::PageUp(shift) => {
                    editor.update_selection(shift);
                    for _ in 0..height {
                        editor.move_up(shift);
                    }
                }
                Action::PageDown(shift) => {
                    editor.update_selection(shift);
                    for _ in 0..height {
                        editor.move_down(shift);
                    }
                }

                Action::InsertChar(c) => {
                    editor.insert_char(c);
                    editor.dirty = true;
                }
                Action::InsertNewline => {
                    editor.insert_newline();
                    editor.dirty = true;
                }
                Action::DeleteChar => {
                    editor.delete_char();
                    editor.dirty = true;
                }
                Action::BackSpace => {
                    editor.backspace_char();
                    editor.dirty = true;
                }
                Action::Tab => {
                    for _ in 0..tab_size {
                        editor.insert_char(' ');
                    }
                    editor.dirty = true;
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
                    if let Some(match_x) = line[x_offset..].find(&search_term.to_string()) {
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
                    editor.lines[editor.cursor_y] = CompactString::from_string_buffer(new_line);
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
                    modal.error_message = Some(CompactString::from("File already exists!"));
                } else {
                    modal.error_message = None;
                }
            }
        }
    }

    fn fill_ws_cache(&mut self) {
        {
            let cache = Arc::clone(&self.whitespace_cache);
            let lines = self.current_editor()
                .map(|e| e.lines.clone())
                .unwrap_or_default();
            rayon::spawn(move || {
                let result: Vec<usize> = lines.par_iter()
                    .enumerate()
                    .filter(|(_, line)| line.as_bytes().iter().any(|&c| c == b' ' || c == b'\t' || c == b'\n'))
                    .map(|(i, _)| i)
                    .collect();
                let mut guard = cache.write();
                *guard = result;
            });
        }
    }

    fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            self.search_results.clear();
            self.search_num_files = 0;
            self.search_num_occurrences = 0;
            return;
        }

        // TODO: Make this happen once when we need it
        self.fill_ws_cache();

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
                                content: CompactString::from(line.trim()),
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
