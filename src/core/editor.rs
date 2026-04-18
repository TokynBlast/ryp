use std::fs;
use std::path::PathBuf;
use std::path::Path;


pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub target_x: usize,
    pub scroll_y: crossbeam::atomic::AtomicCell<usize>,
    pub scroll_x: crossbeam::atomic::AtomicCell<usize>,
    pub selection_start: Option<(usize, usize)>, // (start_x, start_y)
    pub filepath: Option<PathBuf>,
    pub dirty: bool,
    pub is_diff: bool,
    pub highlight_cache: Vec<Vec<(syntect::highlighting::Style, String)>>,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_x: 0,
            cursor_y: 0,
            target_x: 0,
            scroll_y: crossbeam::atomic::AtomicCell::new(0),
            scroll_x: crossbeam::atomic::AtomicCell::new(0),
            selection_start: None,
            filepath: None,
            dirty: false,
            is_diff: false,
            highlight_cache: vec![],
        }
    }

    pub fn rebuild_highlight_cache(&mut self, syntax_set: &syntect::parsing::SyntaxSet, theme: &syntect::highlighting::Theme) {
      let ext = self.filepath.as_ref()
          .and_then(|p| p.extension())
          .and_then(|e| e.to_str())
          .unwrap_or("txt");
      let syntax = syntax_set.find_syntax_by_extension(ext)
          .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
      let mut h = syntect::easy::HighlightLines::new(syntax, theme);

      self.highlight_cache = self.lines.iter().map(|line| {
          let line_with_nl = format!("{}\n", line);
          h.highlight_line(&line_with_nl, syntax_set)
              .unwrap_or_default()
              .into_iter()
              .map(|(s, t)| (s, t.trim_end_matches('\n').to_string()))
              .collect()
      }).collect();
    }

    pub fn load_file(&mut self, path: &Path) -> bool {
        if let Ok(content) = fs::read_to_string(path) {
            self.lines = content.lines().map(|s| s.to_string()).collect();
            if self.lines.is_empty() {
                self.lines.push(String::new());
            }
            self.filepath = Some(path.to_path_buf());
            self.dirty = false;
            self.is_diff = false;
            true
        } else {
            false
        }
    }

    pub fn load_diff(&mut self, path: &Path, content: Vec<String>) {
        self.lines = content;
        if self.lines.is_empty() {
            self.lines.push(String::new());
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
                let mut new_start = self.lines[sy][..bs].to_string();

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
