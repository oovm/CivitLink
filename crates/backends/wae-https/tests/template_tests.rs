use std::path::PathBuf;
use wae_https::template::*;

#[test]
fn test_template_config_default() {
    let config = TemplateConfig::default();
    assert_eq!(config.template_dir, PathBuf::from("templates"));
    assert_eq!(config.extension, "html");
    assert!(config.enable_cache);
}

#[test]
fn test_template_config_builder() {
    let config = TemplateConfig::new().with_template_dir("views").with_extension("tmpl").with_cache(false);

    assert_eq!(config.template_dir, PathBuf::from("views"));
    assert_eq!(config.extension, "tmpl");
    assert!(!config.enable_cache);
}

#[test]
fn test_template_renderer_default() {
    let renderer = TemplateRenderer::default();
    assert_eq!(renderer.template_dir(), &PathBuf::from("templates"));
}
