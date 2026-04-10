use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

pub struct Terminal {
    pub output_lines: Vec<String>,
    pub tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    pub current_line: String,
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
            let mut buf = [0u8; 1024];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                let _ = tx_out.send(buf[..n].to_vec());
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
            output_lines: Vec::new(),
            tx: tx_in,
            rx: rx_out,
            current_line: String::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let mut any_new_data = false;
        while let Ok(data) = self.rx.try_recv() {
            any_new_data = true;

            // Handle some ANSI codes manually before stripping the rest
            let s = String::from_utf8_lossy(&data);

            // Clear screen: Esc[2J or similar
            if s.contains("\x1b[2J") || s.contains("\x1b[H") || s.contains("\x1b[J") {
                self.output_lines.clear();
                self.current_line.clear();
            }

            // Handle \r before stripping (strip_ansi removes it)
            //let data = s.replace('\r', "\x00CARRIAGE\x00");

            let stripped = strip_ansi_escapes::strip(&data);
            let s_clean = String::from_utf8_lossy(&stripped);

            for c in s_clean.chars() {
                match c {
                    '\n' => { self.output_lines.push(self.current_line.clone()); self.current_line.clear(); }
                    '\r' => { self.current_line.clear(); }
                    '\x08' | '\x7f' => { self.current_line.pop(); }
                    '\x1b' | '\x00' => {} // ignore leftover escape chars
                    c => { self.current_line.push(c); }
                }
            }
        }

        if any_new_data && self.output_lines.len() > 1000 {
            self.output_lines.drain(0..self.output_lines.len() - 1000);
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
