//! Swagger UI 模块
//!
//! 提供 Swagger UI 的 HTML 模板。

#![warn(missing_docs)]

use std::fmt::Write;

/// Swagger UI 配置
#[derive(Debug, Clone)]
pub struct SwaggerUiConfig {
    /// OpenAPI 文档 URL
    pub openapi_url: String,
    /// Swagger UI 标题
    pub title: String,
}

impl Default for SwaggerUiConfig {
    /// 创建默认的 Swagger UI 配置
    fn default() -> Self {
        Self { openapi_url: "/openapi.json".to_string(), title: "Swagger UI".to_string() }
    }
}

impl SwaggerUiConfig {
    /// 创建新的 Swagger UI 配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 OpenAPI 文档 URL
    pub fn openapi_url(mut self, url: impl Into<String>) -> Self {
        self.openapi_url = url.into();
        self
    }

    /// 设置 Swagger UI 标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
}

/// 生成 Swagger UI HTML
pub fn generate_html(config: &SwaggerUiConfig) -> String {
    let mut html = String::new();
    writeln!(&mut html, "<!DOCTYPE html>").unwrap();
    writeln!(&mut html, "<html lang=\"zh-CN\">").unwrap();
    writeln!(&mut html, "<head>").unwrap();
    writeln!(&mut html, "  <meta charset=\"UTF-8\">").unwrap();
    writeln!(&mut html, "  <title>{}</title>", config.title).unwrap();
    writeln!(
        &mut html,
        "  <link rel=\"stylesheet\" type=\"text/css\" href=\"https://unpkg.com/swagger-ui-dist@5/swagger-ui.css\">"
    )
    .unwrap();
    writeln!(&mut html, "  <style>").unwrap();
    writeln!(&mut html, "    html {{ box-sizing: border-box; overflow: -moz-scrollbars-vertical; overflow-y: scroll; }}")
        .unwrap();
    writeln!(&mut html, "    *, *:before, *:after {{ box-sizing: inherit; }}").unwrap();
    writeln!(&mut html, "    body {{ margin: 0; background: #fafafa; }}").unwrap();
    writeln!(&mut html, "  </style>").unwrap();
    writeln!(&mut html, "</head>").unwrap();
    writeln!(&mut html, "<body>").unwrap();
    writeln!(&mut html, "  <div id=\"swagger-ui\"></div>").unwrap();
    writeln!(&mut html, "  <script src=\"https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js\"></script>").unwrap();
    writeln!(&mut html, "  <script src=\"https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js\"></script>")
        .unwrap();
    writeln!(&mut html, "  <script>").unwrap();
    writeln!(&mut html, "    window.onload = function() {{").unwrap();
    writeln!(&mut html, "      const ui = SwaggerUIBundle({{").unwrap();
    writeln!(&mut html, "        url: '{}',", config.openapi_url).unwrap();
    writeln!(&mut html, "        dom_id: '#swagger-ui',").unwrap();
    writeln!(&mut html, "        deepLinking: true,").unwrap();
    writeln!(&mut html, "        presets: [").unwrap();
    writeln!(&mut html, "          SwaggerUIBundle.presets.apis,").unwrap();
    writeln!(&mut html, "          SwaggerUIStandalonePreset").unwrap();
    writeln!(&mut html, "        ],").unwrap();
    writeln!(&mut html, "        layout: 'StandaloneLayout'").unwrap();
    writeln!(&mut html, "      }});").unwrap();
    writeln!(&mut html, "      window.ui = ui;").unwrap();
    writeln!(&mut html, "    }};").unwrap();
    writeln!(&mut html, "  </script>").unwrap();
    writeln!(&mut html, "</body>").unwrap();
    writeln!(&mut html, "</html>").unwrap();
    html
}
