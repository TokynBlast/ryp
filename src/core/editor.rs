use std::fs;
use std::path::{PathBuf, Path};
use compact_str::CompactString;
use std::cell::Cell;
use std::sync::Arc;
use arc_swap::ArcSwap;
use std::collections::VecDeque;
use arboard::Clipboard;
use syntect::{
  parsing::ScopeStack,
  highlighting::{
      ThemeSet,
      Highlighter,
      HighlightState,
  }
};

pub struct Editor {
    pub lines: Vec<CompactString>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub target_x: usize,
    pub scroll_y: Cell<usize>,
    pub scroll_x: Cell<usize>,
    pub selection_start: Option<(usize, usize)>, // (start_x, start_y)
    pub filepath: Option<PathBuf>,
    pub dirty: bool,
    pub is_diff: bool,
    pub lang: CompactString,
    pub highlight_cache: Arc<ArcSwap<VecDeque<HighlightState>>>,
    pub clipboard: Option<arboard::Clipboard>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![CompactString::default()],
            cursor_x: 0,
            cursor_y: 0,
            target_x: 0,
            scroll_y: Cell::new(0),
            scroll_x: Cell::new(0),
            selection_start: None,
            filepath: None,
            dirty: false,
            is_diff: false,
            lang: CompactString::default(),
            highlight_cache: Arc::new(
                ArcSwap::from_pointee(
                  VecDeque::from([
                    HighlightState::new(
                        &Highlighter::new(
                            &ThemeSet::load_defaults().themes["base16-ocean.dark"]),
                            ScopeStack::new()
                        )
                    ])
                )
            ),
            clipboard:
                if let Some(clip_board) = Some(Clipboard::new()) {
                    if clip_board.is_ok() {
                        Some(clip_board.unwrap())
                    } else {
                        None
                    }
                } else {
                    None
                },
        }
    }

    pub fn load_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            self.lines = content.lines().map(|s| CompactString::from(s)).collect();
            if self.lines.is_empty() {
                self.lines.push(CompactString::default());
            }
            self.filepath = Some(path.to_path_buf());
            self.dirty = false;
            self.is_diff = false;
            self.lang = if let Some(path) = &self.filepath {
                match path.extension().and_then(|e| e.to_str()) {
                    Some("cpp") => CompactString::new("C++ 󰙲"),
                    Some("hpp") => CompactString::new("C++ Header 󰙲"),
                    Some("rs") => CompactString::new("Rust 󱘗"),
                    Some("lua") => CompactString::new("Lua "),
                    Some("ll") => CompactString::new("LLVM "),
                      Some("asm")
                    | Some("s") => CompactString::new("Assembly"),
                    Some("c") => CompactString::new("C 󰙱"),
                    Some("h")=> CompactString::new("C Header 󰙱"),
                    Some("js") => CompactString::new("JavaScript "),
                      Some("ml")
                    | Some("mli") => CompactString::new("OCaml "),
                    Some("html") => CompactString::new("HTML "),
                    Some("md") => CompactString::new("MarkDown 󰍔"),
                    Some("css") => CompactString::new("CSS "),
                    Some("mi") => CompactString::new("Minis"),
                    Some("cs") => CompactString::new("C# 󰌛"),
                    Some("gd") => CompactString::new("Godot Script "),
                    Some("py") => CompactString::new("Python 󰌠"),
                    Some("java") => CompactString::new("Java 󰬷"),
                    Some("fs") => CompactString::new("F#"),
                    Some("fsx") => CompactString::new("F# Script"),
                    Some("bat") => CompactString::new("Bash "),
                    Some("sh") => CompactString::new("Shell "),
                    Some("go") => CompactString::new("Go 󰟓"),
                    Some("php") => CompactString::new("PHP 󰌟"),
                    Some("rb") => CompactString::new("Ruby "),
                    Some("ts") => CompactString::new("TypeScript 󰛦"),
                      Some("f")
                    | Some("for")
                    | Some("f08")
                    | Some("f90")
                    | Some("f03")
                    | Some("f95")
                    | Some("F90")
                    | Some("F")
                    | Some("f15")
                    | Some("f20") => CompactString::new("Fortran 󱈚"),
                    Some("m") => CompactString::new("Objective-C "),
                    Some("mm") => CompactString::new("Objective-C++"),
                    Some("adb") => CompactString::new("Ada"),
                    Some("d") => CompactString::new("D "),
                    Some("mod") => CompactString::new("Modula"),
                    Some("cob") => CompactString::new("COBOL"),
                    Some("a68") => CompactString::new("ALGOL"),
                    Some("ipynb") => CompactString::new("Jupyter Notebook"),
                    Some("red") => CompactString::new("Red"),
                    Some("json") => CompactString::new("JSON "),
                    Some("r") => CompactString::new("R "),
                    Some("lhs") => CompactString::new("Haskel "),
                    Some("xaml") => CompactString::new("XAML 󰙳"),
                    Some("yaml") => CompactString::new("YAML "),
                    Some("kt") => CompactString::new("Kotlin "),
                    Some("kts") => CompactString::new("Kotlin Script "),
                    Some("txt") => CompactString::new("Plain Text "),
                    _ => CompactString::new("Unknown"),
                }
            } else {
              CompactString::new("Unknown")
            };
            true
        } else {
            false
        }
    }

    pub fn load_diff(&mut self, path: &Path, content: Vec<CompactString>) {
        self.lines = content;
        if self.lines.is_empty() {
            self.lines.push(CompactString::default());
        }
        self.filepath = Some(path.to_path_buf());
        self.dirty = false;
        self.is_diff = true;
    }

    pub fn insert_char(&mut self, c: char) {
        self.delete_selection();
        let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
        self.lines[self.cursor_y].insert(idx, c);
        self.cursor_x += 1;
        self.target_x = self.cursor_x;
    }

    pub fn insert_newline(&mut self) {
        self.delete_selection();
        let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
        let remainder = self.lines[self.cursor_y].split_off(idx);
        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, remainder);
        self.cursor_x = 0;
        self.target_x = self.cursor_x;
    }

    pub fn backspace_char(&mut self) {
        if self.delete_selection() {
            return;
        }

        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
            self.lines[self.cursor_y].remove(idx);
        } else if self.cursor_y > 0 {
            let current_line = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].chars().count();
            self.lines[self.cursor_y].push_str(&current_line);
        }
        self.target_x = self.cursor_x;
    }

    pub fn delete_char(&mut self) {
        if self.delete_selection() {
            return;
        }

        let line_len = self.lines[self.cursor_y].chars().count();

        if self.cursor_x < line_len {
            let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
            self.lines[self.cursor_y].remove(idx);
        } else if self.cursor_y < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_y + 1);
            self.lines[self.cursor_y].push_str(&next_line);
        }
        self.target_x = self.cursor_x;
    }

    // selection logic
    pub fn is_selected(&self, check_x: usize, check_y: usize) -> bool {
        if let Some((start_x, start_y)) = self.selection_start {
            let (first_x, first_y, last_x, last_y) = if start_y < self.cursor_y || (start_y == self.cursor_y && start_x < self.cursor_x) {
                (start_x, start_y, self.cursor_x, self.cursor_y)
            } else {
                (self.cursor_x, self.cursor_y, start_x, start_y)
            };

            if check_y < first_y || check_y > last_y {
                return false;
            }
            if check_y == first_y && check_y == last_y {
                return check_x >= first_x && check_x < last_x;
            }
            if check_y == first_y {
                return check_x >= first_x;
            }
            if check_y == last_y {
                return check_x < last_x;
            }
            return true;
        }
        false
    }

    fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
        s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(s.len())
    }

    pub fn delete_selection(&mut self) -> bool {
        if let Some((start_x, start_y)) = self.selection_start {
            let ((sy, sx), (ey, ex)) = if start_y < self.cursor_y || (start_y == self.cursor_y && start_x < self.cursor_x) {
                ((start_y, start_x), (self.cursor_y, self.cursor_x))
            } else {
                ((self.cursor_y, self.cursor_x), (start_y, start_x))
            };

            if sy == ey {
                let bs = Self::char_to_byte_idx(&self.lines[sy], sx);
                let be = Self::char_to_byte_idx(&self.lines[sy], ex);
                self.lines[sy].replace_range(bs..be, "");
            } else {
                let bs = Self::char_to_byte_idx(&self.lines[sy], sx);
                let mut new_start = CompactString::from(self.lines[sy][..bs].to_string());

                let be = Self::char_to_byte_idx(&self.lines[ey], ex);
                let new_end = self.lines[ey][be..].to_string();

                new_start.push_str(&new_end);

                self.lines.drain(sy..=ey);
                self.lines.insert(sy, new_start);
            }
            self.dirty = true;
            self.cursor_y = sy;
            self.cursor_x = sx;
            self.target_x = sx;
            self.selection_start = None;
            return true;
        }
        false
    }

    pub fn update_selection(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some((self.cursor_x, self.cursor_y));
            }
        } else {
            self.selection_start = None;
        }
    }

    // movement
    pub fn move_up(&mut self, shift: bool) {
        self.update_selection(shift);
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.target_x.min(self.lines[self.cursor_y].len());
        }
    }

    pub fn move_down(&mut self, shift: bool) {
        self.update_selection(shift);
        if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = self.target_x.min(self.lines[self.cursor_y].len());
        }
    }

    pub fn move_left(&mut self, shift: bool, ctrl: bool) {
        self.update_selection(shift);
        if self.cursor_x > 0 {
            if ctrl {
                let line = &self.lines[self.cursor_y];
                let bytes = line.as_bytes();

                // Skip all whitespace
                while self.cursor_x > 0 && bytes[self.cursor_x - 1] == b' ' {
                    self.cursor_x -= 1;
                }

                // Skip what isn't whitespace
                while self.cursor_x > 0 && bytes[self.cursor_x - 1] != b' ' {
                    self.cursor_x -= 1;
                }
            } else {
                self.cursor_x -= 1;
            }
        } else if self.cursor_y > 0 {
            // Move to the end of the previous line
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
        self.target_x = self.cursor_x;
    }

    pub fn move_right(&mut self, shift: bool, ctrl: bool) {
        self.update_selection(shift);

        let line_len = self.lines[self.cursor_y].len();
        if self.cursor_x < line_len {
            if ctrl {
                let line = &self.lines[self.cursor_y];
                let bytes = line.as_bytes();

                // Skip non-whitespace
                while self.cursor_x < line_len && bytes[self.cursor_x] != b' ' {
                    self.cursor_x += 1;
                }
                // Skip whitespace
                while self.cursor_x < line_len && bytes[self.cursor_x] == b' ' {
                    self.cursor_x += 1;
                }
            } else {
                self.cursor_x += 1;
            }
        } else if self.cursor_y < self.lines.len() - 1 {
            // Move to the start of the next line
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.target_x = self.cursor_x;
    }

    fn get_selected_text(&self) -> Option<String> {
        let (start_x, start_y) = self.selection_start?;

        // Normalize coordinates (ensure we know which is start vs end)
        let ((sy, sx), (ey, ex)) = if start_y < self.cursor_y || (start_y == self.cursor_y && start_x < self.cursor_x) {
            ((start_y, start_x), (self.cursor_y, self.cursor_x))
        } else {
            ((self.cursor_y, self.cursor_x), (start_y, start_x))
        };

        if sy == ey {
            // Single line selection
            let line = &self.lines[sy];
            let bs = Self::char_to_byte_idx(line, sx);
            let be = Self::char_to_byte_idx(line, ex);
            Some(line[bs..be].to_string())
        } else {
            // Multi-line selection
            let mut result = String::new();

            // First line: from start_x to end
            let first_line = &self.lines[sy];
            let bs = Self::char_to_byte_idx(first_line, sx);
            result.push_str(&first_line[bs..]);
            result.push('\n');

            // Middle lines: full content
            for y in (sy + 1)..ey {
                result.push_str(&self.lines[y]);
                result.push('\n');
            }

            // Last line: from start to end_x
            let last_line = &self.lines[ey];
            let be = Self::char_to_byte_idx(last_line, ex);
            result.push_str(&last_line[..be]);

            Some(result)
        }
    }

    pub fn copy(&mut self) {
        if let Some(text) = self.get_selected_text() {
            if let Some(clipboard) = &mut self.clipboard {
                let _ = clipboard.set_text(text);
            }
        }
    }

    pub fn cut(&mut self) {
        if self.selection_start.is_some() {
            self.copy();
            self.delete_selection();
            self.dirty = true;
        }
    }

    pub fn paste(&mut self) {
        if let Some(clipboard) = &mut self.clipboard {
            if let Ok(text) = clipboard.get_text() {
                // If we have a selection, delete it first so we "replace" it
                self.delete_selection();

                let paste_lines: Vec<&str> = text.split('\n').collect();

                if paste_lines.is_empty() { return; }

                if paste_lines.len() == 1 {
                    // Simple single line paste
                    let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
                    self.lines[self.cursor_y].insert_str(idx, paste_lines[0]);
                    self.cursor_x += paste_lines[0].chars().count();
                } else {
                    // Multi-line paste
                    let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);

                    // Split the current line at cursor
                    let current_line_suffix = self.lines[self.cursor_y].split_off(idx);

                    // Add the first part of the paste to the current line
                    self.lines[self.cursor_y].push_str(paste_lines[0]);

                    // Insert middle lines
                    for i in 1..paste_lines.len() - 1 {
                        self.lines.insert(self.cursor_y + i, CompactString::from(paste_lines[i]));
                    }

                    // Handle the last line of the paste
                    let last_paste_line = paste_lines.last().unwrap();
                    let mut new_last_line = CompactString::from(*last_paste_line);
                    let final_cursor_x = new_last_line.chars().count();
                    new_last_line.push_str(&current_line_suffix);

                    self.cursor_y += paste_lines.len() - 1;
                    self.lines.insert(self.cursor_y, new_last_line);
                    self.cursor_x = final_cursor_x;
                }

                self.target_x = self.cursor_x;
                self.dirty = true;
            }
        }
    }
}
