# wae-schema

Schema 定义与验证模块，提供统一的 Schema 定义能力，支持 OpenAPI 文档生成。

## 概述

提供流畅的 Builder 模式 API，支持链式调用构建复杂 Schema。与 serde 深度集成，可直接序列化为 JSON Schema 格式。

## 快速开始

### 构建 Schema

```rust
use wae_schema::Schema;

let user_schema = Schema::object()
    .title("用户")
    .property("id", Schema::integer())
    .property("name", Schema::string().min_length(1))
    .required(vec!["id", "name"]);
```

### 生成 OpenAPI 文档

```rust
use wae_schema::{OpenApiDoc, PathItem, Operation, Response};

let doc = OpenApiDoc::new("用户 API", "1.0.0")
    .path("/users", PathItem::new()
        .get(Operation::new()
            .summary("获取用户列表")
            .response("200", Response::new("成功"))));

let json = doc.to_json();
```

## 核心用法

### 基础类型 Schema

```rust
use wae_schema::Schema;

let string_schema = Schema::string()
    .title("用户名")
    .description("用户登录名")
    .min_length(3)
    .max_length(32);

let integer_schema = Schema::integer()
    .title("年龄")
    .minimum(0.0)
    .maximum(150.0);

let bool_schema = Schema::boolean()
    .title("是否激活")
    .with_default(json!(false));

let number_schema = Schema::number()
    .title("价格")
    .minimum(0.0);
```

### 对象 Schema

```rust
use wae_schema::Schema;

let user_schema = Schema::object()
    .title("用户")
    .description("用户信息")
    .property("id", Schema::integer().description("用户ID"))
    .property("name", Schema::string().min_length(1).max_length(100))
    .property("email", Schema::string().format("email"))
    .property("age", Schema::integer().minimum(0.0).maximum(150.0))
    .required(vec!["id", "name", "email"]);
```

### 数组 Schema

```rust
use wae_schema::Schema;

let tags_schema = Schema::array(Schema::string())
    .title("标签列表")
    .description("文章标签");

let users_schema = Schema::array(
    Schema::object()
        .property("id", Schema::integer())
        .property("name", Schema::string())
);
```

### 枚举 Schema

```rust
use wae_schema::Schema;
use serde_json::json;

let status_schema = Schema::string()
    .title("状态")
    .enum_values(vec![
        json!("active"),
        json!("inactive"),
        json!("pending"),
    ]);
```

### 使用 SchemaBuilder

```rust
use wae_schema::SchemaBuilder;

let schema = SchemaBuilder::object()
    .title("产品")
    .description("产品信息")
    .property("id", Schema::integer())
    .property("name", Schema::string())
    .property("price", Schema::number())
    .required(vec!["id", "name", "price"])
    .build();
```

### OpenAPI 文档生成

#### 创建文档

```rust
use wae_schema::{OpenApiDoc, OpenApiInfo};

let doc = OpenApiDoc::new("用户服务 API", "1.0.0")
    .description("用户管理相关接口")
    .server("https://api.example.com", Some("生产环境".to_string()))
    .server("https://dev.api.example.com", Some("开发环境".to_string()));
```

#### 定义路径和操作

```rust
use wae_schema::{OpenApiDoc, PathItem, Operation, Parameter, Response, RequestBody};

let doc = OpenApiDoc::new("用户 API", "1.0.0")
    .path("/users", PathItem::new()
        .get(Operation::new()
            .summary("获取用户列表")
            .tag("用户管理")
            .parameter(Parameter::query("page")
                .description("页码")
                .schema(Schema::integer()))
            .response("200", Response::new("成功")
                .json(Schema::array(Schema::object()
                    .property("id", Schema::integer())
                    .property("name", Schema::string())))))
        .post(Operation::new()
            .summary("创建用户")
            .tag("用户管理")
            .request_body(RequestBody::json(
                Schema::object()
                    .property("name", Schema::string())
                    .property("email", Schema::string())
                    .required(vec!["name", "email"])))
            .response("201", Response::new("创建成功"))));
```

#### 参数定义

```rust
use wae_schema::{Parameter, Schema};

let path_param = Parameter::path("id")
    .description("用户ID")
    .required(true)
    .schema(Schema::integer());

let query_param = Parameter::query("keyword")
    .description("搜索关键词")
    .schema(Schema::string());

let header_param = Parameter::header("Authorization")
    .description("认证令牌")
    .required(true)
    .schema(Schema::string());
```

#### 导出文档

```rust
let json = doc.to_json();

#[cfg(feature = "yaml")]
let yaml = doc.to_yaml()?;
```

### ToSchema Trait

实现 `ToSchema` 自动生成 Schema：

```rust
use wae_schema::{Schema, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: i64,
    name: String,
    email: Option<String>,
}

impl ToSchema for User {
    fn schema() -> Schema {
        Schema::object()
            .property("id", Schema::integer())
            .property("name", Schema::string())
            .property("email", Schema::string())
            .required(vec!["id", "name"])
    }
}
```

内置实现：
- 基础类型：`String`、`i64`、`i32`、`i16`、`i8`、`u64`、`u32`、`u16`、`u8`、`f64`、`f32`、`bool`
- 容器类型：`Vec<T>`、`Option<T>`、`serde_json::Value`

## 最佳实践

### 完整 API 文档示例

```rust
use wae_schema::*;
use serde_json::json;

let user_schema = Schema::object()
    .title("用户")
    .description("用户信息")
    .property("id", Schema::integer().description("用户ID"))
    .property("name", Schema::string()
        .description("用户名")
        .min_length(2)
        .max_length(50))
    .property("email", Schema::string()
        .description("邮箱地址")
        .format("email"))
    .property("role", Schema::string()
        .enum_values(vec![json!("admin"), json!("user"), json!("guest")]))
    .property("created_at", Schema::string()
        .format("date-time")
        .read_only(true))
    .required(vec!["id", "name", "email"]);

let api_doc = OpenApiDoc::new("用户管理 API", "2.0.0")
    .description("用户管理系统的 RESTful API")
    .schema("User", user_schema.clone())
    .path("/users", PathItem::new()
        .get(Operation::new()
            .summary("获取用户列表")
            .tag("用户")
            .parameter(Parameter::query("page")
                .schema(Schema::integer().with_default(json!(1))))
            .parameter(Parameter::query("limit")
                .schema(Schema::integer().with_default(json!(20))))
            .response("200", Response::new("成功")
                .json(Schema::array(user_schema))))
        .post(Operation::new()
            .summary("创建用户")
            .tag("用户")
            .request_body(RequestBody::json(user_schema.clone()))
            .response("201", Response::new("创建成功"))))
    .path("/users/{id}", PathItem::new()
        .get(Operation::new()
            .summary("获取用户详情")
            .tag("用户")
            .parameter(Parameter::path("id")
                .schema(Schema::integer()))
            .response("200", Response::new("成功").json(user_schema))
            .response("404", Response::new("用户不存在"))));

let json_schema = api_doc.to_json();
```

### 复用 Schema 组件

```rust
let error_schema = Schema::object()
    .property("code", Schema::integer())
    .property("message", Schema::string())
    .required(vec!["code", "message"]);

let doc = OpenApiDoc::new("API", "1.0.0")
    .schema("Error", error_schema.clone())
    .path("/users", PathItem::new()
        .get(Operation::new()
            .response("500", Response::new("服务器错误")
                .json(error_schema))));
```

### 设置只读/只写字段

```rust
let user_schema = Schema::object()
    .property("id", Schema::integer().read_only(true))
    .property("password", Schema::string().write_only(true))
    .property("name", Schema::string());
```

## 常见问题

### Q: 如何设置默认值？

```rust
let schema = Schema::boolean()
    .with_default(json!(false));

let schema = Schema::integer()
    .with_default(json!(0));
```

### Q: 如何定义嵌套对象？

```rust
let address_schema = Schema::object()
    .property("city", Schema::string())
    .property("street", Schema::string());

let user_schema = Schema::object()
    .property("name", Schema::string())
    .property("address", address_schema);
```

### Q: 如何定义可选字段？

不添加到 `required` 列表中：

```rust
let schema = Schema::object()
    .property("id", Schema::integer())
    .property("nickname", Schema::string())
    .required(vec!["id"]);
```

或使用 `nullable`：

```rust
let schema = Schema::string()
    .nullable(true);
```

### Q: 如何添加自定义约束？

```rust
let schema = Schema::string()
    .pattern(r"^\d{4}-\d{2}-\d{2}$")
    .format("date");

let schema = Schema::integer()
    .minimum(0.0)
    .maximum(100.0);
```

### Q: 如何生成 YAML 格式？

启用 `yaml` feature：

```rust
#[cfg(feature = "yaml")]
let yaml = doc.to_yaml()?;
```
