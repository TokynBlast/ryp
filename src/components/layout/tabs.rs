use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let mut tab_names = vec![];

    // First pass: collect basic names
    for editor in &app.editors {
        let name = if let Some(path) = &editor.filepath {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            "Untitled".to_string()
        };
        tab_names.push(name);
    }

    // Second pass: resolve duplicates by adding parent dir
    let mut final_names = vec![];
    for (i, name) in tab_names.iter().enumerate() {
        let has_duplicate = tab_names
            .iter()
            .enumerate()
            .any(|(j, n)| i != j && n == name);
        let mut display_name = name.clone();

        if has_duplicate && name != "Untitled" {
            if let Some(path) = &app.editors[i].filepath {
                if let Some(parent) = path.parent() {
                    if let Some(parent_name) = parent.file_name() {
                        display_name = format!("{}/{}", parent_name.to_string_lossy(), name);
                    }
                }
            }
        }

        if app.editors[i].dirty {
            display_name.push('*');
        }
        final_names.push(display_name);
    }

    let mut spans = vec![];
    for (i, name) in final_names.iter().enumerate() {
        let style = if i == app.active_tab {
            Style::default()
                .bg(Color::Green)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        };

        spans.push(Span::styled(format!(" {} ", name), style));
        spans.push(Span::raw(" "));
    }

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    f.render_widget(p, area);
}
