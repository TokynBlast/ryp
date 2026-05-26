fn decode_binding(bind: &'static str) -> (bool, bool, Option<String>) {
    let bind = bind.trim().to_lowercase();
    let parts: Vec<&str> = bind.split('+').collect();

    let mut is_ctrl = false;
    let mut is_shift = false;
    let mut key_start = 0;

    // Drain known modifiers from the front, tracking where we stopped
    for part in parts.iter() {
        match *part {
            "ctrl"  if !is_ctrl  => { is_ctrl  = true; key_start += 1; }
            "shift" if !is_shift => { is_shift = true; key_start += 1; }
            _ => break,
        }
    }

    let key_parts = &parts[key_start..];
    let key: String = key_parts.join("+");

    // Must have at least one modifier and a non-empty key
    if key.is_empty() || (!is_ctrl && !is_shift){
        return (false, false, None);
    }

    (is_ctrl, is_shift, Some(key))
}
