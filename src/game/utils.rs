pub fn wrap_text_to_width(text: &str, width: u16) -> String {
    fn wrap_line(line: &str, width: u16) -> String {
        let mut result = String::new();
        let mut current_line_len = 0;
        for word in line.split_whitespace() {
            if current_line_len > 0 && current_line_len + 1 + word.len() as u16 > width {
                result.push('\n');
                current_line_len = 0;
            }
            if current_line_len > 0 {
                result.push(' ');
                current_line_len += 1;
            }
            // long word handling
            if word.len() as u16 > width {
                let mut chars = word.chars();
                while let Some(c) = chars.next() {
                    if current_line_len >= width {
                        result.push('\n');
                        current_line_len = 0;
                    }
                    result.push(c);
                    current_line_len += 1;
                }
            } else {
                result.push_str(word);
                current_line_len += word.len() as u16;
            }
        }
        result
    }

    text.lines()
        .map(|line| {
            let leading_whitespace: String = line.chars().take_while(|&c| c.is_whitespace()).collect();
            let trimmed_line = line.trim_start();
            if trimmed_line.is_empty() {
                return line.to_string();
            }
            let wrapped_trimmed_line = wrap_line(trimmed_line, width.saturating_sub(leading_whitespace.len() as u16));
            wrapped_trimmed_line.lines().map(|l| format!("{}{}", leading_whitespace, l)).collect::<Vec<_>>().join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
}