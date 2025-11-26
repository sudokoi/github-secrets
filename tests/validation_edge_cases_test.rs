use github_secrets::validation::{
    validate_repo_name, validate_repo_owner, validate_secret_key, validate_token,
};

#[test]
fn test_validate_token_empty_string() {
    let result = validate_token("");
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("empty") || error.contains("invalid") || error.contains("at least"));
}

#[test]
fn test_validate_token_too_short() {
    let result = validate_token("abc");
    assert!(result.is_err());
}

#[test]
fn test_validate_token_valid_classic() {
    // Classic GitHub token format: ghp_...
    let result = validate_token("ghp_1234567890abcdefghijklmnopqrstuvwxyzABCD");
    assert!(result.is_ok());
}

#[test]
fn test_validate_token_valid_fine_grained() {
    // Fine-grained token format: github_pat_...
    let result = validate_token(
        "github_pat_11ABCDEFG0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_unicode_characters() {
    let result = validate_secret_key("SECRET_KEY_日本語");
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("letters")
            || error.contains("numbers")
            || error.contains("underscores")
            || error.contains("hyphens")
    );
}

#[test]
fn test_validate_secret_key_only_special_characters() {
    let result = validate_secret_key("!!!@@@###");
    assert!(result.is_err());
}

#[test]
fn test_validate_secret_key_with_spaces() {
    let result = validate_secret_key("SECRET KEY");
    assert!(result.is_err());
}

#[test]
fn test_validate_secret_key_with_dots() {
    let result = validate_secret_key("SECRET.KEY");
    assert!(result.is_err());
}

#[test]
fn test_validate_secret_key_underscore_only() {
    let result = validate_secret_key("_");
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_hyphen_only() {
    let result = validate_secret_key("-");
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_mixed_valid_chars() {
    let result = validate_secret_key("My-Secret_Key123");
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_starts_with_number() {
    let result = validate_secret_key("123_SECRET");
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_all_uppercase() {
    let result = validate_secret_key("MYSECRET");
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_all_lowercase() {
    let result = validate_secret_key("mysecret");
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_exactly_max_length() {
    let key = "A".repeat(100);
    let result = validate_secret_key(&key);
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_over_max_length() {
    let key = "A".repeat(101);
    let result = validate_secret_key(&key);
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_owner_numbers_only() {
    let result = validate_repo_owner("123456");
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_owner_with_hyphen() {
    let result = validate_repo_owner("my-org");
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_owner_exactly_max_length() {
    let owner = "a".repeat(39);
    let result = validate_repo_owner(&owner);
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_owner_over_max_length() {
    let owner = "a".repeat(40);
    let result = validate_repo_owner(&owner);
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_owner_empty() {
    let result = validate_repo_owner("");
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_name_with_dots() {
    let result = validate_repo_name("my.repo");
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_with_underscores() {
    let result = validate_repo_name("my_repo");
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_with_hyphens() {
    let result = validate_repo_name("my-repo");
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_consecutive_hyphens() {
    let result = validate_repo_name("my--repo");
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_exactly_max_length() {
    let name = "a".repeat(100);
    let result = validate_repo_name(&name);
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_over_max_length() {
    let name = "a".repeat(101);
    let result = validate_repo_name(&name);
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_name_empty() {
    let result = validate_repo_name("");
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_name_numbers_only() {
    let result = validate_repo_name("123456");
    assert!(result.is_ok());
}

// Tests for whitespace handling - validation uses trim() so these should pass
#[test]
fn test_validate_secret_key_leading_whitespace_trimmed() {
    let result = validate_secret_key(" SECRET");
    // Should pass because trim() is called
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_trailing_whitespace_trimmed() {
    let result = validate_secret_key("SECRET ");
    // Should pass because trim() is called
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_owner_leading_whitespace_trimmed() {
    let result = validate_repo_owner(" owner");
    // Should pass because trim() is called
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_owner_trailing_whitespace_trimmed() {
    let result = validate_repo_owner("owner ");
    // Should pass because trim() is called
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_leading_whitespace_trimmed() {
    let result = validate_repo_name(" repo");
    // Should pass because trim() is called
    assert!(result.is_ok());
}

#[test]
fn test_validate_repo_name_trailing_whitespace_trimmed() {
    let result = validate_repo_name("repo ");
    // Should pass because trim() is called
    assert!(result.is_ok());
}

#[test]
fn test_validate_secret_key_empty_after_trim() {
    let result = validate_secret_key("   ");
    // Should fail because it's empty after trim
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_owner_empty_after_trim() {
    let result = validate_repo_owner("   ");
    // Should fail because it's empty after trim
    assert!(result.is_err());
}

#[test]
fn test_validate_repo_name_empty_after_trim() {
    let result = validate_repo_name("   ");
    // Should fail because it's empty after trim
    assert!(result.is_err());
}
