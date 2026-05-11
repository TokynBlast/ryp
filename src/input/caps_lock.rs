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
pub fn is_caps_lock_on() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL};
    // NOTE: This should be safe, as long as the main thread is active.
    //       Which we can garuntee, since we only call it from the main thread.
    unsafe {
        // If the low-order bit is 1, the key is toggled (Caps Lock is ON)
        (GetKeyState(VK_CAPITAL as i32) & 1) != 0
    }
}

#[cfg(target_os = "emscripten")]
pub fn is_caps_lock_on() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::ffi::c_void;
    use std::os::raw::c_int;
    use emscripten_functions_sys::html5::{
        EmscriptenKeyboardEvent,
        emscripten_set_keydown_callback_on_thread,
        EMSCRIPTEN_EVENT_TARGET_WINDOW,
    };

    static CAPS_LOCK_ACTIVE: AtomicBool = AtomicBool::new(false);
    static LISTENER_REGISTERED: AtomicBool = AtomicBool::new(false);

    unsafe extern "C" fn on_keydown(
        _event_type: c_int,
        event: *const EmscriptenKeyboardEvent,
        _user_data: *mut c_void,
    ) -> bool {
        let key = std::ffi::CStr::from_ptr((*event).key.as_ptr())
            .to_str()
            .unwrap_or("");
        if key == "CapsLock" {
            let active = CAPS_LOCK_ACTIVE.load(Ordering::Relaxed);
            CAPS_LOCK_ACTIVE.store(!active, Ordering::Relaxed);
        }
        false
    }

    if LISTENER_REGISTERED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
        // NOTE: This is only unsafe, because it's a call to JS in the end
        unsafe {
            emscripten_set_keydown_callback_on_thread(
                EMSCRIPTEN_EVENT_TARGET_WINDOW,
                std::ptr::null_mut(),
                false,
                Some(on_keydown),
                0 as _,
            );
        }
    }

    CAPS_LOCK_ACTIVE.load(Ordering::Relaxed)
}

#[cfg(target_os = "freebsd")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for FreeBSD is not implemented yet.")
}

#[cfg(target_os = "openbsd")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for OpenBSD is not implemented yet.")
}

#[cfg(target_os = "netbsd")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for NetBSD is not implemented yet.")
}

#[cfg(target_os = "macos")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for MacOS is not implemented yet.")
}

#[cfg(target_os = "ios")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for iOS is not implemented yet.")
}

#[cfg(target_os = "watchos")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for WatchOS is not implemented yet.")
}

#[cfg(target_os = "visionos")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for VisionsOS is not implemented yet.")
}

#[cfg(target_os = "android")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Android is not implemented yet.")
}

#[cfg(target_os = "watchos")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for WatchOS is not implemented yet.")
}

#[cfg(any(target_arch="wasm32", target_arch="wasm64"))]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for WASM is not implemented yet.")
}

#[cfg(target_os = "haiku")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Haiku is not implemented yet.")
}

#[cfg(target_os = "solaris")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Solaris is not implemented yet.")
}

#[cfg(target_os = "fuchsia")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Fuchsia is not implemented yet.")
}

#[cfg(target_os = "horizon")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Nintendo devices is not implemented yet.")
}

#[cfg(target_os = "illumos")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Illumos is not implemented yet.")
}

#[cfg(target_os = "nto")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for the QNX Neutrino is not implemented yet.")
}

#[cfg(target_os = "vita")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for the PS Vita is not implemented yet.")
}

#[cfg(target_os = "redox")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for RedoxOS is not implemented yet.")
}

#[cfg(target_os = "vxworks")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for Wind River VxWorks is not implemented yet.")
}

#[cfg(target_os = "espidf")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for the ESP Board is not implemented yet.")
}

#[cfg(target_os = "uefi")]
pub fn is_caps_lock_on() -> bool {
    compile_error!("Caps lock detection for UEFI is not implemented yet.")
}
