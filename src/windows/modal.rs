use compact_str::CompactString;
use ratatui::layout::Alignment;

pub enum ModalLayout {
    Island,
    Popup,
}

#[derive(PartialEq)]
pub enum ModalType {
    Search,
    Replace,
    ReplaceAll,
    QuitPrompt,
    ConfirmExit,
    Help,
    NewFile,
    CloseTabPrompt,
    DeleteFile,
    CommandPallete,
    Settings,
}

impl ModalType {
    pub fn layout(&self) -> ModalLayout {
        match self {
            ModalType::Search
            | ModalType::Replace
            | ModalType::ReplaceAll
            | ModalType::CommandPallete
            => ModalLayout::Island,

            ModalType::QuitPrompt
            | ModalType::ConfirmExit
            | ModalType::Help
            | ModalType::NewFile
            | ModalType::CloseTabPrompt
            | ModalType::DeleteFile
            | ModalType::Settings
            => ModalLayout::Popup,
        }
    }

    pub fn alignment(&self) -> Alignment {
        match self {
            ModalType::QuitPrompt | ModalType::ConfirmExit | ModalType::NewFile | ModalType::CloseTabPrompt | ModalType::DeleteFile => Alignment::Center,
            _ => Alignment::Left,
        }
    }
}

pub struct Modal {
    pub modal_type: ModalType,
    pub input: CompactString,
    pub replace_input: CompactString,
    pub focus_replace: bool,
    pub active_button: usize,
    pub error_message: Option<CompactString>,
}

impl Modal {
    pub fn new(modal_type: ModalType) -> Self {
        let active_button =
            match modal_type {
                // "No" by default on all
                ModalType::QuitPrompt => 1,
                ModalType::ConfirmExit => 0,
                ModalType::Replace => 0,
                ModalType::ReplaceAll => 0,
                ModalType::Search => 0,
                ModalType::NewFile => 0,
                ModalType::DeleteFile => 0,
                _ => 0
            };

        Self {
            modal_type,
            input: CompactString::default(),
            replace_input: CompactString::default(),
            focus_replace: false,
            active_button,
            error_message: None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.focus_replace {
            self.replace_input.push(c);
        } else {
            self.input.push(c);
        }
    }

    pub fn pop_char(&mut self) {
        if self.focus_replace {
            self.replace_input.pop();
        } else {
            self.input.pop();
        }
    }

    pub fn remove_char(&mut self, idx: usize) {
        if self.focus_replace {
            if idx < self.replace_input.len() {
                self.replace_input.remove(idx);
            }
        } else {
            if idx < self.input.len() {
                self.input.remove(idx);
            }
        }
    }

    pub fn toggle_focus(&mut self) {
        if let ModalType::Replace = self.modal_type {
            self.focus_replace = !self.focus_replace;
        }
    }
}
