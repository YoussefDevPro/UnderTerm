use std::fs;
use std::panic;
use std::backtrace::Backtrace;

pub fn set_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let backtrace = Backtrace::capture();

        let mut message = String::new();
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            message.push_str(s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            message.push_str(s);
        } else {
            message.push_str("Unknown panic payload");
        }

        if let Some(location) = panic_info.location() {
            message.push_str(&format!(
                "\nPanic occurred in file '{}' at line {}",
                location.file(),
                location.line()
            ));
        } else {
            message.push_str("\nPanic location unknown.");
        }

        let log_message = format!("{} 

Backtrace:
{:?}", message, backtrace);

        let _ = fs::write("crash_log.txt", log_message);

        default_hook(panic_info);
    }));
}