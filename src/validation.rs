/// Most messages one /send-batch or /schedule-batch call accepts
pub const MAX_BATCH: usize = 50;

/// Most addresses one /validate-emails call accepts (the upstream verifier's own limit)
pub const MAX_VALIDATE_BATCH: usize = 100;

pub fn is_valid_email(email: &str) -> bool {
    // Basic email validation - you might want to use a more sophisticated library
    let email_regex = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    email_regex.is_match(email)
}
