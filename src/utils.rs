use ratatui::{style::Color, text::Text};

pub fn text_to_ansi_string(text: &Text) -> String {
    let mut ansi_string = String::new();
    let mut current_fg: Option<Color> = None;
    let mut current_bg: Option<Color> = None;

    for line in text.lines.iter() {
        for span in line.spans.iter() {
            let mut codes: Vec<String> = Vec::new();

            // Handle foreground color
            if span.style.fg != current_fg {
                if let Some(color) = span.style.fg {
                    match color {
                        Color::Black => codes.push("30".to_string()),
                        Color::Red => codes.push("31".to_string()),
                        Color::Green => codes.push("32".to_string()),
                        Color::Yellow => codes.push("33".to_string()),
                        Color::Blue => codes.push("34".to_string()),
                        Color::Magenta => codes.push("35".to_string()),
                        Color::Cyan => codes.push("36".to_string()),
                        Color::White => codes.push("37".to_string()),
                        Color::Indexed(i) => codes.push(format!("38;5;{}", i)),
                        Color::Rgb(r, g, b) => codes.push(format!("38;2;{};{};{}", r, g, b)),
                        _ => codes.push("39".to_string()), // Default or unhandled
                    }
                } else {
                    codes.push("39".to_string()); // Reset foreground
                }
                current_fg = span.style.fg;
            }

            // Handle background color
            if span.style.bg != current_bg {
                if let Some(color) = span.style.bg {
                    match color {
                        Color::Black => codes.push("40".to_string()),
                        Color::Red => codes.push("41".to_string()),
                        Color::Green => codes.push("42".to_string()),
                        Color::Yellow => codes.push("43".to_string()),
                        Color::Blue => codes.push("44".to_string()),
                        Color::Magenta => codes.push("45".to_string()),
                        Color::Cyan => codes.push("46".to_string()),
                        Color::White => codes.push("47".to_string()),
                        Color::Indexed(i) => codes.push(format!("48;5;{}", i)),
                        Color::Rgb(r, g, b) => codes.push(format!("48;2;{};{};{}", r, g, b)),
                        _ => codes.push("49".to_string()), // Default or unhandled
                    }
                } else {
                    codes.push("49".to_string()); // Reset background
                }
                current_bg = span.style.bg;
            }

            if !codes.is_empty() {
                ansi_string.push_str(&format!("\x1b[{}m", codes.join(";")));
            }
            ansi_string.push_str(&span.content);
        }
        ansi_string.push_str("\x1b[0m\n"); // Reset style and add newline for each line
        current_fg = None; // Reset for next line
        current_bg = None;
    }
    ansi_string.push_str("\x1b[0m"); // Final reset
    ansi_string
}
