<template>
  <div class="mac-code-container rounded-xl overflow-hidden shadow-2xl bg-slate-900">
    <div v-if="showTitle" class="flex items-center gap-2 px-4 py-3 bg-slate-800 border-b border-slate-700">
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded-full bg-red-500"></div>
        <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
        <div class="w-3 h-3 rounded-full bg-green-500"></div>
      </div>
      <div class="flex-1 text-center">
        <span class="text-slate-400 text-sm font-mono">{{ filename }}</span>
      </div>
      <div class="w-16"></div>
    </div>
    <div v-else class="flex items-center gap-2 px-4 py-3 bg-slate-800">
      <div class="flex items-center gap-2">
        <div class="w-3 h-3 rounded-full bg-red-500"></div>
        <div class="w-3 h-3 rounded-full bg-yellow-500"></div>
        <div class="w-3 h-3 rounded-full bg-green-500"></div>
      </div>
    </div>
    <div class="p-4 overflow-x-auto">
      <pre class="text-sm font-mono" v-html="highlightedCode"></pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { codeToHtml } from 'shiki'

interface Props {
  code: string
  filename?: string
  language?: string
  theme?: string
  showTitle?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  filename: 'main.rs',
  language: 'rust',
  theme: 'one-dark-pro',
  showTitle: true
})

const highlightedCode = ref('')

const highlight = async () => {
  const html = await codeToHtml(props.code, {
    lang: props.language,
    theme: props.theme
  })
  highlightedCode.value = html
}

watch(() => props.code, highlight)
watch(() => props.language, highlight)
watch(() => props.theme, highlight)

onMounted(highlight)
</script>

<style scoped>
.mac-code-container {
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

.mac-code-container :deep(pre) {
  margin: 0;
  padding: 0;
}

.mac-code-container :deep(code) {
  font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
}

.mac-code-container ::-webkit-scrollbar {
  height: 8px;
}

.mac-code-container ::-webkit-scrollbar-track {
  background: #1e293b;
}

.mac-code-container ::-webkit-scrollbar-thumb {
  background: #475569;
  border-radius: 4px;
}

.mac-code-container ::-webkit-scrollbar-thumb:hover {
  background: #64748b;
}
</style>
