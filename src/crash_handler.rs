use std::sync::Mutex;
use std::fs::File;
use std::io::Write;
use std::panic::PanicHookInfo;

static LAST_KNOWN_STATE: Mutex<Option<(u16, u16, bool)>> = Mutex::new(None);

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info: &PanicHookInfo| {
        if let Some(state) = LAST_KNOWN_STATE.lock().unwrap().take() {
            let (width, height, dialogue_active) = state;
            if dialogue_active {
                let debug_message = format!("Crashed in dialogue mode! Terminal dimensions: {}x{}\nPanic Info: {:?}", width + 1, height + 1, panic_info);
                if let Ok(mut file) = File::create("debug.txt") {
                    let _ = file.write_all(debug_message.as_bytes());
                }
            }
        }
    }));
}

pub fn update_last_known_state(width: u16, height: u16, dialogue_active: bool) {
    *LAST_KNOWN_STATE.lock().unwrap() = Some((width, height, dialogue_active));
}
