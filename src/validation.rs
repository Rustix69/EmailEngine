pub fn is_valid_email(email: &str) -> bool {
    // Basic email validation - you might want to use a more sophisticated library
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
}
