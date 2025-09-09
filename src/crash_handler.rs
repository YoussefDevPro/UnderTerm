use std::fs::File;
use std::io::Write;
use std::panic::PanicHookInfo;

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info: &PanicHookInfo| {
        let debug_message = format!("Panic Info: {:?}", panic_info);
        if let Ok(mut file) = File::create("debug_underterm.txt") {
            let _ = file.write_all(debug_message.as_bytes());
        }
    }));
}
