use crate::input::action::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Maps raw terminal key events into application Actions.
/// This allows keybinds to be easily decoupled, remapped, and stored in configuration later.
pub fn map_key(key: KeyEvent, in_modal: bool, is_sidebar_focused: bool) -> Option<Action> {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // Modal Keymap context
    if in_modal {
        return match key.code {
            KeyCode::Esc => Some(Action::CloseModal),
            KeyCode::Char(c) => {
                if !ctrl {
                    Some(Action::ModalInsert(c))
                } else {
                    None
                }
            }
            KeyCode::Backspace => Some(Action::ModalDelete),
            KeyCode::Tab => Some(Action::ModalTab),
            KeyCode::Left => Some(Action::ModalLeft),
            KeyCode::Right => Some(Action::ModalRight),
            KeyCode::Up => Some(Action::ModalUp),
            KeyCode::Down => Some(Action::ModalDown),
            KeyCode::Enter => Some(Action::ModalConfirm),
            _ => None,
        };
    }

    // Sidebar Specific Binds (only if focused)
    if is_sidebar_focused && ctrl {
        match key.code {
            KeyCode::Char('a' | 'A') => return Some(Action::PrevSidebarCategory),
            KeyCode::Char('d' | 'D') => return Some(Action::NextSidebarCategory),
            _ => {}
        }
    }

    // Global / Editor Keymap context
    match key.code {
        KeyCode::Esc => Some(Action::SwitchFocus),
        KeyCode::F(1) => Some(Action::OpenHelp),
        KeyCode::F(2) => Some(Action::OpenSearch),
        KeyCode::F(3) => Some(Action::OpenReplace),
        KeyCode::F(4) => Some(Action::ToggleSidebar),
        KeyCode::F(5) => Some(Action::RefreshGit),
        KeyCode::F(6) => Some(Action::PrevTab),
        KeyCode::F(7) => Some(Action::NextTab),
        KeyCode::F(12) => Some(Action::Quit),

        // Navigation binds
        KeyCode::Up => Some(Action::MoveUp(shift)),
        KeyCode::Down => Some(Action::MoveDown(shift)),
        KeyCode::Left => Some(Action::MoveLeft(shift)),
        KeyCode::Right => Some(Action::MoveRight(shift)),
        KeyCode::PageUp => Some(Action::PageUp(shift)),
        KeyCode::PageDown => Some(Action::PageDown(shift)),

        // Edit binds
        KeyCode::Char(c) => {
            if ctrl {
                match c {
                    'f' | 'F' => Some(Action::OpenSearch),
                    'r' | 'R' => Some(Action::OpenReplace),
                    'k' | 'K' => Some(Action::OpenHelp),
                    'b' | 'B' => Some(Action::ToggleSidebar),
                    'a' | 'A' => Some(Action::PrevTab),
                    'd' | 'D' => Some(Action::NextTab),
                    'w' | 'W' => Some(Action::CloseTab),
                    'q' | 'Q' => Some(Action::Quit),
                    'n' | 'N' => Some(Action::OpenNewFileModal),
                    's' | 'S' => Some(Action::Save),
                    _ => None,
                }
            } else {
                Some(Action::InsertChar(c))
            }
        }
        KeyCode::Enter => {
            if ctrl {
                Some(Action::ModalConfirmForceNewTab)
            } else {
                Some(Action::InsertNewline)
            }
        }
        KeyCode::Backspace => Some(Action::DeleteChar),
        KeyCode::Tab => Some(Action::Tab),
        _ => None,
    }
}
