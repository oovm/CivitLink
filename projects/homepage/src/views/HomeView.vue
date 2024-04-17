<template>
  <div>
    <section class="bg-gradient-to-br from-purple-700 via-violet-800 to-purple-900 relative overflow-hidden">
      <div class="absolute inset-0 overflow-hidden">
        <div class="absolute -top-40 -left-40 w-96 h-96 bg-purple-400 rounded-full mix-blend-multiply filter blur-3xl opacity-30"></div>
        <div class="absolute -bottom-40 -right-40 w-96 h-96 bg-violet-400 rounded-full mix-blend-multiply filter blur-3xl opacity-30"></div>
      </div>
      
      <div class="container mx-auto px-4 py-24 relative z-10">
        <div class="grid lg:grid-cols-2 gap-12 items-center">
          <div class="text-white">
            <h1 class="text-5xl md:text-6xl font-extrabold mb-6 leading-tight">
              GG Game Engine
            </h1>
            <p class="text-xl md:text-2xl text-purple-100 mb-8 leading-relaxed">
              以元游戏为核心的现代游戏引擎，Rust 编写，多平台支持，可组合的 ECS 架构，支持任意游戏类型
            </p>
            <div class="flex flex-wrap gap-4">
              <button 
                @click="$router.push('/d')"
                class="px-8 py-4 bg-white text-purple-700 rounded-xl font-bold text-lg hover:bg-purple-50 transition-all shadow-xl hover:shadow-2xl transform hover:-translate-y-0.5"
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
          <h2 class="text-4xl md:text-5xl font-extrabold bg-gradient-to-r from-purple-700 to-violet-800 bg-clip-text text-transparent mb-4">
            核心特性
          </h2>
          <p class="text-xl text-gray-600 max-w-2xl mx-auto">
            基于元游戏架构的革命性游戏引擎
          </p>
        </div>
        
        <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-purple-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-purple-600/5 to-violet-600/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-purple-600 to-violet-700 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-purple-500/30 transition-all duration-300">
                <span class="text-4xl">⚙️</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">元游戏架构</h3>
              <p class="text-gray-600 leading-relaxed">
                The One → Demiurge → Kosmos 三层架构，可组合任意游戏类型
              </p>
            </div>
          </div>
          
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-purple-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-purple-600/5 to-violet-600/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-blue-500/30 transition-all duration-300">
                <span class="text-4xl">🦀</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">纯血 Rust</h3>
              <p class="text-gray-600 leading-relaxed">
                零 FFI 绑定，内存安全，高性能，跨平台支持
              </p>
            </div>
          </div>
          
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-purple-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-purple-600/5 to-violet-600/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-green-500 to-teal-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-green-500/30 transition-all duration-300">
                <span class="text-4xl">🧩</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">可组合 ECS</h3>
              <p class="text-gray-600 leading-relaxed">
                组件化架构，GOM Schema 定义，即插即用的插件系统
              </p>
            </div>
          </div>
          
          <div class="group relative bg-gradient-to-br from-slate-50 to-slate-100 rounded-2xl p-8 border border-slate-200 hover:border-purple-300 transition-all duration-300 hover:shadow-2xl hover:-translate-y-2 overflow-hidden">
            <div class="absolute inset-0 bg-gradient-to-br from-purple-600/5 to-violet-600/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
            <div class="relative z-10">
              <div class="w-20 h-20 bg-gradient-to-br from-orange-500 to-amber-600 rounded-2xl flex items-center justify-center mb-6 shadow-lg group-hover:shadow-orange-500/30 transition-all duration-300">
                <span class="text-4xl">🎮</span>
              </div>
              <h3 class="text-2xl font-bold text-gray-900 mb-3">多类型支持</h3>
              <p class="text-gray-600 leading-relaxed">
                Galgame、RPG、动作游戏、策略游戏，任意类型均可实现
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section id="quickstart" class="py-24 bg-gradient-to-br from-slate-50 to-slate-100">
      <div class="container mx-auto px-4">
        <div class="text-center mb-16">
          <h2 class="text-4xl md:text-5xl font-extrabold bg-gradient-to-r from-purple-700 to-violet-800 bg-clip-text text-transparent mb-4">
            快速开始
          </h2>
          <p class="text-xl text-gray-600 max-w-2xl mx-auto">
            几分钟内即可创建你的第一个 GG 引擎游戏
          </p>
        </div>

        <div class="max-w-4xl mx-auto space-y-6">
          <div class="bg-white rounded-2xl shadow-lg border border-slate-200 overflow-hidden">
            <div class="flex items-center gap-4 px-6 py-4 bg-slate-50 border-b border-slate-200">
              <div class="w-10 h-10 bg-gradient-to-br from-purple-600 to-violet-700 rounded-lg flex items-center justify-center text-white font-bold text-lg">1</div>
              <h3 class="text-xl font-bold text-gray-900">创建项目</h3>
            </div>
            <div class="p-6">
              <div class="bg-slate-900 rounded-xl p-4 mb-4 overflow-x-auto">
                <pre class="text-sm font-mono text-green-400">cargo new my-first-game
cd my-first-game</pre>
              </div>
              <p class="text-gray-600">创建新的 Rust 项目并进入目录</p>
            </div>
          </div>

          <div class="bg-white rounded-2xl shadow-lg border border-slate-200 overflow-hidden">
            <div class="flex items-center gap-4 px-6 py-4 bg-slate-50 border-b border-slate-200">
              <div class="w-10 h-10 bg-gradient-to-br from-purple-600 to-violet-700 rounded-lg flex items-center justify-center text-white font-bold text-lg">2</div>
              <h3 class="text-xl font-bold text-gray-900">添加依赖</h3>
            </div>
            <div class="p-6">
              <div class="bg-slate-900 rounded-xl p-4 mb-4 overflow-x-auto">
                <pre class="text-sm font-mono text-green-400">[dependencies]
gg-core = { path = "gg-game-engine/projects/core/gg-core" }
gg-galgame = { path = "gg-game-engine/projects/galgame" }</pre>
              </div>
              <p class="text-gray-600">在 Cargo.toml 中添加 GG 引擎依赖</p>
            </div>
          </div>

          <div class="bg-white rounded-2xl shadow-lg border border-slate-200 overflow-hidden">
            <div class="flex items-center gap-4 px-6 py-4 bg-slate-50 border-b border-slate-200">
              <div class="w-10 h-10 bg-gradient-to-br from-purple-600 to-violet-700 rounded-lg flex items-center justify-center text-white font-bold text-lg">3</div>
              <h3 class="text-xl font-bold text-gray-900">编写游戏</h3>
            </div>
            <div class="p-6">
              <div class="bg-slate-900 rounded-xl p-4 mb-4 overflow-x-auto">
                <pre class="text-sm font-mono text-green-400">use gg_galgame::GalgameEngine;

fn main() -> gg_core::GResult<()> {
    let engine = GalgameEngine::load("game.toml")?;
    engine.run()
}</pre>
              </div>
              <p class="text-gray-600">在 src/main.rs 中编写你的游戏代码</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section id="modules" class="py-24 bg-white">
      <div class="container mx-auto px-4">
        <div class="text-center mb-16">
          <h2 class="text-4xl md:text-5xl font-extrabold bg-gradient-to-r from-purple-700 to-violet-800 bg-clip-text text-transparent mb-4">
            模块概览
          </h2>
          <p class="text-xl text-gray-600 max-w-2xl mx-auto">
            模块化的 Crate 设计，每个模块可独立使用
          </p>
        </div>
        
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-6xl mx-auto">
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">gg-core</h3>
            <p class="text-gray-600">核心工具、错误类型、反射系统、插件抽象</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">gg-ecs</h3>
            <p class="text-gray-600">实体组件系统，基于 Archetype 查询，高性能</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">gg-asset</h3>
            <p class="text-gray-600">资源管理系统，异步加载，缓存，热更新</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">gg-render</h3>
            <p class="text-gray-600">渲染抽象，支持多后端，材质，光照，后处理</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">gg-galgame</h3>
            <p class="text-gray-600">Galgame 德穆革，对话、立绘、转场、存档系统</p>
          </div>
          <div class="bg-gradient-to-br from-slate-50 to-slate-100 rounded-xl p-6 border border-slate-200">
            <h3 class="text-xl font-bold text-gray-900 mb-3">gg-editor-*</h3>
            <p class="text-gray-600">编辑器框架和面板，剧本、角色、场景、资源管理</p>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import MacCode from '@/components/MacCode.vue'

const exampleCode = `use gg_core::GResult;
use gg_galgame::GalgameEngine;
use gg_galgame_schema::components::*;
use gg_plugin_dialogue::DialoguePlugin;
use gg_plugin_portrait::PortraitPlugin;

fn main() -> GResult<()> {
    let mut engine = GalgameEngine::new();
    
    engine.register_plugin(DialoguePlugin)?;
    engine.register_plugin(PortraitPlugin)?;
    
    engine.load("game.toml")?;
    engine.run()
}`

function scrollToFeatures() {
  const featuresSection = document.getElementById('features')
  featuresSection?.scrollIntoView({ behavior: 'smooth' })
}
</script>
