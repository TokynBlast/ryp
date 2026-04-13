use crate::app::App;
use crate::windows::modal::ModalType;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn draw_modal(f: &mut Frame, app: &App, area: Rect) {
    let modal = app.modal.as_ref().unwrap();
    let modal_layout = modal.modal_type.layout();

    let modal_area = match modal_layout {
        crate::windows::modal::ModalLayout::Island => {
            let layout_y = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(area.height.saturating_sub(7)),
                    Constraint::Length(5),
                    Constraint::Min(0),
                ])
                .split(area)[1];
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(layout_y)[1]
        }
        crate::windows::modal::ModalLayout::Popup => {
            let layout_y = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(area)[1];
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(layout_y)[1]
        }
    };

    f.render_widget(Clear, modal_area);

    match modal.modal_type {
        ModalType::Search => {
            let block = Block::default()
                .title(" Search (F2) ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(50, 50, 50)));
            let p = Paragraph::new(modal.input.clone())
                .block(block)
                .alignment(modal.modal_type.alignment());
            f.render_widget(p, modal_area);

            f.set_cursor_position((
                modal_area.x + 1 + modal.input.len() as u16,
                modal_area.y + 1,
            ));
        }
        ModalType::Replace => {
            let block = Block::default()
                .title(" Replace (F3) ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(50, 50, 50)));
            let text = vec![
                Line::from(format!("Find: {}", modal.input)),
                Line::from(format!("Replace: {}", modal.replace_input)),
            ];
            let p = Paragraph::new(text)
                .block(block)
                .alignment(modal.modal_type.alignment());
            f.render_widget(p, modal_area);

            if modal.focus_replace {
                f.set_cursor_position((
                    modal_area.x + 10 + modal.replace_input.len() as u16,
                    modal_area.y + 2,
                ));
            } else {
                f.set_cursor_position((
                    modal_area.x + 7 + modal.input.len() as u16,
                    modal_area.y + 1,
                ));
            }
        }
        ModalType::QuitPrompt => {
            let block = Block::default()
                .title(" Unsaved Changes ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(50, 50, 50)));
            let text = vec![
                Line::from(" You have unsaved changes. "),
                Line::from(" Choose an action: "),
                Line::from(""),
                Line::from(vec![Span::styled(
                    if modal.active_button == 0 {
                        " > Discard "
                    } else {
                        "   Discard "
                    },
                    if modal.active_button == 0 {
                        Style::default()
                            .bg(Color::Red)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                )]),
                Line::from(vec![Span::styled(
                    if modal.active_button == 1 {
                        " > Cancel "
                    } else {
                        "   Cancel "
                    },
                    if modal.active_button == 1 {
                        Style::default()
                            .bg(Color::Cyan)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                )]),
                Line::from(vec![Span::styled(
                    if modal.active_button == 2 {
                        " > Save "
                    } else {
                        "   Save "
                    },
                    if modal.active_button == 2 {
                        Style::default()
                            .bg(Color::Green)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                )]),
            ];

            let p = Paragraph::new(text)
                .block(block)
                .alignment(modal.modal_type.alignment());
            f.render_widget(p, modal_area);
        }
        ModalType::ConfirmExit => {
            let block = Block::default()
                .title(" Confirm Exit ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(50, 50, 50)));

            let text = vec![
                Line::from(" You have multiple tabs open. "),
                Line::from(" Are you sure you want to exit? "),
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        if modal.active_button == 0 {
                            " > NO "
                        } else {
                            "   NO "
                        },
                        if modal.active_button == 0 {
                            Style::default()
                                .bg(Color::Gray)
                                .fg(Color::Black)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                    Span::raw("    "),
                    Span::styled(
                        if modal.active_button == 1 {
                            " > YES "
                        } else {
                            "   YES "
                        },
                        if modal.active_button == 1 {
                            Style::default()
                                .bg(Color::Red)
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                ]),
            ];

            let p = Paragraph::new(text)
                .block(block)
                .alignment(modal.modal_type.alignment());
            f.render_widget(p, modal_area);
        }
        ModalType::Help => {
            let block = Block::default()
                .title(" Help Binds ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(50, 50, 50)));

            let text = vec![
                Line::from(" [ NAVIGATION ] "),
                Line::from(" Arrows          : Move Cursor "),
                Line::from(" Shift+Arrow     : Visual Select "),
                Line::from(" PgUp / PgDn     : Fast Scroll "),
                Line::from(" [ COMMANDS ] "),
                Line::from(" F1 / Ctrl+K     : Binds Help "),
                Line::from(" F2 / Ctrl+F     : Search "),
                Line::from(" F3 / Ctrl+R     : Replace "),
                Line::from(" ESC             : Escape Mode "),
                Line::from(" Ctrl+W / Ctrl+C : Quit "),
                Line::from(" Ctrl+G          : Reload Git "),
                Line::from(" Ctrl+A          : Previous Tab "),
                Line::from(" Ctrl+D          : Next Tab "),
                Line::from(" Crtl+T / F5     : Open / Close Builtin Terminal"),
                Line::from(""),
                Line::from(vec![Span::styled(
                    " Press ESC to close ",
                    Style::default().fg(Color::Yellow),
                )]),
            ];

            let p = Paragraph::new(text)
                .block(block)
                .alignment(modal.modal_type.alignment());
            f.render_widget(p, modal_area);
        }
        ModalType::NewFile => {
            let block = Block::default()
                .title(" Create New File ")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Rgb(50, 50, 50)));

            let mut text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Path: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&modal.input, Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" Press Enter to create ", Style::default().fg(Color::DarkGray)),
                ]),
            ];

            if let Some(error) = &modal.error_message {
                text.push(Line::from(vec![
                    Span::styled(format!(" Error: {} ", error), Style::default().fg(Color::Red).bg(Color::Rgb(40, 20, 20))),
                ]));
            }

            let p = Paragraph::new(text)
                .block(block)
                .alignment(modal.modal_type.alignment());
            f.render_widget(p, modal_area);

            f.set_cursor_position((
                modal_area.x + 8 + modal.input.len() as u16,
                modal_area.y + 2,
            ));
        }
    }
}
