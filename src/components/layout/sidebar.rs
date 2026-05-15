use crate::{app::App, input::action::SidebarCategory};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use compact_str::CompactString;

pub fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ws) = &app.workspace {
        if !ws.visible {
            return;
        }

        // Divide area into Activity Bar (icons) and Content
        let sidebar_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4), // Activity bar
                Constraint::Min(0),    // Sidebar content
            ])
            .split(area);

        draw_activity_bar(f, app, sidebar_layout[0]);
        draw_sidebar_content(f, app, sidebar_layout[1]);
    }
}

fn draw_activity_bar(f: &mut Frame, app: &App, area: Rect) {
    let background_style = Style::default().bg(Color::Rgb(30, 30, 30));
    f.render_widget(Block::default().style(background_style), area);

    let categories = [
        (SidebarCategory::FileTree, "  "),    // File icon
        (SidebarCategory::Search, "  "),      // Search icon
        (SidebarCategory::Git, "  "),         // Git icon
        (SidebarCategory::Settings, "  "),    // Gear icon
        (SidebarCategory::MarketPlace, "  "), // Market stall icon
    ];

    let mut lines = vec![];
    for (cat, icon) in categories.iter() {
        let style = if app.sidebar_category == *cat {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        lines.push(Line::from(Span::styled(*icon, style)));
        lines.push(Line::from("")); // Spacer
    }

    let p = Paragraph::new(lines).block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(Color::Rgb(50, 50, 50))));
    f.render_widget(p, area);
}

fn draw_sidebar_content(f: &mut Frame, app: &App, area: Rect) {
    let background_style = Style::default().bg(Color::Rgb(35, 35, 35));
    f.render_widget(Block::default().style(background_style), area);

    match app.sidebar_category {
        SidebarCategory::FileTree => draw_file_tree(f, app, area),
        SidebarCategory::Search => draw_search_view(f, app, area),
        SidebarCategory::Git => draw_git_view(f, app, area),
        SidebarCategory::Settings => draw_settings_view(f, app, area),
        SidebarCategory::MarketPlace => draw_marketplace_view(f, app, area),
    }
}

fn draw_file_tree(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ws) = &app.workspace {
        let active_style = if ws.focused {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD)
        };

        let block = Block::default()
            .title(" Explorer ")
            .borders(Borders::ALL)
            .border_style(active_style);

        let flat = ws.flatten();
        let mut lines = vec![];

        for (i, &(node_idx, depth)) in flat.iter().enumerate() {
            let node = &ws.nodes[node_idx];
            let indent = " ".repeat(depth * 2);

            let icon = if node.is_dir {
                if node.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                " " // no icon for generic file
            };

            let style = if ws.focused && ws.selected == i {
                Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!(
                "{}{}{}",
                indent,
                icon,
                node.path.file_name().unwrap_or_default().to_string_lossy()
            );

            lines.push(Line::from(Span::styled(text, style)));
        }

        let p = Paragraph::new(lines).block(block);
        f.render_widget(p, area);

        if ws.focused {
            f.set_cursor_position((area.x + 1, area.y + 1 + ws.selected as u16));
        }
    }
}

fn draw_search_view(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.workspace.as_ref().map_or(false, |w| w.focused);
    let active_style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    };

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(active_style);

    let inner_area = block.inner(area);
    let search_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header (Query + Metrics)
            Constraint::Min(0),    // Results
        ])
        .split(inner_area);

    f.render_widget(block, area);

    // Search input display
    let query_line = Line::from(vec![
        Span::styled(" Query: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.search_query
            .char_indices()
            .rev()
            .skip(
                app.search_query
                .len()
                .saturating_sub(app.cursor_pos)
            )
            .nth(26)
            .map(|(i, _)| &app.search_query[i..])
            .unwrap_or(&app.search_query),
            Style::default().fg(Color::White)
        ),
    ]);
    let metrics_line = Line::from(vec![
        Span::styled(format!(" Hits: {} in {} files ", app.search_num_occurrences, app.search_num_files), Style::default().fg(Color::Green).bg(Color::Rgb(40, 40, 40))),
    ]);

    let query_p = Paragraph::new(vec![
        query_line,
        metrics_line,
        Line::from(Span::styled("─".repeat(search_layout[0].width as usize), Style::default().fg(Color::Rgb(50, 50, 50))))
    ]);
    f.render_widget(query_p, search_layout[0]);


    f.set_cursor_position((app.cursor_pos.clamp(0, 27) as u16 + 13, 1));

    if app.search_results.is_empty() {
        if !app.search_query.is_empty() {
            f.render_widget(Paragraph::new(" No results found."), search_layout[1]);
        } else {
            f.render_widget(Paragraph::new(" Type to search..."), search_layout[1]);
        }
        return;
    }

    let mut lines = vec![];
    for (i, result) in app.search_results.iter().enumerate() {
        let style = if is_focused && app.search_selected == i {
            Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let filename = std::path::Path::new(&result.filepath)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();

        lines.push(Line::from(Span::styled(
            format!(" {}:{} ", filename, result.line_number),
            style.clone().fg(Color::Cyan)
        )));
        lines.push(Line::from(Span::styled(
            format!("   {}", result.content),
            style
        )));
    }

    let height = search_layout[1].height as usize;
    let scroll_y = if app.search_selected * 2 >= height {
        (app.search_selected * 2) - height + 2
    } else {
        0
    };

    // Paragraph::scroll takes (vertical, horizontal)
    let p = Paragraph::new(lines).scroll((scroll_y as u16, 0));
    f.render_widget(p, search_layout[1]);
}

fn draw_git_view(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.workspace.as_ref().map_or(false, |w| w.focused);
    let active_style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    };

    let block = Block::default()
        .title(" Git Status ")
        .borders(Borders::ALL)
        .border_style(active_style);

    if app.git_changes.is_empty() {
        f.render_widget(Paragraph::new(" No changes detected.").block(block), area);
        return;
    }

    let mut lines = vec![];
    for (i, change) in app.git_changes.iter().enumerate() {
        let is_selected = is_focused && app.git_selected == i;
        let style = if is_selected {
            Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let status_style = match change.status.as_str() {
            "M" => Style::default().fg(Color::Yellow),
            "A" => Style::default().fg(Color::Green),
            "D" => Style::default().fg(Color::Red),
            "??" => Style::default().fg(Color::Magenta),
            _ => Style::default().fg(Color::Gray),
        };

        // Flatten and shorten path
        let path = &change.path;
        let components: Vec<&str> = path.split('/').collect();
        let display_path = if components.len() > 3 {
            CompactString::from(format!("{}/.../{}/{}", components[0], components[components.len()-2], components[components.len()-1]))
        } else {
            path.clone()
        };

        lines.push(Line::from(vec![
            Span::styled(format!(" {} ", change.status), status_style.add_modifier(Modifier::BOLD)),
            Span::styled(display_path, style.add_modifier(Modifier::BOLD)),
        ]));
    }

    let height = area.height as usize;
    let scroll_y = if app.git_selected >= height / 2 {
        app.git_selected - height / 2
    } else {
        0
    };

    let p = Paragraph::new(lines).block(block).scroll((scroll_y as u16, 0));
    f.render_widget(p, area);

      // Top left of box
      f.set_cursor_position((area.x + 1, area.y + 1));
}

//TODO: Make it so that we do less throwing away work, such as the flattened settings
fn draw_settings_view(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.workspace.as_ref().map_or(false, |w| w.focused);
    let active_style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    };

    struct Setting {
        title: String,
        value: String,
    }

    let mut settings: Vec<Setting> = Vec::new();
    for (key, value) in &app.config {
        settings.push(Setting {
            title: key.clone(),
            value: value.to_string(),
        });
    }

    let settings_block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(active_style);

    let inner = settings_block.inner(area);
    f.render_widget(settings_block, area);

    // Calculate how many items can actually fit in the inner area
    // Each setting is 3 lines high
    let visible_count = (inner.height / 3) as usize;

    // Determine the window of items to show based on scroll
    if visible_count == 0 { return; }
    let mut start_index = app.settings_scroll;

    // Ensure selected item is not above the view
    if app.settings_selected < start_index {
        start_index = app.settings_selected;
    } else if app.settings_selected >= start_index + visible_count {
        start_index = app.settings_selected - visible_count + 1;
    }

    let end_index = (start_index + visible_count).min(settings.len());

    // Create chunks ONLY for the visible items
    let mut constraints = vec![Constraint::Length(3); visible_count];
    constraints.push(Constraint::Min(0));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

        // Render only the visible settings
        for (i, setting_idx) in (start_index..end_index).enumerate() {
        let setting = &settings[setting_idx];
        let is_selected = is_focused && app.settings_selected == setting_idx;

        let style = if is_selected {
            Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let block = Block::default()
            .title(format!(" {} ", setting.title))
            .borders(Borders::ALL)
            .border_style(style);

        f.render_widget(Paragraph::new(setting.value.clone()).block(block), chunks[i]);

        if is_selected {
            f.set_cursor_position((chunks[i].x + 1, chunks[i].y + 1));
        }
    }
}

fn draw_marketplace_view(f: &mut Frame, app: &App, area: Rect) {
    // TODO: Implement X scrolling when hovering on a plugin with a description
    //       longer than current view
    let is_focused =
        app.workspace.as_ref()
        .map_or(false, |w| w.focused);
    let active_style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    };

    let block = Block::default()
        .title(" Marketplace ")
        .borders(Borders::ALL)
        .border_style(active_style);

    let inner_area = block.inner(area);
    let marketplace_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Header (ID + Title + description)
            Constraint::Min(0),    // Results
        ])
        .split(inner_area);

    f.render_widget(block, area);

    // Search input display
    let search_line = Line::from(vec![
        Span::styled(" Search: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.market_search_query
            .char_indices()
            .rev()
            .skip(
                app.market_search_query
                .len()
                .saturating_sub(app.cursor_pos)
            )
            .nth(24)
            .map(|(i, _)| &app.market_search_query[i..])
            .unwrap_or(&app.market_search_query),
            Style::default().fg(Color::White)
        ),
    ]);


    // if nothing in query, use a gray "type to search..."
    let search_query = Paragraph::new(vec![
        search_line,
        Line::from(Span::styled("─".repeat(marketplace_layout[0].width as usize), Style::default().fg(Color::Rgb(50, 50, 50))))
    ]);
    f.render_widget(search_query, marketplace_layout[0]);

    f.render_widget(Paragraph::new(
        if app.online {
            if app.marketplace_listed_items.is_empty() {
                if !app.market_search_query.is_empty() {
                    String::from(" No results found.")
                } else {
                  String::from(" Loading plugins...")
                }
            } else {
                String::from(" Here are some hot plugins:")
            }
        } else {
          app.marketplace_error.clone().unwrap_or(String::from(" Trying to connect..."))
        }
    ), marketplace_layout[1]);

    f.set_cursor_position((app.cursor_pos.clamp(0, 25) as u16 + 14, 1));

    let mut lines = vec![];
    for (i, result) in app.marketplace_listed_items.iter().enumerate() {
        let style = if is_focused && app.marketplace_item_selected == i {
            Style::default().bg(Color::Rgb(60, 60, 60)).fg(Color::White)
        } else {
            Style::default().fg(Color::Gray)
        };

        let title = result.title
            .split_once(' ')
            .map(|(_, rest)| rest)
            .unwrap_or(&result.title);

        lines.push(Line::from(Span::styled(
            format!(" {} ", title),
            style.clone().fg(Color::Cyan)
        )));
        lines.push(Line::from(Span::styled(
            format!(" {} ", result.desc.chars().take(5).collect::<String>()),
            style
        )));
    }

    let height = marketplace_layout[1].height as usize;
    let scroll_y = if app.marketplace_item_selected * 2 >= height {
        (app.marketplace_item_selected * 2) - height + 2
    } else {
        0
    };

    // Paragraph::scroll takes (vertical, horizontal)
    let p = Paragraph::new(lines).scroll((scroll_y as u16, 0));
    f.render_widget(p, marketplace_layout[1]);
}
