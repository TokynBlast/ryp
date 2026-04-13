use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    // 1. Separate Status Bar from Content
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main content area
            Constraint::Length(1), // Status Bar
        ])
        .split(size);

    let content_area = outer_layout[0];
    let status_area = outer_layout[1];

    // 2. Separate Sidebar and Editor Column
    let sidebar_visible = app.workspace.as_ref().map_or(false, |ws| ws.visible);
    let main_layout = if sidebar_visible {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Sidebar
                Constraint::Percentage(80), // Tabs + Editor Column
            ])
            .split(content_area)
    } else {
        // If sidebar is hidden, editor column takes full width
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0)])
            .split(content_area)
    };

    let sidebar_area = if sidebar_visible {
        Some(main_layout[0])
    } else {
        None
    };
    let editor_column_area = if sidebar_visible {
        main_layout[1]
    } else {
        main_layout[0]
    };

    if let Some(area) = sidebar_area {
        crate::components::layout::sidebar::draw_sidebar(f, app, area);
    }

    // 3. Separate Tabs and Editor within the column
    let editor_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs
            Constraint::Min(1),    // Editor
        ])
        .split(editor_column_area);

    crate::components::layout::tabs::draw_tabs(f, app, editor_layout[0]);
    crate::components::editor::view::draw_editor(f, app, editor_layout[1]);

    crate::components::layout::status::draw_status_bar(f, app, status_area);

    // Modals
    if app.modal.is_some() {
        crate::components::modals::view::draw_modal(f, app, size);
    }

    if app.terminal_visible {
        draw_terminal(f, app, size);
    }
}

fn draw_terminal(f: &mut ratatui::Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let vertical_margin = area.height.saturating_sub(20) / 2;
    let horizontal_margin = area.width.saturating_sub(80) / 2;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin.saturating_sub(3)),
            Constraint::Min(0),
            Constraint::Length(vertical_margin.saturating_sub(2)),
        ])
        .split(area);

    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(horizontal_margin),
            Constraint::Min(0),
            Constraint::Length(horizontal_margin),
        ])
        .split(popup_layout[1])[1];

    let block = Block::default()
        .title(" Terminal (CTRL+T / F5 / ESC to Hide) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // show scrollback + grid rows as lines
    let mut content: Vec<ratatui::text::Line> = Vec::new();


    // then the active grid
    for (r, row) in app.terminal.grid.cells.iter().enumerate() {
        let line: String = row.iter().map(|c| c.c).collect();

        if r == app.terminal.grid.cursor_row {
            // split at cursor position and insert cursor
            let before = line.chars().take(app.terminal.grid.cursor_col).collect::<String>();
            let after = line.chars().skip(app.terminal.grid.cursor_col + 1).collect::<String>();
            let spans = vec![
                ratatui::text::Span::raw(before),
                ratatui::text::Span::styled("_", Style::default().bg(Color::White).fg(Color::Black)),
                ratatui::text::Span::raw(after),
            ];
            content.push(ratatui::text::Line::from(spans));
        } else {
            content.push(ratatui::text::Line::from(line));
        }
    }

    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(content).block(block), area);
}
