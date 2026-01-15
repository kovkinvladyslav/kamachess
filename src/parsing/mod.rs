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
    
    // Handle castling notation
    let upper = token.to_uppercase();
    if upper == "O-O" || upper == "O-O-O" || upper == "0-0" || upper == "0-0-0" {
        return true;
    }
    
    // Allow alphanumeric and SAN-specific characters: -, +, #, =, x (for captures)
    if !token.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '+' || c == '#' || c == '=' || c == 'x' || c == 'X') {
        return false;
    }
    
    // Must contain at least one digit (rank number) unless it's castling
    if !token.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    
    // First character must be a valid piece letter (K, Q, R, B, N) or file letter (a-h)
    let first = token.chars().next().unwrap();
    let valid_first = matches!(first.to_ascii_uppercase(), 'K' | 'Q' | 'R' | 'B' | 'N')
        || matches!(first.to_ascii_lowercase(), 'a'..='h');
    
    if !valid_first {
        return false;
    }
    
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_move_candidate_valid_moves() {
        // Pawn moves
        assert!(is_move_candidate("e4"));
        assert!(is_move_candidate("d5"));
        assert!(is_move_candidate("a3"));
        assert!(is_move_candidate("h6"));
        
        // Piece moves
        assert!(is_move_candidate("Nf3"));
        assert!(is_move_candidate("Bc4"));
        assert!(is_move_candidate("Qd1"));
        assert!(is_move_candidate("Rfe1"));
        assert!(is_move_candidate("Kf2"));
        
        // Captures
        assert!(is_move_candidate("Nxe5"));
        assert!(is_move_candidate("Bxf7"));
        assert!(is_move_candidate("exd5"));
        
        // Disambiguation
        assert!(is_move_candidate("Nbd7"));
        assert!(is_move_candidate("R1e2"));
        assert!(is_move_candidate("Qh4e1"));
        
        // Promotions
        assert!(is_move_candidate("e8Q"));
        assert!(is_move_candidate("a1=Q"));
        
        // With check/checkmate markers
        assert!(is_move_candidate("Qxf7+"));
        assert!(is_move_candidate("Rd8#"));
        
        // Coordinate notation
        assert!(is_move_candidate("e2e4"));
        assert!(is_move_candidate("g1f3"));
        assert!(is_move_candidate("e7e8q"));
        
        // Castling
        assert!(is_move_candidate("O-O"));
        assert!(is_move_candidate("O-O-O"));
        assert!(is_move_candidate("o-o"));
        assert!(is_move_candidate("0-0"));
        assert!(is_move_candidate("0-0-0"));
    }

    #[test]
    fn test_is_move_candidate_invalid() {
        // Common words that should NOT be moves
        assert!(!is_move_candidate("start"));
        assert!(!is_move_candidate("help"));
        assert!(!is_move_candidate("resign"));
        assert!(!is_move_candidate("draw"));
        assert!(!is_move_candidate("accept"));
        assert!(!is_move_candidate("history"));
        
        // Too short
        assert!(!is_move_candidate("e"));
        assert!(!is_move_candidate("N"));
        
        // Too long
        assert!(!is_move_candidate("Qxf7++++"));
        
        // No digit
        assert!(!is_move_candidate("Nf"));
        assert!(!is_move_candidate("abc"));
        
        // Invalid first character
        assert!(!is_move_candidate("1e4"));
        assert!(!is_move_candidate("xe4"));
    }

    #[test]
    fn test_extract_move_from_start_command() {
        // /start without move
        assert_eq!(extract_move("/start"), None);
        assert_eq!(extract_move("/start @username"), None);
        
        // /start with move
        assert_eq!(extract_move("/start e4"), Some("e4".to_string()));
        assert_eq!(extract_move("/start @username e4"), Some("e4".to_string()));
        assert_eq!(extract_move("/start Nf3"), Some("Nf3".to_string()));
        assert_eq!(extract_move("/start @user d2d4"), Some("d2d4".to_string()));
    }

    #[test]
    fn test_extract_move_preserves_case() {
        // SAN notation should preserve case
        assert_eq!(extract_move("Nf3"), Some("Nf3".to_string()));
        assert_eq!(extract_move("nf3"), Some("nf3".to_string()));
        assert_eq!(extract_move("Qxd5"), Some("Qxd5".to_string()));
    }

    #[test]
    fn test_extract_usernames() {
        assert_eq!(extract_usernames("@user"), vec!["user".to_string()]);
        assert_eq!(extract_usernames("/start @user e4"), vec!["user".to_string()]);
        assert_eq!(
            extract_usernames("/history @user1 @user2"),
            vec!["user1".to_string(), "user2".to_string()]
        );
        assert!(extract_usernames("no usernames here").is_empty());
        assert!(extract_usernames("/start e4").is_empty());
    }
}
