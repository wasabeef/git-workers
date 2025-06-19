use git_workers::config::{Config, RepositoryConfig};
use std::collections::HashMap;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert!(config.hooks.is_empty());
    assert!(config.repository.url.is_none());
}

#[test]
fn test_config_with_hooks() {
    let mut hooks = HashMap::new();
    hooks.insert(
        "post-create".to_string(),
        vec!["echo 'Created'".to_string()],
    );

    let config = Config {
        repository: RepositoryConfig::default(),
        hooks,
    };

    assert_eq!(config.hooks.len(), 1);
    assert!(config.hooks.contains_key("post-create"));
    assert_eq!(config.hooks["post-create"], vec!["echo 'Created'"]);
}

#[test]
fn test_config_with_repository() {
    let config = Config {
        repository: RepositoryConfig {
            url: Some("https://github.com/owner/repo.git".to_string()),
        },
        hooks: HashMap::new(),
    };

    assert_eq!(
        config.repository.url,
        Some("https://github.com/owner/repo.git".to_string())
    );
}
