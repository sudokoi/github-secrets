use github_secrets::paths::{find_config_file, load_env_file};
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
#[serial]
fn test_find_config_file_returns_path() {
    // This test just verifies the function returns a valid PathBuf
    let result = find_config_file();
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("config.toml"));
}

#[test]
#[serial]
fn test_find_config_file_with_config_path_env() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    fs::write(
        &config_path,
        "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
    )
    .unwrap();

    let original = env::var("CONFIG_PATH").ok();

    unsafe {
        env::set_var("CONFIG_PATH", config_path.to_str().unwrap());
    }

    let result = find_config_file();
    assert!(result.is_ok());

    let found = result.unwrap();
    assert!(found.exists());
    assert_eq!(found, config_path);

    // Cleanup
    unsafe {
        if let Some(val) = original {
            env::set_var("CONFIG_PATH", val);
        } else {
            env::remove_var("CONFIG_PATH");
        }
    }
}

#[test]
#[serial]
fn test_find_config_file_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().ok();
    let original_config_path = env::var("CONFIG_PATH").ok();

    // Clear CONFIG_PATH to ensure we test current directory priority
    unsafe {
        env::remove_var("CONFIG_PATH");
    }

    let config_path = temp_dir.path().join("config.toml");
    fs::write(
        &config_path,
        "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
    )
    .unwrap();

    let _ = env::set_current_dir(&temp_dir);

    let result = find_config_file();
    assert!(result.is_ok());

    let found = result.unwrap();
    assert!(found.exists());
    assert_eq!(found.file_name().unwrap(), "config.toml");

    // Cleanup
    if let Some(dir) = original_dir {
        let _ = env::set_current_dir(&dir);
    }

    unsafe {
        if let Some(val) = original_config_path {
            env::set_var("CONFIG_PATH", val);
        }
    }
}

#[test]
#[serial]
fn test_find_config_file_xdg_config_home() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_config_home = temp_dir.path().join("config");
    let app_config_dir = xdg_config_home.join("github-secrets");
    fs::create_dir_all(&app_config_dir).unwrap();

    let config_path = app_config_dir.join("config.toml");
    fs::write(
        &config_path,
        "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
    )
    .unwrap();

    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    let original_config_path = env::var("CONFIG_PATH").ok();

    unsafe {
        env::remove_var("CONFIG_PATH");
        env::set_var("XDG_CONFIG_HOME", xdg_config_home.to_str().unwrap());
    }

    // Ensure we are not in a directory with config.toml
    let temp_cwd = TempDir::new().unwrap();
    let original_cwd = env::current_dir().ok();
    let _ = env::set_current_dir(&temp_cwd);

    let result = find_config_file();
    assert!(result.is_ok());

    let found = result.unwrap();
    assert!(found.exists());
    assert_eq!(found, config_path);

    // Cleanup
    if let Some(cwd) = original_cwd {
        let _ = env::set_current_dir(cwd);
    }

    unsafe {
        if let Some(val) = original_xdg {
            env::set_var("XDG_CONFIG_HOME", val);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }

        if let Some(val) = original_config_path {
            env::set_var("CONFIG_PATH", val);
        }
    }
}

#[test]
#[serial]
fn test_load_env_file_priority() {
    let temp_dir = TempDir::new().unwrap();
    let original_dir = env::current_dir().ok();

    // Create .env in current directory
    let env_path = temp_dir.path().join(".env");
    fs::write(&env_path, "TEST_VAR=current_dir").unwrap();

    let _ = env::set_current_dir(&temp_dir);

    // Should load from current directory
    load_env_file();
    assert_eq!(env::var("TEST_VAR").unwrap(), "current_dir");

    // Cleanup
    unsafe {
        env::remove_var("TEST_VAR");
    }
    if let Some(dir) = original_dir {
        let _ = env::set_current_dir(dir);
    }
}

#[test]
#[serial]
fn test_load_env_file_uses_xdg_config_home() {
    let temp_dir = TempDir::new().unwrap();
    let xdg_config_home = temp_dir.path().join("config");
    let app_config_dir = xdg_config_home.join("github-secrets");
    fs::create_dir_all(&app_config_dir).unwrap();

    let env_path = app_config_dir.join(".env");
    fs::write(&env_path, "TEST_VAR=xdg_home").unwrap();

    // Ensure we are not in a directory with .env
    let temp_cwd = TempDir::new().unwrap();
    let original_cwd = env::current_dir().ok();
    let _ = env::set_current_dir(&temp_cwd);

    let original_xdg = env::var("XDG_CONFIG_HOME").ok();
    unsafe {
        env::set_var("XDG_CONFIG_HOME", xdg_config_home.to_str().unwrap());
    }

    load_env_file();
    assert_eq!(env::var("TEST_VAR").unwrap(), "xdg_home");

    // Cleanup
    unsafe {
        env::remove_var("TEST_VAR");
    }
    if let Some(cwd) = original_cwd {
        let _ = env::set_current_dir(cwd);
    }
    unsafe {
        if let Some(val) = original_xdg {
            env::set_var("XDG_CONFIG_HOME", val);
        } else {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
