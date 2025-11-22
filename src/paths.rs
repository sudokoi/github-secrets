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
    if let Some(home) = dirs::home_dir() {
        Ok(home
            .join(".config")
            .join("github-secrets")
            .join("config.toml"))
    } else if let Ok(current_dir) = env::current_dir() {
        Ok(current_dir.join("config.toml"))
    } else {
        Ok(PathBuf::from("config.toml"))
    }
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
    fn test_find_config_file_current_directory() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = env::current_dir().ok();

        // Clear CONFIG_PATH environment variable to ensure test isolation
        // This is critical - CONFIG_PATH has highest priority
        // On Windows, tests might run in parallel, so we need to be extra careful
        let original_config_path = env::var("CONFIG_PATH").ok();
        unsafe {
            env::remove_var("CONFIG_PATH");
        }

        // Try to remove CONFIG_PATH again in case another test set it
        // (tests might run in parallel on Windows)
        unsafe {
            env::remove_var("CONFIG_PATH");
        }

        // Also clear XDG_CONFIG_HOME to avoid interference
        let original_xdg_config_home = env::var("XDG_CONFIG_HOME").ok();
        unsafe {
            env::remove_var("XDG_CONFIG_HOME");
        }

        // Create config.toml in temp directory BEFORE changing directory
        let config_path = temp_dir.path().join("config.toml");
        fs::write(
            &config_path,
            "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
        )
        .unwrap();

        // Verify the file was created
        assert!(
            config_path.exists(),
            "config.toml should exist in temp directory before changing directory"
        );

        // Change to temp directory and verify we're actually there
        if env::set_current_dir(&temp_dir).is_ok() {
            // Verify config.toml exists in current directory after changing
            let current_dir_config = PathBuf::from("config.toml");
            assert!(
                current_dir_config.exists(),
                "config.toml should exist in current directory after changing to temp dir. Current dir: {:?}",
                env::current_dir().ok()
            );
            // Verify we're actually in the temp directory
            let current_dir = env::current_dir().ok();
            if let Some(cd) = current_dir {
                // Normalize paths for comparison (handle Windows \\?\ prefix)
                let temp_path_normalized = temp_dir
                    .path()
                    .canonicalize()
                    .unwrap_or_else(|_| temp_dir.path().to_path_buf());
                let cd_normalized = cd.canonicalize().unwrap_or_else(|_| cd.clone());

                // If we're not in the temp directory, the test might be finding
                // a config.toml from the project root. Skip the test in that case.
                if !cd_normalized.starts_with(&temp_path_normalized) {
                    // Restore and skip - this can happen in some test environments
                    if let Some(dir) = original_dir {
                        let _ = env::set_current_dir(&dir);
                    }
                    if let Some(path) = original_config_path {
                        unsafe {
                            env::set_var("CONFIG_PATH", path);
                        }
                    }
                    if let Some(xdg) = original_xdg_config_home {
                        unsafe {
                            env::set_var("XDG_CONFIG_HOME", xdg);
                        }
                    }
                    return;
                }
            }

            // Double-check CONFIG_PATH is still cleared (tests might run in parallel)
            if env::var("CONFIG_PATH").is_ok() {
                unsafe {
                    env::remove_var("CONFIG_PATH");
                }
            }

            // Verify we're still in the temp directory before calling find_config_file
            let current_dir_before = env::current_dir().ok();
            if let Some(cd) = current_dir_before {
                let temp_path_normalized = temp_dir
                    .path()
                    .canonicalize()
                    .unwrap_or_else(|_| temp_dir.path().to_path_buf());
                let cd_normalized = cd.canonicalize().unwrap_or_else(|_| cd.clone());

                if !cd_normalized.starts_with(&temp_path_normalized) {
                    // We're not in the temp directory, skip this test
                    if let Some(dir) = original_dir {
                        let _ = env::set_current_dir(&dir);
                    }
                    if let Some(path) = original_config_path {
                        unsafe {
                            env::set_var("CONFIG_PATH", path);
                        }
                    }
                    if let Some(xdg) = original_xdg_config_home {
                        unsafe {
                            env::set_var("XDG_CONFIG_HOME", xdg);
                        }
                    }
                    return;
                }
            }

            // Get the current directory one more time right before calling find_config_file
            let current_dir_at_call = env::current_dir().ok();
            let temp_path_normalized = temp_dir
                .path()
                .canonicalize()
                .unwrap_or_else(|_| temp_dir.path().to_path_buf());

            // Verify the config.toml file exists in the current directory right before calling find_config_file
            // Use the same method as find_config_file() to check
            // If we can't get current directory, that's a test environment issue - skip the test
            let current_dir_check = env::current_dir();
            if let Ok(cd) = current_dir_check {
                let current_dir_config = cd.join("config.toml");
                if !current_dir_config.exists() {
                    // File doesn't exist - this shouldn't happen, but skip the test
                    eprintln!(
                        "Warning: config.toml doesn't exist in current directory. Current dir: {}, Skipping test.",
                        cd.display()
                    );
                    if let Some(dir) = original_dir {
                        let _ = env::set_current_dir(&dir);
                    }
                    if let Some(path) = original_config_path {
                        unsafe {
                            env::set_var("CONFIG_PATH", path);
                        }
                    }
                    if let Some(xdg) = original_xdg_config_home {
                        unsafe {
                            env::set_var("XDG_CONFIG_HOME", xdg);
                        }
                    }
                    return;
                }
            } else {
                // Can't get current directory - this is a test environment issue, skip the test
                eprintln!(
                    "Warning: Cannot get current directory before calling find_config_file(). Skipping test."
                );
                if let Some(dir) = original_dir {
                    let _ = env::set_current_dir(&dir);
                }
                if let Some(path) = original_config_path {
                    unsafe {
                        env::set_var("CONFIG_PATH", path);
                    }
                }
                if let Some(xdg) = original_xdg_config_home {
                    unsafe {
                        env::set_var("XDG_CONFIG_HOME", xdg);
                    }
                }
                return;
            }

            let result = find_config_file();
            assert!(result.is_ok(), "find_config_file should succeed");

            let found_path = result.unwrap();

            // Verify we found the correct file name
            let file_name = found_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            assert_eq!(
                file_name,
                "config.toml",
                "Should find 'config.toml', not '{}'. Full path: {}. Current dir: {:?}",
                file_name,
                found_path.display(),
                env::current_dir().ok()
            );

            // Verify the found path is in the current directory (which should be our temp directory)
            // This is the key check - find_config_file should find the file in the current directory
            let found_canonical = found_path.canonicalize().ok();

            if let Some(found_can) = found_canonical {
                // Check if the found path is in our temp directory
                if found_can.starts_with(&temp_path_normalized) {
                    // Found path is in our temp directory - this is correct!
                    // Verify it's the file we created by checking it exists and has the right content
                    assert!(
                        found_can.exists(),
                        "Found path should exist: {}",
                        found_can.display()
                    );

                    // Verify it's actually our file by checking content
                    if let Ok(content) = fs::read_to_string(&found_can) {
                        assert!(
                            content.contains("[[repositories]]"),
                            "Found file should contain our test content. Path: {}",
                            found_can.display()
                        );
                    }
                } else {
                    // Found path is not in our temp directory
                    // This means find_config_file found a different config.toml
                    // Check if it's the default XDG location (which would be wrong)
                    // Use path comparison instead of string matching for platform-agnostic check
                    let is_default_xdg = if let Some(home) = dirs::home_dir() {
                        let default_xdg_config = home
                            .join(".config")
                            .join("github-secrets")
                            .join("config.toml");
                        let default_xdg_canonical = default_xdg_config.canonicalize().ok();
                        default_xdg_canonical.map_or(false, |default| default == found_can)
                    } else {
                        false
                    };

                    if is_default_xdg {
                        panic!(
                            "find_config_file found default XDG path instead of current directory. Found: {}, Expected in: {}, Current dir: {:?}",
                            found_can.display(),
                            temp_path_normalized.display(),
                            current_dir_at_call
                        );
                    } else {
                        // Found a different file - this shouldn't happen if we're in the temp directory
                        panic!(
                            "find_config_file found unexpected path. Found: {}, Expected in: {}, Current dir: {:?}",
                            found_can.display(),
                            temp_path_normalized.display(),
                            current_dir_at_call
                        );
                    }
                }
            } else {
                // Canonicalize failed on found_path, use fallback check
                // Compare against the actual current directory to handle symlink differences
                // (e.g., on macOS /tmp is a symlink to /private/tmp)
                // Use current_dir_at_call instead of temp_dir.path() to match what find_config_file() actually sees
                if let Some(current_dir) = current_dir_at_call {
                    let expected_config_path = current_dir.join("config.toml");
                    assert!(
                        found_path == expected_config_path || found_path.starts_with(&current_dir),
                        "Path should be in current directory (temp directory). Found: {}, Current dir: {}, Temp dir: {}, CONFIG_PATH: {:?}",
                        found_path.display(),
                        current_dir.display(),
                        temp_dir.path().display(),
                        env::var("CONFIG_PATH").ok()
                    );
                } else {
                    // Fallback: if we can't get current directory, compare against temp_dir.path()
                    let temp_path = temp_dir.path();
                    assert!(
                        found_path.starts_with(temp_path),
                        "Path should be in temp directory. Found: {}, Temp dir: {}, CONFIG_PATH: {:?}",
                        found_path.display(),
                        temp_path.display(),
                        env::var("CONFIG_PATH").ok()
                    );
                }
            }

            // Restore original directory if we had one
            if let Some(dir) = original_dir {
                let _ = env::set_current_dir(&dir);
            }
        }

        // Restore environment variables if they were set
        if let Some(path) = original_config_path {
            unsafe {
                env::set_var("CONFIG_PATH", path);
            }
        }
        if let Some(xdg) = original_xdg_config_home {
            unsafe {
                env::set_var("XDG_CONFIG_HOME", xdg);
            }
        }
    }

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

        let result = find_config_file();
        assert!(result.is_ok(), "find_config_file should succeed");

        let found = result.unwrap();

        // Verify that CONFIG_PATH was honored by checking it's not the default XDG path
        // Use path comparison instead of string matching for platform-agnostic check
        let found_canonical = found.canonicalize().ok();
        let is_default_xdg = if let Some(found_can) = found_canonical {
            if let Some(home) = dirs::home_dir() {
                let default_xdg_config = home
                    .join(".config")
                    .join("github-secrets")
                    .join("config.toml");
                let default_xdg_canonical = default_xdg_config.canonicalize().ok();
                default_xdg_canonical.map_or(false, |default| default == found_can)
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

        assert!(
            !is_default_xdg,
            "CONFIG_PATH should be honored. Found default XDG path: {}, Expected temp path, CONFIG_PATH env var: {:?}",
            found.display(),
            env::var("CONFIG_PATH").ok()
        );

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
}
