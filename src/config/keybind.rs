use modular_bitfield::prelude::*;

#[derive(Debug, Specifier)]
#[bits = 4]
pub enum FnKey {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    None,
}

#[bitfield]
pub struct KeyCombo {
    pub rctrl: bool,
    pub lctrl: bool,
    pub lshift: bool,
    pub rshift: bool,
    pub lalt: bool,
    pub ralt: bool,
    pub lfn: bool,
    pub rfn: bool,
    pub esc: bool,
    pub del: bool,
    pub pgup: bool,
    pub pgdown: bool,
    pub insert: bool,
    pub meta: bool,
    pub enter: bool,
    pub tab: bool,
    pub fn_key: FnKey,
    pub left_arrow_key: bool,
    pub right_arrow_key: bool,
    pub up_arrow_key: bool,
    pub down_arrow_key: bool,
    pub backspace_key: bool,
    pub home_key: bool,
    pub end_key: bool,
    pub print_key: bool,
    // Here to add padding, allowing us to do this, no matter how many keys we look for
    #[allow(dead_code)]
    padding: B4,
}

fn decode_binding(bind: &'static str) -> Option<(KeyCombo, Option<String>)> {
    let bind = bind.trim().to_lowercase();
    let parts: Vec<&str> = bind.split('+').collect();

    let (
        mut is_left_ctrl,
        mut is_right_ctrl,
        mut is_left_shift,
        mut is_right_shift,
        mut is_left_alt,
        mut is_right_alt,
        mut is_left_fn_key,
        mut is_right_fn_key,
        mut is_esc,
        mut is_del,
        mut is_pgup,
        mut is_pgdown,
        mut is_insert,
        mut is_meta,
        mut is_enter,
        mut is_tab,
        mut is_backspace,
        mut is_home_key,
        mut is_end_key,
        mut is_print_key,
        mut is_down_arrow,
        mut is_up_arrow,
        mut is_right_arrow,
        mut is_left_arrow,
        mut fn_key,
    ) = (
        false, false, false, false, false, false,
        false, false, false, false, false, false,
        false, false, false, false, false, false,
        false, false, false, false, false, false,
        FnKey::None
    );

    let mut key_start = 0;

    // Drain known modifiers from the front, tracking where we stopped
    for part in parts.iter() {
        match *part {
            "ctrl" => (is_left_ctrl, is_right_ctrl) = (true, true),
            "shift" => (is_left_shift, is_right_shift) = (true, true),
            "alt" => (is_left_alt, is_right_alt) = (true, true),
            "fn" => (is_left_fn_key, is_right_fn_key) = (true, true),
            "left ctrl" => is_left_ctrl = true,
            "left shift" => is_left_shift = true,
            "left alt" => is_left_alt = true,
            "right ctrl" => is_right_ctrl = true,
            "right shift" => is_right_shift = true,
            "right alt" => is_right_alt = true,
            "meta" => is_meta = true,
            "esc" => is_esc = true,
            "del" => is_del = true,
            "pgup" => is_pgup = true,
            "pgdown" => is_pgdown = true,
            "enter" => is_enter = true,
            "tab" => is_tab = true,
            "backspace" => is_backspace = true,
            "down arrow" => is_down_arrow = true,
            "up arrow" => is_up_arrow = true,
            "left arrow" => is_left_arrow = true,
            "right arrow" => is_right_arrow = true,
            "insert" => is_insert = true,
            "left fn" => is_left_fn_key = true,
            "right fn" => is_right_fn_key = true,
            "home" => is_home_key = true,
            "end" => is_end_key = true,
            "print" => is_print_key = true,
            _ => {
                // Check if it's a function key
                if part.len() <= 3 && part.len() >= 2 && part.starts_with('f'){
                    match part[1..].parse::<u8>().unwrap_or(0) {
                        1 => fn_key = FnKey::F1,
                        2 => fn_key = FnKey::F2,
                        3 => fn_key = FnKey::F3,
                        4 => fn_key = FnKey::F4,
                        5 => fn_key = FnKey::F5,
                        6 => fn_key = FnKey::F6,
                        7 => fn_key = FnKey::F7,
                        8 => fn_key = FnKey::F8,
                        9 => fn_key = FnKey::F9,
                        10 => fn_key = FnKey::F10,
                        11 => fn_key = FnKey::F11,
                        12 => fn_key = FnKey::F12,
                        _ => fn_key = FnKey::None,
                    };
                    key_start += 1;
                } else {
                    break;
                }
            },
        }
        key_start += 1;
    }

    let key_parts = &parts[key_start..];
    let key: String = key_parts.join("+");

    // Must have at least one modifier and a non-empty key
    if key.is_empty() || (
        !is_left_alt
        && !is_right_alt
        && !is_left_ctrl
        && !is_right_ctrl
        && !is_right_fn_key
        && !is_left_fn_key
        && !is_left_shift
        && !is_right_shift
        && !is_del
        && !is_enter
        && !is_esc
        && !is_insert
        && !is_meta
        && !is_pgdown
        && !is_pgup
        && !is_tab
    ){
        return None;
    } else {
        Some((
            KeyCombo::new()
            .with_backspace_key(is_backspace)
            .with_fn_key(fn_key)
            .with_del(is_del)
            .with_down_arrow_key(is_down_arrow)
            .with_up_arrow_key(is_up_arrow)
            .with_left_arrow_key(is_left_arrow)
            .with_right_arrow_key(is_right_arrow)
            .with_home_key(is_home_key)
            .with_pgup(is_pgup)
            .with_pgdown(is_pgdown)
            .with_end_key(is_end_key)
            .with_lalt(is_left_alt)
            .with_ralt(is_right_alt)
            .with_lctrl(is_left_ctrl)
            .with_rctrl(is_right_ctrl)
            .with_insert(is_insert)
            .with_lfn(is_left_fn_key)
            .with_rfn(is_right_fn_key)
            .with_print_key(is_print_key)
            .with_tab(is_tab)
            .with_esc(is_esc)
            .with_meta(is_meta)
            .with_enter(is_enter)
            .with_padding(0)
            ,
            Some(key)
        ))
    }
}
