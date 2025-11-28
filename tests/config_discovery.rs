// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

//! Integration tests for hierarchical configuration file discovery

use std::{
    fs::{self, File},
    io::Write,
};

use noemoji::config::Config;
use tempfile::TempDir;

#[test]
fn load_config_finds_file_in_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".noemoji.toml");

    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "[log]").unwrap();
    writeln!(file, "level = \"debug\"").unwrap();

    let result = Config::load_from(temp_dir.path());

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.log.level, Some(noemoji::logging::LogLevel::Debug));
}

#[test]
fn load_config_searches_parent_directories() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".noemoji.toml");

    // Create config in parent directory
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "[log]").unwrap();
    writeln!(file, "level = \"info\"").unwrap();

    // Create subdirectory
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();

    let result = Config::load_from(sub_dir);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.log.level, Some(noemoji::logging::LogLevel::Info));
}

#[test]
fn load_config_searches_up_to_filesystem_root() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".noemoji.toml");

    // Create config in temp directory
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "[log]").unwrap();
    writeln!(file, "level = \"warn\"").unwrap();

    // Create deep subdirectory structure
    let deep_dir = temp_dir.path().join("a").join("b").join("c").join("d");
    fs::create_dir_all(&deep_dir).unwrap();

    let result = Config::load_from(deep_dir);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config.log.level, Some(noemoji::logging::LogLevel::Warn));
}

#[test]
fn load_config_returns_default_when_no_file_found() {
    // Create a temporary directory that definitely won't have .noemoji.toml
    let temp_dir = TempDir::new().unwrap();
    let sub_dir = temp_dir.path().join("no_config_here");
    fs::create_dir(&sub_dir).unwrap();

    let result = Config::load_from(sub_dir);

    assert!(result.is_ok());
    let config = result.unwrap();
    assert_eq!(config, Config::default());
}

#[test]
#[cfg(unix)] // Permission tests only work on Unix-like systems
fn load_config_handles_permission_errors() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".noemoji.toml");

    // Create config file
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "[log]").unwrap();
    writeln!(file, "level = \"debug\"").unwrap();
    drop(file);

    // Remove read permissions
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&config_path).unwrap().permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(&config_path, perms).unwrap();
    }

    let result = Config::load_from(temp_dir.path());

    // Restore permissions for cleanup
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&config_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o644);
            let _ = fs::set_permissions(&config_path, perms);
        }
    }

    // Should return error for permission denied
    assert!(result.is_err());
}

#[test]
fn load_config_handles_invalid_toml() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".noemoji.toml");

    // Create invalid TOML
    let mut file = File::create(&config_path).unwrap();
    writeln!(file, "[log").unwrap(); // Missing closing bracket
    writeln!(file, "level = debug").unwrap(); // Missing quotes

    let result = Config::load_from(temp_dir.path());

    // Should return error for invalid TOML
    assert!(result.is_err());
}

#[test]
fn load_config_prefers_closer_config_file() {
    let temp_dir = TempDir::new().unwrap();

    // Create config in parent
    let parent_config = temp_dir.path().join(".noemoji.toml");
    let mut file = File::create(&parent_config).unwrap();
    writeln!(file, "[log]").unwrap();
    writeln!(file, "level = \"info\"").unwrap();

    // Create subdirectory with different config
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    let sub_config = sub_dir.join(".noemoji.toml");
    let mut file = File::create(&sub_config).unwrap();
    writeln!(file, "[log]").unwrap();
    writeln!(file, "level = \"debug\"").unwrap();

    let result = Config::load_from(sub_dir);

    assert!(result.is_ok());
    let config = result.unwrap();
    // Should find the closer config with debug level, not the parent with info level
    assert_eq!(config.log.level, Some(noemoji::logging::LogLevel::Debug));
}

// EOF
