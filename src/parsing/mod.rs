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
            .trim_matches(|c: char| {
                // Keep SAN characters: -, +, #, =, x, X, O, 0
                !c.is_alphanumeric() && c != '-' && c != '+' && c != '#' && c != '=' && c != 'x' && c != 'X' && c != 'O' && c != '0'
            })
            .to_string();
        if is_move_candidate(&cleaned) {
            Some(cleaned) // Don't lowercase - preserve case for SAN notation
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
    // Accept SAN notation (2-7 chars: e4, Nf6, Qxd5, O-O-O, e8=Q+) and coordinate notation (2-5 chars)
    if len < 2 || len > 7 {
        return false;
    }
    // Allow alphanumeric and SAN-specific characters: -, +, #, =, x (for captures)
    token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '+' || c == '#' || c == '=' || c == 'x' || c == 'X')
}
