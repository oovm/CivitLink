use wae_schema::swagger_ui::*;

#[test]
fn test_swagger_ui_config_default() {
    let config = SwaggerUiConfig::default();
    assert_eq!(config.openapi_url, "/openapi.json".to_string());
    assert_eq!(config.title, "Swagger UI".to_string());
}

#[test]
fn test_swagger_ui_config_new() {
    let config = SwaggerUiConfig::new();
    assert_eq!(config.openapi_url, "/openapi.json".to_string());
    assert_eq!(config.title, "Swagger UI".to_string());
}

#[test]
fn test_swagger_ui_config_custom() {
    let config = SwaggerUiConfig::new().openapi_url("/api/openapi.json").title("My API Docs");

    assert_eq!(config.openapi_url, "/api/openapi.json".to_string());
    assert_eq!(config.title, "My API Docs".to_string());
}

#[test]
fn test_generate_html() {
    let config = SwaggerUiConfig::new().openapi_url("/openapi.json").title("Test API");

    let html = generate_html(&config);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<html"));
    assert!(html.contains("<title>Test API</title>"));
    assert!(html.contains("url: '/openapi.json'"));
    assert!(html.contains("swagger-ui"));
    assert!(html.contains("SwaggerUIBundle"));
}

#[test]
fn test_generate_html_with_custom_url() {
    let config = SwaggerUiConfig::new().openapi_url("/custom-openapi.json");
    let html = generate_html(&config);
    assert!(html.contains("url: '/custom-openapi.json'"));
}

#[test]
fn test_generate_html_with_custom_title() {
    let config = SwaggerUiConfig::new().title("Custom API");
    let html = generate_html(&config);
    assert!(html.contains("<title>Custom API</title>"));
}
