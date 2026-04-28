use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Frame,
};

// May be useful in the future:
// 🖸🖴🖵🖱🖰🖲🖶🖻🖺🖮🖫🖪🕯
pub fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.current_editor() {
        Some(editor) => {
            format!(
                " {} | Lines: {} | Col: {} | Tab Size: {} | OS: {} ",
                &editor.lang,
                editor.lines.len(),
                editor.cursor_x + 1,
                app.config.get("Tab Size").and_then(|v| v.as_u64()).unwrap_or(4),
                app.os,
            )
        }
        None => format!(" No files open | Tab Size: {} | OS: {} ", app.config.get("Tab Size").and_then(|v| v.as_u64()).unwrap_or(4), app.os),
    };

    let p = Paragraph::new(text).style(Style::default().bg(Color::Blue).fg(Color::White));
    f.render_widget(p, area);
}
