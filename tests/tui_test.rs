//! Integration tests for TUI interactions and validation flows.
//!
//! These tests verify that the TUI validation logic works correctly
//! and that user inputs are properly validated before processing.

use github_secrets::prompt::SecretPair;
use github_secrets::validation;

/// Test that secret pairs can be created and validated.
#[test]
fn test_secret_pair_creation() {
    let pair = SecretPair {
        key: "TEST_KEY".to_string(),
        value: "test-value".to_string(),
    };

    assert_eq!(pair.key, "TEST_KEY");
    assert_eq!(pair.value, "test-value");

    // Test validation of the key
    assert!(validation::validate_secret_key(&pair.key).is_ok());
}

/// Test secret pair cloning (used in retry logic).
#[test]
fn test_secret_pair_cloning() {
    let pair = SecretPair {
        key: "KEY1".to_string(),
        value: "value1".to_string(),
    };

    let cloned = pair.clone();
    assert_eq!(pair.key, cloned.key);
    assert_eq!(pair.value, cloned.value);

    // Modify original
    let mut pair2 = pair;
    pair2.key = "KEY2".to_string();

    // Cloned should be unchanged
    assert_eq!(cloned.key, "KEY1");
}

/// Test validation integration with secret pairs.
#[test]
fn test_secret_pair_validation() {
    // Valid secret pair
    let valid_pair = SecretPair {
        key: "VALID_KEY_123".to_string(),
        value: "some-value".to_string(),
    };
    assert!(validation::validate_secret_key(&valid_pair.key).is_ok());

    // Invalid secret pair keys
    let too_long = "a".repeat(101);
    let invalid_keys = vec!["key with spaces", "key@invalid", "", &too_long];

    for invalid_key in invalid_keys {
        let pair = SecretPair {
            key: invalid_key.to_string(),
            value: "value".to_string(),
        };
        assert!(
            validation::validate_secret_key(&pair.key).is_err(),
            "Key '{}' should be invalid",
            invalid_key
        );
    }
}

/// Test that duplicate key detection works correctly.
#[test]
fn test_duplicate_key_detection() {
    let mut secrets = vec![
        SecretPair {
            key: "KEY1".to_string(),
            value: "value1".to_string(),
        },
        SecretPair {
            key: "KEY2".to_string(),
            value: "value2".to_string(),
        },
    ];

    // Check for duplicate
    let has_duplicate = secrets.iter().any(|s| s.key == "KEY1");
    assert!(has_duplicate);

    let has_duplicate_key2 = secrets.iter().any(|s| s.key == "KEY2");
    assert!(has_duplicate_key2);

    let has_duplicate_key3 = secrets.iter().any(|s| s.key == "KEY3");
    assert!(!has_duplicate_key3);

    // Test removing duplicates
    let key_to_remove = "KEY1";
    secrets.retain(|s| s.key != key_to_remove);
    assert_eq!(secrets.len(), 1);
    assert_eq!(secrets[0].key, "KEY2");
}

/// Test validation error messages are user-friendly.
#[test]
fn test_validation_error_messages() {
    // Test secret key validation errors
    let result = validation::validate_secret_key("");
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("empty") || error_msg.contains("cannot be"));

    let too_long = "a".repeat(101);
    let result = validation::validate_secret_key(&too_long);
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("exceed") || error_msg.contains("100"));

    let result2 = validation::validate_secret_key("key with spaces");
    assert!(result2.is_err());
    let error_msg2 = result2.unwrap_err().to_string();
    assert!(
        error_msg2.contains("letters")
            || error_msg2.contains("numbers")
            || error_msg2.contains("invalid")
    );

    // Test repository owner validation errors
    let result3 = validation::validate_repo_owner("");
    assert!(result3.is_err());
    let error_msg3 = result3.unwrap_err().to_string();
    assert!(error_msg3.contains("empty") || error_msg3.contains("cannot be"));

    // Test repository name validation errors
    let result4 = validation::validate_repo_name("");
    assert!(result4.is_err());
    let error_msg4 = result4.unwrap_err().to_string();
    assert!(error_msg4.contains("empty") || error_msg4.contains("cannot be"));
}

/// Test token validation.
#[test]
fn test_token_validation() {
    // Valid tokens (GitHub token format)
    assert!(validation::validate_token("ghp_1234567890123456789012345678901234567890").is_ok());
    assert!(
        validation::validate_token("github_pat_1234567890123456789012345678901234567890").is_ok()
    );
    assert!(validation::validate_token("gho_1234567890123456789012345678901234567890").is_ok());

    // Invalid tokens
    assert!(validation::validate_token("").is_err());
    assert!(validation::validate_token("short").is_err());
    // Note: "invalid_token_format" might pass if it's long enough and matches the pattern
    // The validation checks for prefix and length, so we test more specific invalid cases
    assert!(validation::validate_token("ghp_").is_err()); // Too short after prefix
    assert!(validation::validate_token("ghp_abc").is_err()); // Too short
    assert!(validation::validate_token("invalid_").is_err()); // Wrong prefix
}

/// Test that all validation functions handle whitespace correctly.
#[test]
fn test_validation_whitespace_handling() {
    // Validation should trim whitespace
    assert!(validation::validate_secret_key("  valid_key  ").is_ok());
    assert!(validation::validate_repo_owner("  valid_owner  ").is_ok());
    assert!(validation::validate_repo_name("  valid_repo  ").is_ok());
    let valid_token = format!("  ghp_{}  ", "a".repeat(30));
    assert!(validation::validate_token(valid_token.trim()).is_ok());

    // But empty after trimming should fail
    assert!(validation::validate_secret_key("   ").is_err());
    assert!(validation::validate_repo_owner("   ").is_err());
    assert!(validation::validate_repo_name("   ").is_err());
}

/// Test comprehensive validation scenarios.
#[test]
fn test_comprehensive_validation_scenarios() {
    // Test all valid patterns
    let max_length_key = "a".repeat(100);
    let valid_secret_keys = vec![
        "UPPER_CASE",
        "lower_case",
        "MixedCase",
        "key-with-hyphens",
        "key_with_underscores",
        "key123",
        "123key",
        "KEY_123",
        "a", // Minimum length
    ];

    for key in &valid_secret_keys {
        assert!(
            validation::validate_secret_key(key).is_ok(),
            "Key '{}' should be valid",
            key
        );
    }

    // Test maximum length separately
    assert!(validation::validate_secret_key(&max_length_key).is_ok());

    // Test all invalid patterns
    let too_long_key = "a".repeat(101);
    let invalid_secret_keys = vec![
        "",  // Empty
        " ", // Whitespace only
        "key with spaces",
        "key@symbol",
        "key#hash",
        "key$dollar",
        "key%percent",
        "key&and",
        "key*star",
        "key+plus",
        "key=equals",
        "key[ bracket",
        "key] bracket",
        "key{ brace",
        "key} brace",
        "key| pipe",
        "key\\ backslash",
        "key: colon",
        "key; semicolon",
        "key\" quote",
        "key' apostrophe",
        "key< less",
        "key> greater",
        "key, comma",
        "key. dot",
        "key? question",
        "key/ slash",
        "key! exclamation",
    ];

    for key in invalid_secret_keys {
        assert!(
            validation::validate_secret_key(key).is_err(),
            "Key '{}' should be invalid",
            key
        );
    }

    // Test too long key separately
    assert!(validation::validate_secret_key(&too_long_key).is_err());
}

#[test]
fn test_render_secret_input_ui() {
    use github_secrets::prompt::{InputMode, SecretPair, render_secret_input_ui};
    use ratatui::{Terminal, backend::TestBackend, style::Color};

    // Setup terminal with TestBackend
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let secrets = vec![SecretPair {
        key: "EXISTING_KEY".to_string(),
        value: "value".to_string(),
    }];
    let current_key = "NEW_KEY";
    let current_value = "";
    let input_mode = InputMode::Key;
    let message = "Test Message";
    let message_color = Color::Yellow;

    terminal
        .draw(|f| {
            render_secret_input_ui(
                f,
                &secrets,
                current_key,
                current_value,
                &input_mode,
                message,
                message_color,
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Verify key input is shown
    // Note: Checking exact content in buffer is complex; rely on no panic and some output

    // Check for title
    let title_found = (0..buffer.area.height).any(|y| {
        let line_str: String = (0..buffer.area.width)
            .map(|x| buffer.get(x, y).symbol())
            .collect();
        line_str.contains("GitHub Secrets")
    });
    assert!(title_found, "Title not found in rendered output");

    // Check for message
    let message_found = (0..buffer.area.height).any(|y| {
        let line_str: String = (0..buffer.area.width)
            .map(|x| buffer.get(x, y).symbol())
            .collect();
        line_str.contains("Test Message")
    });
    assert!(message_found, "Message not found in rendered output");
}

#[test]
fn test_render_selection_ui() {
    use github_secrets::config::Repository;
    use github_secrets::prompt::render_selection_ui;
    use ratatui::{Terminal, backend::TestBackend, widgets::ListState};

    // Setup terminal with TestBackend
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    let repositories = vec![
        Repository {
            owner: "owner1".to_string(),
            name: "repo1".to_string(),
            alias: None,
        },
        Repository {
            owner: "owner2".to_string(),
            name: "repo2".to_string(),
            alias: Some("Alias".to_string()),
        },
    ];

    // Index 0 is "Select All", 1 is repo1, 2 is repo2
    let selected = vec![false, true, false];
    let mut list_state = ListState::default();
    list_state.select(Some(1)); // Select the first repo

    terminal
        .draw(|f| {
            render_selection_ui(f, &repositories, &selected, &mut list_state);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    // Check for repository names
    let repo1_found = (0..buffer.area.height).any(|y| {
        let line_str: String = (0..buffer.area.width)
            .map(|x| buffer.get(x, y).symbol())
            .collect();
        line_str.contains("owner1/repo1")
    });
    assert!(repo1_found, "Repo1 not found in rendered output");

    // Check for checkmark on selected repo
    let checkmark_found = (0..buffer.area.height).any(|y| {
        let line_str: String = (0..buffer.area.width)
            .map(|x| buffer.get(x, y).symbol())
            .collect();
        line_str.contains("[x] owner1/repo1")
    });
    assert!(checkmark_found, "Checkmark not found for selected repo");
}
