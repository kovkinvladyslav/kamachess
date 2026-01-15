pub fn extract_usernames(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            if token.starts_with('@') {
                let trimmed = token
                    .trim_start_matches('@')
                    .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                    .to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn extract_move(text: &str) -> Option<String> {
    text.split_whitespace().rev().find_map(|token| {
        let cleaned = token
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if is_move_candidate(&cleaned) {
            Some(cleaned.to_lowercase())
        } else {
            None
        }
    })
}

pub fn extract_page(text: &str) -> Option<u32> {
    text.split_whitespace()
        .filter_map(|token| token.parse::<u32>().ok())
        .next()
}

fn is_move_candidate(token: &str) -> bool {
    let len = token.len();
    (len == 2 || len == 4 || len == 5) && token.chars().all(|c| c.is_alphanumeric())
}
