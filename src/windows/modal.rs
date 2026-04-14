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
}

impl ModalType {
    pub fn layout(&self) -> ModalLayout {
        match self {
            ModalType::Search | ModalType::Replace | ModalType::ReplaceAll => ModalLayout::Island,
            ModalType::QuitPrompt
            | ModalType::ConfirmExit
            | ModalType::Help
            | ModalType::NewFile => ModalLayout::Popup,
        }
    }

    pub fn alignment(&self) -> Alignment {
        match self {
            ModalType::QuitPrompt | ModalType::ConfirmExit | ModalType::NewFile => Alignment::Center,
            _ => Alignment::Left,
        }
    }
}

pub struct Modal {
    pub modal_type: ModalType,
    pub input: String,
    pub replace_input: String,
    pub focus_replace: bool,
    pub active_button: usize,
    pub error_message: Option<String>,
}

impl Modal {
    pub fn new(modal_type: ModalType) -> Self {
        let active_button = if modal_type == ModalType::QuitPrompt {
            1
        } else if modal_type == ModalType::ConfirmExit {
            0 // "No" by default
        } else {
            0
        };
        Self {
            modal_type,
            input: String::new(),
            replace_input: String::new(),
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

    pub fn toggle_focus(&mut self) {
        if let ModalType::Replace = self.modal_type {
            self.focus_replace = !self.focus_replace;
        }
    }
}
