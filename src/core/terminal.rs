use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

#[derive(Clone)]
pub struct TermCell {
  pub c: char,
  // TODO: style info
}

pub struct TerminalGrid {
  pub cells: Vec<Vec<TermCell>>,  // [row][col]
  pub cursor_row: usize,
  pub cursor_col: usize,
  pub rows: usize,
  pub cols: usize,
  pub scrollback: Vec<Vec<TermCell>>,
}

enum ParseState {
  Normal,
  Esc,        // got \x1b
  Csi,        // got \x1b[, accumulating params
  Osc,
}

pub struct Terminal {
  pub tx: mpsc::Sender<Vec<u8>>,
  rx: mpsc::Receiver<Vec<u8>>,
  pub grid: TerminalGrid,
  parse_state: ParseState,
  pub csi_params: String,
}

impl Terminal {
    pub fn new(cwd: std::path::PathBuf) -> Self {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("Failed to open PTY");

        #[cfg(windows)]
        let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());

        #[cfg(not(windows))]
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(cwd);
        let _child = pair
            .slave
            .spawn_command(cmd)
            .expect("Failed to spawn shell");

        let mut reader = pair
            .master
            .try_clone_reader()
            .expect("Failed to clone PTY reader");
        let mut writer = pair.master.take_writer().expect("Failed to get PTY writer");

        let (tx_in, rx_in) = mpsc::channel::<Vec<u8>>();
        let (tx_out, rx_out) = mpsc::channel::<Vec<u8>>();

        // Read thread (PTY -> App)
        thread::spawn(move || {
            // help prevent fragmentation
            #[cfg(windows)]
            let mut buf = [0u8; 4096];
            #[cfg(not(windows))]
            let mut buf = [0u8; 1024];

            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx_out.send(buf[..n].to_vec()).is_err() {
                            break; // receiver dropped
                        }
                        #[cfg(windows)]
                        std::thread::sleep(std::time::Duration::from_millis(18));
                    }
                    Err(_) => break,
                }
            }
        });

        // Write thread (App -> PTY)
        thread::spawn(move || {
            while let Ok(data) = rx_in.recv() {
                let _ = writer.write_all(&data);
                let _ = writer.flush();
            }
        });

        Self {
            tx: tx_in,
            rx: rx_out,
            grid: TerminalGrid {
                cells: vec![vec![TermCell { c: ' ' }; 80]; 24],
                cursor_row: 0,
                cursor_col: 0,
                rows: 24,
                cols: 80,
                scrollback: Vec::new(),
            },
            parse_state: ParseState::Normal,
            csi_params: String::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let mut any_new_data = false;
        while let Ok(data) = self.rx.try_recv() {
            any_new_data = true;

            // Handle some ANSI codes manually before stripping the rest
            let s = String::from_utf8_lossy(&data);

            for c in s.chars() {
                match self.parse_state {
                  ParseState::Normal => match c {
                      '\x1b' => self.parse_state = ParseState::Esc,
                      '\r' => self.grid.cursor_col = 0,
                      '\n' => {
                          self.grid.cursor_row += 1;
                          if self.grid.cursor_row >= self.grid.rows {
                              // push top row to scrollback, shift grid up
                              let top = self.grid.cells.remove(0);
                              self.grid.scrollback.push(top);
                              self.grid.cells.push(vec![TermCell { c: ' ' }; self.grid.cols]);
                              self.grid.cursor_row = self.grid.rows - 1;
                          }
                      }
                      '\x08' => {
                          if self.grid.cursor_col > 0 {
                              self.grid.cursor_col -= 1;
                          }
                      }
                      c => {
                          self.grid.cells[self.grid.cursor_row][self.grid.cursor_col].c = c;
                          self.grid.cursor_col += 1;
                          if self.grid.cursor_col >= self.grid.cols {
                              self.grid.cursor_col = 0;
                              self.grid.cursor_row += 1;
                          }
                      }
                  },
                  ParseState::Osc => {
                      if c == '\x07' {
                          self.parse_state = ParseState::Normal;
                      }
                      // ignore everything else
                  }
                  ParseState::Esc => match c {
                      '[' => { self.parse_state = ParseState::Csi; self.csi_params.clear(); }
                      ']' => { self.parse_state = ParseState::Osc; self.csi_params.clear(); } // add this
                      _ => self.parse_state = ParseState::Normal,
                  },
                  ParseState::Csi => {
                    let n = self.csi_params.parse::<usize>().unwrap_or(1);

                    // collect modifying digits
                    if c.is_ascii_digit() || c == ';' {
                        self.csi_params.push(c);
                        continue;
                    }

                    match c {
                      // up, clamp to greater than 0
                      'A' => self.grid.cursor_row = self.grid.cursor_row.saturating_sub(n),
                      // left, clamp to greater than 0
                      'D' => self.grid.cursor_col = self.grid.cursor_col.saturating_sub(n),
                      // down, clamp to greater than rows - 1
                      'B' => self.grid.cursor_row = (self.grid.cursor_row + n).min(self.grid.rows - 1),
                      // right, clamp to greater than cols - 1
                      'C' => self.grid.cursor_col = (self.grid.cursor_col + n).min(self.grid.cols - 1),
                      'J' => {
                          for row in &mut self.grid.cells {
                              for cell in row.iter_mut() {
                                  cell.c = ' ';
                              }
                          }
                          self.grid.cursor_row = 0;
                          self.grid.cursor_col = 0;
                      },
                      'K' => {
                          let row = &mut self.grid.cells[self.grid.cursor_row];
                          for cell in row[self.grid.cursor_col..].iter_mut() {
                              cell.c = ' ';
                          }
                      },
                      'H' => {
                          let mut parts = self.csi_params.split(';');
                          let row = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
                          let col = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1).saturating_sub(1);
                          self.grid.cursor_row = row.min(self.grid.rows - 1);
                          self.grid.cursor_col = col.min(self.grid.cols - 1);
                      }
                      'h' | 'l' => { /* ignore mode set/reset */ }
                      _ => {},
                    }

                    self.parse_state = ParseState::Normal;
                  },
              }
            }
        }

        return any_new_data;
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        let mut data = Vec::new();
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // ASCII control codes (A=1, B=2, etc)
                    if c == 'c' || c == 'C' {
                        data.push(3); // SIGINT
                    } else if c == 'l' || c == 'L' {
                        data.push(12); // Form Feed / Control-L
                    } else {
                        let b = (c.to_ascii_uppercase() as u8).saturating_sub(64);
                        data.push(b);
                    }
                } else {
                    data.extend_from_slice(c.to_string().as_bytes());
                }
            }
            KeyCode::Enter => data.push(b'\r'),
            KeyCode::Backspace => data.push(127), // DEL / Backspace
            KeyCode::Tab => data.push(9),
            KeyCode::Esc => data.push(27),
            KeyCode::Left => data.extend_from_slice(b"\x1b[D"),
            KeyCode::Right => data.extend_from_slice(b"\x1b[C"),
            KeyCode::Up => data.extend_from_slice(b"\x1b[A"),
            KeyCode::Down => data.extend_from_slice(b"\x1b[B"),
            _ => {}
        }
        if !data.is_empty() {
            let _ = self.tx.send(data);
        }
    }
}
