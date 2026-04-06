use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.current_editor() {
        Some(editor) => {
            let lang = if let Some(path) = &editor.filepath {
                if path.extension().and_then(|e| e.to_str()) == Some("cpp") {
                    "C++ (cpp)"
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    "Rust (rs)"
                } else {
                    "Plain Text"
                }
            } else {
                "Plain Text"
            };

            format!(
                " {} | Lines: {} | Col: {} | Tab Size: {} ",
                lang,
                editor.lines.len(),
                editor.cursor_x + 1,
                app.config.tab_size
            )
        }
        None => format!(" No files open | Tab Size: {} ", app.config.tab_size),
    };

    let p = Paragraph::new(text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(p, area);
}
