use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

use github_secrets::paths::find_config_file;

#[test]
#[serial]
fn test_find_config_file_default_xdg_home() {
    // Save original home env var and restore later
    let orig_home = env::var("USERPROFILE")
        .ok()
        .or_else(|| env::var("HOME").ok());

    let temp_home = TempDir::new().unwrap();
    // Create ~/.config/github-secrets/config.toml inside temp_home
    let cfg_dir = temp_home.path().join(".config").join("github-secrets");
    fs::create_dir_all(&cfg_dir).unwrap();
    let cfg_file = cfg_dir.join("config.toml");
    fs::write(&cfg_file, "[[repositories]]\nowner=\"x\"\nname=\"y\"").unwrap();

    // Point USERPROFILE (Windows) and HOME (Unix) to temp_home to influence dirs::home_dir()
    unsafe {
        env::set_var("USERPROFILE", temp_home.path());
    }
    unsafe {
        env::set_var("HOME", temp_home.path());
    }
    // Also set XDG_CONFIG_HOME to temp_home so the explicit XDG branch is exercised
    unsafe {
        env::set_var("XDG_CONFIG_HOME", temp_home.path().join(".config"));
    }

    // Change current directory to temp_home so current-dir lookup doesn't shadow XDG lookup
    let orig_dir = std::env::current_dir().ok();
    let _ = env::set_current_dir(&temp_home);

    let found = find_config_file().expect("should find config in default XDG location");
    let found_can = found.canonicalize().unwrap();
    let cfg_can = cfg_file.canonicalize().unwrap();
    assert_eq!(found_can, cfg_can);

    // Restore original
    // Restore original working directory and env
    if let Some(d) = orig_dir {
        let _ = env::set_current_dir(d);
    }

    if let Some(v) = orig_home {
        unsafe {
            env::set_var("USERPROFILE", v.clone());
        }
        unsafe {
            env::set_var("HOME", v);
        }
    } else {
        unsafe {
            env::remove_var("USERPROFILE");
        }
        unsafe {
            env::remove_var("HOME");
        }
    }

    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
}
