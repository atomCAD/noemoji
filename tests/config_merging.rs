// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file,
// You can obtain one at <https://mozilla.org/MPL/2.0/>.

use noemoji::config::{Config, LogConfig};
use noemoji::logging::LogLevel;

#[test]
fn config_or_none_base_none_other_returns_none() {
    let base = Config {
        log: LogConfig { level: None },
        inherit: true,
    };
    let other = Config {
        log: LogConfig { level: None },
        inherit: true,
    };

    let result = base.or(other);
    assert_eq!(result.log.level, None);
    assert!(result.inherit);
}

#[test]
fn config_or_none_base_some_other_returns_other() {
    let base = Config {
        log: LogConfig { level: None },
        inherit: true,
    };
    let other = Config {
        log: LogConfig {
            level: Some(LogLevel::Debug),
        },
        inherit: false,
    };

    let result = base.or(other);
    // base.level is None, so falls back to other.level
    assert_eq!(result.log.level, Some(LogLevel::Debug));
    // inherit comes from other (the fallback)
    assert!(!result.inherit);
}

#[test]
fn config_or_some_base_none_other_returns_base() {
    let base = Config {
        log: LogConfig {
            level: Some(LogLevel::Error),
        },
        inherit: false,
    };
    let other = Config {
        log: LogConfig { level: None },
        inherit: true,
    };

    let result = base.or(other);
    // base.level is Some, so it takes precedence
    assert_eq!(result.log.level, Some(LogLevel::Error));
    // inherit comes from other (the fallback)
    assert!(result.inherit);
}

#[test]
fn config_or_some_base_some_other_returns_base() {
    let base = Config {
        log: LogConfig {
            level: Some(LogLevel::Error),
        },
        inherit: false,
    };
    let other = Config {
        log: LogConfig {
            level: Some(LogLevel::Debug),
        },
        inherit: true,
    };

    let result = base.or(other);
    // base.level is Some, so it takes precedence
    assert_eq!(result.log.level, Some(LogLevel::Error));
    // inherit comes from other (the fallback)
    assert!(result.inherit);
}

#[test]
fn config_load_finds_multiple_configs_and_merges() {
    use std::fs;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let parent_dir = temp_dir.path();
    let child_dir = parent_dir.join("subdir");
    fs::create_dir_all(&child_dir).unwrap();

    // Parent config has log level and inherit = false (stop here)
    let parent_config = r#"
inherit = false

[log]
level = "error"
"#;
    fs::write(parent_dir.join(".noemoji.toml"), parent_config).unwrap();

    // Child config has no log level and inherit = true (default)
    let child_config = r#"
inherit = true
"#;
    fs::write(child_dir.join(".noemoji.toml"), child_config).unwrap();

    let result = Config::load_from(child_dir).unwrap();

    // Parent has inherit = false, so scanning stopped there
    // Child is merged with parent; parent provides log level
    // inherit = false indicates search was terminated early
    assert_eq!(result.log.level, Some(LogLevel::Error));
    assert!(!result.inherit);
}

#[test]
fn config_load_inherit_false_stops_directory_traversal() {
    use std::fs;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let parent_dir = temp_dir.path();
    let child_dir = parent_dir.join("subdir");
    fs::create_dir_all(&child_dir).unwrap();

    // Parent config
    let parent_config = r#"
[log]
level = "error"
"#;
    fs::write(parent_dir.join(".noemoji.toml"), parent_config).unwrap();

    // Child config with inherit = false
    let child_config = r#"
inherit = false

[log]
level = "debug"
"#;
    fs::write(child_dir.join(".noemoji.toml"), child_config).unwrap();

    let result = Config::load_from(child_dir).unwrap();

    // Child has inherit = false, so parent is not scanned
    // Result is just child config merged with default
    // inherit = false indicates search was terminated early
    assert_eq!(result.log.level, Some(LogLevel::Debug));
    assert!(!result.inherit);
}

#[test]
fn config_load_default_base_when_no_configs_found() {
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let empty_dir = temp_dir.path();

    let result = Config::load_from(empty_dir).unwrap();

    // Should get default config
    assert_eq!(result, Config::default());
}

#[test]
fn config_load_partial_configurations_override_base_values() {
    use std::fs;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let parent_dir = temp_dir.path();
    let child_dir = parent_dir.join("subdir");
    fs::create_dir_all(&child_dir).unwrap();

    // Parent config has log level
    let parent_config = r#"
[log]
level = "error"
"#;
    fs::write(parent_dir.join(".noemoji.toml"), parent_config).unwrap();

    // Child config has only inherit setting (no log level)
    let child_config = r#"
inherit = false
"#;
    fs::write(child_dir.join(".noemoji.toml"), child_config).unwrap();

    let result = Config::load_from(child_dir).unwrap();

    // Child has inherit = false, so parent is NOT scanned
    // Result is just child config merged with default (no log level)
    // inherit = false indicates search was terminated early
    assert_eq!(result.log.level, None);
    assert!(!result.inherit);
}

#[test]
fn config_load_merging_applies_general_to_specific() {
    use std::fs;
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let grandparent_dir = temp_dir.path();
    let parent_dir = grandparent_dir.join("parent");
    let child_dir = parent_dir.join("child");
    fs::create_dir_all(&child_dir).unwrap();

    // Grandparent config
    let grandparent_config = r#"
[log]
level = "error"
"#;
    fs::write(grandparent_dir.join(".noemoji.toml"), grandparent_config).unwrap();

    // Parent config overrides log level
    let parent_config = r#"
[log]
level = "warn"
"#;
    fs::write(parent_dir.join(".noemoji.toml"), parent_config).unwrap();

    // Child config overrides log level again
    let child_config = r#"
[log]
level = "debug"
"#;
    fs::write(child_dir.join(".noemoji.toml"), child_config).unwrap();

    let result = Config::load_from(child_dir).unwrap();

    // Should get the most specific (child) config value
    assert_eq!(result.log.level, Some(LogLevel::Debug));
}
