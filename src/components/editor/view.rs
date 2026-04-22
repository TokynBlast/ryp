use crate::app::App;
use crate::windows::modal::ModalType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

pub fn draw_editor(f: &mut Frame, app: &App, area: Rect) {
    let editor = match app.current_editor() {
        Some(e) => e,
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(50, 50, 50)));
            f.render_widget(block.clone(), area);

            let msg = vec![
                Line::from(Span::styled("No files open", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Open file: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Click in Sidebar", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled(" Global Search: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Ctrl + 2", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled(" Close / Quit: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Ctrl + W", Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled(" Help: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Ctrl + K", Style::default().fg(Color::Cyan)),
                ]),
            ];

            let p = Paragraph::new(msg)
                .alignment(ratatui::layout::Alignment::Center)
                .block(block);

            let centered_area = centered_rect(60, 20, area);
            f.render_widget(p, centered_area);
            return;
        }
    };



    let height = area.height as usize;
    let margin = (height / 3).max(1);

    // Retrieve and update scroll_y using Cell to keep it across frames
    let current_scroll = editor.scroll_y.load(std::sync::atomic::Ordering::Relaxed);
    let mut scroll_y = current_scroll;

    let ext = editor
        .filepath
        .as_ref()
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("txt");

    let syntax = app
        .syntax_set
        .find_syntax_by_extension(ext)
        .unwrap_or_else(|| app.syntax_set.find_syntax_plain_text());

    let theme_name = app.config.get("theme")
        .and_then(|v| v.get("Highlighting Theme"))
        .and_then(|v| v.as_str())
        .unwrap_or("base16-ocean.dark");

    let theme = &app.theme_set.themes[theme_name];
    let mut h = syntect::easy::HighlightLines::new(syntax, theme);

    if editor.cursor_y < height {
        scroll_y = 0;
    } else {
        let desired_min_scroll = editor.cursor_y.saturating_sub(height).saturating_add(margin).saturating_add(1);
        let desired_max_scroll = editor.cursor_y.saturating_sub(margin);

        if scroll_y < desired_min_scroll {
            scroll_y = desired_min_scroll;
        } else if scroll_y > desired_max_scroll {
            scroll_y = desired_max_scroll;
        }
    }

    // Clamp to make sure we don't scroll past the content
    scroll_y = scroll_y.min(editor.lines.len().saturating_sub(1));
    editor.scroll_y.store(scroll_y, std::sync::atomic::Ordering::Relaxed);

    let mut lines = vec![];
    for line in editor.lines.iter().take(scroll_y) {
        let line_with_nl = format!("{}\n", line);
        let _ = h.highlight_line(&line_with_nl, &app.syntax_set);
    }

    let search_term = if let Some(modal) = &app.modal {
        if modal.modal_type == ModalType::Search || modal.modal_type == ModalType::Replace {
            Some(modal.input.clone())
        } else {
            None
        }
    } else {
        None
    };

    for (i, line) in editor.lines.iter().skip(scroll_y).take(height).enumerate() {
        let line_num = scroll_y + i + 1;
        let mut bg_color = if line_num % 2 == 0 {
            Color::Rgb(30, 30, 30)
        } else {
            Color::Rgb(40, 40, 40)
        };

        if editor.is_diff {
            if line.starts_with('+') {
                bg_color = Color::Rgb(20, 50, 20);
            } else if line.starts_with('-') {
                bg_color = Color::Rgb(50, 20, 20);
            }
        }

        let num_span = Span::styled(
            format!("{:4} | ", line_num),
            Style::default().fg(Color::Gray).bg(bg_color),
        );

        let mut spans = vec![num_span];

        let mut search_matches = vec![];
        if let Some(ref st) = search_term {
            if !st.is_empty() {
                for (byte_idx, _) in line.match_indices(st.as_str()) {
                    let c_start = line[0..byte_idx].chars().count();
                    for c_off in 0..st.chars().count() {
                        search_matches.push(c_start + c_off);
                    }
                }
            }
        }

        let line_with_nl = format!("{}\n", line);
        if let Ok(ranges) = h.highlight_line(&line_with_nl, &app.syntax_set) {
            let mut char_idx = 0;
            for (style, text) in ranges {
                let style = &style;
                let text = text.trim_end_matches('\n');
                if text.is_empty() {
                    continue;
                }

                let mut fg = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);

                if editor.is_diff && line.starts_with("@@") {
                    fg = Color::Cyan;
                }

                for c in text.chars() {
                    let mut b_bg = bg_color;
                    let mut b_fg = fg;
                    let mut modifier = Modifier::empty();

                    if editor.is_selected(char_idx, scroll_y + i) {
                        b_bg = Color::LightBlue;
                        b_fg = Color::Black;
                    } else if search_matches.contains(&char_idx) {
                        b_bg = Color::LightCyan;
                        b_fg = Color::Black;
                        modifier = Modifier::BOLD;
                    }

                    spans.push(Span::styled(
                        c.to_string(),
                        Style::default().fg(b_fg).bg(b_bg).add_modifier(modifier),
                    ));
                    char_idx += 1;
                }
            }
        }

        let text_len = spans
            .iter()
            .map(|s| s.content.chars().count())
            .sum::<usize>();
        if text_len < area.width as usize {
            let padding = " ".repeat(area.width as usize - text_len);
            spans.push(Span::styled(padding, Style::default().bg(bg_color)));
        }

        lines.push(Line::from(spans));
    }

    let p = Paragraph::new(Text::from(lines));
    f.render_widget(p, area);

    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"));

    let mut scrollbar_state = ScrollbarState::default()
        .content_length(editor.lines.len().saturating_sub(height))
        .position(scroll_y);

    f.render_stateful_widget(
        scrollbar,
        area.inner(Margin {
            vertical: 0,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );

    let is_tree_focused = app.workspace.as_ref().map_or(false, |w| w.focused);

    if !is_tree_focused {
        let cursor_x_visual = editor.cursor_x as u16 + 7 + area.x;
        let cursor_y_visual = (editor.cursor_y - scroll_y) as u16 + area.y;
        f.set_cursor_position((cursor_x_visual, cursor_y_visual));
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
