pub fn wrap_text_to_width(text: &str, width: u16) -> String {
    let mut result_lines = Vec::new();
    for line in text.lines() {
        let leading_spaces_count = line.chars().take_while(|&c| c.is_whitespace() && c != '\n').count();
        let leading_spaces: String = line.chars().take(leading_spaces_count).collect();
        let trimmed_line = line.trim_start();

        if trimmed_line.is_empty() {
            result_lines.push(line.to_string());
            continue;
        }

        let mut current_line = leading_spaces.clone();
        let mut words = trimmed_line.split_whitespace();

        if let Some(first_word) = words.next() {
            current_line.push_str(first_word);

            for word in words {
                if current_line.len() + 1 + word.len() <= width as usize {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    result_lines.push(current_line);
                    current_line = leading_spaces.clone();
                    current_line.push_str(word);
                }
            }
        }
        result_lines.push(current_line);
    }
    result_lines.join("\n")
}