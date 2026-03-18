<template>
  <div class="min-h-screen">
    <header class="bg-white/90 backdrop-blur-xl shadow-lg border-b border-slate-200 sticky top-0 z-50 transition-all duration-300">
      <div class="container mx-auto px-4">
        <div class="flex items-center justify-between h-20">
          <div class="flex items-center gap-3 cursor-pointer" @click="$router.push('/')">
            <div class="w-12 h-12 bg-gradient-to-br from-orange-500 via-red-600 to-orange-700 rounded-xl flex items-center justify-center shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105">
              <span class="text-white font-extrabold text-2xl">W</span>
            </div>
            <span class="text-2xl font-extrabold bg-gradient-to-r from-orange-600 via-red-600 to-orange-700 bg-clip-text text-transparent hover:scale-[1.02] transition-transform">WAE</span>
          </div>
          
          <nav class="flex items-center gap-1">
            <router-link 
              to="/"
              :class="[
                'px-4 py-2.5 rounded-lg font-medium transition-all duration-200 flex items-center gap-2',
                $route.path === '/' 
                  ? 'text-white bg-gradient-to-r from-orange-600 to-red-700 shadow-md' 
                  : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
              ]"
            >
              首页
            </router-link>
            <router-link 
              to="/i"
              :class="[
                'px-4 py-2.5 rounded-lg font-medium transition-all duration-200',
                $route.path === '/i' 
                  ? 'text-white bg-gradient-to-r from-orange-600 to-red-700 shadow-md' 
                  : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
              ]"
            >
              部署
            </router-link>
            <router-link 
              to="/t"
              :class="[
                'px-4 py-2.5 rounded-lg font-medium transition-all duration-200',
                $route.path === '/t' 
                  ? 'text-white bg-gradient-to-r from-orange-600 to-red-700 shadow-md' 
                  : 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
              ]"
            >
              术语
            </router-link>
            <router-link 
              to="/d"
              :class="[
                'px-4 py-2.5 rounded-lg font-medium transition-all duration-200 ml-2',
                $route.path === '/d' 
                  ? 'text-white bg-gradient-to-r from-orange-700 to-red-800 shadow-md' 
                  : 'text-white bg-gradient-to-r from-orange-600 to-red-700 hover:from-orange-700 hover:to-red-800 shadow-md'
              ]"
            >
              文档
            </router-link>
          </nav>
        </div>
      </div>
    </header>

    <router-view />

    <footer class="bg-slate-900 text-slate-300">
      <div class="container mx-auto px-4 py-16">
        <div class="grid md:grid-cols-4 gap-12">
          <div class="md:col-span-2">
            <div class="flex items-center gap-3 mb-6">
              <div class="w-12 h-12 bg-gradient-to-br from-orange-500 via-red-600 to-orange-700 rounded-xl flex items-center justify-center">
                <span class="text-white font-extrabold text-2xl">W</span>
              </div>
              <span class="text-2xl font-extrabold text-white">WAE</span>
            </div>
            <p class="text-slate-400 mb-6 leading-relaxed max-w-md">
              微服务优先的 Rust 异步框架，完全替代 axum，深度融合 tokio，提供一站式全栈解决方案。
            </p>
            <div class="flex gap-4">
              <a href="https://github.com/oovm/wae" class="w-10 h-10 bg-slate-800 hover:bg-slate-700 rounded-lg flex items-center justify-center transition-colors">
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
              </a>
            </div>
          </div>
          
          <div>
            <h4 class="text-white font-semibold text-lg mb-6">文档资源</h4>
            <ul class="space-y-3">
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">使用文档</a></li>
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">API 参考</a></li>
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">示例项目</a></li>
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">教程指南</a></li>
            </ul>
          </div>
          
          <div>
            <h4 class="text-white font-semibold text-lg mb-6">项目信息</h4>
            <ul class="space-y-3">
              <li><a href="https://github.com/oovm/wae" class="text-slate-400 hover:text-white transition-colors">GitHub</a></li>
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">更新日志</a></li>
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">贡献指南</a></li>
              <li><a href="#" class="text-slate-400 hover:text-white transition-colors">问题反馈</a></li>
            </ul>
          </div>
        </div>
        
        <div class="border-t border-slate-800 mt-12 pt-8 flex flex-col md:flex-row justify-between items-center gap-4">
          <p class="text-slate-500 text-sm">
            © 2024 WAE. 保留所有权利。
          </p>
          <div class="flex gap-6 text-sm">
            <a href="#" class="text-slate-500 hover:text-white transition-colors">隐私政策</a>
            <a href="#" class="text-slate-500 hover:text-white transition-colors">服务条款</a>
            <a href="#" class="text-slate-500 hover:text-white transition-colors">行为准则</a>
          </div>
        </div>
      </div>
    </footer>
  </div>
</template>

<script setup lang="ts">
</script>
