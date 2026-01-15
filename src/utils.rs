pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn format_username(username: &Option<String>) -> String {
    match username {
        Some(name) => format!("@{}", escape_html(name)),
        None => "unknown".to_string(),
    }
}
