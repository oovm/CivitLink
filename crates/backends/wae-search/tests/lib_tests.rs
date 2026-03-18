use wae_search::SearchConfig;

#[test]
fn test_search_config_default() {
    let config = SearchConfig::default();
    assert_eq!(config.url, "http://localhost:9200");
}
