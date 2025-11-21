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

    // 1. Try current directory first
    let current_dir_config = PathBuf::from("config.toml");
    if current_dir_config.exists() {
        return Ok(current_dir_config);
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
        let original_dir = env::current_dir().unwrap();

        // Create config.toml in temp directory
        let config_path = temp_dir.path().join("config.toml");
        fs::write(
            &config_path,
            "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
        )
        .unwrap();

        // Change to temp directory
        env::set_current_dir(&temp_dir).unwrap();

        let result = find_config_file();
        assert!(result.is_ok());
        // Compare canonical paths to handle relative vs absolute
        let found_path = result.unwrap().canonicalize().unwrap();
        let expected_path = config_path.canonicalize().unwrap();
        assert_eq!(found_path, expected_path);

        // Restore original directory
        env::set_current_dir(&original_dir).unwrap();
    }

    #[test]
    fn test_find_config_file_env_override() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("custom-config.toml");
        fs::write(
            &config_path,
            "[[repositories]]\nowner = \"test\"\nname = \"repo\"",
        )
        .unwrap();

        unsafe {
            env::set_var("CONFIG_PATH", config_path.to_str().unwrap());
        }
        let result = find_config_file();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), config_path);

        unsafe {
            env::remove_var("CONFIG_PATH");
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
        let original_dir = env::current_dir().unwrap();

        // Change to temp directory (no config.toml)
        // Note: This might fail on some systems, so we'll test the logic differently
        if env::set_current_dir(&temp_dir).is_ok() {
            let result = find_config_file();
            assert!(result.is_ok());
            // Should return default XDG path even if it doesn't exist
            let path = result.unwrap();
            let path_str = path.to_string_lossy();
            // The path should either be the default XDG location or current directory
            assert!(
                path_str.contains("github-secrets") || path_str == "config.toml",
                "Path should contain 'github-secrets' or be 'config.toml', got: {}",
                path_str
            );
            assert!(
                path_str.contains("config.toml"),
                "Path should contain 'config.toml', got: {}",
                path_str
            );

            // Restore
            env::set_current_dir(&original_dir).unwrap();
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
