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
        KeyCode::F(5) => Some(Action::ToggleTerminal),
        KeyCode::F(6) => Some(Action::PrevTab),
        KeyCode::F(7) => Some(Action::NextTab),
        KeyCode::F(12) => Some(Action::ToggleDebugConsole),

        // Navigation binds
        KeyCode::Up => Some(Action::MoveUp(shift)),
        KeyCode::Down => Some(Action::MoveDown(shift)),
        KeyCode::Left => Some(Action::MoveLeft(shift, ctrl)),
        KeyCode::Right => Some(Action::MoveRight(shift, ctrl)),
        KeyCode::PageUp => Some(Action::PageUp(shift)),
        KeyCode::PageDown => Some(Action::PageDown(shift)),

        // Edit binds
        KeyCode::Char(mut c) => {
            // TODO: Use config json
            if ctrl {
                // TODO: When we have CTRL pressed, SHIFT is ignored,
                //       But only when a character is also pressed...
                //
                // We could do it, so that depending on where we are
                // (TTY, GUI, etc.), we can use the appropriate method

                // Caps lock cancelling
                c = if shift {
                    c.to_uppercase().next().take().unwrap()
                } else {
                    c
                };

                match c {
                    'f' => Some(Action::OpenSearch),
                    'r' => Some(Action::OpenReplace),
                    'k' => Some(Action::OpenHelp),
                    'b' => Some(Action::ToggleSidebar),
                    // TODO: Make CTRL+SHIFT+A/D to tab movement
                    'a' | 'A' => Some(Action::PrevTab),
                    'd' | 'D' => Some(Action::NextTab),
                    'w' => Some(Action::CloseTab),
                    'q' => Some(Action::Quit),
                    'n' => Some(Action::OpenNewFileModal),
                    //'N' => Some(Action::OpenNewRyp)
                    's' => Some(Action::Save),
                    't' => Some(Action::ToggleTerminal),
                    'g' => Some(Action::RefreshGit),
                    'e' => Some(Action::ToggleDebugConsole),
                    'c' | 'C' => Some(Action::Copy()),
                    'x' | 'X' => Some(Action::Cut()),
                    'v' | 'V' => Some(Action::Paste()),
                    'z' | 'Z' => Some(Action::Undo),
                    'y' | 'Y' => Some(Action::Redo),
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
        KeyCode::Backspace => Some(Action::BackSpace),
        KeyCode::Delete => Some(Action::DeleteChar),
        KeyCode::Tab => Some(Action::Tab),
        _ => None,
    }
}
