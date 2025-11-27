use github_secrets::app::{App, UpdateResult};

#[test]
fn test_update_result_new_success() {
    let result = UpdateResult::new_success("API_KEY".to_string(), "owner/repo".to_string());

    assert_eq!(result.secret_name, "API_KEY");
    assert_eq!(result.repository, "owner/repo");
    assert!(result.success);
    assert_eq!(result.error, None);
    assert!(result.is_success());
    assert!(!result.is_failure());
}

#[test]
fn test_update_result_new_failure() {
    let result = UpdateResult::new_failure(
        "API_KEY".to_string(),
        "owner/repo".to_string(),
        "Network error".to_string(),
    );

    assert_eq!(result.secret_name, "API_KEY");
    assert_eq!(result.repository, "owner/repo");
    assert!(!result.success);
    assert_eq!(result.error, Some("Network error".to_string()));
    assert!(!result.is_success());
    assert!(result.is_failure());
}

#[test]
fn test_update_result_equality() {
    let result1 = UpdateResult::new_success("KEY1".to_string(), "repo1".to_string());
    let result2 = UpdateResult::new_success("KEY1".to_string(), "repo1".to_string());
    let result3 =
        UpdateResult::new_failure("KEY1".to_string(), "repo1".to_string(), "error".to_string());

    assert_eq!(result1, result2);
    assert_ne!(result1, result3);
}

#[test]
fn test_count_results_all_success() {
    let results = vec![
        UpdateResult::new_success("KEY1".to_string(), "repo1".to_string()),
        UpdateResult::new_success("KEY2".to_string(), "repo1".to_string()),
        UpdateResult::new_success("KEY3".to_string(), "repo2".to_string()),
    ];

    let (success, failure) = App::count_results(&results);
    assert_eq!(success, 3);
    assert_eq!(failure, 0);
}

#[test]
fn test_count_results_all_failure() {
    let results = vec![
        UpdateResult::new_failure("KEY1".to_string(), "repo1".to_string(), "err1".to_string()),
        UpdateResult::new_failure("KEY2".to_string(), "repo1".to_string(), "err2".to_string()),
        UpdateResult::new_failure("KEY3".to_string(), "repo2".to_string(), "err3".to_string()),
    ];

    let (success, failure) = App::count_results(&results);
    assert_eq!(success, 0);
    assert_eq!(failure, 3);
}

#[test]
fn test_count_results_mixed() {
    let results = vec![
        UpdateResult::new_success("KEY1".to_string(), "repo1".to_string()),
        UpdateResult::new_failure("KEY2".to_string(), "repo1".to_string(), "err".to_string()),
        UpdateResult::new_success("KEY3".to_string(), "repo2".to_string()),
        UpdateResult::new_failure("KEY4".to_string(), "repo2".to_string(), "err".to_string()),
        UpdateResult::new_success("KEY5".to_string(), "repo3".to_string()),
    ];

    let (success, failure) = App::count_results(&results);
    assert_eq!(success, 3);
    assert_eq!(failure, 2);
}

#[test]
fn test_count_results_empty() {
    let results: Vec<UpdateResult> = vec![];

    let (success, failure) = App::count_results(&results);
    assert_eq!(success, 0);
    assert_eq!(failure, 0);
}

#[test]
fn test_aggregate_by_repository_single_repo() {
    let results = vec![
        UpdateResult::new_success("KEY1".to_string(), "owner/repo1".to_string()),
        UpdateResult::new_success("KEY2".to_string(), "owner/repo1".to_string()),
        UpdateResult::new_failure(
            "KEY3".to_string(),
            "owner/repo1".to_string(),
            "err".to_string(),
        ),
    ];

    let aggregated = App::aggregate_by_repository(&results);

    assert_eq!(aggregated.len(), 1);
    assert!(aggregated.contains_key("owner/repo1"));
    assert_eq!(aggregated["owner/repo1"].len(), 3);
}

#[test]
fn test_aggregate_by_repository_multiple_repos() {
    let results = vec![
        UpdateResult::new_success("KEY1".to_string(), "owner/repo1".to_string()),
        UpdateResult::new_success("KEY2".to_string(), "owner/repo2".to_string()),
        UpdateResult::new_failure(
            "KEY3".to_string(),
            "owner/repo1".to_string(),
            "err".to_string(),
        ),
        UpdateResult::new_success("KEY4".to_string(), "owner/repo3".to_string()),
        UpdateResult::new_failure(
            "KEY5".to_string(),
            "owner/repo2".to_string(),
            "err".to_string(),
        ),
    ];

    let aggregated = App::aggregate_by_repository(&results);

    assert_eq!(aggregated.len(), 3);
    assert_eq!(aggregated["owner/repo1"].len(), 2);
    assert_eq!(aggregated["owner/repo2"].len(), 2);
    assert_eq!(aggregated["owner/repo3"].len(), 1);
}

#[test]
fn test_aggregate_by_repository_preserves_order() {
    let results = vec![
        UpdateResult::new_success("KEY1".to_string(), "repo1".to_string()),
        UpdateResult::new_success("KEY2".to_string(), "repo1".to_string()),
        UpdateResult::new_success("KEY3".to_string(), "repo1".to_string()),
    ];

    let aggregated = App::aggregate_by_repository(&results);

    let repo1_results = &aggregated["repo1"];
    assert_eq!(repo1_results[0].secret_name, "KEY1");
    assert_eq!(repo1_results[1].secret_name, "KEY2");
    assert_eq!(repo1_results[2].secret_name, "KEY3");
}

#[test]
fn test_aggregate_by_repository_empty() {
    let results: Vec<UpdateResult> = vec![];

    let aggregated = App::aggregate_by_repository(&results);

    assert_eq!(aggregated.len(), 0);
}

#[test]
fn test_aggregate_by_repository_counts_per_repo() {
    let results = vec![
        UpdateResult::new_success("KEY1".to_string(), "repo1".to_string()),
        UpdateResult::new_failure("KEY2".to_string(), "repo1".to_string(), "err".to_string()),
        UpdateResult::new_success("KEY3".to_string(), "repo2".to_string()),
        UpdateResult::new_success("KEY4".to_string(), "repo2".to_string()),
    ];

    let aggregated = App::aggregate_by_repository(&results);

    // Count successes per repo
    let repo1_success = aggregated["repo1"]
        .iter()
        .filter(|r| r.is_success())
        .count();
    let repo1_failure = aggregated["repo1"]
        .iter()
        .filter(|r| r.is_failure())
        .count();

    assert_eq!(repo1_success, 1);
    assert_eq!(repo1_failure, 1);

    let repo2_success = aggregated["repo2"]
        .iter()
        .filter(|r| r.is_success())
        .count();
    let repo2_failure = aggregated["repo2"]
        .iter()
        .filter(|r| r.is_failure())
        .count();

    assert_eq!(repo2_success, 2);
    assert_eq!(repo2_failure, 0);
}
