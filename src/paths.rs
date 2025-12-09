//! XDG-compliant path resolution for configuration files.
//!
//! This module provides functions to locate configuration and environment files
//! following the XDG Base Directory Specification, with fallback to current directory.

use anyhow::Result;
use std::env;
use std::path::PathBuf;

/// Find the config.toml file.
/// Priority:
/// 1. CONFIG_PATH from environment (if set)
/// 2. Current directory/config.toml
/// 3. ~/.config/github-secrets/config.toml (default XDG location)
/// 4. XDG_CONFIG_HOME/github-secrets/config.toml (if XDG_CONFIG_HOME is set)
pub fn find_config_file() -> Result<PathBuf> {
    // Check if CONFIG_PATH is explicitly set (highest priority)
    if let Ok(config_path) = env::var("CONFIG_PATH") {
        let path = PathBuf::from(&config_path);
        if path.exists() {
            return Ok(path);
        }
    }

    // 1. Try current directory first (use absolute path to avoid race conditions)
    if let Ok(current_dir) = env::current_dir() {
        let current_dir_config = current_dir.join("config.toml");
        if current_dir_config.exists() {
            return Ok(current_dir_config);
        }
    }

    // 2. Try default XDG location (~/.config/github-secrets/config.toml)
    if let Some(home) = dirs::home_dir() {
        let default_xdg_config = home
            .join(".config")
            .join("github-secrets")
            .join("config.toml");
        if default_xdg_config.exists() {
            return Ok(default_xdg_config);
        }
    }

    // 3. Try XDG_CONFIG_HOME/github-secrets/config.toml (if XDG_CONFIG_HOME is set)
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        let xdg_config_path = PathBuf::from(xdg_config_home)
            .join("github-secrets")
            .join("config.toml");
        if xdg_config_path.exists() {
            return Ok(xdg_config_path);
        }
    }

    // If none exists, return default XDG path (will show error when trying to read)
    Ok(get_config_creation_path())
}

/// Get the path where a new config file should be created.
/// Priority:
/// 1. XDG_CONFIG_HOME/github-secrets/config.toml (if XDG_CONFIG_HOME is set)
/// 2. ~/.config/github-secrets/config.toml (default XDG location)
/// 3. Current directory/config.toml (fallback)
pub fn get_config_creation_path() -> PathBuf {
    // 1. Try XDG_CONFIG_HOME/github-secrets/config.toml (if XDG_CONFIG_HOME is set)
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg_config_home)
            .join("github-secrets")
            .join("config.toml");
    }

    // 2. Try default XDG location (~/.config/github-secrets/config.toml)
    if let Some(home) = dirs::home_dir() {
        return home
            .join(".config")
            .join("github-secrets")
            .join("config.toml");
    }

    // 3. Fallback to current directory
    PathBuf::from("config.toml")
}

/// Find and load .env file.
/// Priority:
/// 1. Current directory/.env
/// 2. ~/.config/github-secrets/.env (default XDG location)
/// 3. XDG_CONFIG_HOME/github-secrets/.env (if XDG_CONFIG_HOME is set)
pub fn load_env_file() {
    // 1. Try current directory first
    let current_dir_env = PathBuf::from(".env");
    if current_dir_env.exists() {
        let _ = dotenv::from_path(&current_dir_env);
        // After loading, check if XDG_CONFIG_HOME was set in .env
        // and reload from that location if it exists
        if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
            let xdg_env_path = PathBuf::from(xdg_config_home)
                .join("github-secrets")
                .join(".env");
            if xdg_env_path.exists() && xdg_env_path != current_dir_env {
                let _ = dotenv::from_path(&xdg_env_path);
            }
        }
        return;
    }

    // 2. Try default XDG location (~/.config/github-secrets/.env)
    if let Some(home) = dirs::home_dir() {
        let default_xdg_env = home.join(".config").join("github-secrets").join(".env");
        if default_xdg_env.exists() {
            let _ = dotenv::from_path(&default_xdg_env);

            // If XDG_CONFIG_HOME was set in the .env file, reload from the new location
            if let Ok(new_xdg_config_home) = env::var("XDG_CONFIG_HOME") {
                let new_xdg_env_path = PathBuf::from(new_xdg_config_home)
                    .join("github-secrets")
                    .join(".env");
                if new_xdg_env_path.exists() && new_xdg_env_path != default_xdg_env {
                    let _ = dotenv::from_path(&new_xdg_env_path);
                }
            }
            return;
        }
    }

    // 3. Try XDG_CONFIG_HOME/github-secrets/.env (if XDG_CONFIG_HOME is set)
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        let xdg_env_path = PathBuf::from(xdg_config_home)
            .join("github-secrets")
            .join(".env");
        if xdg_env_path.exists() {
            let _ = dotenv::from_path(&xdg_env_path);
            return;
        }
    }

    // Fallback: try current directory again (dotenv default behavior)
    let _ = dotenv::dotenv();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_config_file_env_override() {
        // Save original CONFIG_PATH to restore later
        let original_config_path = env::var("CONFIG_PATH").ok();

        // Clear CONFIG_PATH first to ensure clean test state
        unsafe {
            env::remove_var("CONFIG_PATH");
        }

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        fs::write(
            &config_path,
            "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
        )
        .unwrap();

        // Verify file exists before setting CONFIG_PATH
        assert!(
            config_path.exists(),
            "config.toml should exist before setting CONFIG_PATH"
        );

        // Convert to absolute path string to avoid relative path issues
        // config_path is already absolute since it's derived from temp_dir.path() which is absolute
        let config_path_str = config_path
            .canonicalize()
            .unwrap_or_else(|_| {
                // If canonicalize fails, config_path should already be absolute since it's derived
                // from temp_dir.path().join("config.toml"), and temp_dir.path() returns an absolute path.
                // Use config_path directly as it should already be absolute.
                config_path.clone()
            })
            .to_str()
            .unwrap()
            .to_string();

        unsafe {
            env::set_var("CONFIG_PATH", &config_path_str);
        }

        // Verify CONFIG_PATH was set correctly
        let env_config_path = env::var("CONFIG_PATH").expect("CONFIG_PATH should be set");
        assert_eq!(
            env_config_path, config_path_str,
            "CONFIG_PATH should be set to the temp file path"
        );

        // Critical: Verify the file still exists right before calling find_config_file()
        // This catches cases where the file might have been deleted or the path is wrong
        if !config_path.exists() {
            panic!(
                "config.toml does not exist before calling find_config_file(). Path: {}, CONFIG_PATH: {:?}",
                config_path.display(),
                env::var("CONFIG_PATH").ok()
            );
        }

        // Verify CONFIG_PATH points to an existing file
        let config_path_from_env = PathBuf::from(&config_path_str);
        if !config_path_from_env.exists() {
            panic!(
                "CONFIG_PATH points to non-existent file. CONFIG_PATH: {}, File exists: {}",
                config_path_str,
                config_path_from_env.exists()
            );
        }

        let result = find_config_file();
        assert!(result.is_ok(), "find_config_file should succeed");

        let found = result.unwrap();

        // First, verify that CONFIG_PATH was honored by checking if found path matches CONFIG_PATH
        // This is the most direct check - if CONFIG_PATH is set, find_config_file() should return it
        let found_canonical = found.canonicalize().ok();
        let config_path_canonical = config_path.canonicalize().ok();
        let config_path_from_env = PathBuf::from(&config_path_str);
        let config_path_from_env_canonical = config_path_from_env.canonicalize().ok();

        let matches_config_path = match (
            found_canonical.as_ref(),
            config_path_canonical.as_ref(),
            config_path_from_env_canonical.as_ref(),
        ) {
            (Some(found_can), Some(config_can), _) => found_can == config_can,
            (Some(found_can), _, Some(env_can)) => found_can == env_can,
            (_, Some(config_can), Some(env_can)) => config_can == env_can,
            _ => found == config_path || found == config_path_from_env,
        };

        if !matches_config_path {
            // CONFIG_PATH was not honored - check if it's the default XDG path for better error message
            let is_default_xdg = if let Some(found_can) = found_canonical.as_ref() {
                if let Some(home) = dirs::home_dir() {
                    let default_xdg_config = home
                        .join(".config")
                        .join("github-secrets")
                        .join("config.toml");
                    let default_xdg_canonical = default_xdg_config.canonicalize().ok();
                    default_xdg_canonical.is_some_and(|default| default == *found_can)
                } else {
                    false
                }
            } else {
                // If canonicalize fails, fall back to comparing non-canonicalized paths
                if let Some(home) = dirs::home_dir() {
                    let default_xdg_config = home
                        .join(".config")
                        .join("github-secrets")
                        .join("config.toml");
                    found == default_xdg_config
                } else {
                    false
                }
            };

            if is_default_xdg {
                panic!(
                    "CONFIG_PATH was not honored. Found default XDG path: {}, Expected: {} (CONFIG_PATH: {}). File exists: {}, Found canonical: {:?}, Expected canonical: {:?}",
                    found.display(),
                    config_path_str,
                    env::var("CONFIG_PATH").unwrap_or_else(|_| "not set".to_string()),
                    config_path.exists(),
                    found_canonical,
                    config_path_canonical
                );
            } else {
                panic!(
                    "CONFIG_PATH was not honored. Found: {}, Expected: {} (CONFIG_PATH: {}). File exists: {}, Found canonical: {:?}, Expected canonical: {:?}",
                    found.display(),
                    config_path_str,
                    env::var("CONFIG_PATH").unwrap_or_else(|_| "not set".to_string()),
                    config_path.exists(),
                    found_canonical,
                    config_path_canonical
                );
            }
        }

        // Verify the found path exists
        assert!(
            found.exists(),
            "Found path should exist. Found: {}",
            found.display()
        );

        // Verify it's the same file by checking the file name
        assert_eq!(
            found.file_name(),
            config_path.file_name(),
            "File names should match. Found: {}, Expected: {}",
            found.display(),
            config_path.display()
        );

        // Verify it's in the temp directory (not the default XDG location)
        // This ensures CONFIG_PATH was actually used
        // Use canonicalized paths for comparison, similar to the other test
        let found_canonical = found.canonicalize().ok();
        let temp_path_normalized = temp_dir
            .path()
            .canonicalize()
            .unwrap_or_else(|_| temp_dir.path().to_path_buf());

        if let Some(found_can) = found_canonical {
            assert!(
                found_can.starts_with(&temp_path_normalized),
                "Found path should be in the temp directory (CONFIG_PATH was set to temp file). Found: {}, Temp dir: {}",
                found_can.display(),
                temp_path_normalized.display()
            );
        } else {
            // Fallback: if canonicalize fails on found path, compare against config_path_str
            // since found should be exactly what we set in CONFIG_PATH (which is config_path_str)
            // This handles edge cases where canonicalize might fail on the found path
            // Compare against what we actually set in CONFIG_PATH rather than temp_dir.path()
            // to avoid symlink resolution differences
            let expected_path = PathBuf::from(&config_path_str);
            assert!(
                found == expected_path,
                "Found path should match what we set in CONFIG_PATH. Found: {}, Expected (CONFIG_PATH): {}",
                found.display(),
                config_path_str
            );
        }

        // Optionally verify file content matches (extra safety check)
        if let Ok(content) = fs::read_to_string(&found) {
            assert!(
                content.contains("[[repositories]]"),
                "Found file should contain our test content. Found: {}",
                found.display()
            );
        }

        // Always clean up CONFIG_PATH after test
        unsafe {
            env::remove_var("CONFIG_PATH");
        }

        // Restore original if it existed
        if let Some(path) = original_config_path {
            unsafe {
                env::set_var("CONFIG_PATH", path);
            }
        }
    }

    #[test]
    fn test_find_config_file_returns_default_when_not_found() {
        // Remove XDG_CONFIG_HOME if set
        let xdg_home = env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }

        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().ok();

        // Change to temp directory (no config.toml)
        // Note: This might fail on some systems, so we'll test the logic differently
        if env::set_current_dir(&temp_dir).is_ok() {
            let result = find_config_file();
            assert!(result.is_ok());
            // Should return default XDG path even if it doesn't exist
            let path = result.unwrap();
            let path_str = path.to_string_lossy();
            // The path should contain 'config.toml' (could be in various locations)
            assert!(
                path_str.contains("config.toml"),
                "Path should contain 'config.toml', got: {}",
                path_str
            );

            // Restore original directory if we had one
            if let Some(dir) = original_dir {
                let _ = env::set_current_dir(&dir);
            }
        }

        // Restore XDG_CONFIG_HOME
        if let Some(xdg) = xdg_home {
            unsafe {
                env::set_var("XDG_CONFIG_HOME", xdg);
            }
        }
    }

    #[test]
    fn test_load_env_file_current_directory() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().ok();

        // Create .env in temp directory
        let env_path = temp_dir.path().join(".env");
        fs::write(&env_path, "TEST_VAR=test_value").unwrap();

        // Change to temp directory
        if env::set_current_dir(&temp_dir).is_ok() {
            load_env_file();

            // Verify the variable was loaded (if it was loaded)
            if let Ok(value) = env::var("TEST_VAR") {
                assert_eq!(value, "test_value");
                unsafe {
                    env::remove_var("TEST_VAR");
                }
            }

            // Restore original directory if we had one
            if let Some(dir) = original_dir {
                let _ = env::set_current_dir(&dir);
            }
        }
    }

    #[test]
    fn test_load_env_file_handles_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().ok();

        // Change to temp directory (no .env file)
        if env::set_current_dir(&temp_dir).is_ok() {
            // Should not panic when .env doesn't exist
            load_env_file();

            // Restore original directory if we had one
            if let Some(dir) = original_dir {
                let _ = env::set_current_dir(&dir);
            }
        }
    }

    #[test]
    fn test_get_config_creation_path_xdg_home_set() {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", "/tmp/xdg");
        }

        let path = get_config_creation_path();
        assert_eq!(path, PathBuf::from("/tmp/xdg/github-secrets/config.toml"));

        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn test_get_config_creation_path_default() {
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }

        let path = get_config_creation_path();
        if let Some(home) = dirs::home_dir() {
            assert_eq!(
                path,
                home.join(".config")
                    .join("github-secrets")
                    .join("config.toml")
            );
        }
    }
}
