use std::fs;

#[cfg(target_os = "linux")]
pub fn is_caps_lock_on() -> bool {
    use glob::glob;
    let capslock_led_file = "/sys/class/leds/input*::capslock/brightness";

    if let Ok(paths) = glob(capslock_led_file) {
        for entry in paths.flatten() {
            if let Ok(content) = fs::read_to_string(&entry) {
                // '0' means off, '1' (or higher) means on
                if content.trim() != "0" {
                    return true;
                }
            }
        }
    }
    false
}



#[cfg(target_os = "windows")]
fn is_caps_lock_on() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL};
    // NOTE: This should be safe, as long as the main thread is active.
    //       Which we can garuntee, since we only call it from the main thread.
    unsafe {
        // If the low-order bit is 1, the key is toggled (Caps Lock is ON)
        (GetKeyState(VK_CAPITAL as i32) & 1) != 0
    }
}
