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
