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
                .bg({
                    let hex = app.config.get("Active Tab BG Color")
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim_start_matches('#'))
                        .filter(|s| s.len() == 6)
                        .unwrap_or("2E7D32");
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                    Color::Rgb(r, g, b)
                })
                .fg({
                    let hex = app.config.get("Active Tab FG Color")
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim_start_matches('#'))
                        .filter(|s| s.len() == 6)
                        .unwrap_or("FFFFFF");
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                    Color::Rgb(r, g, b)
                })
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg({
                    let hex = app.config.get("Tab BG Color")
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim_start_matches('#'))
                        .filter(|s| s.len() == 6)
                        .unwrap_or("A9A9A9");
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                    Color::Rgb(r, g, b)
                })
                .fg({
                    let hex = app.config.get("Tab FG Color")
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim_start_matches('#'))
                        .filter(|s| s.len() == 6)
                        .unwrap_or("FFFFFF");
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
                    Color::Rgb(r, g, b)
                })
        };

        spans.push(Span::styled(format!(" {} ", name), style));
        spans.push(Span::raw(" "));
    }

    let p = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));
    f.render_widget(p, area);
}
