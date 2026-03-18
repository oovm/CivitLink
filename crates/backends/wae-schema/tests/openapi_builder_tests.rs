use serde_json::json;
use std::collections::BTreeMap;
use wae_schema::{openapi::builder::OpenApiBuilder, *};

#[test]
fn test_basic_builder() {
    let doc = OpenApiBuilder::new().title("Test API").version("1.0.0").description("This is a test API").build();

    assert_eq!(doc.info.title, "Test API");
    assert_eq!(doc.info.version, "1.0.0");
    assert_eq!(doc.info.description, Some("This is a test API".to_string()));
}

#[test]
fn test_with_title_and_version() {
    let doc = OpenApiBuilder::with_title_and_version("My API", "2.0.0").build();

    assert_eq!(doc.info.title, "My API");
    assert_eq!(doc.info.version, "2.0.0");
}

#[test]
fn test_add_path() {
    let path_item = PathItem::new();
    let doc = OpenApiBuilder::new().path("/users", path_item).build();

    assert!(doc.paths.contains_key("/users"));
}

#[test]
fn test_add_schema() {
    let schema = Schema::object();
    let doc = OpenApiBuilder::new().schema("User", schema).build();

    let components = doc.components.as_ref().unwrap();
    assert!(components.schemas.contains_key("User"));
}

#[test]
fn test_openapi_doc_new() {
    let doc = OpenApiDoc::new("Test API", "1.0.0");
    assert_eq!(doc.info.title, "Test API");
    assert_eq!(doc.info.version, "1.0.0");
    assert_eq!(doc.openapi, "3.1.0");
}

#[test]
fn test_openapi_doc_default() {
    let doc = OpenApiDoc::default();
    assert_eq!(doc.openapi, "3.1.0");
}

#[test]
fn test_openapi_doc_methods() {
    let user_schema = Schema::object().property("id", Schema::integer());
    let param = Parameter::query("page");
    let request_body = RequestBody::json(Schema::string());
    let response = Response::new("Ok").json(Schema::string());
    let security_scheme = SecurityScheme::api_key("X-API-Key", ApiKeyLocation::Header);
    let security_req = SecurityRequirement::new().scheme("api_key", vec![]);

    let doc = OpenApiDoc::new("Test", "1.0")
        .description("Desc")
        .path("/users", PathItem::new())
        .schema("User", user_schema)
        .parameter("PageParam", param)
        .request_body("UserRequest", request_body)
        .response("UserResponse", response)
        .security_scheme("api_key", security_scheme)
        .server("https://api.example.com", Some("Prod".to_string()))
        .tag("users", Some("User management".to_string()))
        .external_docs("https://docs.example.com", Some("External docs".to_string()))
        .security(security_req);

    assert_eq!(doc.info.description, Some("Desc".to_string()));
    assert!(doc.paths.contains_key("/users"));
    assert!(doc.components.is_some());
    let comps = doc.components.as_ref().unwrap();
    assert!(comps.schemas.contains_key("User"));
    assert!(comps.parameters.contains_key("PageParam"));
    assert!(comps.request_bodies.contains_key("UserRequest"));
    assert!(comps.responses.contains_key("UserResponse"));
    assert!(comps.security_schemes.contains_key("api_key"));
    assert!(doc.servers.is_some());
    assert!(doc.tags.is_some());
    assert!(doc.external_docs.is_some());
    assert!(doc.security.is_some());
}

#[test]
fn test_openapi_doc_to_json() {
    let doc = OpenApiDoc::new("Test", "1.0");
    let json = doc.to_json();
    assert!(json.is_object());
}

#[test]
fn test_path_item_new() {
    let item = PathItem::new();
    assert!(item.get.is_none());
    assert!(item.post.is_none());
}

#[test]
fn test_path_item_default() {
    let item = PathItem::default();
    assert!(item.get.is_none());
}

#[test]
fn test_path_item_methods() {
    let item = PathItem::new()
        .get(Operation::new())
        .post(Operation::new())
        .put(Operation::new())
        .delete(Operation::new())
        .patch(Operation::new())
        .head(Operation::new())
        .options(Operation::new())
        .trace(Operation::new());

    assert!(item.get.is_some());
    assert!(item.post.is_some());
    assert!(item.put.is_some());
    assert!(item.delete.is_some());
    assert!(item.patch.is_some());
    assert!(item.head.is_some());
    assert!(item.options.is_some());
    assert!(item.trace.is_some());
}

#[test]
fn test_operation_new() {
    let op = Operation::new();
    assert!(op.responses.is_empty());
}

#[test]
fn test_operation_default() {
    let op = Operation::default();
    assert!(op.responses.is_empty());
}

#[test]
fn test_operation_methods() {
    let param = ParameterOrReference::Parameter(Parameter::query("page"));
    let request_body = RequestBodyOrReference::RequestBody(RequestBody::json(Schema::string()));
    let response = ResponseOrReference::Response(Response::new("Ok"));

    let op = Operation::new()
        .summary("Get users")
        .description("Get all users")
        .operation_id("getUsers")
        .tag("users")
        .parameter(param)
        .request_body(request_body)
        .response("200", response)
        .deprecated(true);

    assert_eq!(op.summary, Some("Get users".to_string()));
    assert_eq!(op.description, Some("Get all users".to_string()));
    assert_eq!(op.operation_id, Some("getUsers".to_string()));
    assert!(op.tags.is_some());
    assert!(op.parameters.is_some());
    assert!(op.request_body.is_some());
    assert!(op.responses.contains_key("200"));
    assert_eq!(op.deprecated, true);
}

#[test]
fn test_parameter_methods() {
    let path_param = Parameter::path("id")
        .description("User ID")
        .required(true)
        .schema(Schema::integer())
        .deprecated(false)
        .allow_empty_value(false)
        .example(json!(1));

    assert_eq!(path_param.name, "id");
    assert_eq!(path_param.location, ParameterLocation::Path);
    assert_eq!(path_param.description, Some("User ID".to_string()));
    assert_eq!(path_param.required, true);
    assert!(path_param.schema.is_some());
    assert_eq!(path_param.example, Some(json!(1)));

    let query_param = Parameter::query("page");
    assert_eq!(query_param.location, ParameterLocation::Query);

    let header_param = Parameter::header("Authorization");
    assert_eq!(header_param.location, ParameterLocation::Header);

    let cookie_param = Parameter::cookie("session");
    assert_eq!(cookie_param.location, ParameterLocation::Cookie);
}

#[test]
fn test_request_body_json() {
    let body = RequestBody::json(Schema::string()).description("Request body").required(true);

    assert_eq!(body.description, Some("Request body".to_string()));
    assert_eq!(body.required, true);
    assert!(body.content.contains_key("application/json"));
}

#[test]
fn test_response_new() {
    let resp = Response::new("Success");
    assert_eq!(resp.description, "Success");
}

#[test]
fn test_response_json() {
    let resp = Response::new("Success").json(Schema::string());
    assert!(resp.content.is_some());
    let content = resp.content.as_ref().unwrap();
    assert!(content.contains_key("application/json"));
}

#[test]
fn test_openapi_info_default() {
    let info = OpenApiInfo::default();
    assert_eq!(info.title, "API");
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn test_security_scheme_api_key() {
    let scheme = SecurityScheme::api_key("X-API-Key", ApiKeyLocation::Header).description("API Key authentication");

    assert_eq!(scheme.scheme_type, SecuritySchemeType::ApiKey);
    assert_eq!(scheme.name, Some("X-API-Key".to_string()));
    assert_eq!(scheme.location, Some(ApiKeyLocation::Header));
    assert_eq!(scheme.description, Some("API Key authentication".to_string()));
}

#[test]
fn test_security_scheme_basic() {
    let scheme = SecurityScheme::basic();
    assert_eq!(scheme.scheme_type, SecuritySchemeType::Http);
    assert_eq!(scheme.scheme, Some("basic".to_string()));
}

#[test]
fn test_security_scheme_bearer() {
    let scheme = SecurityScheme::bearer(Some("JWT".to_string()));
    assert_eq!(scheme.scheme_type, SecuritySchemeType::Http);
    assert_eq!(scheme.scheme, Some("bearer".to_string()));
    assert_eq!(scheme.bearer_format, Some("JWT".to_string()));
}

#[test]
fn test_security_scheme_oauth2() {
    let flow = OAuthFlow::new()
        .authorization_url("https://auth.example.com")
        .token_url("https://token.example.com")
        .refresh_url("https://refresh.example.com")
        .scope("read", "Read access")
        .scope("write", "Write access");

    let flows = OAuthFlows { implicit: Some(flow), password: None, client_credentials: None, authorization_code: None };

    let scheme = SecurityScheme::oauth2(flows);
    assert_eq!(scheme.scheme_type, SecuritySchemeType::OAuth2);
    assert!(scheme.flows.is_some());
}

#[test]
fn test_security_scheme_open_id_connect() {
    let scheme = SecurityScheme::open_id_connect("https://oidc.example.com");
    assert_eq!(scheme.scheme_type, SecuritySchemeType::OpenIdConnect);
    assert_eq!(scheme.open_id_connect_url, Some("https://oidc.example.com".to_string()));
}

#[test]
fn test_oauth_flow() {
    let flow = OAuthFlow::new()
        .authorization_url("https://auth.example.com")
        .token_url("https://token.example.com")
        .refresh_url("https://refresh.example.com")
        .scope("read", "Read access");

    let _flow2 = OAuthFlow::default();

    assert_eq!(flow.authorization_url, Some("https://auth.example.com".to_string()));
    assert_eq!(flow.token_url, Some("https://token.example.com".to_string()));
    assert_eq!(flow.refresh_url, Some("https://refresh.example.com".to_string()));
    assert!(flow.scopes.contains_key("read"));
}

#[test]
fn test_security_requirement() {
    let req = SecurityRequirement::new().scheme("api_key", vec![]).scheme("oauth2", vec!["read", "write"]);

    assert!(req.schemes.contains_key("api_key"));
    assert!(req.schemes.contains_key("oauth2"));
}

#[test]
fn test_security_requirement_default() {
    let req = SecurityRequirement::default();
    assert!(req.schemes.is_empty());
}

#[test]
fn test_components_default() {
    let comps = Components::default();
    assert!(comps.schemas.is_empty());
    assert!(comps.responses.is_empty());
    assert!(comps.parameters.is_empty());
}

#[test]
fn test_openapi_builder_with_paths() {
    let mut paths = BTreeMap::new();
    paths.insert("/users".to_string(), PathItem::new());
    paths.insert("/posts".to_string(), PathItem::new());

    let doc = OpenApiBuilder::new().paths(paths).build();
    assert!(doc.paths.contains_key("/users"));
    assert!(doc.paths.contains_key("/posts"));
}

#[test]
fn test_openapi_builder_with_schemas() {
    let mut schemas = BTreeMap::new();
    schemas.insert("User".to_string(), Schema::object());
    schemas.insert("Post".to_string(), Schema::object());

    let doc = OpenApiBuilder::new().schemas(schemas).build();
    let comps = doc.components.as_ref().unwrap();
    assert!(comps.schemas.contains_key("User"));
    assert!(comps.schemas.contains_key("Post"));
}

#[test]
fn test_openapi_builder_with_servers() {
    let servers = vec![
        Server { url: "https://api1.example.com".to_string(), description: None, variables: None },
        Server { url: "https://api2.example.com".to_string(), description: None, variables: None },
    ];

    let doc = OpenApiBuilder::new().servers(servers).build();
    assert_eq!(doc.servers.as_ref().unwrap().len(), 2);
}

#[test]
fn test_openapi_builder_with_tags() {
    let tags = vec![
        Tag { name: "users".to_string(), description: None, external_docs: None },
        Tag { name: "posts".to_string(), description: None, external_docs: None },
    ];

    let doc = OpenApiBuilder::new().tags(tags).build();
    assert_eq!(doc.tags.as_ref().unwrap().len(), 2);
}

#[test]
fn test_openapi_builder_with_securities() {
    let securities =
        vec![SecurityRequirement::new().scheme("api_key", vec![]), SecurityRequirement::new().scheme("oauth2", vec!["read"])];

    let doc = OpenApiBuilder::new().securities(securities).build();
    assert_eq!(doc.security.as_ref().unwrap().len(), 2);
}
