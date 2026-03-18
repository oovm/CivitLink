<template>
  <div class="shiki-code-wrapper">
    <div v-if="html" class="shiki-container" v-html="html"></div>
    <div v-else class="loading-container">
      <div class="loading-spinner"></div>
      <span>Loading syntax highlighting...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { codeToHtml, createHighlighter } from 'shiki'

const props = defineProps<{
  code: string
  lang: string
  theme?: string
}>()

const html = ref<string>('')
let highlighter: any = null

async function initHighlighter() {
  try {
    highlighter = await createHighlighter({
      themes: ['vitesse-dark', 'vitesse-light'],
      langs: ['rust', 'javascript', 'typescript', 'html', 'css']
    })
    
    await highlightCode()
  } catch (error) {
    console.error('Failed to initialize highlighter:', error)
  }
}

async function highlightCode() {
  if (!highlighter) {
    await initHighlighter()
    return
  }

  try {
    const result = await codeToHtml(props.code, {
      lang: props.lang,
      theme: props.theme || 'vitesse-dark'
    })
    html.value = result
  } catch (error) {
    console.error('Failed to highlight code:', error)
  }
}

watch(() => [props.code, props.lang, props.theme], highlightCode)

onMounted(() => {
  initHighlighter()
})
</script>

<style scoped>
.shiki-code-wrapper {
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.shiki-container {
  width: 100%;
  height: 100%;
  overflow: auto;
  border-radius: 0.5rem;
}

.shiki-container :deep(.shiki) {
  margin: 0;
  border-radius: 0.5rem;
  padding: 1rem;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 0.875rem;
  line-height: 1.7;
}

.shiki-container :deep(.shiki code) {
  font-family: inherit;
}

.loading-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 1rem;
  color: #64748b;
  font-size: 0.875rem;
}

.loading-spinner {
  width: 2rem;
  height: 2rem;
  border: 3px solid #e2e8f0;
  border-top-color: #3b82f6;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
