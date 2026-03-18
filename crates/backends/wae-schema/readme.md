# wae-schema

Schema 模块 - 提供数据结构定义、验证和 OpenAPI 文档生成。

## 主要功能

- **Schema 定义**: 类型安全的数据结构
- **Schema 构建器**: 流畅的 API 构建 JSON Schema
- **OpenAPI 文档生成**: 自动生成 OpenAPI 3.1 规范的 API 文档
- **Swagger UI 集成**: 内置 Swagger UI 界面，方便 API 测试和文档查看
- **axum 集成**: 与 axum 框架无缝集成，提供文档服务端点
- **ToSchema trait**: 为 Rust 标准类型提供自动 Schema 生成

## 技术栈

- **序列化**: serde, serde_json
- **OpenAPI 规范**: OpenAPI 3.1
- **Web 框架**: axum (可选功能)
- **Swagger UI**: 从 CDN 加载的 Swagger UI 5.x

## 功能特性

### Schema 构建

使用流畅的 API 构建 JSON Schema：

```rust
use wae_schema::Schema;

let user_schema = Schema::object()
    .title("User")
    .description("用户信息")
    .property("id", Schema::integer().description("用户 ID"))
    .property("username", Schema::string().min_length(1).max_length(50))
    .property("email", Schema::string().format("email"))
    .property("age", Schema::integer().minimum(0).maximum(150))
    .required(vec!["id", "username", "email"]);
```

### 使用 SchemaBuilder

```rust
use wae_schema::SchemaBuilder;

let product_schema = SchemaBuilder::object()
    .title("Product")
    .property("name", Schema::string())
    .property("price", Schema::number())
    .build();
```

### OpenAPI 文档构建

使用 OpenApiBuilder 构建完整的 API 文档：

```rust
use wae_schema::{
    OpenApiBuilder, PathItem, Operation, Response, Schema,
    RequestBody, Parameter, ParameterLocation
};

let doc = OpenApiBuilder::with_title_and_version("用户 API", "1.0.0")
    .description("用户管理相关的 API 接口")
    .tag("用户", Some("用户管理操作"))
    .path(
        "/users/{id}",
        PathItem::new()
            .get(
                Operation::new()
                    .summary("获取用户信息")
                    .description("根据用户 ID 获取用户详细信息")
                    .operation_id("get_user")
                    .tag("用户")
                    .parameter(Parameter::path("id")
                        .description("用户 ID")
                        .schema(Schema::integer()))
                    .response("200", Response::new("成功").json(Schema::object()))
            )
    )
    .build();
```

### 与 axum 集成

将 OpenAPI 文档作为服务端点提供：

```rust
use axum::Router;
use wae_schema::{OpenApiDoc, OpenApiBuilder};
use wae_schema::openapi::axum::openapi_router;

#[tokio::main]
async fn main() {
    let doc = OpenApiBuilder::with_title_and_version("我的 API", "1.0.0").build();
    
    let app = Router::new()
        .merge(openapi_router(doc));
    
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

访问 `/openapi.json` 获取 JSON 格式的 OpenAPI 文档。

### 使用自定义配置

```rust
use wae_schema::openapi::axum::{OpenApiService, OpenApiServiceConfig};

let config = OpenApiServiceConfig::new()
    .json_path("/api-docs.json")
    .yaml_path("/api-docs.yaml")
    .enable_json(true)
    .enable_yaml(false);

let service = OpenApiService::with_config(doc, config);
let router = service.into_router();
```

### 合并到现有路由

```rust
use axum::Router;
use wae_schema::openapi::axum::merge_openapi_router;

let mut app = Router::new();
app = merge_openapi_router(app, doc);
```

### Swagger UI 集成

添加 Swagger UI 界面：

```rust
use axum::Router;
use wae_schema::swagger_ui::SwaggerUiService;
use wae_schema::swagger_ui::SwaggerUiConfig;

let swagger_config = SwaggerUiConfig::new()
    .openapi_url("/openapi.json")
    .title("我的 API 文档");

let swagger_service = SwaggerUiService::new(swagger_config);
let swagger_router = swagger_service.router();

let app = Router::new()
    .merge(openapi_router(doc))
    .merge(swagger_router);
```

访问 `/swagger-ui` 查看交互式 API 文档界面。

### 完整示例

结合 OpenAPI 文档和 Swagger UI 的完整示例：

```rust
use axum::Router;
use wae_schema::{
    OpenApiBuilder, OpenApiDoc, PathItem, Operation, Response, Schema,
    openapi::axum::openapi_router,
    swagger_ui::{SwaggerUiService, SwaggerUiConfig}
};

fn create_openapi_doc() -> OpenApiDoc {
    OpenApiBuilder::with_title_and_version("电商 API", "2.0.0")
        .description("电商平台的 RESTful API 文档")
        .server("http://localhost:3000", Some("本地开发服务器"))
        .tag("商品", Some("商品管理接口"))
        .tag("订单", Some("订单管理接口"))
        .path(
            "/products",
            PathItem::new()
                .get(
                    Operation::new()
                        .summary("获取商品列表")
                        .tag("商品")
                        .response("200", Response::new("成功").json(Schema::array(Schema::object())))
                )
                .post(
                    Operation::new()
                        .summary("创建商品")
                        .tag("商品")
                        .response("201", Response::new("创建成功").json(Schema::object()))
                )
        )
        .path(
            "/products/{id}",
            PathItem::new()
                .get(
                    Operation::new()
                        .summary("获取商品详情")
                        .tag("商品")
                        .parameter(Parameter::path("id").schema(Schema::integer()))
                        .response("200", Response::new("成功").json(Schema::object()))
                )
        )
        .build()
}

#[tokio::main]
async fn main() {
    let doc = create_openapi_doc();
    
    let swagger_config = SwaggerUiConfig::new()
        .title("电商 API 文档");
    
    let app = Router::new()
        .merge(openapi_router(doc))
        .merge(SwaggerUiService::new(swagger_config).router());
    
    println!("服务器运行在 http://127.0.0.1:3000");
    println!("OpenAPI JSON: http://127.0.0.1:3000/openapi.json");
    println!("Swagger UI: http://127.0.0.1:3000/swagger-ui");
    
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## 特性标志

- `axum`: 启用 axum 框架集成
- `yaml`: 启用 YAML 格式的 OpenAPI 文档输出

在 Cargo.toml 中启用：

```toml
[dependencies]
wae-schema = { version = "0.0.0", features = ["axum", "yaml"] }
```

## ToSchema trait

为常见 Rust 类型提供自动 Schema 生成：

```rust
use wae_schema::ToSchema;

let string_schema = String::schema();
let int_schema = i32::schema();
let vec_schema = Vec::<String>::schema();
let option_schema = Option::<i64>::schema();
```

支持的类型：
- 字符串: `String`
- 整数: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- 浮点数: `f32`, `f64`
- 布尔: `bool`
- 数组: `Vec<T>` where T: ToSchema
- 可选: `Option<T>` where T: ToSchema
- JSON 值: `serde_json::Value`
