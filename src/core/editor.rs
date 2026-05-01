use std::fs;
use std::path::PathBuf;
use std::path::Path;
use compact_str::CompactString;
use std::cell::Cell;

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
                    Some("cpp") => CompactString::from("C++ 󰙲"),
                    Some("hpp") => CompactString::from("C++ Header 󰙲"),
                    Some("rs") => CompactString::from("Rust 󱘗"),
                    Some("lua") => CompactString::from("Lua "),
                    Some("ll") => CompactString::from("LLVM "),
                    Some("asm") | Some("s") => CompactString::from("Assembly"),
                    Some("c") => CompactString::from("C 󰙱"),
                    Some("h")=> CompactString::from("C Header 󰙱"),
                    Some("js") => CompactString::from("JavaScript "),
                    Some("ml") | Some("mli") => CompactString::from("OCaml "),
                    Some("html") => CompactString::from("HTML "),
                    Some("md") => CompactString::from("MarkDown 󰍔"),
                    Some("css") => CompactString::from("CSS "),
                    Some("mi") => CompactString::from("Minis"),
                    Some("cs") => CompactString::from("C# 󰌛"),
                    Some("gd") => CompactString::from("Godot Script "),
                    Some("py") => CompactString::from("Python 󰌠"),
                    Some("java") => CompactString::from("Java 󰬷"),
                    Some("fs") => CompactString::from("F#"),
                    Some("fsx") => CompactString::from("F# Script"),
                    Some("bat") => CompactString::from("Bash "),
                    Some("sh") => CompactString::from("Shell "),
                    Some("go") => CompactString::from("Go 󰟓"),
                    Some("php") => CompactString::from("PHP 󰌟"),
                    Some("rb") => CompactString::from("Ruby "),
                    Some("ts") => CompactString::from("TypeScript 󰛦"),
                      Some("f")
                    | Some("for")
                    | Some("f08")
                    | Some("f90")
                    | Some("f03")
                    | Some("f95")
                    | Some("F90")
                    | Some("F")
                    | Some("f15")
                    | Some("f20") => CompactString::from("Fortran 󱈚"),
                    Some("m") => CompactString::from("Objective-C "),
                    Some("mm") => CompactString::from("Objective-C++"),
                    Some("adb") => CompactString::from("Ada"),
                    Some("d") => CompactString::from("D "),
                    Some("mod") => CompactString::from("Modula"),
                    Some("cob") => CompactString::from("COBOL"),
                    Some("a68") => CompactString::from("ALGOL"),
                    Some("ipynb") => CompactString::from("Jupyter Notebook"),
                    Some("red") => CompactString::from("Red"),
                    Some("json") => CompactString::from("JSON "),
                    Some("r") => CompactString::from("R "),
                    Some("lhs") => CompactString::from("Haskel "),
                    Some("xaml") => CompactString::from("XAML 󰙳"),
                    Some("yaml") => CompactString::from("YAML "),
                    Some("kt") => CompactString::from("Kotlin "),
                    Some("kts") => CompactString::from("Kotlin Script "),
                    Some("txt") => CompactString::from("Plain Text "),
                    _ => CompactString::from("Unknown"),
                }
            } else {
              CompactString::from("Unknown")
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
        self.dirty = true;
        self.delete_selection();
        let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
        self.lines[self.cursor_y].insert(idx, c);
        self.cursor_x += 1;
        self.target_x = self.cursor_x;
    }

    pub fn insert_newline(&mut self) {
        self.dirty = true;
        self.delete_selection();
        let idx = Self::char_to_byte_idx(&self.lines[self.cursor_y], self.cursor_x);
        let remainder = self.lines[self.cursor_y].split_off(idx);
        self.cursor_y += 1;
        self.lines.insert(self.cursor_y, remainder);
        self.cursor_x = 0;
        self.target_x = self.cursor_x;
    }

    pub fn delete_char(&mut self) {
        self.dirty = true;
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

    pub fn move_left(&mut self, shift: bool) {
        self.update_selection(shift);
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines[self.cursor_y].len();
        }
        self.target_x = self.cursor_x;
    }

    pub fn move_right(&mut self, shift: bool) {
        self.update_selection(shift);
        if self.cursor_x < self.lines[self.cursor_y].len() {
            self.cursor_x += 1;
        } else if self.cursor_y < self.lines.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.target_x = self.cursor_x;
    }
}
