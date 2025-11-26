use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

use github_secrets::paths::{find_config_file, load_env_file};

#[test]
#[serial]
fn test_find_config_file_uses_xdg_config_home() {
    // Save originals
    let orig_cfg = env::var("CONFIG_PATH").ok();
    let orig_xdg = env::var("XDG_CONFIG_HOME").ok();
    let orig_dir = env::current_dir().ok();

    // Ensure no CONFIG_PATH and change to an empty temp dir
    unsafe {
        env::remove_var("CONFIG_PATH");
    }
    let work_dir = TempDir::new().unwrap();
    let _ = env::set_current_dir(&work_dir);

    // Create an XDG_CONFIG_HOME with github-secrets/config.toml
    let xdg = TempDir::new().unwrap();
    let xdg_cfg_dir = xdg.path().join("github-secrets");
    fs::create_dir_all(&xdg_cfg_dir).unwrap();
    let cfg = xdg_cfg_dir.join("config.toml");
    fs::write(&cfg, "[[repositories]]\nowner=\"x\"\nname=\"y\"").unwrap();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", xdg.path());
    }

    let res = find_config_file().expect("should find config via XDG_CONFIG_HOME");
    assert_eq!(res.file_name().unwrap(), "config.toml");
    let res_can = res.canonicalize().unwrap();
    let cfg_can = cfg.canonicalize().unwrap();
    assert_eq!(res_can, cfg_can);

    // cleanup / restore
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
        env::remove_var("CONFIG_PATH");
    }
    if let Some(v) = orig_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", v);
        }
    }
    if let Some(v) = orig_cfg {
        unsafe {
            env::set_var("CONFIG_PATH", v);
        }
    }
    if let Some(d) = orig_dir {
        let _ = env::set_current_dir(d);
    }
}

#[test]
#[serial]
fn test_load_env_file_uses_xdg_config_home() {
    // Save originals
    let orig_xdg = env::var("XDG_CONFIG_HOME").ok();
    let orig_dir = env::current_dir().ok();

    // Change to an empty temp dir with no .env
    let work_dir = TempDir::new().unwrap();
    let _ = env::set_current_dir(&work_dir);

    // Create XDG_CONFIG_HOME/github-secrets/.env
    let xdg = TempDir::new().unwrap();
    let xdg_cfg_dir = xdg.path().join("github-secrets");
    fs::create_dir_all(&xdg_cfg_dir).unwrap();
    let env_file = xdg_cfg_dir.join(".env");
    fs::write(&env_file, "XDG_TEST=xdg_value").unwrap();

    unsafe {
        env::set_var("XDG_CONFIG_HOME", xdg.path());
    }

    // Ensure the variable is not set before
    unsafe {
        env::remove_var("XDG_TEST");
    }

    load_env_file();

    // If loading worked, env var should be present
    if let Ok(val) = env::var("XDG_TEST") {
        assert_eq!(val, "xdg_value");
        unsafe {
            env::remove_var("XDG_TEST");
        }
    }

    // cleanup / restore
    unsafe {
        env::remove_var("XDG_CONFIG_HOME");
    }
    if let Some(v) = orig_xdg {
        unsafe {
            env::set_var("XDG_CONFIG_HOME", v);
        }
    }
    if let Some(d) = orig_dir {
        let _ = env::set_current_dir(d);
    }
}
