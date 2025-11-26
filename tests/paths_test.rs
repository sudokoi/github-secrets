use github_secrets::paths::find_config_file;
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
