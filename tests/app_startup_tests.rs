use serial_test::serial;
use std::env;

#[tokio::test]
#[serial]
async fn test_run_fails_on_empty_token() {
    // Preserve existing value
    let orig = env::var_os("GITHUB_TOKEN");
    unsafe {
        env::set_var("GITHUB_TOKEN", "");
    }

    let res = github_secrets::app::App::run().await;

    // Restore
    match orig {
        Some(v) => unsafe { env::set_var("GITHUB_TOKEN", v) },
        None => unsafe { env::remove_var("GITHUB_TOKEN") },
    }

    assert!(res.is_err());
    let msg = format!("{}", res.unwrap_err());
    assert!(msg.contains("Invalid GitHub token format"));
}

#[tokio::test]
#[serial]
async fn test_run_fails_on_invalid_token_format() {
    // Preserve existing value
    let orig = env::var_os("GITHUB_TOKEN");
    unsafe {
        env::set_var("GITHUB_TOKEN", "invalid_token");
    }

    let res = github_secrets::app::App::run().await;

    // Restore
    match orig {
        Some(v) => unsafe { env::set_var("GITHUB_TOKEN", v) },
        None => unsafe { env::remove_var("GITHUB_TOKEN") },
    }

    assert!(res.is_err());
    let msg = format!("{}", res.unwrap_err());
    assert!(msg.contains("Invalid GitHub token format"));
}
