//! OpenAPI 文档构建器
//!
//! 提供流畅的 API 来构建符合 OpenAPI 3.1 规范的文档。

use super::super::*;
use std::collections::BTreeMap;

/// OpenAPI 文档构建器
///
/// 提供流畅的 API 来构建 OpenAPI 文档。
pub struct OpenApiBuilder {
    doc: OpenApiDoc,
}

impl OpenApiBuilder {
    /// 创建新的 OpenAPI 构建器
    pub fn new() -> Self {
        Self { doc: OpenApiDoc::default() }
    }

    /// 创建指定标题和版本的 OpenAPI 构建器
    pub fn with_title_and_version(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self { doc: OpenApiDoc::new(title, version) }
    }

    /// 设置 API 标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.doc.info.title = title.into();
        self
    }

    /// 设置 API 版本
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.doc.info.version = version.into();
        self
    }

    /// 设置 API 描述
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.doc.info.description = Some(desc.into());
        self
    }

    /// 设置服务条款 URL
    pub fn terms_of_service(mut self, url: impl Into<String>) -> Self {
        self.doc.info.terms_of_service = Some(url.into());
        self
    }

    /// 设置联系信息
    pub fn contact(mut self, contact: Contact) -> Self {
        self.doc.info.contact = Some(contact);
        self
    }

    /// 设置许可信息
    pub fn license(mut self, license: License) -> Self {
        self.doc.info.license = Some(license);
        self
    }

    /// 添加路径
    pub fn path(mut self, path: impl Into<String>, item: PathItem) -> Self {
        self.doc.paths.insert(path.into(), item);
        self
    }

    /// 添加多个路径
    pub fn paths(mut self, paths: BTreeMap<String, PathItem>) -> Self {
        self.doc.paths.extend(paths);
        self
    }

    /// 添加 Schema 组件
    pub fn schema(mut self, name: impl Into<String>, schema: Schema) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.schemas.insert(name.into(), schema);
        self
    }

    /// 添加多个 Schema 组件
    pub fn schemas(mut self, schemas: BTreeMap<String, Schema>) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.schemas.extend(schemas);
        self
    }

    /// 添加响应组件
    pub fn response(mut self, name: impl Into<String>, response: Response) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.responses.insert(name.into(), response);
        self
    }

    /// 添加多个响应组件
    pub fn responses(mut self, responses: BTreeMap<String, Response>) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.responses.extend(responses);
        self
    }

    /// 添加参数组件
    pub fn parameter(mut self, name: impl Into<String>, parameter: Parameter) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.parameters.insert(name.into(), parameter);
        self
    }

    /// 添加多个参数组件
    pub fn parameters(mut self, parameters: BTreeMap<String, Parameter>) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.parameters.extend(parameters);
        self
    }

    /// 添加请求体组件
    pub fn request_body(mut self, name: impl Into<String>, body: RequestBody) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.request_bodies.insert(name.into(), body);
        self
    }

    /// 添加多个请求体组件
    pub fn request_bodies(mut self, bodies: BTreeMap<String, RequestBody>) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.request_bodies.extend(bodies);
        self
    }

    /// 添加安全方案组件
    pub fn security_scheme(mut self, name: impl Into<String>, scheme: SecurityScheme) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.security_schemes.insert(name.into(), scheme);
        self
    }

    /// 添加多个安全方案组件
    pub fn security_schemes(mut self, schemes: BTreeMap<String, SecurityScheme>) -> Self {
        let components = self.doc.components.get_or_insert_with(Components::default);
        components.security_schemes.extend(schemes);
        self
    }

    /// 添加服务器
    pub fn server(mut self, url: impl Into<String>, description: Option<String>) -> Self {
        let servers = self.doc.servers.get_or_insert_with(Vec::new);
        servers.push(Server { url: url.into(), description, variables: None });
        self
    }

    /// 添加多个服务器
    pub fn servers(mut self, servers: Vec<Server>) -> Self {
        let doc_servers = self.doc.servers.get_or_insert_with(Vec::new);
        doc_servers.extend(servers);
        self
    }

    /// 添加标签
    pub fn tag(mut self, name: impl Into<String>, description: Option<String>) -> Self {
        let tags = self.doc.tags.get_or_insert_with(Vec::new);
        tags.push(Tag { name: name.into(), description, external_docs: None });
        self
    }

    /// 添加多个标签
    pub fn tags(mut self, tags: Vec<Tag>) -> Self {
        let doc_tags = self.doc.tags.get_or_insert_with(Vec::new);
        doc_tags.extend(tags);
        self
    }

    /// 设置外部文档
    pub fn external_docs(mut self, url: impl Into<String>, description: Option<String>) -> Self {
        self.doc.external_docs = Some(ExternalDocumentation { url: url.into(), description });
        self
    }

    /// 添加安全要求
    pub fn security(mut self, security: SecurityRequirement) -> Self {
        let securities = self.doc.security.get_or_insert_with(Vec::new);
        securities.push(security);
        self
    }

    /// 添加多个安全要求
    pub fn securities(mut self, securities: Vec<SecurityRequirement>) -> Self {
        let doc_securities = self.doc.security.get_or_insert_with(Vec::new);
        doc_securities.extend(securities);
        self
    }

    /// 构建 OpenAPI 文档
    pub fn build(self) -> OpenApiDoc {
        self.doc
    }
}

impl Default for OpenApiBuilder {
    /// 创建默认的 OpenAPI 构建器
    fn default() -> Self {
        Self::new()
    }
}
