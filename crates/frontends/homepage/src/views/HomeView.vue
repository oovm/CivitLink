<template>
  <div>
    <section class="bg-gradient-to-br from-orange-600 via-red-700 to-orange-800 relative overflow-hidden">
      <div class="absolute inset-0 overflow-hidden">
        <div class="absolute -top-40 -left-40 w-96 h-96 bg-orange-400 rounded-full mix-blend-multiply filter blur-3xl opacity-30"></div>
        <div class="absolute -bottom-40 -right-40 w-96 h-96 bg-red-400 rounded-full mix-blend-multiply filter blur-3xl opacity-30"></div>
      </div>
      
      <div class="container mx-auto px-4 py-24 relative z-10">
        <div class="grid lg:grid-cols-2 gap-12 items-center">
          <div class="text-white">
            <h1 class="text-5xl md:text-6xl font-extrabold mb-6 leading-tight">
              WAE - Rust Async Utilities
            </h1>
            <p class="text-xl md:text-2xl text-orange-100 mb-8 leading-relaxed">
              微服务优先的 Rust 异步框架，完全替代 axum，深度融合 tokio，提供一站式全栈解决方案
            </p>
            <div class="flex flex-wrap gap-4">
              <button 
                @click="$router.push('/d')"
                class="px-8 py-4 bg-white text-orange-600 rounded-xl font-bold text-lg hover:bg-orange-50 transition-all shadow-xl hover:shadow-2xl transform hover:-translate-y-0.5"
              >
                查看文档
              </button>
              <button 
                @click="scrollToFeatures"
                class="px-8 py-4 border-2 border-white text-white rounded-xl font-bold text-lg hover:bg-white/10 transition-all"
              >
                了解更多
              </button>
            </div>
          </div>
          
          <div class="backdrop-blur-sm rounded-2xl">
            <MacCode :code="exampleCode" filename="main.rs" :show-title="false" theme="one-dark-pro" />
          </div>
        </div>
      </div>
    </section>

    <section id="features" class="py-24 bg-white">
      <div class="container mx-auto px-4">
        <div class="text-center mb-16">
          <h2 class="text-4xl md:text-5xl font-extrabold bg-gradient-to-r from-orange-600 to-red-700 bg-clip-text text-transparent mb-4">
            核心特性
          </h2>
          <p class="text-xl text-gray-600 max-w-2xl mx-auto">
            专为现代微服务设计的高性能全栈解决方案
          </p>
        </div>
        
        <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-orange-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-orange-500/5 to-red-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-orange-500 to-red-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-orange-500/30 transition-all duration-300">
                <span class="text-4xl">🚀</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">微服务优先</h3>
              <p class="text-gray-600 leading-relaxed">
                服务发现、配置中心、链路追踪、健康检查，开箱即用
              </p>
            </div>
          </div>
          
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-orange-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-orange-500/5 to-red-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-blue-500/30 transition-all duration-300">
                <span class="text-4xl">⚡</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">深度融合 tokio</h3>
              <p class="text-gray-600 leading-relaxed">
                基于 axum 构建，原生 async/await，极致性能
              </p>
            </div>
          </div>
          
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-orange-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-orange-500/5 to-red-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-green-500 to-teal-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-green-500/30 transition-all duration-300">
                <span class="text-4xl">🤖</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">AI 友好</h3>
              <p class="text-gray-600 leading-relaxed">
                清晰的 Trait 抽象，统一的错误处理，最小化样板代码
              </p>
            </div>
          </div>
          
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-orange-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-orange-500/5 to-red-500/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-purple-500 to-pink-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-purple-500/30 transition-all duration-300">
                <span class="text-4xl">🦀</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">纯血 Rust</h3>
              <p class="text-gray-600 leading-relaxed">
                零 FFI 绑定，跨平台支持，内存安全，可审计
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section id="quickstart" class="py-24 bg-gradient-to-br from-slate-50 to-slate-100">
      <div class="container mx-auto px-4">
        <div class="text-center mb-16">
          <h2 class="text-4xl md:text-5xl font-extrabold bg-gradient-to-r from-orange-600 to-red-700 bg-clip-text text-transparent mb-4">
            快速开始
          </h2>
          <p class="text-xl text-gray-600 max-w-2xl mx-auto">
            几分钟内即可启动你的 WAE 服务
          </p>
        </div>

        <div class="max-w-4xl mx-auto space-y-6">
          <div class="bg-white rounded-2xl shadow-lg border border-slate-200 overflow-hidden">
            <div class="flex items-center gap-4 px-6 py-4 bg-slate-50 border-b border-slate-200">
              <div class="w-10 h-10 bg-gradient-to-br from-orange-500 to-red-600 rounded-lg flex items-center justify-center text-white font-bold text-lg">1</div>
              <h3 class="text-xl font-bold text-gray-900">添加依赖</h3>
            </div>
            <div class="p-6">
              <div class="bg-slate-900 rounded-xl p-4 mb-4 overflow-x-auto">
                <pre class="text-sm font-mono text-green-400">[dependencies]
wae = { path = "wae" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }</pre>
              </div>
              <p class="text-gray-600">在 Cargo.toml 中添加 WAE 及其核心依赖</p>
            </div>
          </div>

          <div class="bg-white rounded-2xl shadow-lg border border-slate-200 overflow-hidden">
            <div class="flex items-center gap-4 px-6 py-4 bg-slate-50 border-b border-slate-200">
              <div class="w-10 h-10 bg-gradient-to-br from-orange-500 to-red-600 rounded-lg flex items-center justify-center text-white font-bold text-lg">2</div>
              <h3 class="text-xl font-bold text-gray-900">创建主程序</h3>
            </div>
            <div class="p-6">
              <div class="bg-slate-900 rounded-xl p-4 mb-4 overflow-x-auto">
                <pre class="text-sm font-mono text-green-400">use axum::Router;
use wae_https::HttpsServerBuilder;

#[tokio::main]
async fn main() {
    let router = Router::new();
    let server = HttpsServerBuilder::new()
        .router(router)
        .build();
    server.serve().await;
}</pre>
              </div>
              <p class="text-gray-600">在 src/main.rs 中编写你的服务代码</p>
            </div>
          </div>

          <div class="bg-white rounded-2xl shadow-lg border border-slate-200 overflow-hidden">
            <div class="flex items-center gap-4 px-6 py-4 bg-slate-50 border-b border-slate-200">
              <div class="w-10 h-10 bg-gradient-to-br from-orange-500 to-red-600 rounded-lg flex items-center justify-center text-white font-bold text-lg">3</div>
              <h3 class="text-xl font-bold text-gray-900">启动服务</h3>
            </div>
            <div class="p-6">
              <div class="bg-slate-900 rounded-xl p-4 mb-4 overflow-x-auto">
                <pre class="text-sm font-mono text-green-400">cargo run</pre>
              </div>
              <p class="text-gray-600">执行命令启动服务，访问 http://localhost:3000 查看效果</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section id="modules" class="py-24 bg-white">
      <div class="container mx-auto px-4">
        <div class="text-center mb-16">
          <h2 class="text-4xl md:text-5xl font-extrabold bg-gradient-to-r from-orange-600 to-red-700 bg-clip-text text-transparent mb-4">
            模块概览
          </h2>
          <p class="text-xl text-gray-600 max-w-2xl mx-auto">
            模块化的 Crate 设计，每个模块可独立使用
          </p>
        </div>
        
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-6xl mx-auto">
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">wae-types</h3>
            <p class="text-gray-600">核心类型定义，统一错误类型与结果类型</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">wae-https</h3>
            <p class="text-gray-600">HTTP/HTTPS 服务，基于 axum 构建，统一响应结构</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">wae-authentication</h3>
            <p class="text-gray-600">认证服务 (JWT, OAuth2, SAML, TOTP)</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">wae-ai</h3>
            <p class="text-gray-600">AI 服务抽象，支持腾讯混元、火山引擎等</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">wae-storage</h3>
            <p class="text-gray-600">存储服务抽象，支持腾讯云 COS、阿里云 OSS、本地存储</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">wae-config</h3>
            <p class="text-gray-600">多层级配置管理，支持 TOML/YAML/环境变量</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import MacCode from '@/components/MacCode.vue'

const exampleCode = `use wae::{ApiResponse, Router, get, ToSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct HelloResponse {
    message: String,
}

async fn hello() -> ApiResponse<HelloResponse> {
    ApiResponse::success(HelloResponse {
        message: "Hello, WAE!".into(),
    })
}

#[tokio::main]
async fn main() -> wae::Result<()> {
    Router::new()
        .route("/hello", get(hello))
        .serve("0.0.0.0:3000", "wae-demo")
        .await
}`

function scrollToFeatures() {
  const featuresSection = document.getElementById('features')
  featuresSection?.scrollIntoView({ behavior: 'smooth' })
}
</script>
